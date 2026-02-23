//! New Panel container rendering with dual Y-axis support

use egui::{Rect, Ui};
use egui_plot::{Bar, BarChart, HLine, Line, Plot, PlotPoints};

use std::collections::HashMap;

use trdelnik_core::{AxisCoordinate, Color, YAxis};
use trdelnik_data::Panel;
use trdelnik_theme::ChartTheme;

use crate::axis_interaction::{
    draw_vertical_crosshair, handle_drag_pan, handle_scroll_zoom, handle_x_axis_interaction,
    handle_y_axis_interaction, DragPanState,
};
use crate::axis_scale::{CursorState, PanelYState, SharedXState};
use crate::config::ChartConfig;
use crate::egui_theme::ToEguiColor;

/// Result from rendering a panel container
#[derive(Debug, Clone)]
pub struct PanelContainerResult {
    /// The inner rect of the panel (data area)
    pub inner_rect: Rect,
    /// Current X bounds (after pan/zoom)
    pub x_bounds: (f64, f64),
    /// Current Y bounds (after pan/zoom)
    pub y_bounds: (f64, f64),
}

/// Render a panel container with multiple indicators and dual Y-axis support
pub fn render_panel_container<X: AxisCoordinate>(
    ui: &mut Ui,
    panel: &Panel<X>,
    theme: &ChartTheme,
    config: &ChartConfig,
    chart_id: egui::Id,
    x_spacing: f64,
    height: f32,
    show_x_axis: bool,
    data_x_range: (f64, f64),
) -> PanelContainerResult {
    let (data_y_min_left, data_y_max_left) = panel.left_y_range();
    let (data_y_min_right, data_y_max_right) = panel.right_y_range();
    let (data_x_min, data_x_max) = data_x_range;
    let bar_width = x_spacing * config.candle_width_ratio * 0.8;
    let has_right_axis = panel.has_right_axis();

    // Load axis scale states
    let panel_id = panel.id();
    let y_state_id = chart_id.with(format!("{}_y", panel_id));
    let y_state_right_id = chart_id.with(format!("{}_y_right", panel_id));
    let x_state_id = chart_id.with("shared_x");
    let mut y_state_left = PanelYState::load(ui.ctx(), y_state_id);
    let mut y_state_right = PanelYState::load(ui.ctx(), y_state_right_id);
    let mut x_state = SharedXState::load(ui.ctx(), x_state_id);

    // Initialize bounds if needed
    y_state_left.init_if_needed(data_y_min_left, data_y_max_left);
    if has_right_axis {
        y_state_right.init_if_needed(data_y_min_right, data_y_max_right);
    }

    let plot_response = Plot::new(chart_id.with(panel_id))
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
        .include_y(y_state_left.y_min)
        .include_y(y_state_left.y_max)
        .y_axis_position(egui_plot::HPlacement::Right)
        .y_axis_min_width(60.0)
        .link_axis(chart_id.with("link"), [true, false])
        .show(ui, |plot_ui| {
            // Draw reference lines from all indicators
            for level in panel.all_reference_lines() {
                let color = theme.grid.to_egui();

                if level == 0.0 {
                    plot_ui.hline(HLine::new(level).color(color));
                } else {
                    plot_ui.hline(
                        HLine::new(level)
                            .color(color)
                            .style(egui_plot::LineStyle::Dashed { length: 4.0 }),
                    );
                }
            }

            // Draw indicators
            for indicator in &panel.indicators {
                // Draw histogram if present - group bars by color for efficiency
                if let Some(histogram) = &indicator.histogram {
                    let mut bars_by_color: HashMap<[u8; 4], Vec<Bar>> = HashMap::new();

                    for bar_data in histogram {
                        let value = if bar_data.axis == YAxis::Right && has_right_axis {
                            transform_to_left_axis(
                                bar_data.value,
                                &y_state_right,
                                &y_state_left,
                            )
                        } else {
                            bar_data.value
                        };

                        let bar = Bar::new(bar_data.x.to_plot_value(), value).width(bar_width);
                        let color_key = bar_data.color.to_array();
                        bars_by_color.entry(color_key).or_default().push(bar);
                    }

                    // Render each color group as a separate BarChart
                    for (color_key, bars) in bars_by_color {
                        let color = Color::from_array(color_key);
                        plot_ui.bar_chart(BarChart::new(bars).color(color.to_egui()));
                    }
                }

                // Draw lines
                for line in &indicator.lines {
                    // Use line's color if set, otherwise get from theme
                    let color = line.color
                        .map(|c| c.to_egui())
                        .unwrap_or_else(|| get_line_color(&line.line_id, theme));

                    // Transform right-axis values to left-axis coordinates
                    let points: Vec<[f64; 2]> = if line.axis == YAxis::Right && has_right_axis {
                        line.points
                            .iter()
                            .filter_map(|(x, y)| {
                                y.map(|y_val| {
                                    let transformed_y =
                                        transform_to_left_axis(y_val, &y_state_right, &y_state_left);
                                    [x.to_plot_value(), transformed_y]
                                })
                            })
                            .collect()
                    } else {
                        line.valid_plot_points()
                    };

                    let plot_points: PlotPoints = points.into();
                    plot_ui.line(Line::new(plot_points).color(color).name(&line.name));
                }
            }
        });

    let outer_rect = plot_response.response.rect;

    // Handle scroll zoom (X-axis only)
    handle_scroll_zoom(ui, outer_rect, &mut x_state);

    // Handle drag to pan (both axes)
    let pan_state_id = chart_id.with(format!("{}_pan", panel_id));
    let mut pan_state = DragPanState::load(ui.ctx(), pan_state_id);
    handle_drag_pan(
        ui,
        outer_rect,
        &mut x_state,
        &mut y_state_left,
        &mut pan_state,
        pan_state_id,
    );

    // Handle axis interactions
    handle_y_axis_interaction(
        ui,
        outer_rect,
        &mut y_state_left,
        y_state_id,
        data_y_min_left,
        data_y_max_left,
    );
    if show_x_axis {
        handle_x_axis_interaction(ui, outer_rect, &mut x_state, x_state_id, data_x_min, data_x_max);
    }

    // Draw vertical crosshair (synchronized with main panel cursor)
    if config.show_crosshair {
        let cursor_state_id = chart_id.with("cursor");
        let cursor_state = CursorState::load(ui.ctx(), cursor_state_id);
        draw_vertical_crosshair(ui, outer_rect, &cursor_state, &x_state);
    }

    // Store states
    y_state_left.store(ui.ctx(), y_state_id);
    if has_right_axis {
        y_state_right.store(ui.ctx(), y_state_right_id);
    }
    x_state.store(ui.ctx(), x_state_id);

    // Get actual bounds from plot transform
    let plot_bounds = plot_response.transform.bounds();

    PanelContainerResult {
        inner_rect: *plot_response.transform.frame(),
        x_bounds: (plot_bounds.min()[0], plot_bounds.max()[0]),
        y_bounds: (plot_bounds.min()[1], plot_bounds.max()[1]),
    }
}

/// Transform a value from right-axis coordinates to left-axis coordinates
fn transform_to_left_axis(value: f64, right_state: &PanelYState, left_state: &PanelYState) -> f64 {
    let right_range = right_state.y_max - right_state.y_min;
    let left_range = left_state.y_max - left_state.y_min;

    if right_range == 0.0 {
        return value;
    }

    let normalized = (value - right_state.y_min) / right_range;
    left_state.y_min + normalized * left_range
}

/// Get the color for a line based on its ID
fn get_line_color(line_id: &str, theme: &ChartTheme) -> egui::Color32 {
    match line_id {
        "rsi" => theme.rsi.to_egui(),
        "macd_line" => theme.macd_line.to_egui(),
        "macd_signal" => theme.macd_signal.to_egui(),
        "stoch_k" => theme.rsi.to_egui(),      // Use RSI color for %K
        "stoch_d" => theme.macd_signal.to_egui(), // Use signal color for %D
        "atr" => theme.sma.to_egui(),
        _ => theme.sma.to_egui(),
    }
}
