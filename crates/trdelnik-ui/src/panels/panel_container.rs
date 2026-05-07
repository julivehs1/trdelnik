//! Panel container rendering with dual Y-axis support

use egui::{Rect, Ui};
use egui_plot::{HLine, Plot};

use trdelnik_core::AxisCoordinate;
use trdelnik_render::HLineStyle;
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
    let _ = x_spacing;
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
            // Draw horizontal reference lines from the panel
            for hl in panel.hlines() {
                let color = hl.color
                    .map(|c| c.to_egui())
                    .unwrap_or_else(|| theme.grid.to_egui());

                match hl.style {
                    HLineStyle::Solid => {
                        plot_ui.hline(HLine::new("ref", hl.level).color(color));
                    }
                    HLineStyle::Dashed => {
                        plot_ui.hline(
                            HLine::new("ref", hl.level)
                                .color(color)
                                .style(egui_plot::LineStyle::Dashed { length: 4.0 }),
                        );
                    }
                }
            }

            // Render every plot through trdelnik-render-egui — this is the
            // single egui-aware Plot rendering site. Right-axis values are
            // rescaled into the left-axis range (egui_plot only exposes
            // one Y axis).
            let bounds = plot_ui.plot_bounds();
            let render_bounds = trdelnik_render::Bounds2D::new(
                (bounds.min()[0], bounds.min()[1]),
                (bounds.max()[0], bounds.max()[1]),
            );
            if has_right_axis {
                let right = trdelnik_render_egui::RightAxisTransform {
                    right_min: y_state_right.y_min,
                    right_max: y_state_right.y_max,
                    left_min: y_state_left.y_min,
                    left_max: y_state_left.y_max,
                };
                for plot_data in panel.plots() {
                    trdelnik_render_egui::render_plot_dual_axis(
                        plot_data.as_ref(),
                        plot_ui,
                        render_bounds,
                        right,
                        theme,
                    );
                }
            } else {
                for plot_data in panel.plots() {
                    trdelnik_render_egui::render_plot(
                        plot_data.as_ref(),
                        plot_ui,
                        render_bounds,
                        theme,
                    );
                }
            }
        });

    // Draw markers (after plot so they appear on top)
    crate::panels::main::draw_markers(ui, panel.markers(), &plot_response.transform);

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

