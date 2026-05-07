//! # Trdelnik Script Bridge
//!
//! Glue between [`trdelnik_script`] execution results and the chart
//! system. Converts a `CompiledStrategy` + `ExecutionResult` into the
//! visualisation primitives the chart layer consumes (`Plot` overlays,
//! signal markers, summary stats).
//!
//! Lives in its own crate to keep `trdelnik-data` free of any script
//! dependency — `data` should be neutral plumbing for any source of
//! plot data, not an integration point for one specific strategy DSL.

use trdelnik_core::{
    AxisCoordinate, CandleSeries, Color, IndicatorLine, IndicatorMarker,
};
use trdelnik_graph::ExecutionResult;
use trdelnik_render::{Plot, StandardPlot};
use trdelnik_script::ast::ExitTarget;
use trdelnik_script::CompiledStrategy;

/// Configuration for a single plot from a script (for UI customisation).
#[derive(Debug, Clone)]
pub struct ScriptPlotConfig {
    /// Index of the plot in the script
    pub index: usize,
    /// Display name
    pub name: String,
    /// Current color (initially from script, can be overridden by user)
    pub color: Color,
    /// Original color from the script (for reset functionality)
    pub script_color: Color,
    /// Whether this plot is visible
    pub visible: bool,
    /// Panel name (None = overlay on main chart)
    pub panel: Option<String>,
}

impl ScriptPlotConfig {
    /// Create configs from a compiled strategy.
    pub fn from_strategy(strategy: &CompiledStrategy) -> Vec<Self> {
        strategy
            .plots
            .iter()
            .enumerate()
            .map(|(i, plot)| ScriptPlotConfig {
                index: i,
                name: format!("Plot {}", i + 1),
                color: plot.color,
                script_color: plot.color,
                visible: true,
                panel: plot.panel.clone(),
            })
            .collect()
    }
}

/// Convert a compiled strategy's plots to boxed `Plot` overlays.
pub fn plots_to_overlays<X: AxisCoordinate>(
    strategy: &CompiledStrategy,
    result: &ExecutionResult,
    series: &CandleSeries<X>,
) -> Vec<Box<dyn Plot<X>>> {
    plots_to_overlays_with_config(strategy, result, series, None)
}

/// Convert plots with custom configurations into boxed `Plot` overlays.
pub fn plots_to_overlays_with_config<X: AxisCoordinate>(
    strategy: &CompiledStrategy,
    result: &ExecutionResult,
    series: &CandleSeries<X>,
    configs: Option<&[ScriptPlotConfig]>,
) -> Vec<Box<dyn Plot<X>>> {
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();

    strategy
        .plots
        .iter()
        .enumerate()
        .filter_map(|(idx, plot)| {
            if let Some(cfgs) = configs {
                if let Some(cfg) = cfgs.get(idx) {
                    if !cfg.visible {
                        return None;
                    }
                }
            }

            let color = configs
                .and_then(|cfgs| cfgs.get(idx))
                .map(|cfg| cfg.color)
                .unwrap_or(plot.color);

            let values: Vec<Option<f64>> = result
                .get_output(plot.node)
                .iter()
                .map(|v| v.as_number())
                .collect();

            let indicator_id = format!("script_plot_{}", idx);
            let name = configs
                .and_then(|cfgs| cfgs.get(idx))
                .map(|cfg| cfg.name.clone())
                .unwrap_or_else(|| format!("Script Plot {}", idx + 1));

            let mut data = StandardPlot::new(&indicator_id);

            let line = IndicatorLine::from_xy(&name, &indicator_id, &x_values, &values)
                .with_color(color);

            data.add_line(line);
            Some(Box::new(data) as Box<dyn Plot<X>>)
        })
        .collect()
}

/// Convert entry/exit signals to `IndicatorMarker`s for chart annotation.
pub fn signals_to_markers<X: AxisCoordinate>(
    strategy: &CompiledStrategy,
    result: &ExecutionResult,
    series: &CandleSeries<X>,
) -> Vec<IndicatorMarker<X>> {
    let mut markers = Vec::new();

    let green = Color::from_name("green");
    let red = Color::from_name("red");
    let yellow = Color::from_name("yellow");

    if let Some(entry_long) = strategy.entry_long {
        let values = result.get_output(entry_long);
        for (i, val) in values.iter().enumerate() {
            if val.as_bool() == Some(true) {
                if let Some(candle) = series.get(i) {
                    markers.push(
                        IndicatorMarker::arrow_up(candle.x, candle.low, green)
                            .with_label("Long"),
                    );
                }
            }
        }
    }

    if let Some(entry_short) = strategy.entry_short {
        let values = result.get_output(entry_short);
        for (i, val) in values.iter().enumerate() {
            if val.as_bool() == Some(true) {
                if let Some(candle) = series.get(i) {
                    markers.push(
                        IndicatorMarker::arrow_down(candle.x, candle.high, red)
                            .with_label("Short"),
                    );
                }
            }
        }
    }

    for (target, exit_node) in &strategy.exit_signals {
        let values = result.get_output(*exit_node);
        let label = match target {
            ExitTarget::Long => "Exit Long",
            ExitTarget::Short => "Exit Short",
            ExitTarget::All => "Exit",
        };

        for (i, val) in values.iter().enumerate() {
            if val.as_bool() == Some(true) {
                if let Some(candle) = series.get(i) {
                    markers.push(
                        IndicatorMarker::cross(candle.x, candle.close, yellow).with_label(label),
                    );
                }
            }
        }
    }

    markers
}

/// Compute signal markers ready to be attached to `ChartData` or a `Panel`.
pub fn signals_to_overlay_markers<X: AxisCoordinate>(
    strategy: &CompiledStrategy,
    result: &ExecutionResult,
    series: &CandleSeries<X>,
) -> Vec<IndicatorMarker<X>> {
    signals_to_markers(strategy, result, series)
}

/// Counts of long, short, and exit signals across an execution result.
#[derive(Debug, Clone, Default)]
pub struct SignalStats {
    /// Number of long entry signals
    pub long_entries: usize,
    /// Number of short entry signals
    pub short_entries: usize,
    /// Number of exit signals
    pub exits: usize,
}

impl SignalStats {
    /// Calculate signal statistics from an execution result.
    pub fn from_execution(strategy: &CompiledStrategy, result: &ExecutionResult) -> Self {
        let long_entries = strategy
            .entry_long
            .map(|node| {
                result
                    .get_output(node)
                    .iter()
                    .filter(|v| v.as_bool() == Some(true))
                    .count()
            })
            .unwrap_or(0);

        let short_entries = strategy
            .entry_short
            .map(|node| {
                result
                    .get_output(node)
                    .iter()
                    .filter(|v| v.as_bool() == Some(true))
                    .count()
            })
            .unwrap_or(0);

        let exits = strategy
            .exit_signals
            .iter()
            .map(|(_, node)| {
                result
                    .get_output(*node)
                    .iter()
                    .filter(|v| v.as_bool() == Some(true))
                    .count()
            })
            .sum();

        Self {
            long_entries,
            short_entries,
            exits,
        }
    }

    /// Total number of signals.
    pub fn total(&self) -> usize {
        self.long_entries + self.short_entries + self.exits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{Candle, Timestamp};
    use trdelnik_graph::Executor;
    use trdelnik_script::compile;

    fn test_series() -> CandleSeries<Timestamp> {
        let mut series = CandleSeries::new();
        // Down-up-down-up trajectory so a 3/5 SMA cross fires both
        // crossover (long) and crossunder (short) at least once.
        let prices = [
            110.0, 108.0, 106.0, 104.0, 102.0, 100.0, 102.0, 104.0, 106.0, 108.0,
            110.0, 108.0, 106.0, 104.0, 102.0, 100.0, 102.0, 104.0, 106.0, 108.0,
        ];
        for (i, &p) in prices.iter().enumerate() {
            series.push(Candle::new(
                Timestamp(i as i64 * 60_000),
                p - 0.5,
                p + 1.0,
                p - 1.0,
                p,
                1_000.0,
            ));
        }
        series
    }

    fn run(src: &str) -> (CompiledStrategy, ExecutionResult, CandleSeries<Timestamp>) {
        let strategy = compile(src).expect("compile");
        let series = test_series();
        // Re-compile for the executor so we can move the graph out.
        let compiled = compile(src).expect("compile (executor copy)");
        let mut executor = Executor::new(compiled.graph);
        let result = executor.process_series(&series);
        (strategy, result, series)
    }

    // ---------- ScriptPlotConfig ----------

    #[test]
    fn test_script_plot_config_from_strategy_one_per_plot() {
        let (strategy, _result, _series) = run(
            r#"
            let fast = sma(close, 3)
            let slow = sma(close, 5)
            plot fast
            plot slow
            "#,
        );
        let cfgs = ScriptPlotConfig::from_strategy(&strategy);
        assert_eq!(cfgs.len(), 2);
        assert_eq!(cfgs[0].index, 0);
        assert_eq!(cfgs[1].index, 1);
        assert_eq!(cfgs[0].name, "Plot 1");
        assert_eq!(cfgs[1].name, "Plot 2");
        assert!(cfgs[0].visible);
        // Initial color matches the script's color (which the from_strategy mirror
        // copies into both `color` and `script_color`).
        assert_eq!(cfgs[0].color, cfgs[0].script_color);
    }

    #[test]
    fn test_script_plot_config_no_plots_yields_empty() {
        let (strategy, _, _) = run(
            r#"
            let fast = sma(close, 3)
            let slow = sma(close, 5)
            entry long when crossover(fast, slow)
            "#,
        );
        assert!(ScriptPlotConfig::from_strategy(&strategy).is_empty());
    }

    // ---------- plots_to_overlays / plots_to_overlays_with_config ----------

    #[test]
    fn test_plots_to_overlays_returns_one_box_per_plot() {
        let (strategy, result, series) = run(
            r#"
            let fast = sma(close, 3)
            let slow = sma(close, 5)
            plot fast
            plot slow
            "#,
        );
        let overlays = plots_to_overlays::<Timestamp>(&strategy, &result, &series);
        assert_eq!(overlays.len(), 2);
        // Indicator IDs are unique per plot
        let id1 = overlays[0].indicator_id().to_string();
        let id2 = overlays[1].indicator_id().to_string();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_plots_to_overlays_no_plots_returns_empty() {
        let (strategy, result, series) = run(
            r#"
            let fast = sma(close, 3)
            entry long when fast > 100
            "#,
        );
        let overlays = plots_to_overlays::<Timestamp>(&strategy, &result, &series);
        assert!(overlays.is_empty());
    }

    #[test]
    fn test_plots_to_overlays_with_config_filters_invisible() {
        let (strategy, result, series) = run(
            r#"
            let fast = sma(close, 3)
            let slow = sma(close, 5)
            plot fast
            plot slow
            "#,
        );
        let mut cfgs = ScriptPlotConfig::from_strategy(&strategy);
        cfgs[0].visible = false;
        let overlays =
            plots_to_overlays_with_config::<Timestamp>(&strategy, &result, &series, Some(&cfgs));
        assert_eq!(overlays.len(), 1);
    }

    #[test]
    fn test_plots_to_overlays_with_config_none_equivalent_to_default() {
        let (strategy, result, series) = run(
            r#"
            let fast = sma(close, 3)
            plot fast
            "#,
        );
        let a = plots_to_overlays::<Timestamp>(&strategy, &result, &series);
        let b = plots_to_overlays_with_config::<Timestamp>(&strategy, &result, &series, None);
        assert_eq!(a.len(), b.len());
    }

    // ---------- signals_to_markers ----------

    #[test]
    fn test_signals_to_markers_emits_for_long_entries() {
        let (strategy, result, series) = run(
            r#"
            let fast = sma(close, 3)
            let slow = sma(close, 5)
            entry long when crossover(fast, slow)
            "#,
        );
        let markers = signals_to_markers::<Timestamp>(&strategy, &result, &series);
        // The chosen test_series rises monotonically for the first half, so a
        // crossover happens at least once.
        assert!(!markers.is_empty(), "expected at least one long-entry marker");
    }

    #[test]
    fn test_signals_to_markers_no_signals_yields_empty() {
        // Nothing crosses: both entries are constant false because we don't
        // declare them at all.
        let (strategy, result, series) = run(
            r#"
            let fast = sma(close, 3)
            plot fast
            "#,
        );
        let markers = signals_to_markers::<Timestamp>(&strategy, &result, &series);
        assert!(markers.is_empty());
    }

    #[test]
    fn test_signals_to_overlay_markers_matches_signals_to_markers() {
        let (strategy, result, series) = run(
            r#"
            let fast = sma(close, 3)
            let slow = sma(close, 5)
            entry long when crossover(fast, slow)
            entry short when crossunder(fast, slow)
            "#,
        );
        let a = signals_to_markers::<Timestamp>(&strategy, &result, &series);
        let b = signals_to_overlay_markers::<Timestamp>(&strategy, &result, &series);
        assert_eq!(a.len(), b.len());
    }

    // ---------- SignalStats ----------

    #[test]
    fn test_signal_stats_default_is_zero() {
        let s = SignalStats::default();
        assert_eq!(s.long_entries, 0);
        assert_eq!(s.short_entries, 0);
        assert_eq!(s.exits, 0);
        assert_eq!(s.total(), 0);
    }

    #[test]
    fn test_signal_stats_total_sums_components() {
        let s = SignalStats { long_entries: 2, short_entries: 3, exits: 4 };
        assert_eq!(s.total(), 9);
    }

    #[test]
    fn test_signal_stats_from_execution_counts_signals() {
        let (strategy, result, _series) = run(
            r#"
            let fast = sma(close, 3)
            let slow = sma(close, 5)
            entry long when crossover(fast, slow)
            entry short when crossunder(fast, slow)
            "#,
        );
        let stats = SignalStats::from_execution(&strategy, &result);
        // The series rises then falls, so we expect at least one of each.
        assert!(stats.long_entries >= 1);
        assert!(stats.short_entries >= 1);
        assert_eq!(stats.total(), stats.long_entries + stats.short_entries + stats.exits);
    }

    #[test]
    fn test_signal_stats_no_entries_yields_zero() {
        let (strategy, result, _series) = run(
            r#"
            let fast = sma(close, 3)
            plot fast
            "#,
        );
        let stats = SignalStats::from_execution(&strategy, &result);
        assert_eq!(stats.total(), 0);
    }
}
