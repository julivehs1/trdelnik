//! Vertical line drawing
//!
//! A simple vertical line at a fixed X (time) value.
//! Commonly used for event markers.

use crate::{
    AnchorPoint, ChartPoint, Drawing, DrawingHandle, DrawingLabel, DrawingOutput,
    DrawingStyle, TextAnchor, VerticalLineOutput,
};
use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;
use uuid::Uuid;

/// A vertical line at a fixed X (time) value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerticalLine<X: AxisCoordinate> {
    id: Uuid,
    /// The X value where the line is drawn
    x_value: X,
    /// Optional label text
    pub label: Option<String>,
}

impl<X: AxisCoordinate> VerticalLine<X> {
    /// Create a new vertical line at the given X value
    pub fn new(x_value: X) -> Self {
        Self {
            id: Uuid::new_v4(),
            x_value,
            label: None,
        }
    }

    /// Create with a label
    pub fn with_label(x_value: X, label: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            x_value,
            label: Some(label.into()),
        }
    }

    /// Get the X value
    pub fn x_value(&self) -> X {
        self.x_value
    }

    /// Set the X value
    pub fn set_x_value(&mut self, x: X) {
        self.x_value = x;
    }
}

impl<X: AxisCoordinate> Drawing<X> for VerticalLine<X> {
    fn type_id(&self) -> &'static str {
        "vline"
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn display_name(&self) -> &str {
        "Vertical Line"
    }

    fn required_points(&self) -> usize {
        1
    }

    fn anchor_points(&self) -> &[AnchorPoint<X>] {
        &[]
    }

    fn anchor_points_mut(&mut self) -> &mut [AnchorPoint<X>] {
        &mut []
    }

    fn set_anchor_point(&mut self, _index: usize, point: AnchorPoint<X>) {
        self.x_value = point.point.x;
    }

    fn is_complete(&self) -> bool {
        true
    }

    fn compute(&self, style: &DrawingStyle) -> DrawingOutput<X> {
        let mut output = DrawingOutput::new();

        // Use the dedicated vertical line output type
        let vline = VerticalLineOutput::new(self.x_value, style.color)
            .with_width(style.line_width)
            .with_style(style.line_style);

        output.add_vertical_line(vline);

        // Add label at the top
        if style.show_labels {
            if let Some(ref label_text) = self.label {
                let label = DrawingLabel::new(
                    ChartPoint::new(self.x_value, f64::MAX),
                    label_text.clone(),
                    style.color,
                )
                .with_anchor(TextAnchor::BottomCenter)
                .with_background(style.color.with_alpha(200));

                output.add_label(label);
            }
        }

        // Handle
        let handle = DrawingHandle::new(ChartPoint::new(self.x_value, 0.0), 0);
        output.add_handle(handle);

        output
    }

    fn hit_test(&self, point: &ChartPoint<X>, tolerance: f64) -> bool {
        (point.x.to_plot_value() - self.x_value.to_plot_value()).abs() <= tolerance
    }

    fn bounds(&self) -> Option<(ChartPoint<X>, ChartPoint<X>)> {
        None // No Y bounds
    }

    fn clone_box(&self) -> Box<dyn Drawing<X>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DrawingStyle;
    use trdelnik_core::Index;

    #[test]
    fn test_new_sets_x_value_and_no_label() {
        let v = VerticalLine::new(Index(7));
        assert_eq!(v.x_value(), Index(7));
        assert!(v.label.is_none());
    }

    #[test]
    fn test_new_assigns_unique_id() {
        let a: VerticalLine<Index> = VerticalLine::new(Index(0));
        let b: VerticalLine<Index> = VerticalLine::new(Index(0));
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn test_with_label() {
        let v: VerticalLine<Index> = VerticalLine::with_label(Index(5), "earnings");
        assert_eq!(v.x_value(), Index(5));
        assert_eq!(v.label.as_deref(), Some("earnings"));
    }

    #[test]
    fn test_set_x_value() {
        let mut v: VerticalLine<Index> = VerticalLine::new(Index(0));
        v.set_x_value(Index(42));
        assert_eq!(v.x_value(), Index(42));
    }

    #[test]
    fn test_drawing_metadata() {
        let v: VerticalLine<Index> = VerticalLine::new(Index(0));
        assert_eq!(v.type_id(), "vline");
        assert_eq!(v.display_name(), "Vertical Line");
        assert_eq!(v.required_points(), 1);
        assert!(v.is_complete());
        assert!(v.bounds().is_none());
        assert!(v.anchor_points().is_empty());
    }

    #[test]
    fn test_set_anchor_point_updates_x() {
        let mut v: VerticalLine<Index> = VerticalLine::new(Index(0));
        v.set_anchor_point(0, AnchorPoint::new(Index(99), 12.5));
        assert_eq!(v.x_value(), Index(99));
    }

    #[test]
    fn test_compute_emits_vertical_line_and_handle_no_label() {
        let v: VerticalLine<Index> = VerticalLine::new(Index(3));
        let style = DrawingStyle::default();
        let out = v.compute(&style);

        assert_eq!(out.vertical_lines.len(), 1);
        assert_eq!(out.vertical_lines[0].x, Index(3));
        assert_eq!(out.handles.len(), 1);
        // No label was set on the line
        assert!(out.labels.is_empty());
    }

    #[test]
    fn test_compute_emits_label_when_set_and_show_labels_true() {
        let v: VerticalLine<Index> = VerticalLine::with_label(Index(2), "event");
        let style = DrawingStyle::default(); // show_labels = true
        let out = v.compute(&style);

        assert_eq!(out.labels.len(), 1);
        assert_eq!(out.labels[0].text, "event");
    }

    #[test]
    fn test_compute_omits_label_when_show_labels_disabled() {
        let v: VerticalLine<Index> = VerticalLine::with_label(Index(2), "event");
        let style = DrawingStyle::default().no_labels();
        let out = v.compute(&style);
        assert!(out.labels.is_empty());
    }

    #[test]
    fn test_hit_test_within_tolerance() {
        let v: VerticalLine<Index> = VerticalLine::new(Index(10));
        let probe = ChartPoint::new(Index(11), 50.0);
        assert!(v.hit_test(&probe, 2.0));
        assert!(!v.hit_test(&probe, 0.5));
    }

    #[test]
    fn test_hit_test_y_is_irrelevant() {
        let v: VerticalLine<Index> = VerticalLine::new(Index(5));
        let near = ChartPoint::new(Index(5), 1_000_000.0);
        assert!(v.hit_test(&near, 0.0001));
    }

    #[test]
    fn test_clone_box_preserves_x_and_label() {
        let v: VerticalLine<Index> = VerticalLine::with_label(Index(8), "ann");
        let boxed: Box<dyn Drawing<Index>> = v.clone_box();
        // The cloned trait object reports the same metadata
        assert_eq!(boxed.type_id(), "vline");
        // hit_test returns true exactly at the cloned x_value
        assert!(boxed.hit_test(&ChartPoint::new(Index(8), 0.0), 0.0));
    }
}
