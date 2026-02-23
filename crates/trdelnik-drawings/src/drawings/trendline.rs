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
}
