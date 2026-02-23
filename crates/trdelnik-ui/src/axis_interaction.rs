//! Helper functions for axis interaction

use egui::{Color32, CursorIcon, Id, PointerButton, Pos2, Rect, Sense, Ui};

use crate::axis_scale::{CursorState, PanelYState, SharedXState};

/// Width of the Y-axis area in pixels
pub const Y_AXIS_WIDTH: f32 = 60.0;
/// Height of the X-axis area in pixels
pub const X_AXIS_HEIGHT: f32 = 24.0;
/// Sensitivity for drag scaling
pub const DRAG_SENSITIVITY: f64 = 0.005;

/// Background color when hovering (very subtle)
const AXIS_HOVER_COLOR: Color32 = Color32::from_rgba_premultiplied(100, 100, 120, 20);
/// Background color when dragging (slightly more visible)
const AXIS_DRAG_COLOR: Color32 = Color32::from_rgba_premultiplied(100, 100, 140, 40);

/// Calculate the Y-axis rect (right side of plot, where axis labels are - OUTSIDE the plot)
pub fn y_axis_rect(outer_rect: Rect) -> Rect {
    // Y-axis labels are to the RIGHT of the plot rect
    Rect::from_min_max(
        Pos2::new(outer_rect.right(), outer_rect.top()),
        Pos2::new(outer_rect.right() + Y_AXIS_WIDTH, outer_rect.bottom()),
    )
}

/// Calculate the X-axis rect (bottom of plot, where axis labels are - OUTSIDE the plot)
pub fn x_axis_rect(outer_rect: Rect) -> Rect {
    // X-axis labels are BELOW the plot rect
    Rect::from_min_max(
        Pos2::new(outer_rect.left(), outer_rect.bottom()),
        Pos2::new(outer_rect.right(), outer_rect.bottom() + X_AXIS_HEIGHT),
    )
}

/// Calculate the inner plot rect (data area, excluding axes)
pub fn inner_plot_rect(inner_rect: Rect) -> Rect {
    inner_rect
}

/// Handle Y-axis interaction for a panel
pub fn handle_y_axis_interaction(
    ui: &mut Ui,
    outer_rect: Rect,
    state: &mut PanelYState,
    state_id: Id,
    data_y_min: f64,
    data_y_max: f64,
) {
    let axis_rect = y_axis_rect(outer_rect);
    let response = ui.interact(axis_rect, state_id.with("y_interact"), Sense::click_and_drag());

    // Draw visual feedback only when interacting
    if response.dragged() {
        ui.painter().rect_filled(axis_rect, 0.0, AXIS_DRAG_COLOR);
    } else if response.hovered() {
        ui.painter().rect_filled(axis_rect, 0.0, AXIS_HOVER_COLOR);
    }

    // Handle drag start
    if response.drag_started() {
        state.dragging = true;
        state.drag_start_y = response.interact_pointer_pos().map(|p| p.y);
        state.drag_start_bounds = Some((state.y_min, state.y_max));
    }

    // Handle dragging
    if response.dragged() && state.dragging {
        if let (Some(start_y), Some(current_pos), Some((start_min, start_max))) = (
            state.drag_start_y,
            response.interact_pointer_pos(),
            state.drag_start_bounds,
        ) {
            let delta = start_y - current_pos.y; // Drag up = zoom in (smaller range)
            let factor = 1.0 + delta as f64 * DRAG_SENSITIVITY;
            let factor = factor.clamp(0.1, 10.0);

            let center = (start_min + start_max) / 2.0;
            let half_range = (start_max - start_min) / 2.0;
            let new_half_range = half_range / factor;

            state.y_min = center - new_half_range;
            state.y_max = center + new_half_range;
        }
    }

    // Handle drag end
    if response.drag_stopped() {
        state.dragging = false;
        state.drag_start_y = None;
        state.drag_start_bounds = None;
    }

    // Handle double-click reset
    if response.double_clicked() {
        state.reset(data_y_min, data_y_max);
    }

    // Set cursor
    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }

    state.clone().store(ui.ctx(), state_id);
}

/// Handle X-axis interaction (shared across panels)
pub fn handle_x_axis_interaction(
    ui: &mut Ui,
    outer_rect: Rect,
    state: &mut SharedXState,
    state_id: Id,
    data_x_min: f64,
    data_x_max: f64,
) {
    let axis_rect = x_axis_rect(outer_rect);
    let response = ui.interact(axis_rect, state_id.with("x_interact"), Sense::click_and_drag());

    // Draw visual feedback only when interacting
    if response.dragged() {
        ui.painter().rect_filled(axis_rect, 0.0, AXIS_DRAG_COLOR);
    } else if response.hovered() {
        ui.painter().rect_filled(axis_rect, 0.0, AXIS_HOVER_COLOR);
    }

    // Handle drag start
    if response.drag_started() {
        state.dragging = true;
        state.drag_start_x = response.interact_pointer_pos().map(|p| p.x);
        state.drag_start_bounds = Some((state.x_min, state.x_max));
    }

    // Handle dragging
    if response.dragged() && state.dragging {
        if let (Some(start_x), Some(current_pos), Some((start_min, start_max))) = (
            state.drag_start_x,
            response.interact_pointer_pos(),
            state.drag_start_bounds,
        ) {
            let delta = start_x - current_pos.x; // Drag left = zoom out, drag right = zoom in
            let factor = 1.0 + delta as f64 * DRAG_SENSITIVITY;
            let factor = factor.clamp(0.1, 10.0);

            let center = (start_min + start_max) / 2.0;
            let half_range = (start_max - start_min) / 2.0;
            let new_half_range = half_range / factor;

            state.x_min = center - new_half_range;
            state.x_max = center + new_half_range;
        }
    }

    // Handle drag end
    if response.drag_stopped() {
        state.dragging = false;
        state.drag_start_x = None;
        state.drag_start_bounds = None;
    }

    // Handle double-click reset
    if response.double_clicked() {
        state.x_min = data_x_min;
        state.x_max = data_x_max;
    }

    // Set cursor
    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
    }

    state.clone().store(ui.ctx(), state_id);
}

/// Calculate the inner data rect (the plot area itself, axes are outside)
pub fn data_rect(outer_rect: Rect) -> Rect {
    // The outer_rect IS the data area - axes are drawn outside
    outer_rect
}

/// Handle scroll wheel zoom on the plot area (X-axis, centered on cursor position)
pub fn handle_scroll_zoom(
    ui: &Ui,
    outer_rect: Rect,
    x_state: &mut SharedXState,
) {
    let data_area = data_rect(outer_rect);
    if let Some(pos) = ui.ctx().pointer_hover_pos() {
        if data_area.contains(pos) {
            let scroll = ui.input(|i| i.raw_scroll_delta);

            if scroll.y != 0.0 {
                // Scroll up (positive) = zoom in = smaller range
                // Scroll down (negative) = zoom out = larger range
                let zoom_factor = if scroll.y > 0.0 { 0.9 } else { 1.1 };

                // Calculate cursor position in data coordinates
                let rel_x = (pos.x - data_area.left()) / data_area.width();
                let cursor_x = x_state.x_min + rel_x as f64 * (x_state.x_max - x_state.x_min);

                // Zoom centered on cursor position
                // Keep the cursor at the same relative position after zoom
                let range = x_state.x_max - x_state.x_min;
                let new_range = range * zoom_factor;

                // Distance from cursor to min/max edges (as fraction of range)
                let left_frac = (cursor_x - x_state.x_min) / range;
                let right_frac = (x_state.x_max - cursor_x) / range;

                x_state.x_min = cursor_x - left_frac * new_range;
                x_state.x_max = cursor_x + right_frac * new_range;
            }
        }
    }
}

/// State for drag panning
#[derive(Clone, Debug, Default)]
pub struct DragPanState {
    pub dragging: bool,
    pub drag_start: Option<Pos2>,
    pub x_start: Option<(f64, f64)>,
    pub y_start: Option<(f64, f64)>,
}

impl DragPanState {
    pub fn load(ctx: &egui::Context, id: Id) -> Self {
        ctx.data(|d| d.get_temp::<Self>(id)).unwrap_or_default()
    }

    pub fn store(self, ctx: &egui::Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
}

/// Handle drag to pan on the plot area (both axes)
pub fn handle_drag_pan(
    ui: &Ui,
    outer_rect: Rect,
    x_state: &mut SharedXState,
    y_state: &mut PanelYState,
    pan_state: &mut DragPanState,
    pan_state_id: Id,
) {
    let data_area = data_rect(outer_rect);
    let response = ui.interact(data_area, pan_state_id.with("drag_pan"), Sense::drag());

    if response.drag_started_by(PointerButton::Primary) {
        pan_state.dragging = true;
        pan_state.drag_start = response.interact_pointer_pos();
        pan_state.x_start = Some((x_state.x_min, x_state.x_max));
        pan_state.y_start = Some((y_state.y_min, y_state.y_max));
    }

    if response.dragged_by(PointerButton::Primary) && pan_state.dragging {
        if let (
            Some(start),
            Some(current),
            Some((x_min_start, x_max_start)),
            Some((y_min_start, y_max_start)),
        ) = (
            pan_state.drag_start,
            response.interact_pointer_pos(),
            pan_state.x_start,
            pan_state.y_start,
        ) {
            let dx = current.x - start.x;
            let dy = current.y - start.y;

            // Convert pixel delta to data delta
            let x_range = x_max_start - x_min_start;
            let y_range = y_max_start - y_min_start;

            let x_delta = -(dx as f64 / data_area.width() as f64) * x_range;
            let y_delta = (dy as f64 / data_area.height() as f64) * y_range;

            x_state.x_min = x_min_start + x_delta;
            x_state.x_max = x_max_start + x_delta;
            y_state.y_min = y_min_start + y_delta;
            y_state.y_max = y_max_start + y_delta;
        }
    }

    if response.drag_stopped() {
        pan_state.dragging = false;
        pan_state.drag_start = None;
        pan_state.x_start = None;
        pan_state.y_start = None;
    }

    // Show grab cursor when hovering or dragging
    if response.hovered() && !pan_state.dragging {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    } else if pan_state.dragging {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    }

    pan_state.clone().store(ui.ctx(), pan_state_id);
}

/// Color for crosshair lines
const CROSSHAIR_COLOR: Color32 = Color32::from_rgba_premultiplied(150, 150, 150, 180);

/// Update cursor state and draw crosshair if enabled
pub fn update_cursor_and_draw_crosshair(
    ui: &Ui,
    outer_rect: Rect,
    x_state: &SharedXState,
    y_state: &PanelYState,
    cursor_state: &mut CursorState,
    show_crosshair: bool,
) {
    let data_area = data_rect(outer_rect);
    let hover_pos = ui.ctx().pointer_hover_pos();

    // Update cursor position in data coordinates
    cursor_state.update_from_screen(
        hover_pos,
        data_area,
        x_state.x_min,
        x_state.x_max,
        y_state.y_min,
        y_state.y_max,
    );

    // Draw crosshair if enabled and cursor is active
    if show_crosshair {
        if let Some(pos) = hover_pos {
            if data_area.contains(pos) {
                let painter = ui.painter();

                // Draw vertical line (full height of data area)
                painter.line_segment(
                    [
                        Pos2::new(pos.x, data_area.top()),
                        Pos2::new(pos.x, data_area.bottom()),
                    ],
                    egui::Stroke::new(1.0, CROSSHAIR_COLOR),
                );

                // Draw horizontal line (full width of data area)
                painter.line_segment(
                    [
                        Pos2::new(data_area.left(), pos.y),
                        Pos2::new(data_area.right(), pos.y),
                    ],
                    egui::Stroke::new(1.0, CROSSHAIR_COLOR),
                );
            }
        }
    }
}

/// Draw only the vertical crosshair line (for panels that share X-axis but have different Y)
pub fn draw_vertical_crosshair(
    ui: &Ui,
    outer_rect: Rect,
    cursor_state: &CursorState,
    x_state: &SharedXState,
) {
    if !cursor_state.active {
        return;
    }

    if let Some(cursor_x) = cursor_state.x {
        let data_area = data_rect(outer_rect);

        // Convert data X to screen X
        let rel_x = (cursor_x - x_state.x_min) / (x_state.x_max - x_state.x_min);
        let screen_x = data_area.left() + rel_x as f32 * data_area.width();

        // Only draw if within the data area
        if screen_x >= data_area.left() && screen_x <= data_area.right() {
            let painter = ui.painter();
            painter.line_segment(
                [
                    Pos2::new(screen_x, data_area.top()),
                    Pos2::new(screen_x, data_area.bottom()),
                ],
                egui::Stroke::new(1.0, CROSSHAIR_COLOR),
            );
        }
    }
}
