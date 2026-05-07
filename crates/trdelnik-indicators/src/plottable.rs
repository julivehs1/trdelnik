//! The `Plottable` trait and helper functions.
//!
//! `Plottable` is the bridge between math indicators and chart rendering.
//! It returns a boxed `Plot` trait object, so any indicator can produce any
//! visualisation type (lines, histogram, or a custom `Plot` impl).

use trdelnik_core::{AxisCoordinate, CandleSeries, Color, IndicatorLine};
use trdelnik_render::{Plot, StandardPlot};

use crate::indicator::Indicator;
use crate::{Ohlc, Ohlcv};

/// Trait for indicators that can produce chart-ready plot data.
///
/// Returns a boxed `Plot` so each indicator can pick whatever visualisation
/// fits — `StandardPlot` for lines + histogram, or a custom `Plot` impl.
/// Panel configuration (hlines, y_range, placement) is NOT part of this
/// trait — that belongs on `PanelHandle` / `Panel`.
pub trait Plottable: Send + Sync {
    /// Compute the plot for the given candle series.
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>>;

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
) -> Box<dyn Plot<X>>
where
    X: AxisCoordinate,
    I: Indicator<Input = f64, Output = f64> + Clone,
{
    let mut ind = indicator.clone();
    let x_values: Vec<X> = series.candles().iter().map(|c| c.x).collect();
    let y_values: Vec<Option<f64>> = series.closes().iter().map(|&close| ind.next(close)).collect();

    let mut data = StandardPlot::new(indicator_id);
    data.add_line(IndicatorLine::from_xy(name, indicator_id, &x_values, &y_values));
    Box::new(data)
}

/// Plot a single-line Ohlc→f64 indicator.
pub fn plot_ohlc<X, I>(
    indicator: &I,
    series: &CandleSeries<X>,
    name: &str,
    indicator_id: &str,
) -> Box<dyn Plot<X>>
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

    let mut data = StandardPlot::new(indicator_id);
    data.add_line(IndicatorLine::from_xy(name, indicator_id, &x_values, &y_values));
    Box::new(data)
}

/// Plot a single-line Ohlcv→f64 indicator.
pub fn plot_ohlcv<X, I>(
    indicator: &I,
    series: &CandleSeries<X>,
    name: &str,
    indicator_id: &str,
) -> Box<dyn Plot<X>>
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

    let mut data = StandardPlot::new(indicator_id);
    data.add_line(IndicatorLine::from_xy(name, indicator_id, &x_values, &y_values));
    Box::new(data)
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
            fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
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
            fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
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
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
        let (x_values, values) = compute_close(self, series);

        let middle: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.middle)).collect();
        let upper: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.upper)).collect();
        let lower: Vec<Option<f64>> = values.iter().map(|v| v.map(|b| b.lower)).collect();

        let mut data = StandardPlot::new("bollinger");
        data.add_line(IndicatorLine::from_xy("Middle", "bb_middle", &x_values, &middle));
        data.add_line(IndicatorLine::from_xy("Upper", "bb_upper", &x_values, &upper));
        data.add_line(IndicatorLine::from_xy("Lower", "bb_lower", &x_values, &lower));
        Box::new(data)
    }
    fn name(&self) -> String {
        format!("BB({}, {})", self.period(), self.std_dev_mult())
    }
    fn indicator_id(&self) -> &'static str {
        "bollinger"
    }
}

impl Plottable for Macd {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
        let (x_values, values) = compute_close(self, series);

        let macd_line: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.macd)).collect();
        let signal: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.signal)).collect();
        let histogram: Vec<Option<f64>> = values.iter().map(|v| v.map(|m| m.histogram)).collect();

        let mut data = StandardPlot::new("macd");
        data.add_line(IndicatorLine::from_xy("MACD", "macd_line", &x_values, &macd_line));
        data.add_line(IndicatorLine::from_xy("Signal", "macd_signal", &x_values, &signal));

        let pos_color = Color::rgb(0, 180, 0);
        let neg_color = Color::rgb(180, 0, 0);
        data.set_histogram_pos_neg(&x_values, &histogram, pos_color, neg_color);

        Box::new(data)
    }
    fn name(&self) -> String {
        format!("MACD({}, {}, {})", self.fast_period(), self.slow_period(), self.signal_period())
    }
    fn indicator_id(&self) -> &'static str {
        "macd"
    }
}

impl Plottable for Ppo {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
        let (x_values, values) = compute_close(self, series);

        let ppo_line: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.ppo)).collect();
        let signal: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.signal)).collect();
        let histogram: Vec<Option<f64>> = values.iter().map(|v| v.map(|p| p.histogram)).collect();

        let mut data = StandardPlot::new("ppo");
        data.add_line(IndicatorLine::from_xy("PPO", "ppo_line", &x_values, &ppo_line));
        data.add_line(IndicatorLine::from_xy("Signal", "ppo_signal", &x_values, &signal));

        let pos_color = Color::rgb(0, 180, 0);
        let neg_color = Color::rgb(180, 0, 0);
        data.set_histogram_pos_neg(&x_values, &histogram, pos_color, neg_color);

        Box::new(data)
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
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
        let (x_values, values) = compute_ohlc(self, series);

        let k_values: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.k)).collect();
        let d_values: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.d)).collect();

        let mut data = StandardPlot::new("stochastic");
        data.add_line(IndicatorLine::from_xy("%K", "stoch_k", &x_values, &k_values));
        data.add_line(IndicatorLine::from_xy("%D", "stoch_d", &x_values, &d_values));
        Box::new(data)
    }
    fn name(&self) -> String {
        format!("Stoch({}, {})", self.k_period(), self.d_period())
    }
    fn indicator_id(&self) -> &'static str {
        "stochastic"
    }
}

impl Plottable for Keltner {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
        let (x_values, values) = compute_ohlc(self, series);

        let middle: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.middle)).collect();
        let upper: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.upper)).collect();
        let lower: Vec<Option<f64>> = values.iter().map(|v| v.map(|k| k.lower)).collect();

        let mut data = StandardPlot::new("keltner");
        data.add_line(IndicatorLine::from_xy("Middle", "kc_middle", &x_values, &middle));
        data.add_line(IndicatorLine::from_xy("Upper", "kc_upper", &x_values, &upper));
        data.add_line(IndicatorLine::from_xy("Lower", "kc_lower", &x_values, &lower));
        Box::new(data)
    }
    fn name(&self) -> String {
        format!("KC({}, {})", self.ema_period(), self.atr_mult())
    }
    fn indicator_id(&self) -> &'static str {
        "keltner"
    }
}

impl Plottable for Chandelier {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
        let (x_values, values) = compute_ohlc(self, series);

        let long_exit: Vec<Option<f64>> = values.iter().map(|v| v.map(|c| c.long_exit)).collect();
        let short_exit: Vec<Option<f64>> = values.iter().map(|v| v.map(|c| c.short_exit)).collect();

        let mut data = StandardPlot::new("chandelier");
        data.add_line(IndicatorLine::from_xy("Long Exit", "ce_long", &x_values, &long_exit));
        data.add_line(IndicatorLine::from_xy("Short Exit", "ce_short", &x_values, &short_exit));
        Box::new(data)
    }
    fn name(&self) -> String {
        format!("CE({}, {})", self.period(), self.atr_mult())
    }
    fn indicator_id(&self) -> &'static str {
        "chandelier"
    }
}

impl Plottable for ChandeKrollStop {
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
        let (x_values, values) = compute_ohlc(self, series);

        let stop_long: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.stop_long)).collect();
        let stop_short: Vec<Option<f64>> = values.iter().map(|v| v.map(|s| s.stop_short)).collect();

        let mut data = StandardPlot::new("chande_kroll");
        data.add_line(IndicatorLine::from_xy("Stop Long", "ck_stop_long", &x_values, &stop_long));
        data.add_line(IndicatorLine::from_xy("Stop Short", "ck_stop_short", &x_values, &stop_short));
        Box::new(data)
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
    fn plot<X: AxisCoordinate>(&self, series: &CandleSeries<X>) -> Box<dyn Plot<X>> {
        let (x_values, values) = compute_hl2(self, series);

        let fisher: Vec<Option<f64>> = values.iter().map(|v| v.map(|f| f.fisher)).collect();
        let trigger: Vec<Option<f64>> = values.iter().map(|v| v.map(|f| f.trigger)).collect();

        let mut data = StandardPlot::new("fisher_transform");
        data.add_line(IndicatorLine::from_xy("Fisher", "fisher", &x_values, &fisher));
        data.add_line(IndicatorLine::from_xy("Trigger", "fisher_trigger", &x_values, &trigger));
        Box::new(data)
    }
    fn name(&self) -> String {
        format!("Fisher({})", self.period())
    }
    fn indicator_id(&self) -> &'static str {
        "fisher_transform"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{Candle, Index};

    fn series(n: usize) -> CandleSeries<Index> {
        let mut s = CandleSeries::<Index>::new();
        for i in 0..n {
            let p = 100.0 + ((i as f64) * 0.1).sin() * 5.0 + (i as f64) * 0.2;
            s.push(Candle::new(Index(i), p - 0.5, p + 1.0, p - 1.0, p, 1000.0 + i as f64));
        }
        s
    }

    // ---------- compute_close / compute_ohlc / compute_hl2 ----------

    #[test]
    fn test_compute_close_returns_paired_xs_and_values() {
        let s = series(40);
        let sma = Sma::new(5);
        let (xs, vs) = compute_close(&sma, &s);
        assert_eq!(xs.len(), s.len());
        assert_eq!(vs.len(), s.len());
        // Warmup: first 4 values are None, then we should have values
        assert!(vs[0].is_none());
        assert!(vs[5].is_some());
    }

    #[test]
    fn test_compute_ohlc_uses_high_low_close() {
        let s = series(30);
        let atr = Atr::new(5);
        let (xs, vs) = compute_ohlc(&atr, &s);
        assert_eq!(xs.len(), s.len());
        assert!(vs.iter().any(|v| v.is_some()));
    }

    #[test]
    fn test_compute_hl2_feeds_indicator() {
        let s = series(30);
        let f = FisherTransform::new(5);
        let (xs, vs) = compute_hl2(&f, &s);
        assert_eq!(xs.len(), s.len());
        assert!(vs.iter().any(|v| v.is_some()));
    }

    // ---------- plot_close / plot_ohlc / plot_ohlcv ----------

    #[test]
    fn test_plot_close_makes_one_line() {
        let s = series(30);
        let sma = Sma::new(5);
        let plot = plot_close(&sma, &s, "SMA 5", "sma");
        assert_eq!(plot.lines().len(), 1);
        assert_eq!(plot.indicator_id(), "sma");
    }

    #[test]
    fn test_plot_ohlc_makes_one_line() {
        let s = series(30);
        let atr = Atr::new(5);
        let plot = plot_ohlc(&atr, &s, "ATR 5", "atr");
        assert_eq!(plot.lines().len(), 1);
    }

    #[test]
    fn test_plot_ohlcv_makes_one_line() {
        let s = series(30);
        let mfi = Mfi::new(5);
        let plot = plot_ohlcv(&mfi, &s, "MFI 5", "mfi");
        assert_eq!(plot.lines().len(), 1);
    }

    // ---------- impl_plottable_period — single-line indicators ----------

    #[test]
    fn test_sma_plottable() {
        let s = series(30);
        let sma = Sma::new(5);
        let plot = sma.plot(&s);
        assert_eq!(sma.name(), "SMA 5");
        assert_eq!(sma.indicator_id(), "sma");
        assert_eq!(plot.lines().len(), 1);
    }

    #[test]
    fn test_ema_plottable() {
        let ema = Ema::new(7);
        assert_eq!(ema.name(), "EMA 7");
        assert_eq!(ema.indicator_id(), "ema");
        assert_eq!(ema.plot(&series(30)).lines().len(), 1);
    }

    #[test]
    fn test_wma_plottable() {
        let wma = Wma::new(4);
        assert_eq!(wma.name(), "WMA 4");
        assert_eq!(wma.indicator_id(), "wma");
        assert_eq!(wma.plot(&series(30)).lines().len(), 1);
    }

    #[test]
    fn test_rsi_plottable() {
        let rsi = Rsi::new(6);
        assert_eq!(rsi.name(), "RSI 6");
        assert_eq!(rsi.indicator_id(), "rsi");
        assert_eq!(rsi.plot(&series(30)).lines().len(), 1);
    }

    #[test]
    fn test_roc_plottable() {
        let roc = Roc::new(3);
        assert_eq!(roc.name(), "ROC 3");
        assert_eq!(roc.indicator_id(), "roc");
        assert_eq!(roc.plot(&series(30)).lines().len(), 1);
    }

    #[test]
    fn test_std_dev_plottable() {
        let sd = StdDev::new(5);
        assert_eq!(sd.name(), "StdDev 5");
        assert_eq!(sd.indicator_id(), "std_dev");
        assert_eq!(sd.plot(&series(30)).lines().len(), 1);
    }

    #[test]
    fn test_efficiency_ratio_plottable() {
        let er = EfficiencyRatio::new(5);
        assert_eq!(er.name(), "ER 5");
        assert_eq!(er.indicator_id(), "efficiency_ratio");
        assert_eq!(er.plot(&series(30)).lines().len(), 1);
    }

    #[test]
    fn test_atr_plottable() {
        let atr = Atr::new(5);
        assert_eq!(atr.name(), "ATR 5");
        assert_eq!(atr.indicator_id(), "atr");
        assert_eq!(atr.plot(&series(30)).lines().len(), 1);
    }

    #[test]
    fn test_cci_plottable() {
        let cci = Cci::new(5, 0.015);
        assert_eq!(cci.name(), "CCI 5");
        assert_eq!(cci.indicator_id(), "cci");
        assert_eq!(cci.plot(&series(30)).lines().len(), 1);
    }

    // ---------- impl_plottable_simple ----------

    #[test]
    fn test_obv_plottable_uses_simple_label() {
        let obv = Obv::new();
        assert_eq!(obv.name(), "OBV");
        assert_eq!(obv.indicator_id(), "obv");
        assert_eq!(obv.plot(&series(30)).lines().len(), 1);
    }

    #[test]
    fn test_mfi_plottable() {
        let mfi = Mfi::new(5);
        assert_eq!(mfi.name(), "MFI 5");
        assert_eq!(mfi.indicator_id(), "mfi");
        assert_eq!(mfi.plot(&series(30)).lines().len(), 1);
    }

    // ---------- multi-line / histogram indicators ----------

    #[test]
    fn test_bollinger_plot_has_three_lines() {
        let bb = Bollinger::new(5, 2.0);
        let plot = bb.plot(&series(30));
        assert_eq!(plot.lines().len(), 3);
        assert_eq!(bb.name(), "BB(5, 2)");
        assert_eq!(bb.indicator_id(), "bollinger");
        assert!(plot.histogram().is_none());
    }

    #[test]
    fn test_macd_plot_has_two_lines_and_histogram() {
        let macd = Macd::new(3, 5, 2);
        let plot = macd.plot(&series(40));
        assert_eq!(plot.lines().len(), 2);
        assert!(plot.histogram().is_some());
        assert_eq!(macd.name(), "MACD(3, 5, 2)");
        assert_eq!(macd.indicator_id(), "macd");
    }

    #[test]
    fn test_ppo_plot_has_two_lines_and_histogram() {
        let ppo = Ppo::new(3, 5, 2);
        let plot = ppo.plot(&series(40));
        assert_eq!(plot.lines().len(), 2);
        assert!(plot.histogram().is_some());
        assert_eq!(ppo.name(), "PPO(3, 5, 2)");
        assert_eq!(ppo.indicator_id(), "ppo");
    }

    #[test]
    fn test_stochastic_plot_has_two_lines() {
        let stoch = Stochastic::new(5, 3);
        let plot = stoch.plot(&series(30));
        assert_eq!(plot.lines().len(), 2);
        assert_eq!(stoch.name(), "Stoch(5, 3)");
        assert_eq!(stoch.indicator_id(), "stochastic");
    }

    #[test]
    fn test_keltner_plot_has_three_lines() {
        let kc = Keltner::new(5, 5, 2.0);
        let plot = kc.plot(&series(30));
        assert_eq!(plot.lines().len(), 3);
        assert_eq!(kc.name(), "KC(5, 2)");
        assert_eq!(kc.indicator_id(), "keltner");
    }

    #[test]
    fn test_chandelier_plot_has_two_lines() {
        let ce = Chandelier::new(5, 3.0);
        let plot = ce.plot(&series(30));
        assert_eq!(plot.lines().len(), 2);
        assert_eq!(ce.name(), "CE(5, 3)");
        assert_eq!(ce.indicator_id(), "chandelier");
    }

    #[test]
    fn test_chande_kroll_stop_plottable() {
        let ck = ChandeKrollStop::new(5, 3, 2.0);
        let plot = ck.plot(&series(30));
        assert_eq!(plot.lines().len(), 2);
        assert_eq!(ck.indicator_id(), "chande_kroll");
        // CK label: format!("CK({}, {}, {})", atr_period, stop_period, atr_mult)
        assert_eq!(ck.name(), "CK(5, 3, 2)");
    }

    #[test]
    fn test_fisher_transform_plottable() {
        let f = FisherTransform::new(5);
        let plot = f.plot(&series(30));
        assert_eq!(plot.lines().len(), 2);
        assert_eq!(f.name(), "Fisher(5)");
        assert_eq!(f.indicator_id(), "fisher_transform");
    }
}
