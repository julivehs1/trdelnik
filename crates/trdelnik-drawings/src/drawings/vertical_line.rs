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
