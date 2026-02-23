//! Resize handle component for resizable panels
//!
//! Provides a draggable handle for adjusting panel heights.

use egui::{Color32, CursorIcon, Rect, Response, Sense, Ui};

use crate::panel_state::PanelHeightState;

/// Render a resize handle between panels
///
/// # Arguments
///
/// * `ui` - The egui UI context
/// * `panel_id` - ID of the panel below the handle
/// * `state` - Panel height state for tracking resize operations
/// * `min_height` - Minimum allowed height
/// * `max_height` - Maximum allowed height
/// * `handle_color` - Default color of the handle
/// * `hover_color` - Color when hovered or dragging
///
/// # Returns
///
/// The response from the handle interaction
pub fn render_resize_handle(
    ui: &mut Ui,
    panel_id: &str,
    state: &mut PanelHeightState,
    min_height: f32,
    max_height: f32,
    handle_color: Color32,
    hover_color: Color32,
) -> Response {
    // Allocate space for the handle
    // Interaction area: 6px, visible bar: 2px (centered)
    let interaction_height = 6.0;
    let visible_height = 2.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), interaction_height),
        Sense::click_and_drag(),
    );

    // Change cursor on hover
    if response.hovered() || state.is_resizing_panel(panel_id) {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }

    // Handle drag start
    if response.drag_started() {
        let current_height = state.get_height(panel_id, 100.0);
        if let Some(pos) = response.interact_pointer_pos() {
            state.start_resize(panel_id, pos.y, current_height);
        }
    }

    // Handle drag
    if response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            state.update_resize(pos.y, min_height, max_height);
        }
    }

    // Handle drag end
    if response.drag_stopped() {
        state.end_resize();
    }

    // Draw the visible handle
    if ui.is_rect_visible(rect) {
        let is_active = response.hovered() || state.is_resizing_panel(panel_id);
        let color = if is_active { hover_color } else { handle_color };

        // Full-width line centered within the interaction area
        let visible_rect = Rect::from_center_size(
            rect.center(),
            egui::vec2(rect.width(), visible_height),
        );

        ui.painter().rect_filled(visible_rect, 0.0, color);
    }

    response
}

/// Render a resize handle between two panels that resizes both
///
/// Dragging down increases the panel above and decreases the panel below.
/// The total height of both panels remains constant.
pub fn render_resize_handle_dual(
    ui: &mut Ui,
    panel_above: &str,
    panel_below: &str,
    state: &mut PanelHeightState,
    min_height_above: f32,
    max_height_above: f32,
    min_height_below: f32,
    max_height_below: f32,
    handle_color: Color32,
    hover_color: Color32,
) -> Response {
    let interaction_height = 6.0;
    let visible_height = 2.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), interaction_height),
        Sense::click_and_drag(),
    );

    let is_active = response.hovered() || response.dragged();

    // Change cursor on hover
    if is_active {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }

    // Handle drag - use delta instead of absolute position to avoid feedback loop
    // (the handle moves as panels resize, so absolute position doesn't work)
    if response.dragged() {
        let delta = response.drag_delta().y;
        if delta.abs() > 0.0 {
            state.apply_resize_delta(
                panel_above,
                panel_below,
                delta,
                min_height_above,
                max_height_above,
                min_height_below,
                max_height_below,
            );
        }
    }

    // Handle drag end - clear resizing state
    if response.drag_stopped() {
        state.end_resize();
    }

    // Draw the visible handle
    if ui.is_rect_visible(rect) {
        let color = if is_active { hover_color } else { handle_color };

        let visible_rect = Rect::from_center_size(
            rect.center(),
            egui::vec2(rect.width(), visible_height),
        );

        ui.painter().rect_filled(visible_rect, 0.0, color);
    }

    response
}

/// A simpler resize handle that returns the delta
///
/// This is useful when you want more control over how the resize is applied.
pub fn simple_resize_handle(
    ui: &mut Ui,
    _id: impl std::hash::Hash,
    handle_color: Color32,
    hover_color: Color32,
) -> f32 {
    let interaction_height = 6.0;
    let visible_height = 2.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), interaction_height),
        Sense::click_and_drag(),
    );

    // Change cursor on hover
    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }

    // Draw the visible handle
    if ui.is_rect_visible(rect) {
        let is_active = response.hovered() || response.dragged();
        let color = if is_active { hover_color } else { handle_color };

        let visible_rect = Rect::from_center_size(
            rect.center(),
            egui::vec2(rect.width(), visible_height),
        );

        ui.painter().rect_filled(visible_rect, 0.0, color);
    }

    // Return the drag delta
    if response.dragged() {
        response.drag_delta().y
    } else {
        0.0
    }
}
