//! Position Tool (Gain/Loss measurement)
//!
//! Shows the price difference, percentage change, and optionally
//! profit/loss between two price points.

use crate::{
    point_to_segment_distance, Anchor, AnchorPoint, ChartPoint, Drawing, DrawingFill,
    DrawingHandle, DrawingLabel, DrawingLine, DrawingOutput, DrawingStyle, TextAnchor,
};
use serde::{Deserialize, Serialize};
use trdelnik_core::{AxisCoordinate, Color};
use uuid::Uuid;

/// Position direction (for coloring)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionDirection {
    /// Long position (profit when price goes up)
    Long,
    /// Short position (profit when price goes down)
    Short,
}

impl Default for PositionDirection {
    fn default() -> Self {
        Self::Long
    }
}

/// Position measurement tool
///
/// Shows a box between two points with:
/// - Price difference
/// - Percentage change
/// - Pips (for forex)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionTool<X: AxisCoordinate> {
    id: Uuid,
    /// The two anchor points (entry and exit)
    points: Vec<AnchorPoint<X>>,
    /// Position direction
    pub direction: PositionDirection,
    /// Optional position size (for P&L calculation)
    pub quantity: Option<f64>,
    /// Pip multiplier (10000 for forex, 1 for crypto/stocks)
    pub pip_multiplier: f64,
}

impl<X: AxisCoordinate> PositionTool<X> {
    /// Create a new position tool
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            points: Vec::with_capacity(2),
            direction: PositionDirection::Long,
            quantity: None,
            pip_multiplier: 1.0, // Default for stocks/crypto
        }
    }

    /// Create for forex (pips)
    pub fn forex() -> Self {
        Self {
            id: Uuid::new_v4(),
            points: Vec::with_capacity(2),
            direction: PositionDirection::Long,
            quantity: None,
            pip_multiplier: 10000.0,
        }
    }

    /// Create from two points
    pub fn from_points(entry: ChartPoint<X>, exit: ChartPoint<X>) -> Self {
        let direction = if exit.y >= entry.y {
            PositionDirection::Long
        } else {
            PositionDirection::Short
        };

        Self {
            id: Uuid::new_v4(),
            points: vec![
                AnchorPoint {
                    point: entry,
                    anchor: Anchor::DataPoint,
                },
                AnchorPoint {
                    point: exit,
                    anchor: Anchor::DataPoint,
                },
            ],
            direction,
            quantity: None,
            pip_multiplier: 1.0,
        }
    }

    /// Set the direction
    pub fn with_direction(mut self, direction: PositionDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the quantity
    pub fn with_quantity(mut self, quantity: f64) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Calculate the price difference
    fn price_diff(&self) -> Option<f64> {
        if self.points.len() < 2 {
            return None;
        }
        Some(self.points[1].point.y - self.points[0].point.y)
    }

    /// Calculate the percentage change
    fn percent_change(&self) -> Option<f64> {
        if self.points.len() < 2 {
            return None;
        }
        let entry = self.points[0].point.y;
        if entry == 0.0 {
            return None;
        }
        Some(((self.points[1].point.y - entry) / entry) * 100.0)
    }

    /// Check if the position is profitable
    fn is_profitable(&self) -> bool {
        let diff = self.price_diff().unwrap_or(0.0);
        match self.direction {
            PositionDirection::Long => diff > 0.0,
            PositionDirection::Short => diff < 0.0,
        }
    }
}

impl<X: AxisCoordinate> Default for PositionTool<X> {
    fn default() -> Self {
        Self::new()
    }
}

impl<X: AxisCoordinate> Drawing<X> for PositionTool<X> {
    fn type_id(&self) -> &'static str {
        "position"
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn display_name(&self) -> &str {
        "Position"
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

        // Update direction based on points
        if self.points.len() >= 2 {
            let diff = self.points[1].point.y - self.points[0].point.y;
            self.direction = if diff >= 0.0 {
                PositionDirection::Long
            } else {
                PositionDirection::Short
            };
        }
    }

    fn compute(&self, style: &DrawingStyle) -> DrawingOutput<X> {
        let mut output = DrawingOutput::new();

        if self.points.len() < 2 {
            if let Some(p) = self.points.first() {
                output.add_handle(DrawingHandle::new(p.point.clone(), 0));
            }
            return output;
        }

        let entry = &self.points[0].point;
        let exit = &self.points[1].point;

        // Colors based on profit/loss
        let is_profit = self.is_profitable();
        let main_color = if is_profit {
            Color::from_hex("#26A69A").unwrap_or(Color::rgb(38, 166, 154)) // Green
        } else {
            Color::from_hex("#EF5350").unwrap_or(Color::rgb(239, 83, 80)) // Red
        };

        // Draw the filled rectangle
        let fill = DrawingFill::rect(entry.clone(), exit.clone(), main_color, 0.15);
        output.add_fill(fill);

        // Draw the border
        let border_points = vec![
            entry.clone(),
            ChartPoint::new(exit.x, entry.y),
            exit.clone(),
            ChartPoint::new(entry.x, exit.y),
            entry.clone(), // Close the rectangle
        ];
        let border = DrawingLine::polyline(border_points, main_color).with_width(1.0);
        output.add_line(border);

        // Draw the vertical line in the middle with arrow
        let mid_x_f64 = (entry.x.to_plot_value() + exit.x.to_plot_value()) / 2.0;
        let mid_x = X::from_plot_value(mid_x_f64);

        let vertical_line = DrawingLine::new(
            ChartPoint::new(mid_x, entry.y),
            ChartPoint::new(mid_x, exit.y),
            main_color,
        )
        .with_width(1.0);
        output.add_line(vertical_line);

        // Draw horizontal connector at entry
        let connector = DrawingLine::new(
            ChartPoint::new(entry.x, entry.y),
            ChartPoint::new(exit.x, entry.y),
            main_color,
        )
        .with_width(1.0);
        output.add_line(connector);

        // Calculate values for label
        let price_diff = self.price_diff().unwrap_or(0.0);
        let pct_change = self.percent_change().unwrap_or(0.0);
        let pips = (price_diff * self.pip_multiplier).round();

        // Build label text
        let label_text = if self.pip_multiplier > 1.0 {
            format!("{:.0}  ({:.2}%)  {:.2}", pips.abs(), pct_change.abs(), price_diff.abs())
        } else {
            format!("{:.2}  ({:.2}%)", price_diff.abs(), pct_change.abs())
        };

        // Position label above/below the box based on direction
        let label_y = if exit.y > entry.y {
            exit.y // Above for profit
        } else {
            entry.y // Above the box
        };

        let label = DrawingLabel::new(
            ChartPoint::new(mid_x, label_y),
            label_text,
            Color::white(),
        )
        .with_font_size(style.font_size)
        .with_background(main_color)
        .with_anchor(TextAnchor::BottomCenter);
        output.add_label(label);

        // Handles at corners
        output.add_handle(DrawingHandle::square(entry.clone(), 0));
        output.add_handle(DrawingHandle::square(exit.clone(), 1));

        output
    }

    fn hit_test(&self, point: &ChartPoint<X>, tolerance: f64) -> bool {
        if self.points.len() < 2 {
            return false;
        }

        let entry = &self.points[0].point;
        let exit = &self.points[1].point;

        // Check if point is inside the rectangle
        let min_x = entry.x.to_plot_value().min(exit.x.to_plot_value());
        let max_x = entry.x.to_plot_value().max(exit.x.to_plot_value());
        let min_y = entry.y.min(exit.y);
        let max_y = entry.y.max(exit.y);

        let px = point.x.to_plot_value();
        let py = point.y;

        // Inside the box
        if px >= min_x && px <= max_x && py >= min_y && py <= max_y {
            return true;
        }

        // Near the border
        let corners = [
            ChartPoint::new(entry.x, entry.y),
            ChartPoint::new(exit.x, entry.y),
            ChartPoint::new(exit.x, exit.y),
            ChartPoint::new(entry.x, exit.y),
        ];

        for i in 0..4 {
            let j = (i + 1) % 4;
            if point_to_segment_distance(point, &corners[i], &corners[j]) <= tolerance {
                return true;
            }
        }

        false
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
    fn test_position_tool() {
        let tool = PositionTool::from_points(
            ChartPoint::new(Index(0), 100.0),
            ChartPoint::new(Index(10), 110.0),
        );

        assert!(tool.is_complete());
        assert_eq!(tool.price_diff(), Some(10.0));
        assert_eq!(tool.percent_change(), Some(10.0));
        assert!(tool.is_profitable());
    }

    #[test]
    fn test_position_tool_loss() {
        // Long position where price dropped — that is the loss case.
        // (`from_points` auto-infers direction from price movement, so we
        // override it to force Long.)
        let tool = PositionTool::from_points(
            ChartPoint::new(Index(0), 100.0),
            ChartPoint::new(Index(10), 90.0),
        )
        .with_direction(PositionDirection::Long);

        assert_eq!(tool.direction, PositionDirection::Long);
        assert!(!tool.is_profitable());
    }

    #[test]
    fn test_position_tool_short_profit() {
        // Short position where price dropped — Short profits when price falls.
        let tool = PositionTool::from_points(
            ChartPoint::new(Index(0), 100.0),
            ChartPoint::new(Index(10), 90.0),
        );

        assert_eq!(tool.direction, PositionDirection::Short);
        assert!(tool.is_profitable());
    }
}
