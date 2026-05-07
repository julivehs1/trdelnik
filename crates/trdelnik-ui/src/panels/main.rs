//! Main candlestick chart panel

use egui::{Color32, Rect, Ui};
use egui_plot::{BoxElem, BoxPlot, BoxSpread, Corner, Legend, Plot, PlotUi};

use trdelnik_core::{AxisCoordinate, MarkerShape};
use trdelnik_render::Plot as PlotTrait;
use trdelnik_data::ChartData;
use trdelnik_theme::ChartTheme;

use crate::axis_interaction::{
    handle_drag_pan, handle_scroll_zoom, handle_x_axis_interaction, handle_y_axis_interaction,
    update_cursor_and_draw_crosshair, DragPanState,
};
use crate::axis_scale::{CursorState, PanelYState, SharedXState};
use crate::config::ChartConfig;
use crate::egui_theme::ToEguiColor;

/// Result from rendering the main chart
#[derive(Debug, Clone)]
pub struct MainChartResult {
    /// Whether the chart is hovered
    pub hovered: bool,
    /// The inner rect of the plot (data area, excluding axes)
    pub inner_rect: Rect,
    /// Current X bounds (after pan/zoom)
    pub x_bounds: (f64, f64),
    /// Current Y bounds (after pan/zoom)
    pub y_bounds: (f64, f64),
}

/// Render the main candlestick chart
pub fn render_main_chart<X: AxisCoordinate>(
    ui: &mut Ui,
    chart_data: &ChartData<X>,
    theme: &ChartTheme,
    config: &ChartConfig,
    chart_id: egui::Id,
    height: f32,
    show_x_axis: bool,
) -> MainChartResult {
    let candles = chart_data.series().candles();
    if candles.is_empty() {
        ui.label("No data to display");
        return MainChartResult {
            hovered: false,
            inner_rect: Rect::NOTHING,
            x_bounds: (0.0, 1.0),
            y_bounds: (0.0, 100.0),
        };
    }

    let spacing = chart_data.x_spacing();
    let (price_min, price_max) = chart_data.price_range().unwrap_or((0.0, 100.0));
    let (data_x_min, data_x_max) = chart_data.x_range().unwrap_or((0.0, 1.0));
    let price_padding = (price_max - price_min) * 0.05;
    let data_y_min = price_min - price_padding;
    let data_y_max = price_max + price_padding;

    // Load axis scale states
    let y_state_id = chart_id.with("main_y");
    let x_state_id = chart_id.with("shared_x");
    let mut y_state = PanelYState::load(ui.ctx(), y_state_id);
    let mut x_state = SharedXState::load(ui.ctx(), x_state_id);

    // Initialize bounds if needed
    y_state.init_if_needed(data_y_min, data_y_max);
    x_state.init_if_needed(data_x_min, data_x_max);

    // Build plot
    let mut plot = Plot::new(chart_id.with("main"))
        .height(height)
        .show_axes([show_x_axis, true])
        .show_grid(config.show_grid)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_double_click_reset(false)
        .auto_bounds([false, false])
        .reset()
        .include_x(x_state.x_min)
        .include_x(x_state.x_max)
        .include_y(y_state.y_min)
        .include_y(y_state.y_max)
        .y_axis_position(egui_plot::HPlacement::Right)
        .y_axis_min_width(60.0)
        .link_axis(chart_id.with("link"), [true, false])
        .label_formatter(move |_name, value| {
            let x_coord = X::from_plot_value(value.x);
            format!("Price: {:.2}\nTime: {}", value.y, x_coord.format_label())
        });

    if config.show_legend {
        plot = plot.legend(Legend::default().position(Corner::LeftTop));
    }

    let plot_response = plot.show(ui, |plot_ui| {
        draw_candlesticks(plot_ui, chart_data, theme, spacing, config.candle_width_ratio);

        // Draw overlays
        for overlay in chart_data.overlay_plots() {
            draw_overlay(plot_ui, overlay.as_ref(), theme);
        }
    });

    // Draw overlay markers (after plot so they appear on top)
    draw_markers(ui, chart_data.overlay_markers(), &plot_response.transform);

    let outer_rect = plot_response.response.rect;

    // Handle scroll zoom (X-axis only)
    handle_scroll_zoom(ui, outer_rect, &mut x_state);

    // Handle drag to pan (both axes)
    let pan_state_id = chart_id.with("main_pan");
    let mut pan_state = DragPanState::load(ui.ctx(), pan_state_id);
    handle_drag_pan(ui, outer_rect, &mut x_state, &mut y_state, &mut pan_state, pan_state_id);

    // Handle axis interactions
    handle_y_axis_interaction(ui, outer_rect, &mut y_state, y_state_id, data_y_min, data_y_max);
    if show_x_axis {
        handle_x_axis_interaction(ui, outer_rect, &mut x_state, x_state_id, data_x_min, data_x_max);
    }

    // Update cursor state and draw crosshair
    let cursor_state_id = chart_id.with("cursor");
    let mut cursor_state = CursorState::load(ui.ctx(), cursor_state_id);
    update_cursor_and_draw_crosshair(
        ui,
        outer_rect,
        &x_state,
        &y_state,
        &mut cursor_state,
        config.show_crosshair,
    );
    cursor_state.store(ui.ctx(), cursor_state_id);

    // Store states
    y_state.store(ui.ctx(), y_state_id);
    x_state.store(ui.ctx(), x_state_id);

    let plot_bounds = plot_response.transform.bounds();

    MainChartResult {
        hovered: plot_response.response.hovered(),
        inner_rect: *plot_response.transform.frame(),
        x_bounds: (plot_bounds.min()[0], plot_bounds.max()[0]),
        y_bounds: (plot_bounds.min()[1], plot_bounds.max()[1]),
    }
}

fn draw_candlesticks<X: AxisCoordinate>(
    plot_ui: &mut PlotUi,
    chart_data: &ChartData<X>,
    theme: &ChartTheme,
    spacing: f64,
    width_ratio: f64,
) {
    let candles = chart_data.series().candles();
    let candle_width = spacing * width_ratio;
    let wick_width = candle_width * 0.1;

    let mut bullish_boxes = Vec::new();
    let mut bearish_boxes = Vec::new();

    for candle in candles {
        let x = candle.x.to_plot_value();
        let is_bullish = candle.is_bullish();

        let (lower, upper) = if is_bullish {
            (candle.open, candle.close)
        } else {
            (candle.close, candle.open)
        };

        let box_elem = BoxElem::new(
            x,
            BoxSpread::new(
                candle.low,
                lower,
                (candle.open + candle.close) / 2.0,
                upper,
                candle.high,
            ),
        )
        .box_width(candle_width)
        .whisker_width(wick_width);

        if is_bullish {
            bullish_boxes.push(box_elem);
        } else {
            bearish_boxes.push(box_elem);
        }
    }

    if !bullish_boxes.is_empty() {
        plot_ui.box_plot(
            BoxPlot::new("Bullish", bullish_boxes)
                .color(theme.bullish.to_egui()),
        );
    }

    if !bearish_boxes.is_empty() {
        plot_ui.box_plot(
            BoxPlot::new("Bearish", bearish_boxes)
                .color(theme.bearish.to_egui()),
        );
    }
}

fn draw_overlay<X: AxisCoordinate>(
    plot_ui: &mut PlotUi,
    overlay: &dyn PlotTrait<X>,
    theme: &ChartTheme,
) {
    let bounds = plot_ui.plot_bounds();
    let render_bounds = trdelnik_render::Bounds2D::new(
        (bounds.min()[0], bounds.min()[1]),
        (bounds.max()[0], bounds.max()[1]),
    );
    trdelnik_render_egui::render_plot(overlay, plot_ui, render_bounds, theme);
}

/// Draw markers using the plot transform
pub(crate) fn draw_markers<X: AxisCoordinate>(
    ui: &Ui,
    markers: &[trdelnik_core::IndicatorMarker<X>],
    transform: &egui_plot::PlotTransform,
) {
    if markers.is_empty() {
        return;
    }

    let painter = ui.painter();
    let bounds = transform.bounds();
    let frame = transform.frame();

    for marker in markers {
        let x = marker.x.to_plot_value();
        let y = marker.y;

        // Skip if outside bounds
        if x < bounds.min()[0] || x > bounds.max()[0] ||
           y < bounds.min()[1] || y > bounds.max()[1] {
            continue;
        }

        // Convert to screen coordinates
        let screen_pos = transform.position_from_point(&egui_plot::PlotPoint::new(x, y));

        // Skip if outside frame
        if !frame.contains(screen_pos) {
            continue;
        }

        let color = Color32::from_rgba_unmultiplied(
            marker.color.r,
            marker.color.g,
            marker.color.b,
            marker.color.a,
        );

        match marker.shape {
            MarkerShape::ArrowUp => {
                draw_arrow_up(painter, screen_pos, color);
            }
            MarkerShape::ArrowDown => {
                draw_arrow_down(painter, screen_pos, color);
            }
            MarkerShape::Cross => {
                draw_cross(painter, screen_pos, color);
            }
            MarkerShape::Circle => {
                draw_circle(painter, screen_pos, color);
            }
            MarkerShape::Square => {
                draw_square(painter, screen_pos, color);
            }
        }
    }
}

/// Draw an upward-pointing arrow
fn draw_arrow_up(painter: &egui::Painter, pos: egui::Pos2, color: Color32) {
    let size = 8.0;
    let points = vec![
        egui::pos2(pos.x, pos.y - size),
        egui::pos2(pos.x - size * 0.6, pos.y),
        egui::pos2(pos.x + size * 0.6, pos.y),
    ];
    painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

/// Draw a downward-pointing arrow
fn draw_arrow_down(painter: &egui::Painter, pos: egui::Pos2, color: Color32) {
    let size = 8.0;
    let points = vec![
        egui::pos2(pos.x, pos.y + size),
        egui::pos2(pos.x - size * 0.6, pos.y),
        egui::pos2(pos.x + size * 0.6, pos.y),
    ];
    painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

/// Draw an X marker
fn draw_cross(painter: &egui::Painter, pos: egui::Pos2, color: Color32) {
    let size = 5.0;
    let stroke = egui::Stroke::new(2.0, color);
    painter.line_segment(
        [egui::pos2(pos.x - size, pos.y - size), egui::pos2(pos.x + size, pos.y + size)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(pos.x + size, pos.y - size), egui::pos2(pos.x - size, pos.y + size)],
        stroke,
    );
}

/// Draw a circle marker
fn draw_circle(painter: &egui::Painter, pos: egui::Pos2, color: Color32) {
    let radius = 5.0;
    painter.circle_filled(pos, radius, color);
}

/// Draw a square marker
fn draw_square(painter: &egui::Painter, pos: egui::Pos2, color: Color32) {
    let size = 5.0;
    let rect = egui::Rect::from_center_size(pos, egui::vec2(size * 2.0, size * 2.0));
    painter.rect_filled(rect, 0.0, color);
}
