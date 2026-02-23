//! Bridge to IndicatorOutput for integration with the rendering system

use crate::context::ExecutionResult;
use crate::node::NodeId;
use trdelnik_core::{IndicatorLine, IndicatorOutput, Placement, Timestamp};

/// Builder for converting graph execution results to IndicatorOutput.
///
/// This provides a bridge between the graph execution engine and
/// the existing indicator rendering system.
pub struct OutputBuilder<'a> {
    result: &'a ExecutionResult,
}

impl<'a> OutputBuilder<'a> {
    /// Create a new output builder from execution results
    pub fn new(result: &'a ExecutionResult) -> Self {
        Self { result }
    }

    /// Build a single-line overlay indicator output
    pub fn single_overlay(
        &self,
        name: impl Into<String>,
        indicator_id: impl Into<String>,
        line_name: impl Into<String>,
        line_id: impl Into<String>,
        node_id: NodeId,
    ) -> IndicatorOutput<Timestamp> {
        let name = name.into();
        let x_values = self.x_values_as_timestamps();
        let y_values = self.result.get_output_f64(node_id);

        let mut output = IndicatorOutput::overlay(name.clone(), indicator_id);
        output.add_line(IndicatorLine::from_xy(
            line_name,
            line_id,
            &x_values,
            &y_values,
        ));
        output
    }

    /// Build a single-line panel indicator output
    pub fn single_panel(
        &self,
        name: impl Into<String>,
        indicator_id: impl Into<String>,
        line_name: impl Into<String>,
        line_id: impl Into<String>,
        node_id: NodeId,
    ) -> IndicatorOutput<Timestamp> {
        let name = name.into();
        let x_values = self.x_values_as_timestamps();
        let y_values = self.result.get_output_f64(node_id);

        let mut output = IndicatorOutput::panel(name.clone(), indicator_id);
        output.add_line(IndicatorLine::from_xy(
            line_name,
            line_id,
            &x_values,
            &y_values,
        ));
        output
    }

    /// Build a multi-line indicator output
    pub fn multi_line(
        &self,
        name: impl Into<String>,
        indicator_id: impl Into<String>,
        placement: Placement,
        lines: &[(impl AsRef<str>, impl AsRef<str>, NodeId)],
    ) -> IndicatorOutput<Timestamp> {
        let x_values = self.x_values_as_timestamps();

        let mut output = IndicatorOutput::new(name, indicator_id, placement);
        for (line_name, line_id, node_id) in lines {
            let y_values = self.result.get_output_f64(*node_id);
            output.add_line(IndicatorLine::from_xy(
                line_name.as_ref(),
                line_id.as_ref(),
                &x_values,
                &y_values,
            ));
        }
        output
    }

    /// Get the raw output values for a node
    pub fn get_output(&self, node_id: NodeId) -> Vec<Option<f64>> {
        self.result.get_output_f64(node_id)
    }

    /// Get the X values as f64
    pub fn x_values(&self) -> &[f64] {
        self.result.x_values()
    }

    /// Get the X values as Timestamps
    pub fn x_values_as_timestamps(&self) -> Vec<Timestamp> {
        self.result
            .x_values()
            .iter()
            .map(|&x| Timestamp(x as i64))
            .collect()
    }

    /// Build an indicator output from a struct node and its field extraction nodes.
    ///
    /// This is a convenience method for multi-output indicators where you have:
    /// - A struct node that computes all values
    /// - Multiple field nodes that extract individual values
    ///
    /// # Arguments
    /// * `name` - Display name for the indicator
    /// * `indicator_id` - Unique identifier for the indicator type
    /// * `placement` - Where to render the indicator (overlay or panel)
    /// * `field_nodes` - Slice of (field_node_id, line_name, line_id) tuples
    ///
    /// # Example
    /// ```ignore
    /// let (boll, middle, upper, lower) = bollinger_with_fields(&mut graph, close, 20, 2.0);
    /// let output = builder.from_field_nodes(
    ///     "Bollinger Bands",
    ///     "bollinger",
    ///     Placement::Overlay,
    ///     &[
    ///         (middle, "Middle", "middle"),
    ///         (upper, "Upper", "upper"),
    ///         (lower, "Lower", "lower"),
    ///     ],
    /// );
    /// ```
    pub fn from_field_nodes(
        &self,
        name: impl Into<String>,
        indicator_id: impl Into<String>,
        placement: Placement,
        field_nodes: &[(NodeId, &str, &str)],
    ) -> IndicatorOutput<Timestamp> {
        let x_values = self.x_values_as_timestamps();

        let mut output = IndicatorOutput::new(name, indicator_id, placement);
        for &(node_id, line_name, line_id) in field_nodes {
            let y_values = self.result.get_output_f64(node_id);
            output.add_line(IndicatorLine::from_xy(
                line_name,
                line_id,
                &x_values,
                &y_values,
            ));
        }
        output
    }
}

/// Extension trait for ExecutionResult to easily create OutputBuilder
pub trait ExecutionResultExt {
    fn output_builder(&self) -> OutputBuilder<'_>;
}

impl ExecutionResultExt for ExecutionResult {
    fn output_builder(&self) -> OutputBuilder<'_> {
        OutputBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::OutputStore;

    #[test]
    fn test_output_builder_x_values() {
        let mut result = ExecutionResult::new();
        let mut store = OutputStore::new();
        store.set(NodeId(0), crate::value::Value::number(100.0));

        result.push_bar(1000.0, &store);
        result.push_bar(2000.0, &store);
        result.push_bar(3000.0, &store);

        let builder = OutputBuilder::new(&result);
        let timestamps = builder.x_values_as_timestamps();

        assert_eq!(timestamps.len(), 3);
        assert_eq!(timestamps[0], Timestamp(1000));
        assert_eq!(timestamps[1], Timestamp(2000));
        assert_eq!(timestamps[2], Timestamp(3000));
    }
}
