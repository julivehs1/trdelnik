//! The `Plottable` trait and helper functions.
//!
//! `Plottable` replaces `ChartIndicator` as the bridge between
//! math indicators and chart rendering. Unlike `ChartIndicator`,
//! it returns `PlotData` (pure data) instead of `IndicatorOutput`
//! (which mixed data with panel configuration).

use trdelnik_core::{
    AxisCoordinate, CandleSeries, Color, IndicatorLine, PlotData,
};

use crate::indicator::Indicator;
use crate::{Ohlc, Ohlcv};

// Re-export PlotData for convenience
pub use trdelnik_core::PlotData as PlotDataExport;

/// Trait for indicators that can produce chart-ready plot data.
///
/// Panel configuration (hlines, y_range, placement) is NOT part of
/// this trait — that belongs on `PanelHandle` / `Panel`.
pub trait Plottable: Send + Sync {
    /// Compute plot data for the given candle series.
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X>;

    /// Display name (e.g. "RSI 14", "SMA 20")
    fn name(&self) -> String;

    /// Identifier for theming (e.g. "rsi", "sma")
    fn indicator_id(&self) -> &'static str;
}

// ============================================================================
// Helper functions — make Plottable impls trivial
// ============================================================================

/// Plot a single-line f64→f64 indicator against close prices.
pub fn plot_close<X, I>(
    indicator: &I,
    series: &CandleSeries<X>,
    name: &str,
    indicator_id: &str,
) -> PlotData<X>
where
    X: AxisCoordinate,
    I: Indicator<Input = f64, Output = f64> + Clone,
{
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let y_values: Vec<Option<f64>> = series.closes().iter().map(|&close| ind.next(close)).collect();

    let mut data = PlotData::new(indicator_id);
    data.add_line(IndicatorLine::from_xy(name, indicator_id, &x_values, &y_values));
    data
}

/// Plot a single-line Ohlc→f64 indicator.
pub fn plot_ohlc<X, I>(
    indicator: &I,
    series: &CandleSeries<X>,
    name: &str,
    indicator_id: &str,
) -> PlotData<X>
where
    X: AxisCoordinate,
    I: Indicator<Input = Ohlc, Output = f64> + Clone,
{
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let y_values: Vec<Option<f64>> = series
        .candles()
        .iter()
        .map(|c| ind.next(Ohlc::new(c.high, c.low, c.close)))
        .collect();

    let mut data = PlotData::new(indicator_id);
    data.add_line(IndicatorLine::from_xy(name, indicator_id, &x_values, &y_values));
    data
}

/// Plot a single-line Ohlcv→f64 indicator.
pub fn plot_ohlcv<X, I>(
    indicator: &I,
    series: &CandleSeries<X>,
    name: &str,
    indicator_id: &str,
) -> PlotData<X>
where
    X: AxisCoordinate,
    I: Indicator<Input = Ohlcv, Output = f64> + Clone,
{
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let y_values: Vec<Option<f64>> = series
        .candles()
        .iter()
        .map(|c| ind.next(Ohlcv::new(c.high, c.low, c.close, c.volume)))
        .collect();

    let mut data = PlotData::new(indicator_id);
    data.add_line(IndicatorLine::from_xy(name, indicator_id, &x_values, &y_values));
    data
}

/// Compute a multi-output f64 indicator, returning raw (x, outputs) for custom line building.
pub fn compute_close<X, I, O>(
    indicator: &I,
    series: &CandleSeries<X>,
) -> (Vec<X>, Vec<Option<O>>)
where
    X: AxisCoordinate,
    I: Indicator<Input = f64, Output = O> + Clone,
    O: Copy,
{
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let values: Vec<Option<O>> = series.closes().iter().map(|&close| ind.next(close)).collect();
    (x_values, values)
}

/// Compute a multi-output Ohlc indicator, returning raw (x, outputs).
pub fn compute_ohlc<X, I, O>(
    indicator: &I,
    series: &CandleSeries<X>,
) -> (Vec<X>, Vec<Option<O>>)
where
    X: AxisCoordinate,
    I: Indicator<Input = Ohlc, Output = O> + Clone,
    O: Copy,
{
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let values: Vec<Option<O>> = series
        .candles()
        .iter()
        .map(|c| ind.next(Ohlc::new(c.high, c.low, c.close)))
        .collect();
    (x_values, values)
}

/// Compute feeding (high+low)/2 for indicators like Fisher Transform.
pub fn compute_hl2<X, I, O>(
    indicator: &I,
    series: &CandleSeries<X>,
) -> (Vec<X>, Vec<Option<O>>)
where
    X: AxisCoordinate,
    I: Indicator<Input = f64, Output = O> + Clone,
    O: Copy,
{
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let values: Vec<Option<O>> = series
        .candles()
        .iter()
        .map(|c| ind.next((c.high + c.low) / 2.0))
        .collect();
    (x_values, values)
}

// ============================================================================
// Plottable implementations for all indicators
// ============================================================================

use crate::{
    Atr, Bollinger, Cci, Chandelier, ChandeKrollStop, EfficiencyRatio, Ema, FisherTransform,
    Keltner, Macd, Mfi, Obv, Ppo, Roc, Rsi, Sma, StdDev, Stochastic, Wma,
};

/// Implement `Plottable` for an indicator that produces a single line and
/// has a `period()` getter. The display label is `"$label $period"`.
macro_rules! impl_plottable_period {
    ($plot_fn:ident, $struct:ty, $label:literal, $id:literal) => {
        impl Plottable for $struct {
            fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
                $plot_fn(self, series, &self.name(), self.indicator_id())
            }
            fn name(&self) -> String {
                format!(concat!($label, " {}"), self.period())
            }
            fn indicator_id(&self) -> &'static str {
                $id
            }
        }
    };
}

/// Implement `Plottable` for an indicator with no parameters in its label.
macro_rules! impl_plottable_simple {
    ($plot_fn:ident, $struct:ty, $label:literal, $id:literal) => {
        impl Plottable for $struct {
            fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
                $plot_fn(self, series, &self.name(), self.indicator_id())
            }
            fn name(&self) -> String {
                $label.to_string()
            }
            fn indicator_id(&self) -> &'static str {
                $id
            }
        }
    };
}

// --- Single-line f64→f64 ---
impl_plottable_period!(plot_close, Sma, "SMA", "sma");
impl_plottable_period!(plot_close, Ema, "EMA", "ema");
impl_plottable_period!(plot_close, Wma, "WMA", "wma");
impl_plottable_period!(plot_close, Rsi, "RSI", "rsi");
impl_plottable_period!(plot_close, Roc, "ROC", "roc");
impl_plottable_period!(plot_close, StdDev, "StdDev", "std_dev");
impl_plottable_period!(plot_close, EfficiencyRatio, "ER", "efficiency_ratio");

// --- Multi-output f64 indicators ---

impl Plottable for Bollinger {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
        let (x_values, values) = compute_close(self, series);

        let middle: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.middle)).collect();
        let upper: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.upper)).collect();
        let lower: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.lower)).collect();

        let mut data = PlotData::new("bollinger");
        data.add_line(IndicatorLine::from_xy("Middle", "bb_middle", &x_values, &middle));
        data.add_line(IndicatorLine::from_xy("Upper", "bb_upper", &x_values, &upper));
        data.add_line(IndicatorLine::from_xy("Lower", "bb_lower", &x_values, &lower));
        data
    }
    fn name(&self) -> String {
        format!("BB({}, {})", self.period(), self.std_dev_mult())
    }
    fn indicator_id(&self) -> &'static str {
        "bollinger"
    }
}

impl Plottable for Macd {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
        let (x_values, values) = compute_close(self, series);

        let macd_line: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.macd)).collect();
        let signal: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.signal)).collect();
        let histogram: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.histogram)).collect();

        let mut data = PlotData::new("macd");
        data.add_line(IndicatorLine::from_xy("MACD", "macd_line", &x_values, &macd_line));
        data.add_line(IndicatorLine::from_xy("Signal", "macd_signal", &x_values, &signal));

        let pos_color = Color::rgb(0, 180, 0);
        let neg_color = Color::rgb(180, 0, 0);
        data.set_histogram_pos_neg(&x_values, &histogram, pos_color, neg_color);

        data
    }
    fn name(&self) -> String {
        format!("MACD({}, {}, {})", self.fast_period(), self.slow_period(), self.signal_period())
    }
    fn indicator_id(&self) -> &'static str {
        "macd"
    }
}

impl Plottable for Ppo {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
        let (x_values, values) = compute_close(self, series);

        let ppo_line: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.ppo)).collect();
        let signal: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.signal)).collect();
        let histogram: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.histogram)).collect();

        let mut data = PlotData::new("ppo");
        data.add_line(IndicatorLine::from_xy("PPO", "ppo_line", &x_values, &ppo_line));
        data.add_line(IndicatorLine::from_xy("Signal", "ppo_signal", &x_values, &signal));

        let pos_color = Color::rgb(0, 180, 0);
        let neg_color = Color::rgb(180, 0, 0);
        data.set_histogram_pos_neg(&x_values, &histogram, pos_color, neg_color);

        data
    }
    fn name(&self) -> String {
        format!("PPO({}, {}, {})", self.fast_period(), self.slow_period(), self.signal_period())
    }
    fn indicator_id(&self) -> &'static str {
        "ppo"
    }
}

// --- Single-line Ohlc→f64 ---
impl_plottable_period!(plot_ohlc, Atr, "ATR", "atr");
impl_plottable_period!(plot_ohlc, Cci, "CCI", "cci");

impl Plottable for Stochastic {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
        let (x_values, values) = compute_ohlc(self, series);

        let k_values: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.k)).collect();
        let d_values: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.d)).collect();

        let mut data = PlotData::new("stochastic");
        data.add_line(IndicatorLine::from_xy("%K", "stoch_k", &x_values, &k_values));
        data.add_line(IndicatorLine::from_xy("%D", "stoch_d", &x_values, &d_values));
        data
    }
    fn name(&self) -> String {
        format!("Stoch({}, {})", self.k_period(), self.d_period())
    }
    fn indicator_id(&self) -> &'static str {
        "stochastic"
    }
}

impl Plottable for Keltner {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
        let (x_values, values) = compute_ohlc(self, series);

        let middle: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.middle)).collect();
        let upper: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.upper)).collect();
        let lower: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.lower)).collect();

        let mut data = PlotData::new("keltner");
        data.add_line(IndicatorLine::from_xy("Middle", "kc_middle", &x_values, &middle));
        data.add_line(IndicatorLine::from_xy("Upper", "kc_upper", &x_values, &upper));
        data.add_line(IndicatorLine::from_xy("Lower", "kc_lower", &x_values, &lower));
        data
    }
    fn name(&self) -> String {
        format!("KC({}, {})", self.ema_period(), self.atr_mult())
    }
    fn indicator_id(&self) -> &'static str {
        "keltner"
    }
}

impl Plottable for Chandelier {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
        let (x_values, values) = compute_ohlc(self, series);

        let long_exit: Vec<Option<f64>> = values.iter().map(|v| v.map(|c| c.long_exit)).collect();
        let short_exit: Vec<Option<f64>> = values.iter().map(|v| v.map(|c| c.short_exit)).collect();

        let mut data = PlotData::new("chandelier");
        data.add_line(IndicatorLine::from_xy("Long Exit", "ce_long", &x_values, &long_exit));
        data.add_line(IndicatorLine::from_xy("Short Exit", "ce_short", &x_values, &short_exit));
        data
    }
    fn name(&self) -> String {
        format!("CE({}, {})", self.period(), self.atr_mult())
    }
    fn indicator_id(&self) -> &'static str {
        "chandelier"
    }
}

impl Plottable for ChandeKrollStop {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
        let (x_values, values) = compute_ohlc(self, series);

        let stop_long: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.stop_long)).collect();
        let stop_short: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.stop_short)).collect();

        let mut data = PlotData::new("chande_kroll");
        data.add_line(IndicatorLine::from_xy("Stop Long", "ck_stop_long", &x_values, &stop_long));
        data.add_line(IndicatorLine::from_xy("Stop Short", "ck_stop_short", &x_values, &stop_short));
        data
    }
    fn name(&self) -> String {
        format!("CK({}, {}, {})", self.atr_period(), self.stop_period(), self.atr_mult())
    }
    fn indicator_id(&self) -> &'static str {
        "chande_kroll"
    }
}

// --- Single-line Ohlcv→f64 ---
impl_plottable_simple!(plot_ohlcv, Obv, "OBV", "obv");
impl_plottable_period!(plot_ohlcv, Mfi, "MFI", "mfi");

// --- Fisher Transform (uses HL2 input) ---

impl Plottable for FisherTransform {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> PlotData<X> {
        let (x_values, values) = compute_hl2(self, series);

        let fisher: Vec<Option<f64>> = values.iter().map(|v| v.map(|f| f.fisher)).collect();
        let trigger: Vec<Option<f64>> = values.iter().map(|v| v.map(|f| f.trigger)).collect();

        let mut data = PlotData::new("fisher_transform");
        data.add_line(IndicatorLine::from_xy("Fisher", "fisher", &x_values, &fisher));
        data.add_line(IndicatorLine::from_xy("Trigger", "fisher_trigger", &x_values, &trigger));
        data
    }
    fn name(&self) -> String {
        format!("Fisher({})", self.period())
    }
    fn indicator_id(&self) -> &'static str {
        "fisher_transform"
    }
}
