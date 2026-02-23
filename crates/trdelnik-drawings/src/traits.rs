//! Drawing trait definition
//!
//! The core trait that all drawing types must implement.

use crate::{AnchorPoint, ChartPoint, DrawingOutput, DrawingStyle};
use trdelnik_core::AxisCoordinate;
use uuid::Uuid;

/// The core trait for all drawing tools
///
/// This trait defines the interface that all drawings must implement.
/// It follows a similar pattern to the `Indicator` trait in trdelnik-indicators.
///
/// # Example
///
/// ```rust,ignore
/// impl<X: AxisCoordinate> Drawing<X> for TrendLine<X> {
///     fn type_id(&self) -> &'static str { "trendline" }
///     fn id(&self) -> Uuid { self.id }
///     fn display_name(&self) -> &str { "Trend Line" }
///     fn required_points(&self) -> usize { 2 }
///
///     fn anchor_points(&self) -> &[AnchorPoint<X>] { &self.points }
///
///     fn set_anchor_point(&mut self, index: usize, point: AnchorPoint<X>) {
///         // ...
///     }
///
///     fn compute(&self, style: &DrawingStyle) -> DrawingOutput<X> {
///         // Generate lines, fills, labels for rendering
///     }
///
///     fn hit_test(&self, point: &ChartPoint<X>, tolerance: f64) -> bool {
///         // Check if point is on the drawing
///     }
/// }
/// ```
pub trait Drawing<X: AxisCoordinate>: Send + Sync {
    /// Unique identifier for this drawing type (e.g., "trendline", "fib_retracement")
    fn type_id(&self) -> &'static str;

    /// Unique instance ID (generated when created)
    fn id(&self) -> Uuid;

    /// Human-readable name for the UI
    fn display_name(&self) -> &str;

    /// Number of anchor points required to complete this drawing
    fn required_points(&self) -> usize;

    /// Get the current anchor points
    fn anchor_points(&self) -> &[AnchorPoint<X>];

    /// Get mutable access to anchor points
    fn anchor_points_mut(&mut self) -> &mut [AnchorPoint<X>];

    /// Set or add an anchor point at the given index
    fn set_anchor_point(&mut self, index: usize, point: AnchorPoint<X>);

    /// Check if the drawing is complete (all required points set)
    fn is_complete(&self) -> bool {
        self.anchor_points().len() >= self.required_points()
    }

    /// Compute the render output for this drawing
    ///
    /// This is the main rendering method - it produces lines, fills, and labels
    /// that will be drawn on the chart.
    fn compute(&self, style: &DrawingStyle) -> DrawingOutput<X>;

    /// Check if a point is on this drawing (for selection)
    ///
    /// # Arguments
    /// * `point` - The point to test (in data coordinates)
    /// * `tolerance` - The hit tolerance (in data coordinate units)
    fn hit_test(&self, point: &ChartPoint<X>, tolerance: f64) -> bool;

    /// Get the bounding box of this drawing (for culling)
    ///
    /// Returns `None` if the drawing has no bounds (e.g., horizontal line)
    fn bounds(&self) -> Option<(ChartPoint<X>, ChartPoint<X>)> {
        let points = self.anchor_points();
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].point.x;
        let mut max_x = points[0].point.x;
        let mut min_y = points[0].point.y;
        let mut max_y = points[0].point.y;

        for p in points.iter().skip(1) {
            if p.point.x < min_x {
                min_x = p.point.x;
            }
            if p.point.x > max_x {
                max_x = p.point.x;
            }
            if p.point.y < min_y {
                min_y = p.point.y;
            }
            if p.point.y > max_y {
                max_y = p.point.y;
            }
        }

        Some((
            ChartPoint::new(min_x, min_y),
            ChartPoint::new(max_x, max_y),
        ))
    }

    /// Clone this drawing into a boxed trait object
    fn clone_box(&self) -> Box<dyn Drawing<X>>;
}

/// Extension trait for cloning boxed drawings
impl<X: AxisCoordinate> Clone for Box<dyn Drawing<X>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
