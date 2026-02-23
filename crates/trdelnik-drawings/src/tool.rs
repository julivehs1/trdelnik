//! Drawing tool state machine
//!
//! Manages the active drawing tool and the state of drawings being created.

use crate::{
    drawings::{HorizontalLine, PositionTool, TrendLine, VerticalLine},
    Anchor, AnchorPoint, ChartPoint, Drawing,
};
use trdelnik_core::AxisCoordinate;
use uuid::Uuid;

/// Available drawing tools
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DrawingTool {
    /// No tool active (selection/pointer mode)
    #[default]
    None,
    /// Crosshair cursor (no drawing)
    Crosshair,
    /// Trend line (2 points)
    TrendLine,
    /// Ray (2 points, extends right)
    Ray,
    /// Extended line (2 points, extends both ways)
    ExtendedLine,
    /// Horizontal line (1 point)
    HorizontalLine,
    /// Vertical line (1 point)
    VerticalLine,
    /// Position/Measurement tool (2 points)
    Position,
}

impl DrawingTool {
    /// Get the display name for this tool
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::None => "Pointer",
            Self::Crosshair => "Crosshair",
            Self::TrendLine => "Trend Line",
            Self::Ray => "Ray",
            Self::ExtendedLine => "Extended Line",
            Self::HorizontalLine => "Horizontal Line",
            Self::VerticalLine => "Vertical Line",
            Self::Position => "Position",
        }
    }

    /// Get the number of points required
    pub fn required_points(&self) -> usize {
        match self {
            Self::None | Self::Crosshair => 0,
            Self::HorizontalLine | Self::VerticalLine => 1,
            Self::TrendLine | Self::Ray | Self::ExtendedLine | Self::Position => 2,
        }
    }

    /// Check if this is a drawing tool (not a selection tool)
    pub fn is_drawing_tool(&self) -> bool {
        !matches!(self, Self::None | Self::Crosshair)
    }
}

/// Action returned by the tool state machine
pub enum ToolAction<X: AxisCoordinate> {
    /// No action
    None,
    /// Continue building the current drawing
    Continue,
    /// Try to select a drawing at this point
    SelectAt(ChartPoint<X>),
    /// A drawing is complete and ready to be added
    Complete(Box<dyn Drawing<X>>),
    /// Cancel the current drawing
    Cancel,
}

impl<X: AxisCoordinate> std::fmt::Debug for ToolAction<X> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Continue => write!(f, "Continue"),
            Self::SelectAt(point) => write!(f, "SelectAt({:?})", point.to_f64()),
            Self::Complete(drawing) => write!(f, "Complete({})", drawing.type_id()),
            Self::Cancel => write!(f, "Cancel"),
        }
    }
}

/// Tool state machine
///
/// Manages the state of the active tool and any drawing being created.
#[derive(Clone)]
pub struct ToolState<X: AxisCoordinate> {
    /// Currently active tool
    tool: DrawingTool,
    /// Drawing currently being created
    pending: Option<Box<dyn Drawing<X>>>,
    /// Current preview point (mouse position)
    preview_point: Option<ChartPoint<X>>,
    /// Index of anchor being dragged (if editing)
    dragging_anchor: Option<(Uuid, usize)>,
}

impl<X: AxisCoordinate> Default for ToolState<X> {
    fn default() -> Self {
        Self::new()
    }
}

impl<X: AxisCoordinate> ToolState<X> {
    /// Create a new tool state
    pub fn new() -> Self {
        Self {
            tool: DrawingTool::None,
            pending: None,
            preview_point: None,
            dragging_anchor: None,
        }
    }

    /// Get the current tool
    pub fn tool(&self) -> DrawingTool {
        self.tool
    }

    /// Set the active tool
    pub fn set_tool(&mut self, tool: DrawingTool) {
        // Cancel any pending drawing
        self.pending = None;
        self.preview_point = None;
        self.tool = tool;
    }

    /// Get the pending drawing (if any)
    pub fn pending(&self) -> Option<&dyn Drawing<X>> {
        self.pending.as_ref().map(|d| d.as_ref())
    }

    /// Get the preview point
    pub fn preview_point(&self) -> Option<&ChartPoint<X>> {
        self.preview_point.as_ref()
    }

    /// Check if there's an active drawing operation
    pub fn is_drawing(&self) -> bool {
        self.pending.is_some()
    }

    /// Check if we're dragging an anchor
    pub fn is_dragging(&self) -> bool {
        self.dragging_anchor.is_some()
    }

    /// Get the dragging anchor info
    pub fn dragging_anchor(&self) -> Option<(Uuid, usize)> {
        self.dragging_anchor
    }

    /// Start dragging an anchor
    pub fn start_drag(&mut self, drawing_id: Uuid, anchor_index: usize) {
        self.dragging_anchor = Some((drawing_id, anchor_index));
    }

    /// Stop dragging
    pub fn stop_drag(&mut self) {
        self.dragging_anchor = None;
    }

    /// Handle mouse move
    pub fn handle_move(&mut self, point: ChartPoint<X>) {
        self.preview_point = Some(point);
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, point: ChartPoint<X>) -> ToolAction<X> {
        match self.tool {
            DrawingTool::None => {
                // Selection mode - try to select at point
                ToolAction::SelectAt(point)
            }

            DrawingTool::Crosshair => {
                // Crosshair doesn't do anything on click
                ToolAction::None
            }

            DrawingTool::HorizontalLine => {
                // Single-click tool - create and complete immediately
                let drawing = HorizontalLine::new(point.y);
                ToolAction::Complete(Box::new(drawing))
            }

            DrawingTool::VerticalLine => {
                let drawing = VerticalLine::new(point.x);
                ToolAction::Complete(Box::new(drawing))
            }

            DrawingTool::TrendLine => self.handle_two_point_click(point, || TrendLine::new()),

            DrawingTool::Ray => self.handle_two_point_click(point, || TrendLine::ray()),

            DrawingTool::ExtendedLine => {
                self.handle_two_point_click(point, || TrendLine::extended())
            }

            DrawingTool::Position => self.handle_two_point_click(point, || PositionTool::new()),
        }
    }

    /// Handle a two-point tool click
    fn handle_two_point_click<D: Drawing<X> + 'static>(
        &mut self,
        point: ChartPoint<X>,
        create_drawing: impl FnOnce() -> D,
    ) -> ToolAction<X> {
        if let Some(ref mut pending) = self.pending {
            // Second click - complete the drawing
            let anchor = AnchorPoint {
                point,
                anchor: Anchor::DataPoint,
            };
            pending.set_anchor_point(1, anchor);

            let completed = self.pending.take().unwrap();
            ToolAction::Complete(completed)
        } else {
            // First click - start the drawing
            let mut drawing = create_drawing();
            let anchor = AnchorPoint {
                point,
                anchor: Anchor::DataPoint,
            };
            drawing.set_anchor_point(0, anchor);
            self.pending = Some(Box::new(drawing));
            ToolAction::Continue
        }
    }

    /// Handle escape key
    pub fn handle_escape(&mut self) -> ToolAction<X> {
        if self.pending.is_some() {
            self.pending = None;
            self.preview_point = None;
            ToolAction::Cancel
        } else if self.tool != DrawingTool::None {
            self.tool = DrawingTool::None;
            ToolAction::Cancel
        } else {
            ToolAction::None
        }
    }

    /// Cancel the current drawing operation
    pub fn cancel(&mut self) {
        self.pending = None;
        self.preview_point = None;
        self.dragging_anchor = None;
    }

    /// Reset to pointer mode
    pub fn reset(&mut self) {
        self.tool = DrawingTool::None;
        self.pending = None;
        self.preview_point = None;
        self.dragging_anchor = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::Index;

    #[test]
    fn test_tool_state() {
        let mut state: ToolState<Index> = ToolState::new();
        assert_eq!(state.tool(), DrawingTool::None);
        assert!(!state.is_drawing());

        state.set_tool(DrawingTool::TrendLine);
        assert_eq!(state.tool(), DrawingTool::TrendLine);
    }

    #[test]
    fn test_horizontal_line_click() {
        let mut state: ToolState<Index> = ToolState::new();
        state.set_tool(DrawingTool::HorizontalLine);

        let action = state.handle_click(ChartPoint::new(Index(5), 100.0));

        match action {
            ToolAction::Complete(drawing) => {
                assert_eq!(drawing.type_id(), "hline");
            }
            _ => panic!("Expected Complete action"),
        }
    }

    #[test]
    fn test_trendline_two_clicks() {
        let mut state: ToolState<Index> = ToolState::new();
        state.set_tool(DrawingTool::TrendLine);

        // First click
        let action1 = state.handle_click(ChartPoint::new(Index(0), 100.0));
        assert!(matches!(action1, ToolAction::Continue));
        assert!(state.is_drawing());

        // Second click
        let action2 = state.handle_click(ChartPoint::new(Index(10), 110.0));
        match action2 {
            ToolAction::Complete(drawing) => {
                assert_eq!(drawing.type_id(), "trendline");
                assert!(drawing.is_complete());
            }
            _ => panic!("Expected Complete action"),
        }

        assert!(!state.is_drawing());
    }

    #[test]
    fn test_escape_cancels() {
        let mut state: ToolState<Index> = ToolState::new();
        state.set_tool(DrawingTool::TrendLine);

        // First click
        state.handle_click(ChartPoint::new(Index(0), 100.0));
        assert!(state.is_drawing());

        // Escape
        let action = state.handle_escape();
        assert!(matches!(action, ToolAction::Cancel));
        assert!(!state.is_drawing());
    }
}
