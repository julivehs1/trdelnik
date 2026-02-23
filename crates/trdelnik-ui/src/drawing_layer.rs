//! Drawing layer for rendering drawings on the chart
//!
//! This module provides functions to render drawings and handle user interaction.

use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};
use trdelnik_core::AxisCoordinate;
use trdelnik_drawings::{
    ChartPoint, DrawingFill, DrawingHandle, DrawingLabel, DrawingLine, DrawingOutput,
    DrawingStore, DrawingStyle, HandleType, HorizontalLineOutput, LineStyle, TextAnchor,
    ToolState, VerticalLineOutput,
};

use crate::egui_theme::to_egui_color;

/// Theme settings for drawing rendering
#[derive(Clone, Debug)]
pub struct DrawingRenderTheme {
    /// Handle radius in pixels
    pub handle_radius: f32,
    /// Handle fill color
    pub handle_fill: Color32,
    /// Handle stroke color
    pub handle_stroke: Color32,
    /// Selected handle fill color
    pub selected_handle_fill: Color32,
    /// Preview opacity (0.0 - 1.0)
    pub preview_opacity: f32,
}

impl Default for DrawingRenderTheme {
    fn default() -> Self {
        Self {
            handle_radius: 4.0,
            handle_fill: Color32::WHITE,
            handle_stroke: Color32::from_rgb(41, 98, 255),
            selected_handle_fill: Color32::from_rgb(41, 98, 255),
            preview_opacity: 0.6,
        }
    }
}

/// View bounds for coordinate transformation
#[derive(Clone, Copy, Debug)]
pub struct ViewBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl ViewBounds {
    pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
}

/// Render all drawings from a store (for main panel - backwards compatible)
pub fn render_drawings<X: AxisCoordinate>(
    painter: &Painter,
    plot_rect: Rect,
    store: &DrawingStore<X>,
    tool_state: &ToolState<X>,
    bounds: ViewBounds,
    theme: &DrawingRenderTheme,
) {
    render_drawings_for_panel(painter, plot_rect, store, tool_state, bounds, theme, "main");
}

/// Render drawings for a specific panel
pub fn render_drawings_for_panel<X: AxisCoordinate>(
    painter: &Painter,
    plot_rect: Rect,
    store: &DrawingStore<X>,
    tool_state: &ToolState<X>,
    bounds: ViewBounds,
    theme: &DrawingRenderTheme,
    panel_id: &str,
) {
    // Create a clipped painter to ensure drawings don't extend beyond the plot area
    let clipped_painter = painter.with_clip_rect(plot_rect);

    let selected_id = store.selected();

    // Render all visible drawings for this panel
    for (drawing, style) in store.visible_for_panel(panel_id) {
        let is_selected = selected_id == Some(drawing.id());
        let output = drawing.compute(style);
        render_drawing_output(&clipped_painter, plot_rect, &output, bounds, theme, is_selected);
    }

    // Render pending drawing (preview) - only on the panel being drawn to
    // For now, always show on main panel
    if panel_id == "main" {
        if let Some(pending) = tool_state.pending() {
            let style = DrawingStyle::default();
            let output = pending.compute(&style);

            // Render with preview opacity
            render_drawing_output_preview(
                &clipped_painter,
                plot_rect,
                &output,
                bounds,
                theme,
                tool_state.preview_point(),
            );
        }
    }
}

/// Render a single drawing's output
fn render_drawing_output<X: AxisCoordinate>(
    painter: &Painter,
    plot_rect: Rect,
    output: &DrawingOutput<X>,
    bounds: ViewBounds,
    theme: &DrawingRenderTheme,
    is_selected: bool,
) {
    // Helper to convert data coordinates to screen coordinates
    let to_screen = |point: &ChartPoint<X>| -> Pos2 {
        let x_pct = (point.x.to_plot_value() - bounds.x_min) / (bounds.x_max - bounds.x_min);
        let y_pct = (point.y - bounds.y_min) / (bounds.y_max - bounds.y_min);

        Pos2::new(
            plot_rect.left() + (x_pct as f32) * plot_rect.width(),
            plot_rect.bottom() - (y_pct as f32) * plot_rect.height(),
        )
    };

    // Render fills first (background)
    for fill in &output.fills {
        render_fill(painter, fill, &to_screen);
    }

    // Render horizontal lines (span full width)
    for hline in &output.horizontal_lines {
        render_horizontal_line(painter, plot_rect, hline, bounds);
    }

    // Render vertical lines (span full height)
    for vline in &output.vertical_lines {
        render_vertical_line(painter, plot_rect, vline, bounds);
    }

    // Render regular lines
    for line in &output.lines {
        render_line(painter, plot_rect, line, bounds, &to_screen);
    }

    // Render labels
    for label in &output.labels {
        render_label(painter, plot_rect, label, bounds, &to_screen);
    }

    // Render handles (only if selected)
    if is_selected {
        for handle in &output.handles {
            render_handle(painter, handle, theme, &to_screen);
        }
    }
}

/// Render a preview drawing (with transparency and preview line to cursor)
fn render_drawing_output_preview<X: AxisCoordinate>(
    painter: &Painter,
    plot_rect: Rect,
    output: &DrawingOutput<X>,
    bounds: ViewBounds,
    theme: &DrawingRenderTheme,
    preview_point: Option<&ChartPoint<X>>,
) {
    let to_screen = |point: &ChartPoint<X>| -> Pos2 {
        let x_pct = (point.x.to_plot_value() - bounds.x_min) / (bounds.x_max - bounds.x_min);
        let y_pct = (point.y - bounds.y_min) / (bounds.y_max - bounds.y_min);

        Pos2::new(
            plot_rect.left() + (x_pct as f32) * plot_rect.width(),
            plot_rect.bottom() - (y_pct as f32) * plot_rect.height(),
        )
    };

    // Render existing elements with preview opacity
    for fill in &output.fills {
        render_fill(painter, fill, &to_screen);
    }

    // Render horizontal lines
    for hline in &output.horizontal_lines {
        render_horizontal_line(painter, plot_rect, hline, bounds);
    }

    // Render vertical lines
    for vline in &output.vertical_lines {
        render_vertical_line(painter, plot_rect, vline, bounds);
    }

    for line in &output.lines {
        render_line(painter, plot_rect, line, bounds, &to_screen);
    }

    // Draw preview line from last handle to cursor
    if let Some(preview) = preview_point {
        if let Some(last_handle) = output.handles.last() {
            let start = to_screen(&last_handle.point);
            let end = to_screen(preview);

            let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 100, 150));
            render_dashed_line(painter, start, end, stroke, 5.0, 3.0);
        }
    }

    // Always render handles for preview
    for handle in &output.handles {
        render_handle(painter, handle, theme, &to_screen);
    }
}

/// Render a fill polygon
fn render_fill<X: AxisCoordinate>(
    painter: &Painter,
    fill: &DrawingFill<X>,
    to_screen: impl Fn(&ChartPoint<X>) -> Pos2,
) {
    if fill.points.len() < 3 {
        return;
    }

    let points: Vec<Pos2> = fill.points.iter().map(&to_screen).collect();
    let mut color = to_egui_color(fill.color);
    color = color.gamma_multiply(fill.opacity);

    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

/// Render a horizontal line that spans the full plot width
fn render_horizontal_line(
    painter: &Painter,
    plot_rect: Rect,
    hline: &HorizontalLineOutput,
    bounds: ViewBounds,
) {
    // Convert Y data coordinate to screen coordinate
    let y_pct = (hline.y - bounds.y_min) / (bounds.y_max - bounds.y_min);
    let screen_y = plot_rect.bottom() - (y_pct as f32) * plot_rect.height();

    // Check if line is within visible area
    if screen_y < plot_rect.top() || screen_y > plot_rect.bottom() {
        return;
    }

    let start = Pos2::new(plot_rect.left(), screen_y);
    let end = Pos2::new(plot_rect.right(), screen_y);
    let color = to_egui_color(hline.color);
    let stroke = Stroke::new(hline.width, color);

    match hline.style {
        LineStyle::Solid => {
            painter.line_segment([start, end], stroke);
        }
        LineStyle::Dashed { dash, gap } => {
            render_dashed_line(painter, start, end, stroke, dash, gap);
        }
        LineStyle::Dotted => {
            render_dashed_line(painter, start, end, stroke, 2.0, 4.0);
        }
    }
}

/// Render a vertical line that spans the full plot height
fn render_vertical_line<X: AxisCoordinate>(
    painter: &Painter,
    plot_rect: Rect,
    vline: &VerticalLineOutput<X>,
    bounds: ViewBounds,
) {
    // Convert X data coordinate to screen coordinate
    let x_pct = (vline.x.to_plot_value() - bounds.x_min) / (bounds.x_max - bounds.x_min);
    let screen_x = plot_rect.left() + (x_pct as f32) * plot_rect.width();

    // Check if line is within visible area
    if screen_x < plot_rect.left() || screen_x > plot_rect.right() {
        return;
    }

    let start = Pos2::new(screen_x, plot_rect.top());
    let end = Pos2::new(screen_x, plot_rect.bottom());
    let color = to_egui_color(vline.color);
    let stroke = Stroke::new(vline.width, color);

    match vline.style {
        LineStyle::Solid => {
            painter.line_segment([start, end], stroke);
        }
        LineStyle::Dashed { dash, gap } => {
            render_dashed_line(painter, start, end, stroke, dash, gap);
        }
        LineStyle::Dotted => {
            render_dashed_line(painter, start, end, stroke, 2.0, 4.0);
        }
    }
}

/// Render a line or polyline
fn render_line<X: AxisCoordinate>(
    painter: &Painter,
    plot_rect: Rect,
    line: &DrawingLine<X>,
    _bounds: ViewBounds,
    to_screen: impl Fn(&ChartPoint<X>) -> Pos2,
) {
    if line.points.len() < 2 {
        return;
    }

    let color = to_egui_color(line.color);
    let stroke = Stroke::new(line.width, color);

    // Handle extended lines
    let mut screen_points: Vec<Pos2> = line.points.iter().map(&to_screen).collect();

    if line.extend_left || line.extend_right {
        if screen_points.len() >= 2 {
            let p1 = screen_points[0];
            let p2 = screen_points[1];

            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;

            if line.extend_left && dx.abs() > 0.001 {
                // Extend to left edge
                let t = (plot_rect.left() - p1.x) / dx;
                let extended_y = p1.y + t * dy;
                screen_points.insert(0, Pos2::new(plot_rect.left(), extended_y));
            }

            if line.extend_right && dx.abs() > 0.001 {
                // Extend to right edge
                let t = (plot_rect.right() - p1.x) / dx;
                let extended_y = p1.y + t * dy;
                screen_points.push(Pos2::new(plot_rect.right(), extended_y));
            }
        }
    }

    // Draw the line based on style
    match line.style {
        LineStyle::Solid => {
            painter.add(egui::Shape::line(screen_points, stroke));
        }
        LineStyle::Dashed { dash, gap } => {
            for window in screen_points.windows(2) {
                render_dashed_line(painter, window[0], window[1], stroke, dash, gap);
            }
        }
        LineStyle::Dotted => {
            for window in screen_points.windows(2) {
                render_dashed_line(painter, window[0], window[1], stroke, 2.0, 4.0);
            }
        }
    }
}

/// Render a dashed line between two points
fn render_dashed_line(painter: &Painter, start: Pos2, end: Pos2, stroke: Stroke, dash: f32, gap: f32) {
    let dir = end - start;
    let len = dir.length();

    if len < 0.001 {
        return;
    }

    let dir_norm = dir / len;
    let mut pos = 0.0;
    let mut drawing = true;
    let mut current_start = start;

    while pos < len {
        let segment_len = if drawing { dash } else { gap };
        let next_pos = (pos + segment_len).min(len);

        if drawing {
            let segment_end = start + dir_norm * next_pos;
            painter.line_segment([current_start, segment_end], stroke);
        }

        pos = next_pos;
        current_start = start + dir_norm * pos;
        drawing = !drawing;
    }
}

/// Render a text label
fn render_label<X: AxisCoordinate>(
    painter: &Painter,
    plot_rect: Rect,
    label: &DrawingLabel<X>,
    _bounds: ViewBounds,
    to_screen: impl Fn(&ChartPoint<X>) -> Pos2,
) {
    let mut pos = to_screen(&label.position);

    // Handle special positions (for horizontal/vertical lines)
    if label.position.x.to_plot_value() == f64::MAX {
        pos.x = plot_rect.right() - 5.0;
    }
    if label.position.y == f64::MAX {
        pos.y = plot_rect.top() + 5.0;
    }

    // Clamp position to plot rect
    pos.x = pos.x.clamp(plot_rect.left(), plot_rect.right());
    pos.y = pos.y.clamp(plot_rect.top(), plot_rect.bottom());

    let text_color = to_egui_color(label.color);
    let font_id = egui::FontId::proportional(label.font_size);

    // Draw background if specified
    if let Some(bg_color) = label.background {
        let galley = painter.layout_no_wrap(label.text.clone(), font_id.clone(), text_color);
        let text_size = galley.size();

        let padding = label.padding;
        let bg_rect = match label.anchor {
            TextAnchor::TopLeft => {
                Rect::from_min_size(pos, text_size + Vec2::splat(padding * 2.0))
            }
            TextAnchor::TopCenter => Rect::from_min_size(
                pos - Vec2::new(text_size.x / 2.0 + padding, 0.0),
                text_size + Vec2::splat(padding * 2.0),
            ),
            TextAnchor::TopRight => Rect::from_min_size(
                pos - Vec2::new(text_size.x + padding * 2.0, 0.0),
                text_size + Vec2::splat(padding * 2.0),
            ),
            TextAnchor::MiddleLeft => Rect::from_min_size(
                pos - Vec2::new(0.0, text_size.y / 2.0 + padding),
                text_size + Vec2::splat(padding * 2.0),
            ),
            TextAnchor::Center => Rect::from_center_size(pos, text_size + Vec2::splat(padding * 2.0)),
            TextAnchor::MiddleRight => Rect::from_min_size(
                pos - Vec2::new(text_size.x + padding * 2.0, text_size.y / 2.0 + padding),
                text_size + Vec2::splat(padding * 2.0),
            ),
            TextAnchor::BottomLeft => Rect::from_min_size(
                pos - Vec2::new(0.0, text_size.y + padding * 2.0),
                text_size + Vec2::splat(padding * 2.0),
            ),
            TextAnchor::BottomCenter => Rect::from_min_size(
                pos - Vec2::new(text_size.x / 2.0 + padding, text_size.y + padding * 2.0),
                text_size + Vec2::splat(padding * 2.0),
            ),
            TextAnchor::BottomRight => Rect::from_min_size(
                pos - Vec2::new(text_size.x + padding * 2.0, text_size.y + padding * 2.0),
                text_size + Vec2::splat(padding * 2.0),
            ),
        };

        painter.rect_filled(bg_rect, 2.0, to_egui_color(bg_color));
    }

    // Convert anchor to egui alignment
    let align = match label.anchor {
        TextAnchor::TopLeft => egui::Align2::LEFT_TOP,
        TextAnchor::TopCenter => egui::Align2::CENTER_TOP,
        TextAnchor::TopRight => egui::Align2::RIGHT_TOP,
        TextAnchor::MiddleLeft => egui::Align2::LEFT_CENTER,
        TextAnchor::Center => egui::Align2::CENTER_CENTER,
        TextAnchor::MiddleRight => egui::Align2::RIGHT_CENTER,
        TextAnchor::BottomLeft => egui::Align2::LEFT_BOTTOM,
        TextAnchor::BottomCenter => egui::Align2::CENTER_BOTTOM,
        TextAnchor::BottomRight => egui::Align2::RIGHT_BOTTOM,
    };

    painter.text(pos, align, &label.text, font_id, text_color);
}

/// Render an interactive handle
fn render_handle<X: AxisCoordinate>(
    painter: &Painter,
    handle: &DrawingHandle<X>,
    theme: &DrawingRenderTheme,
    to_screen: impl Fn(&ChartPoint<X>) -> Pos2,
) {
    let pos = to_screen(&handle.point);
    let radius = theme.handle_radius;

    match handle.handle_type {
        HandleType::Circle => {
            painter.circle_filled(pos, radius, theme.handle_fill);
            painter.circle_stroke(pos, radius, Stroke::new(1.5, theme.handle_stroke));
        }
        HandleType::Square => {
            let rect = Rect::from_center_size(pos, Vec2::splat(radius * 2.0));
            painter.rect_filled(rect, 0.0, theme.handle_fill);
            // Draw stroke manually using line segments
            let stroke = Stroke::new(1.5, theme.handle_stroke);
            painter.line_segment([rect.left_top(), rect.right_top()], stroke);
            painter.line_segment([rect.right_top(), rect.right_bottom()], stroke);
            painter.line_segment([rect.right_bottom(), rect.left_bottom()], stroke);
            painter.line_segment([rect.left_bottom(), rect.left_top()], stroke);
        }
        HandleType::Diamond => {
            // Rotated square (diamond)
            let size = radius * 1.5;
            let points = vec![
                pos + Vec2::new(0.0, -size),
                pos + Vec2::new(size, 0.0),
                pos + Vec2::new(0.0, size),
                pos + Vec2::new(-size, 0.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                points.clone(),
                theme.handle_fill,
                Stroke::NONE,
            ));
            painter.add(egui::Shape::closed_line(
                points,
                Stroke::new(1.5, theme.handle_stroke),
            ));
        }
    }
}

/// Convert screen position to data coordinates
pub fn screen_to_data<X: AxisCoordinate>(
    screen_pos: Pos2,
    plot_rect: Rect,
    bounds: ViewBounds,
) -> ChartPoint<X> {
    let x_pct = ((screen_pos.x - plot_rect.left()) / plot_rect.width()) as f64;
    let y_pct = ((plot_rect.bottom() - screen_pos.y) / plot_rect.height()) as f64;

    let x = bounds.x_min + x_pct * (bounds.x_max - bounds.x_min);
    let y = bounds.y_min + y_pct * (bounds.y_max - bounds.y_min);

    ChartPoint::new(X::from_plot_value(x), y)
}

/// Handle drawing input events
pub fn handle_drawing_input<X: AxisCoordinate>(
    ui: &egui::Ui,
    response: &egui::Response,
    plot_rect: Rect,
    bounds: ViewBounds,
    store: &mut DrawingStore<X>,
    tool_state: &mut ToolState<X>,
) -> bool {
    let mut handled = false;

    // Update preview point on mouse move
    if let Some(hover_pos) = response.hover_pos() {
        if plot_rect.contains(hover_pos) {
            let data_point = screen_to_data(hover_pos, plot_rect, bounds);
            tool_state.handle_move(data_point);
        }
    }

    // Handle clicks
    if response.clicked() {
        if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
            if plot_rect.contains(pointer_pos) {
                let data_point = screen_to_data(pointer_pos, plot_rect, bounds);

                match tool_state.handle_click(data_point.clone()) {
                    trdelnik_drawings::ToolAction::SelectAt(point) => {
                        // Calculate tolerance based on view scale
                        let tolerance =
                            5.0 * (bounds.x_max - bounds.x_min) / plot_rect.width() as f64;
                        if let Some(id) = store.hit_test(&point, tolerance) {
                            store.select(Some(id));
                        } else {
                            store.select(None);
                        }
                    }
                    trdelnik_drawings::ToolAction::Complete(drawing) => {
                        store.add_boxed(drawing);
                        handled = true;
                    }
                    trdelnik_drawings::ToolAction::Continue => {
                        handled = true;
                    }
                    _ => {}
                }
            }
        }
    }

    // Handle escape
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        tool_state.handle_escape();
        store.deselect();
        handled = true;
    }

    // Handle delete
    if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
        if let Some(selected) = store.selected() {
            store.remove(selected);
            handled = true;
        }
    }

    handled
}
