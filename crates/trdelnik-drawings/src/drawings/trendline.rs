//! Trend line drawing
//!
//! A line between two points, optionally extended.

use crate::{
    point_to_segment_distance, Anchor, AnchorPoint, ChartPoint, Drawing, DrawingHandle,
    DrawingLine, DrawingOutput, DrawingStyle,
};
use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;
use uuid::Uuid;

/// A trend line between two points
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrendLine<X: AxisCoordinate> {
    id: Uuid,
    /// The two anchor points
    points: Vec<AnchorPoint<X>>,
    /// Extend the line to the left
    pub extend_left: bool,
    /// Extend the line to the right
    pub extend_right: bool,
}

impl<X: AxisCoordinate> TrendLine<X> {
    /// Create a new empty trend line
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            points: Vec::with_capacity(2),
            extend_left: false,
            extend_right: false,
        }
    }

    /// Create a trend line from two points
    pub fn from_points(start: ChartPoint<X>, end: ChartPoint<X>) -> Self {
        Self {
            id: Uuid::new_v4(),
            points: vec![
                AnchorPoint {
                    point: start,
                    anchor: Anchor::DataPoint,
                },
                AnchorPoint {
                    point: end,
                    anchor: Anchor::DataPoint,
                },
            ],
            extend_left: false,
            extend_right: false,
        }
    }

    /// Create a ray (extends only to the right)
    pub fn ray() -> Self {
        Self {
            id: Uuid::new_v4(),
            points: Vec::with_capacity(2),
            extend_left: false,
            extend_right: true,
        }
    }

    /// Create an extended line (extends in both directions)
    pub fn extended() -> Self {
        Self {
            id: Uuid::new_v4(),
            points: Vec::with_capacity(2),
            extend_left: true,
            extend_right: true,
        }
    }

    /// Set whether to extend left
    pub fn with_extend_left(mut self, extend: bool) -> Self {
        self.extend_left = extend;
        self
    }

    /// Set whether to extend right
    pub fn with_extend_right(mut self, extend: bool) -> Self {
        self.extend_right = extend;
        self
    }
}

impl<X: AxisCoordinate> Default for TrendLine<X> {
    fn default() -> Self {
        Self::new()
    }
}

impl<X: AxisCoordinate> Drawing<X> for TrendLine<X> {
    fn type_id(&self) -> &'static str {
        if self.extend_left && self.extend_right {
            "extended_line"
        } else if self.extend_right {
            "ray"
        } else {
            "trendline"
        }
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn display_name(&self) -> &str {
        if self.extend_left && self.extend_right {
            "Extended Line"
        } else if self.extend_right {
            "Ray"
        } else {
            "Trend Line"
        }
    }

    fn required_points(&self) -> usize {
        2
    }

    fn anchor_points(&self) -> &[AnchorPoint<X>] {
        &self.points
    }

    fn anchor_points_mut(&mut self) -> &mut [AnchorPoint<X>] {
        &mut self.points
    }

    fn set_anchor_point(&mut self, index: usize, point: AnchorPoint<X>) {
        if index < self.points.len() {
            self.points[index] = point;
        } else if index == self.points.len() {
            self.points.push(point);
        }
    }

    fn compute(&self, style: &DrawingStyle) -> DrawingOutput<X> {
        let mut output = DrawingOutput::new();

        if self.points.len() < 2 {
            // Not enough points yet - could draw a preview point
            if let Some(p) = self.points.first() {
                let handle = DrawingHandle::new(p.point.clone(), 0);
                output.add_handle(handle);
            }
            return output;
        }

        let p1 = &self.points[0].point;
        let p2 = &self.points[1].point;

        // Create the line
        let mut line = DrawingLine::new(p1.clone(), p2.clone(), style.color)
            .with_width(style.line_width)
            .with_style(style.line_style);

        if self.extend_left {
            line = line.extend_left();
        }
        if self.extend_right {
            line = line.extend_right();
        }

        output.add_line(line);

        // Add handles at both endpoints
        output.add_handle(DrawingHandle::new(p1.clone(), 0));
        output.add_handle(DrawingHandle::new(p2.clone(), 1));

        output
    }

    fn hit_test(&self, point: &ChartPoint<X>, tolerance: f64) -> bool {
        if self.points.len() < 2 {
            return false;
        }

        let p1 = &self.points[0].point;
        let p2 = &self.points[1].point;

        // For extended lines, we need to use infinite line distance
        // For regular lines, use segment distance
        if self.extend_left || self.extend_right {
            // Use line distance (not segment)
            crate::point_to_line_distance(point, p1, p2) <= tolerance
        } else {
            point_to_segment_distance(point, p1, p2) <= tolerance
        }
    }

    fn clone_box(&self) -> Box<dyn Drawing<X>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::Index;

    #[test]
    fn test_trendline_creation() {
        let line: TrendLine<Index> = TrendLine::new();
        assert!(!line.is_complete());
        assert_eq!(line.required_points(), 2);
    }

    #[test]
    fn test_trendline_from_points() {
        let line = TrendLine::from_points(
            ChartPoint::new(Index(0), 100.0),
            ChartPoint::new(Index(10), 110.0),
        );
        assert!(line.is_complete());
        assert_eq!(line.anchor_points().len(), 2);
    }

    #[test]
    fn test_trendline_hit_test() {
        let line = TrendLine::from_points(
            ChartPoint::new(Index(0), 0.0),
            ChartPoint::new(Index(10), 10.0),
        );

        // Point on the line
        let point_on = ChartPoint::new(Index(5), 5.0);
        assert!(line.hit_test(&point_on, 1.0));

        // Point off the line
        let point_off = ChartPoint::new(Index(5), 10.0);
        assert!(!line.hit_test(&point_off, 1.0));
    }

    use crate::DrawingStyle;

    fn cp(x: usize, y: f64) -> ChartPoint<Index> {
        ChartPoint::new(Index(x), y)
    }

    #[test]
    fn test_default_is_new() {
        let a: TrendLine<Index> = TrendLine::default();
        assert!(!a.extend_left);
        assert!(!a.extend_right);
        assert!(a.anchor_points().is_empty());
    }

    #[test]
    fn test_ray_only_extends_right() {
        let r: TrendLine<Index> = TrendLine::ray();
        assert!(!r.extend_left);
        assert!(r.extend_right);
    }

    #[test]
    fn test_extended_extends_both() {
        let e: TrendLine<Index> = TrendLine::extended();
        assert!(e.extend_left);
        assert!(e.extend_right);
    }

    #[test]
    fn test_with_extend_builders() {
        let a: TrendLine<Index> = TrendLine::new()
            .with_extend_left(true)
            .with_extend_right(true);
        assert!(a.extend_left);
        assert!(a.extend_right);
        let b: TrendLine<Index> = TrendLine::new().with_extend_right(true);
        assert!(!b.extend_left);
        assert!(b.extend_right);
    }

    #[test]
    fn test_type_id_for_each_variant() {
        let plain: TrendLine<Index> = TrendLine::new();
        assert_eq!(plain.type_id(), "trendline");
        assert_eq!(plain.display_name(), "Trend Line");

        let ray: TrendLine<Index> = TrendLine::ray();
        assert_eq!(ray.type_id(), "ray");
        assert_eq!(ray.display_name(), "Ray");

        let ext: TrendLine<Index> = TrendLine::extended();
        assert_eq!(ext.type_id(), "extended_line");
        assert_eq!(ext.display_name(), "Extended Line");
    }

    #[test]
    fn test_required_points_is_two() {
        let t: TrendLine<Index> = TrendLine::new();
        assert_eq!(t.required_points(), 2);
    }

    #[test]
    fn test_id_unique() {
        let a: TrendLine<Index> = TrendLine::new();
        let b: TrendLine<Index> = TrendLine::new();
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn test_set_anchor_point_appends_then_overwrites() {
        let mut t: TrendLine<Index> = TrendLine::new();
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 1.0));
        t.set_anchor_point(1, AnchorPoint::new(Index(10), 5.0));
        assert_eq!(t.anchor_points().len(), 2);
        // Overwrite index 0
        t.set_anchor_point(0, AnchorPoint::new(Index(7), 7.7));
        assert_eq!(t.anchor_points()[0].point.x, Index(7));
    }

    #[test]
    fn test_set_anchor_point_skip_indices_does_nothing() {
        let mut t: TrendLine<Index> = TrendLine::new();
        // index 5 with empty points: neither overwrites nor appends
        t.set_anchor_point(5, AnchorPoint::new(Index(0), 1.0));
        assert!(t.anchor_points().is_empty());
    }

    #[test]
    fn test_anchor_points_mut_can_modify() {
        let mut t = TrendLine::from_points(cp(0, 0.0), cp(10, 10.0));
        t.anchor_points_mut()[0].point.y = 99.0;
        assert!((t.anchor_points()[0].point.y - 99.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_with_one_point_emits_only_handle() {
        let mut t: TrendLine<Index> = TrendLine::new();
        t.set_anchor_point(0, AnchorPoint::new(Index(3), 5.0));
        let out = t.compute(&DrawingStyle::default());
        assert!(out.lines.is_empty());
        assert_eq!(out.handles.len(), 1);
    }

    #[test]
    fn test_compute_zero_points_emits_nothing() {
        let t: TrendLine<Index> = TrendLine::new();
        let out = t.compute(&DrawingStyle::default());
        assert!(out.lines.is_empty());
        assert!(out.handles.is_empty());
    }

    #[test]
    fn test_compute_two_points_emits_line_and_two_handles() {
        let t = TrendLine::from_points(cp(0, 0.0), cp(10, 10.0));
        let out = t.compute(&DrawingStyle::default());
        assert_eq!(out.lines.len(), 1);
        assert_eq!(out.handles.len(), 2);
        // Plain trend line: no extension
        assert!(!out.lines[0].extend_left);
        assert!(!out.lines[0].extend_right);
    }

    #[test]
    fn test_compute_extended_propagates_extend_flags() {
        let t = TrendLine::extended().with_extend_left(true).with_extend_right(true);
        let mut t = t;
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 0.0));
        t.set_anchor_point(1, AnchorPoint::new(Index(10), 10.0));
        let out = t.compute(&DrawingStyle::default());
        assert!(out.lines[0].extend_left);
        assert!(out.lines[0].extend_right);
    }

    #[test]
    fn test_compute_ray_propagates_only_extend_right() {
        let mut t: TrendLine<Index> = TrendLine::ray();
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 0.0));
        t.set_anchor_point(1, AnchorPoint::new(Index(10), 10.0));
        let out = t.compute(&DrawingStyle::default());
        assert!(!out.lines[0].extend_left);
        assert!(out.lines[0].extend_right);
    }

    #[test]
    fn test_hit_test_one_point_returns_false() {
        let mut t: TrendLine<Index> = TrendLine::new();
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 0.0));
        assert!(!t.hit_test(&cp(0, 0.0), 1.0));
    }

    #[test]
    fn test_hit_test_segment_ignores_outside_endpoints() {
        let t = TrendLine::from_points(cp(0, 0.0), cp(10, 10.0));
        // x=20 is outside the [0,10] segment; even though the infinite line
        // would pass through (20,20), the segment hit test rejects it.
        let far_off_segment = cp(20, 5.0);
        assert!(!t.hit_test(&far_off_segment, 1.0));
    }

    #[test]
    fn test_hit_test_extended_line_uses_infinite_distance() {
        let mut t = TrendLine::extended();
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 0.0));
        t.set_anchor_point(1, AnchorPoint::new(Index(10), 10.0));
        // Outside segment but on the infinite line through (0,0) → (10,10):
        // (20, 20) is on that line. Tolerance 0.5 catches it.
        assert!(t.hit_test(&cp(20, 20.0), 0.5));
    }

    #[test]
    fn test_clone_box_preserves_extension_flags() {
        let t = TrendLine::extended();
        let mut t = t;
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 0.0));
        t.set_anchor_point(1, AnchorPoint::new(Index(10), 10.0));
        let boxed: Box<dyn Drawing<Index>> = t.clone_box();
        assert_eq!(boxed.type_id(), "extended_line");
    }
}
