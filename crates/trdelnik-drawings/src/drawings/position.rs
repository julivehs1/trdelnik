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

    fn cp(x: usize, y: f64) -> ChartPoint<Index> {
        ChartPoint::new(Index(x), y)
    }

    // ---------- PositionDirection ----------

    #[test]
    fn test_direction_default_is_long() {
        let d: PositionDirection = Default::default();
        assert_eq!(d, PositionDirection::Long);
    }

    // ---------- Constructors ----------

    #[test]
    fn test_new_defaults() {
        let t: PositionTool<Index> = PositionTool::new();
        assert_eq!(t.direction, PositionDirection::Long);
        assert!(t.quantity.is_none());
        assert!((t.pip_multiplier - 1.0).abs() < 1e-6);
        assert!(t.anchor_points().is_empty());
    }

    #[test]
    fn test_default_is_new() {
        let a: PositionTool<Index> = PositionTool::default();
        assert!(a.anchor_points().is_empty());
        assert!((a.pip_multiplier - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_forex_uses_pip_multiplier_10000() {
        let t: PositionTool<Index> = PositionTool::forex();
        assert!((t.pip_multiplier - 10_000.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_points_with_equal_y_is_long() {
        // Edge case: exit.y == entry.y → direction = Long (>= comparison)
        let t = PositionTool::from_points(cp(0, 100.0), cp(10, 100.0));
        assert_eq!(t.direction, PositionDirection::Long);
    }

    // ---------- Builders ----------

    #[test]
    fn test_with_direction_overrides() {
        let t: PositionTool<Index> =
            PositionTool::new().with_direction(PositionDirection::Short);
        assert_eq!(t.direction, PositionDirection::Short);
    }

    #[test]
    fn test_with_quantity_sets_some() {
        let t: PositionTool<Index> = PositionTool::new().with_quantity(2.5);
        assert_eq!(t.quantity, Some(2.5));
    }

    // ---------- Internal calculations ----------

    #[test]
    fn test_price_diff_under_two_points_is_none() {
        let t: PositionTool<Index> = PositionTool::new();
        assert!(t.price_diff().is_none());
        assert!(t.percent_change().is_none());
    }

    #[test]
    fn test_percent_change_zero_entry_is_none() {
        let t = PositionTool::from_points(cp(0, 0.0), cp(10, 5.0));
        assert!(t.percent_change().is_none());
    }

    #[test]
    fn test_short_loss_when_price_rises() {
        let t: PositionTool<Index> = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0))
            .with_direction(PositionDirection::Short);
        assert!(!t.is_profitable());
    }

    // ---------- Drawing trait ----------

    #[test]
    fn test_metadata() {
        let t: PositionTool<Index> = PositionTool::new();
        assert_eq!(t.type_id(), "position");
        assert_eq!(t.display_name(), "Position");
        assert_eq!(t.required_points(), 2);
    }

    #[test]
    fn test_set_anchor_point_updates_direction_long() {
        let mut t: PositionTool<Index> = PositionTool::new();
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 100.0));
        t.set_anchor_point(1, AnchorPoint::new(Index(10), 110.0));
        assert_eq!(t.direction, PositionDirection::Long);
    }

    #[test]
    fn test_set_anchor_point_updates_direction_short() {
        let mut t: PositionTool<Index> = PositionTool::new();
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 100.0));
        t.set_anchor_point(1, AnchorPoint::new(Index(10), 90.0));
        assert_eq!(t.direction, PositionDirection::Short);
    }

    #[test]
    fn test_set_anchor_point_overwrites_existing_index() {
        let mut t = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0));
        t.set_anchor_point(1, AnchorPoint::new(Index(5), 120.0));
        assert_eq!(t.anchor_points()[1].point.x, Index(5));
        assert!((t.anchor_points()[1].point.y - 120.0).abs() < 1e-9);
    }

    // ---------- compute ----------

    #[test]
    fn test_compute_under_two_points_emits_nothing_or_handle() {
        let t: PositionTool<Index> = PositionTool::new();
        let out0 = t.compute(&DrawingStyle::default());
        assert!(out0.handles.is_empty());
        assert!(out0.lines.is_empty());

        let mut t1: PositionTool<Index> = PositionTool::new();
        t1.set_anchor_point(0, AnchorPoint::new(Index(3), 50.0));
        let out1 = t1.compute(&DrawingStyle::default());
        assert_eq!(out1.handles.len(), 1);
        assert!(out1.lines.is_empty());
    }

    #[test]
    fn test_compute_full_long_position_emits_box_lines_handles_and_label() {
        let t = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0));
        let out = t.compute(&DrawingStyle::default());
        // 1 fill rectangle + 1 border polyline + 1 vertical mid-line + 1 horizontal connector
        assert_eq!(out.fills.len(), 1);
        assert_eq!(out.lines.len(), 3);
        assert_eq!(out.labels.len(), 1);
        assert_eq!(out.handles.len(), 2);
    }

    #[test]
    fn test_compute_label_text_for_stocks() {
        // pip_multiplier = 1.0 → label format is "diff (pct%)"
        let t = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0));
        let out = t.compute(&DrawingStyle::default());
        let text = &out.labels[0].text;
        assert!(text.contains("10.00"));   // diff
        assert!(text.contains("10.00%"));
    }

    #[test]
    fn test_compute_label_text_for_forex_includes_pips() {
        // pip_multiplier > 1 → label includes pip count
        let mut t = PositionTool::forex();
        t.set_anchor_point(0, AnchorPoint::new(Index(0), 1.1000));
        t.set_anchor_point(1, AnchorPoint::new(Index(10), 1.1050));
        let out = t.compute(&DrawingStyle::default());
        let text = &out.labels[0].text;
        // 0.0050 * 10000 = 50 pips
        assert!(text.contains("50"));
    }

    // ---------- hit_test ----------

    #[test]
    fn test_hit_test_under_two_points_is_false() {
        let t: PositionTool<Index> = PositionTool::new();
        assert!(!t.hit_test(&cp(0, 100.0), 1.0));
    }

    #[test]
    fn test_hit_test_inside_box_returns_true() {
        let t = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0));
        assert!(t.hit_test(&cp(5, 105.0), 0.001));
    }

    #[test]
    fn test_hit_test_on_border_within_tolerance() {
        let t = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0));
        // Just outside the box vertically, but within tolerance of the top border.
        assert!(t.hit_test(&cp(5, 110.5), 1.0));
    }

    #[test]
    fn test_hit_test_far_outside_returns_false() {
        let t = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0));
        assert!(!t.hit_test(&cp(50, 50.0), 0.5));
    }

    // ---------- bounds (default impl) ----------

    #[test]
    fn test_bounds_uses_default_impl() {
        let t = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0));
        let (min, max) = t.bounds().unwrap();
        assert_eq!(min.x, Index(0));
        assert_eq!(max.x, Index(10));
        assert!((min.y - 100.0).abs() < 1e-9);
        assert!((max.y - 110.0).abs() < 1e-9);
    }

    // ---------- clone_box ----------

    #[test]
    fn test_clone_box_preserves_metadata() {
        let t = PositionTool::from_points(cp(0, 100.0), cp(10, 110.0));
        let id = t.id();
        let boxed: Box<dyn Drawing<Index>> = t.clone_box();
        assert_eq!(boxed.id(), id);
        assert_eq!(boxed.type_id(), "position");
    }
}
