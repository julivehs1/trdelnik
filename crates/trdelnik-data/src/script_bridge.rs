//! Bridge between trdelnik-script and the chart system
//!
//! This module provides functions to convert script execution results
//! into chart-ready data structures like PlotData with markers.

use trdelnik_core::{
    AxisCoordinate, CandleSeries, Color, IndicatorLine, IndicatorMarker, PlotData,
};
use trdelnik_graph::ExecutionResult;
use trdelnik_script::ast::ExitTarget;
use trdelnik_script::CompiledStrategy;

/// Configuration for a single plot from a script (for UI customization)
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
    /// Create configs from a compiled strategy
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

/// Convert a compiled strategy's plots to PlotData
pub fn plots_to_overlays<X: AxisCoordinate>(
    strategy: &CompiledStrategy,
    result: &ExecutionResult,
    series: &CandleSeries<X>,
) -> Vec<PlotData<X>> {
    plots_to_overlays_with_config(strategy, result, series, None)
}

/// Convert plots with custom configurations
pub fn plots_to_overlays_with_config<X: AxisCoordinate>(
    strategy: &CompiledStrategy,
    result: &ExecutionResult,
    series: &CandleSeries<X>,
    configs: Option<&[ScriptPlotConfig]>,
) -> Vec<PlotData<X>> {
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

            let mut data = PlotData::new(name.clone(), &indicator_id);

            let line = IndicatorLine::from_xy(&name, &indicator_id, &x_values, &values)
                .with_color(color);

            data.add_line(line);
            Some(data)
        })
        .collect()
}

/// Convert entry/exit signals to IndicatorMarkers
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

/// Create a PlotData for signals (as an overlay with markers only)
pub fn create_signals_overlay<X: AxisCoordinate>(
    strategy: &CompiledStrategy,
    result: &ExecutionResult,
    series: &CandleSeries<X>,
) -> PlotData<X> {
    let mut data = PlotData::new("Signals", "script_signals");
    let markers = signals_to_markers(strategy, result, series);
    data.add_markers(markers);
    data
}

/// Statistics about signal counts
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
    /// Calculate signal statistics from execution result
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

    /// Total number of signals
    pub fn total(&self) -> usize {
        self.long_entries + self.short_entries + self.exits
    }
}
