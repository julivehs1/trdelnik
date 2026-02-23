//! Coordinate types for drawings
//!
//! Provides types for positioning drawings on the chart.

use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;

/// A point on the chart in data coordinates
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChartPoint<X: AxisCoordinate> {
    /// X coordinate (time/index)
    pub x: X,
    /// Y coordinate (price/value)
    pub y: f64,
}

impl<X: AxisCoordinate> ChartPoint<X> {
    /// Create a new chart point
    pub fn new(x: X, y: f64) -> Self {
        Self { x, y }
    }

    /// Convert to f64 tuple for calculations
    pub fn to_f64(&self) -> (f64, f64) {
        (self.x.to_plot_value(), self.y)
    }
}

/// How a point is anchored to the chart
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Anchor {
    /// Fixed at data coordinates (moves with zoom/pan)
    DataPoint,
    /// Fixed relative to a candle index (e.g., "3rd candle from right")
    CandleRelative {
        /// Offset from the end (0 = last candle, 1 = second to last, etc.)
        offset_from_end: i32,
    },
    /// Percentage position in the visible viewport
    ViewportRelative {
        /// X position as percentage (0.0 = left, 1.0 = right)
        x_pct: f64,
        /// Y position as percentage (0.0 = bottom, 1.0 = top)
        y_pct: f64,
    },
}

impl Default for Anchor {
    fn default() -> Self {
        Self::DataPoint
    }
}

/// A point with its anchor type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnchorPoint<X: AxisCoordinate> {
    /// The position in data coordinates
    pub point: ChartPoint<X>,
    /// How this point is anchored
    pub anchor: Anchor,
}

impl<X: AxisCoordinate> AnchorPoint<X> {
    /// Create a new anchor point with default (DataPoint) anchor
    pub fn new(x: X, y: f64) -> Self {
        Self {
            point: ChartPoint::new(x, y),
            anchor: Anchor::DataPoint,
        }
    }

    /// Create with a specific anchor type
    pub fn with_anchor(x: X, y: f64, anchor: Anchor) -> Self {
        Self {
            point: ChartPoint::new(x, y),
            anchor,
        }
    }
}

/// Calculate distance from a point to a line segment
pub fn point_to_segment_distance<X: AxisCoordinate>(
    point: &ChartPoint<X>,
    line_start: &ChartPoint<X>,
    line_end: &ChartPoint<X>,
) -> f64 {
    let (px, py) = point.to_f64();
    let (x1, y1) = line_start.to_f64();
    let (x2, y2) = line_end.to_f64();

    let dx = x2 - x1;
    let dy = y2 - y1;

    if dx == 0.0 && dy == 0.0 {
        // Line is a point
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }

    // Parameter t for the closest point on the line
    let t = ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy);
    let t = t.clamp(0.0, 1.0);

    // Closest point on segment
    let closest_x = x1 + t * dx;
    let closest_y = y1 + t * dy;

    ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt()
}

/// Calculate distance from a point to an infinite line
pub fn point_to_line_distance<X: AxisCoordinate>(
    point: &ChartPoint<X>,
    line_start: &ChartPoint<X>,
    line_end: &ChartPoint<X>,
) -> f64 {
    let (px, py) = point.to_f64();
    let (x1, y1) = line_start.to_f64();
    let (x2, y2) = line_end.to_f64();

    let dx = x2 - x1;
    let dy = y2 - y1;

    if dx == 0.0 && dy == 0.0 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }

    // Distance = |cross product| / |line vector|
    let cross = (px - x1) * dy - (py - y1) * dx;
    cross.abs() / (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::Index;

    #[test]
    fn test_point_to_segment() {
        let point = ChartPoint::new(Index(5), 5.0);
        let start = ChartPoint::new(Index(0), 0.0);
        let end = ChartPoint::new(Index(10), 0.0);

        // Point is above the middle of the segment
        let dist = point_to_segment_distance(&point, &start, &end);
        assert!((dist - 5.0).abs() < 0.001);
    }
}
