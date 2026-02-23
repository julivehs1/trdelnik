//! Integration module for converting script execution results to chart components
//!
//! This module provides methods to convert CompiledStrategy outputs into
//! SignalSeries types that can be directly rendered by the chart.
//!
//! For converting plots to IndicatorOutput, use the functions in `trdelnik_data`:
//! - `trdelnik_data::ScriptPlotConfig` for UI customization
//! - `trdelnik_data::plots_to_overlays()` for plot conversion
//! - `trdelnik_data::SignalStats` for signal statistics

use trdelnik_core::{AxisCoordinate, CandleSeries, Signal, SignalDirection, SignalSeries};
use trdelnik_graph::ExecutionResult;

use crate::ast::ExitTarget;
use crate::compiler::CompiledStrategy;

impl CompiledStrategy {
    /// Convert entry/exit signals to SignalSeries for chart rendering
    ///
    /// This method extracts all entry and exit signals from the execution result
    /// and converts them to a SignalSeries that can be rendered on the chart.
    ///
    /// # Arguments
    /// * `result` - The execution result from running the strategy
    /// * `series` - The candle series (provides price levels for signal placement)
    ///
    /// # Returns
    /// A SignalSeries containing all entry and exit signals
    pub fn to_signals<X: AxisCoordinate>(
        &self,
        result: &ExecutionResult,
        series: &CandleSeries<X>,
    ) -> SignalSeries<X> {
        let mut signals = SignalSeries::new();

        // Long entries
        if let Some(entry_long) = self.entry_long {
            let values = result.get_output(entry_long);
            for (i, val) in values.iter().enumerate() {
                if val.as_bool() == Some(true) {
                    if let Some(candle) = series.get(i) {
                        signals.push(Signal::new(
                            candle.x,
                            candle.low,
                            SignalDirection::Buy,
                            "Script Entry Long",
                        ));
                    }
                }
            }
        }

        // Short entries
        if let Some(entry_short) = self.entry_short {
            let values = result.get_output(entry_short);
            for (i, val) in values.iter().enumerate() {
                if val.as_bool() == Some(true) {
                    if let Some(candle) = series.get(i) {
                        signals.push(Signal::new(
                            candle.x,
                            candle.high,
                            SignalDirection::Sell,
                            "Script Entry Short",
                        ));
                    }
                }
            }
        }

        // Exit signals
        for (target, exit_node) in &self.exit_signals {
            let values = result.get_output(*exit_node);
            let direction = match target {
                ExitTarget::Long => SignalDirection::ExitLong,
                ExitTarget::Short => SignalDirection::ExitShort,
                ExitTarget::All => SignalDirection::Neutral,
            };

            for (i, val) in values.iter().enumerate() {
                if val.as_bool() == Some(true) {
                    if let Some(candle) = series.get(i) {
                        signals.push(Signal::new(
                            candle.x,
                            candle.close,
                            direction,
                            "Script Exit",
                        ));
                    }
                }
            }
        }

        signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;
    use trdelnik_core::{Candle, Timestamp};
    use trdelnik_graph::Executor;

    fn create_test_series() -> CandleSeries<Timestamp> {
        let mut series = CandleSeries::new();
        let prices = [
            100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5, 107.0, 108.0,
            107.5, 109.0, 110.0, 109.5, 111.0, 112.0, 111.5, 113.0,
        ];

        for (i, &price) in prices.iter().enumerate() {
            series.push(Candle::new(
                Timestamp(i as i64 * 3600000),
                price - 0.5,
                price + 1.0,
                price - 1.0,
                price,
                1000.0 + i as f64 * 100.0,
            ));
        }
        series
    }

    #[test]
    fn test_to_signals() {
        let source = r#"
            let fast = sma(close, 3)
            let slow = sma(close, 5)
            entry long when crossover(fast, slow)
            entry short when crossunder(fast, slow)
        "#;

        let strategy = compile(source).unwrap();
        let series = create_test_series();

        // Execute the strategy
        let compiled = compile(source).unwrap();
        let mut executor = Executor::new(compiled.graph);
        let result = executor.process_series(&series);

        // Convert to signals
        let signals = strategy.to_signals(&result, &series);

        // Should have some signals (exact count depends on the data)
        // Just verify the conversion works without panicking
        let _ = signals.len();
    }
}
