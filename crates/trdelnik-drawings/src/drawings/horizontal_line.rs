//! Horizontal line drawing
//!
//! A simple horizontal line at a fixed price level.
//! Commonly used for support/resistance levels.

use crate::{
    AnchorPoint, ChartPoint, Drawing, DrawingHandle, DrawingLabel,
    DrawingOutput, DrawingStyle, HorizontalLineOutput, TextAnchor,
};
use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;
use uuid::Uuid;

/// A horizontal line at a fixed Y (price) value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HorizontalLine<X: AxisCoordinate> {
    id: Uuid,
    /// The Y value where the line is drawn
    y_value: f64,
    /// Optional label text
    pub label: Option<String>,
    /// Phantom data for generic type
    #[serde(skip)]
    _phantom: std::marker::PhantomData<X>,
}

impl<X: AxisCoordinate> HorizontalLine<X> {
    /// Create a new horizontal line at the given Y value
    pub fn new(y_value: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            y_value,
            label: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create with a label
    pub fn with_label(y_value: f64, label: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            y_value,
            label: Some(label.into()),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the Y value
    pub fn y_value(&self) -> f64 {
        self.y_value
    }

    /// Set the Y value
    pub fn set_y_value(&mut self, y: f64) {
        self.y_value = y;
    }
}

impl<X: AxisCoordinate> Drawing<X> for HorizontalLine<X> {
    fn type_id(&self) -> &'static str {
        "hline"
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn display_name(&self) -> &str {
        "Horizontal Line"
    }

    fn required_points(&self) -> usize {
        1 // Just one click to set the Y value
    }

    fn anchor_points(&self) -> &[AnchorPoint<X>] {
        // Horizontal lines don't store anchor points in the traditional sense
        // The Y value is the "anchor"
        &[]
    }

    fn anchor_points_mut(&mut self) -> &mut [AnchorPoint<X>] {
        &mut []
    }

    fn set_anchor_point(&mut self, _index: usize, point: AnchorPoint<X>) {
        // Use the Y value from the point
        self.y_value = point.point.y;
    }

    fn is_complete(&self) -> bool {
        true // Always complete once created
    }

    fn compute(&self, style: &DrawingStyle) -> DrawingOutput<X> {
        let mut output = DrawingOutput::new();

        // Use the dedicated horizontal line output type
        let hline = HorizontalLineOutput::new(self.y_value, style.color)
            .with_width(style.line_width)
            .with_style(style.line_style);

        output.add_horizontal_line(hline);

        // Add price label on the right side
        if style.show_prices {
            let label_text = if let Some(ref custom_label) = self.label {
                format!("{} ({:.2})", custom_label, self.y_value)
            } else {
                format!("{:.2}", self.y_value)
            };

            let label = DrawingLabel::new(
                ChartPoint::new(X::from_plot_value(f64::MAX), self.y_value),
                label_text,
                style.color,
            )
            .with_anchor(TextAnchor::MiddleRight)
            .with_background(style.color.with_alpha(200));

            output.add_label(label);
        }

        // Add a handle at a dummy position (UI will position it correctly)
        let handle = DrawingHandle::new(ChartPoint::new(X::from_plot_value(0.0), self.y_value), 0);
        output.add_handle(handle);

        output
    }

    fn hit_test(&self, point: &ChartPoint<X>, tolerance: f64) -> bool {
        // Check if the point is within tolerance of the line's Y value
        (point.y - self.y_value).abs() <= tolerance
    }

    fn bounds(&self) -> Option<(ChartPoint<X>, ChartPoint<X>)> {
        // Horizontal lines have no X bounds (they extend infinitely)
        None
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
    fn test_horizontal_line_creation() {
        let line: HorizontalLine<Index> = HorizontalLine::new(100.0);
        assert_eq!(line.y_value(), 100.0);
        assert!(line.is_complete());
    }

    #[test]
    fn test_horizontal_line_hit_test() {
        let line: HorizontalLine<Index> = HorizontalLine::new(100.0);
        let point_on = ChartPoint::new(Index(5), 100.5);
        let point_off = ChartPoint::new(Index(5), 105.0);

        assert!(line.hit_test(&point_on, 1.0));
        assert!(!line.hit_test(&point_off, 1.0));
    }

    #[test]
    fn test_new_assigns_unique_id() {
        let a: HorizontalLine<Index> = HorizontalLine::new(100.0);
        let b: HorizontalLine<Index> = HorizontalLine::new(100.0);
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn test_with_label_sets_label() {
        let h: HorizontalLine<Index> = HorizontalLine::with_label(50.0, "support");
        assert_eq!(h.label.as_deref(), Some("support"));
        assert_eq!(h.y_value(), 50.0);
    }

    #[test]
    fn test_set_y_value() {
        let mut h: HorizontalLine<Index> = HorizontalLine::new(100.0);
        h.set_y_value(200.0);
        assert_eq!(h.y_value(), 200.0);
    }

    #[test]
    fn test_drawing_metadata() {
        let h: HorizontalLine<Index> = HorizontalLine::new(100.0);
        assert_eq!(h.type_id(), "hline");
        assert_eq!(h.display_name(), "Horizontal Line");
        assert_eq!(h.required_points(), 1);
        assert!(h.is_complete());
        assert!(h.bounds().is_none());
        assert!(h.anchor_points().is_empty());
    }

    #[test]
    fn test_set_anchor_point_uses_y_only() {
        let mut h: HorizontalLine<Index> = HorizontalLine::new(0.0);
        h.set_anchor_point(0, AnchorPoint::new(Index(99), 42.5));
        assert_eq!(h.y_value(), 42.5);
    }

    #[test]
    fn test_compute_emits_horizontal_line_and_handle_no_label_with_show_prices_off() {
        // show_prices = false → no label even though Y has a numeric value
        let h: HorizontalLine<Index> = HorizontalLine::new(150.0);
        let mut style = DrawingStyle::default();
        style.show_prices = false;
        let out = h.compute(&style);

        assert_eq!(out.horizontal_lines.len(), 1);
        assert!((out.horizontal_lines[0].y - 150.0).abs() < 1e-9);
        assert_eq!(out.handles.len(), 1);
        assert!(out.labels.is_empty());
    }

    #[test]
    fn test_compute_default_style_emits_price_label() {
        let h: HorizontalLine<Index> = HorizontalLine::new(99.5);
        let style = DrawingStyle::default(); // show_prices = true
        let out = h.compute(&style);
        assert_eq!(out.labels.len(), 1);
        // Without a custom label, the text is just the formatted price
        assert!(out.labels[0].text.contains("99.50"));
    }

    #[test]
    fn test_compute_with_custom_label_includes_label_and_price() {
        let h: HorizontalLine<Index> = HorizontalLine::with_label(99.5, "Resistance");
        let out = h.compute(&DrawingStyle::default());
        assert_eq!(out.labels.len(), 1);
        let text = &out.labels[0].text;
        assert!(text.contains("Resistance"));
        assert!(text.contains("99.50"));
    }

    #[test]
    fn test_hit_test_y_only() {
        let h: HorizontalLine<Index> = HorizontalLine::new(100.0);
        // X is irrelevant for hit_test — both close-x and far-x match if Y is close
        assert!(h.hit_test(&ChartPoint::new(Index(0), 100.4), 0.5));
        assert!(h.hit_test(&ChartPoint::new(Index(99999), 99.6), 0.5));
        assert!(!h.hit_test(&ChartPoint::new(Index(0), 102.0), 0.5));
    }

    #[test]
    fn test_clone_box_preserves_y_and_label() {
        let h: HorizontalLine<Index> =
            HorizontalLine::with_label(77.7, "Pivot");
        let boxed: Box<dyn Drawing<Index>> = h.clone_box();
        // Cloned trait object reports same metadata
        assert_eq!(boxed.type_id(), "hline");
        // hit_test at exactly the cloned y_value
        assert!(boxed.hit_test(&ChartPoint::new(Index(0), 77.7), 0.0));
    }
}
