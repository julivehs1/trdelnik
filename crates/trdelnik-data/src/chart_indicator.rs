//! Bridge between stateful indicators and chart rendering.
//!
//! This module provides the `ChartIndicator` trait that converts stateful
//! indicators into `IndicatorOutput` for chart rendering.

use trdelnik_core::{AxisCoordinate, CandleSeries, IndicatorLine, IndicatorOutput, Placement};
use trdelnik_indicators::{
    indicator::Indicator as StatefulIndicator,
    Atr, Bollinger, BollingerValue, Cci, Chandelier, ChandelierValue,
    EfficiencyRatio, Ema, Keltner, KeltnerValue, Macd, MacdValue,
    Mfi, Obv, Ohlc, Ohlcv, Ppo, PpoValue, Roc, Rsi, Sma, StdDev,
    Stochastic, StochasticValue, Wma,
};

/// Trait for indicators that can be rendered on a chart.
pub trait ChartIndicator: Send + Sync {
    /// Compute the indicator output for the given candle series.
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X>;

    /// Get the display name of this indicator.
    fn name(&self) -> String;

    /// Get the indicator identifier for theming.
    fn indicator_id(&self) -> &'static str;

    /// Get the default placement for this indicator.
    fn default_placement(&self) -> Placement;
}

// ============================================================================
// Helper functions
// ============================================================================

fn compute_f64_indicator<X: AxisCoordinate, I: StatefulIndicator<Input = f64, Output = f64> + Clone>(
    indicator: &I,
    series: &CandleSeries<X>,
    name: &str,
    line_id: &str,
) -> IndicatorOutput<X> {
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let y_values: Vec<Option<f64>> = series.closes().iter().map(|&close| ind.next(close)).collect();

    let mut output = IndicatorOutput::overlay(name, line_id);
    output.add_line(IndicatorLine::from_xy(name, line_id, &x_values, &y_values));
    output
}

fn compute_ohlc_indicator<X, I, O, F>(
    indicator: &I,
    series: &CandleSeries<X>,
    name: &str,
    indicator_id: &str,
    placement: Placement,
    add_lines: F,
) -> IndicatorOutput<X>
where
    X: AxisCoordinate,
    I: StatefulIndicator<Input = Ohlc, Output = O> + Clone,
    O: Copy,
    F: FnOnce(&mut IndicatorOutput<X>, &[X], &[Option<O>]),
{
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let y_values: Vec<Option<O>> = series
        .candles()
        .iter()
        .map(|c| ind.next(Ohlc::new(c.high, c.low, c.close)))
        .collect();

    let mut output = IndicatorOutput::new(name, indicator_id, placement);
    add_lines(&mut output, &x_values, &y_values);
    output
}

// ============================================================================
// Implementations for f64 -> f64 indicators (overlays)
// ============================================================================

impl ChartIndicator for Sma {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        compute_f64_indicator(self, series, &self.name(), self.indicator_id())
    }

    fn name(&self) -> String {
        format!("SMA {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "sma"
    }

    fn default_placement(&self) -> Placement {
        Placement::Overlay
    }
}

impl ChartIndicator for Ema {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        compute_f64_indicator(self, series, &self.name(), self.indicator_id())
    }

    fn name(&self) -> String {
        format!("EMA {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "ema"
    }

    fn default_placement(&self) -> Placement {
        Placement::Overlay
    }
}

impl ChartIndicator for Wma {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        compute_f64_indicator(self, series, &self.name(), self.indicator_id())
    }

    fn name(&self) -> String {
        format!("WMA {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "wma"
    }

    fn default_placement(&self) -> Placement {
        Placement::Overlay
    }
}

// ============================================================================
// Implementations for f64 -> f64 indicators (panels)
// ============================================================================

impl ChartIndicator for Rsi {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut output = compute_f64_indicator(self, series, &self.name(), self.indicator_id());
        output.default_placement = Placement::Panel;
        output.set_reference_lines(vec![30.0, 70.0]);
        output.set_y_range((0.0, 100.0));
        output
    }

    fn name(&self) -> String {
        format!("RSI {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "rsi"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

impl ChartIndicator for Roc {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut output = compute_f64_indicator(self, series, &self.name(), self.indicator_id());
        output.default_placement = Placement::Panel;
        output.set_reference_lines(vec![0.0]);
        output
    }

    fn name(&self) -> String {
        format!("ROC {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "roc"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

impl ChartIndicator for StdDev {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut output = compute_f64_indicator(self, series, &self.name(), self.indicator_id());
        output.default_placement = Placement::Panel;
        output
    }

    fn name(&self) -> String {
        format!("StdDev {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "std_dev"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

impl ChartIndicator for EfficiencyRatio {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut output = compute_f64_indicator(self, series, &self.name(), self.indicator_id());
        output.default_placement = Placement::Panel;
        output.set_y_range((0.0, 1.0));
        output
    }

    fn name(&self) -> String {
        format!("ER {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "efficiency_ratio"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

// ============================================================================
// Bollinger Bands
// ============================================================================

impl ChartIndicator for Bollinger {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut ind = self.clone();
        let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
        let values: Vec<Option<BollingerValue>> = series.closes().iter().map(|&c| ind.next(c)).collect();

        let middle: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.middle)).collect();
        let upper: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.upper)).collect();
        let lower: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.lower)).collect();

        let mut output = IndicatorOutput::overlay(&self.name(), "bollinger");
        output.add_line(IndicatorLine::from_xy("Middle", "bb_middle", &x_values, &middle));
        output.add_line(IndicatorLine::from_xy("Upper", "bb_upper", &x_values, &upper));
        output.add_line(IndicatorLine::from_xy("Lower", "bb_lower", &x_values, &lower));
        output
    }

    fn name(&self) -> String {
        format!("BB({}, {})", self.period(), self.std_dev_mult())
    }

    fn indicator_id(&self) -> &'static str {
        "bollinger"
    }

    fn default_placement(&self) -> Placement {
        Placement::Overlay
    }
}

// ============================================================================
// MACD
// ============================================================================

impl ChartIndicator for Macd {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut ind = self.clone();
        let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
        let values: Vec<Option<MacdValue>> = series.closes().iter().map(|&c| ind.next(c)).collect();

        let macd_line: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.macd)).collect();
        let signal: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.signal)).collect();
        let histogram: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.histogram)).collect();

        let mut output = IndicatorOutput::panel(&self.name(), "macd");
        output.add_line(IndicatorLine::from_xy("MACD", "macd_line", &x_values, &macd_line));
        output.add_line(IndicatorLine::from_xy("Signal", "macd_signal", &x_values, &signal));
        output.set_reference_lines(vec![0.0]);

        // Add histogram with positive/negative coloring
        let pos_color = trdelnik_core::Color::rgb(0, 180, 0);
        let neg_color = trdelnik_core::Color::rgb(180, 0, 0);
        output.set_histogram_pos_neg(&x_values, &histogram, pos_color, neg_color);

        output
    }

    fn name(&self) -> String {
        format!("MACD({}, {}, {})", self.fast_period(), self.slow_period(), self.signal_period())
    }

    fn indicator_id(&self) -> &'static str {
        "macd"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

// ============================================================================
// PPO
// ============================================================================

impl ChartIndicator for Ppo {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut ind = self.clone();
        let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
        let values: Vec<Option<PpoValue>> = series.closes().iter().map(|&c| ind.next(c)).collect();

        let ppo_line: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.ppo)).collect();
        let signal: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.signal)).collect();
        let histogram: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.histogram)).collect();

        let mut output = IndicatorOutput::panel(&self.name(), "ppo");
        output.add_line(IndicatorLine::from_xy("PPO", "ppo_line", &x_values, &ppo_line));
        output.add_line(IndicatorLine::from_xy("Signal", "ppo_signal", &x_values, &signal));
        output.set_reference_lines(vec![0.0]);

        let pos_color = trdelnik_core::Color::rgb(0, 180, 0);
        let neg_color = trdelnik_core::Color::rgb(180, 0, 0);
        output.set_histogram_pos_neg(&x_values, &histogram, pos_color, neg_color);

        output
    }

    fn name(&self) -> String {
        format!("PPO({}, {}, {})", self.fast_period(), self.slow_period(), self.signal_period())
    }

    fn indicator_id(&self) -> &'static str {
        "ppo"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

// ============================================================================
// OHLC-Input Indicators
// ============================================================================

impl ChartIndicator for Atr {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        compute_ohlc_indicator(
            self,
            series,
            &self.name(),
            "atr",
            Placement::Panel,
            |output, x_values, y_values| {
                let values: Vec<Option<f64>> = y_values.iter().copied().collect();
                output.add_line(IndicatorLine::from_xy(&output.name, "atr", x_values, &values));
            },
        )
    }

    fn name(&self) -> String {
        format!("ATR {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "atr"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

impl ChartIndicator for Cci {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut output = compute_ohlc_indicator(
            self,
            series,
            &self.name(),
            "cci",
            Placement::Panel,
            |output, x_values, y_values| {
                let values: Vec<Option<f64>> = y_values.iter().copied().collect();
                output.add_line(IndicatorLine::from_xy(&output.name, "cci", x_values, &values));
            },
        );
        output.set_reference_lines(vec![-100.0, 0.0, 100.0]);
        output
    }

    fn name(&self) -> String {
        format!("CCI {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "cci"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

impl ChartIndicator for Stochastic {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut ind = self.clone();
        let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
        let values: Vec<Option<StochasticValue>> = series
            .candles()
            .iter()
            .map(|c| ind.next(Ohlc::new(c.high, c.low, c.close)))
            .collect();

        let k_values: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.k)).collect();
        let d_values: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.d)).collect();

        let mut output = IndicatorOutput::panel(&self.name(), "stochastic");
        output.add_line(IndicatorLine::from_xy("%K", "stoch_k", &x_values, &k_values));
        output.add_line(IndicatorLine::from_xy("%D", "stoch_d", &x_values, &d_values));
        output.set_reference_lines(vec![20.0, 80.0]);
        output.set_y_range((0.0, 100.0));
        output
    }

    fn name(&self) -> String {
        format!("Stoch({}, {})", self.k_period(), self.d_period())
    }

    fn indicator_id(&self) -> &'static str {
        "stochastic"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

impl ChartIndicator for Keltner {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut ind = self.clone();
        let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
        let values: Vec<Option<KeltnerValue>> = series
            .candles()
            .iter()
            .map(|c| ind.next(Ohlc::new(c.high, c.low, c.close)))
            .collect();

        let middle: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.middle)).collect();
        let upper: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.upper)).collect();
        let lower: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.lower)).collect();

        let mut output = IndicatorOutput::overlay(&self.name(), "keltner");
        output.add_line(IndicatorLine::from_xy("Middle", "kc_middle", &x_values, &middle));
        output.add_line(IndicatorLine::from_xy("Upper", "kc_upper", &x_values, &upper));
        output.add_line(IndicatorLine::from_xy("Lower", "kc_lower", &x_values, &lower));
        output
    }

    fn name(&self) -> String {
        format!("KC({}, {})", self.ema_period(), self.atr_mult())
    }

    fn indicator_id(&self) -> &'static str {
        "keltner"
    }

    fn default_placement(&self) -> Placement {
        Placement::Overlay
    }
}

impl ChartIndicator for Chandelier {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut ind = self.clone();
        let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
        let values: Vec<Option<ChandelierValue>> = series
            .candles()
            .iter()
            .map(|c| ind.next(Ohlc::new(c.high, c.low, c.close)))
            .collect();

        let long_exit: Vec<Option<f64>> = values.iter().map(|v| v.map(|c| c.long_exit)).collect();
        let short_exit: Vec<Option<f64>> = values.iter().map(|v| v.map(|c| c.short_exit)).collect();

        let mut output = IndicatorOutput::overlay(&self.name(), "chandelier");
        output.add_line(IndicatorLine::from_xy("Long Exit", "ce_long", &x_values, &long_exit));
        output.add_line(IndicatorLine::from_xy("Short Exit", "ce_short", &x_values, &short_exit));
        output
    }

    fn name(&self) -> String {
        format!("CE({}, {})", self.period(), self.atr_mult())
    }

    fn indicator_id(&self) -> &'static str {
        "chandelier"
    }

    fn default_placement(&self) -> Placement {
        Placement::Overlay
    }
}

// ============================================================================
// OHLCV-Input Indicators
// ============================================================================

impl ChartIndicator for Obv {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut ind = self.clone();
        let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
        let y_values: Vec<Option<f64>> = series
            .candles()
            .iter()
            .map(|c| ind.next(Ohlcv::new(c.high, c.low, c.close, c.volume)))
            .collect();

        let mut output = IndicatorOutput::panel("OBV", "obv");
        output.add_line(IndicatorLine::from_xy("OBV", "obv", &x_values, &y_values));
        output
    }

    fn name(&self) -> String {
        "OBV".to_string()
    }

    fn indicator_id(&self) -> &'static str {
        "obv"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}

impl ChartIndicator for Mfi {
    fn compute<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> IndicatorOutput<X> {
        let mut ind = self.clone();
        let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
        let y_values: Vec<Option<f64>> = series
            .candles()
            .iter()
            .map(|c| ind.next(Ohlcv::new(c.high, c.low, c.close, c.volume)))
            .collect();

        let mut output = IndicatorOutput::panel(&self.name(), "mfi");
        output.add_line(IndicatorLine::from_xy(&output.name, "mfi", &x_values, &y_values));
        output.set_reference_lines(vec![20.0, 80.0]);
        output.set_y_range((0.0, 100.0));
        output
    }

    fn name(&self) -> String {
        format!("MFI {}", self.period())
    }

    fn indicator_id(&self) -> &'static str {
        "mfi"
    }

    fn default_placement(&self) -> Placement {
        Placement::Panel
    }
}
