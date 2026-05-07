//! Higher-timeframe (HTF) indicator wrappers.
//!
//! `HtfIndicatorNode<T>` runs an inner indicator on aggregated higher-timeframe
//! bars. The aggregator collects every `every_n` primary candles into one
//! synthetic HTF candle (open from the first, close from the last, high/low
//! over the window, summed volume). The inner indicator advances **only**
//! when the HTF window closes; between closes the node forward-fills its
//! last output.
//!
//! Boundaries are bar-count-based, so this works on any [`AxisCoordinate`]
//! — no Timestamp dependency. For irregular series you trade some semantic
//! precision for portability; pick `every_n` to match your series cadence.
//!
//! ```ignore
//! // 5m primary bars, run RSI(14) over 1h aggregates → every_n = 12
//! let htf_rsi = graph.add_node(htf_rsi(close, 12, 14));
//! ```

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::context::ExecutionContext;
use crate::node::{Node, NodeId};
use crate::value::{OutputToValue, Value};
use trdelnik_indicators::{Indicator, Ohlc, Ohlcv};

// ============================================================================
// Aggregated HTF candle
// ============================================================================

/// One closed higher-timeframe candle, built up over `every_n` primary bars.
#[derive(Debug, Clone, Copy)]
pub struct AggregatedCandle {
    /// Open of the first primary bar in the window.
    pub open: f64,
    /// Maximum high across the window.
    pub high: f64,
    /// Minimum low across the window.
    pub low: f64,
    /// Close of the last primary bar in the window.
    pub close: f64,
    /// Sum of volume across the window.
    pub volume: f64,
}

// ============================================================================
// BarAggregator
// ============================================================================

/// Bar-count-based HTF aggregator.
///
/// Feed it every primary candle via [`BarAggregator::feed`]; it returns
/// `Some(candle)` exactly on the bar where `every_n` primary candles have
/// been accumulated, and `None` otherwise.
#[derive(Debug, Clone)]
pub struct BarAggregator {
    every_n: usize,
    bars_collected: usize,
    current: Option<AggregatedCandle>,
}

impl BarAggregator {
    /// Construct an aggregator that closes a candle every `every_n` bars.
    /// Panics if `every_n == 0`.
    pub fn new(every_n: usize) -> Self {
        assert!(every_n > 0, "BarAggregator: every_n must be > 0");
        Self {
            every_n,
            bars_collected: 0,
            current: None,
        }
    }

    /// Reset to start a fresh window.
    pub fn reset(&mut self) {
        self.bars_collected = 0;
        self.current = None;
    }

    /// Feed a primary candle. Returns the closed HTF candle on the
    /// `every_n`-th call since construction or last `feed` that returned
    /// `Some`; otherwise `None`.
    pub fn feed(&mut self, ctx: &ExecutionContext) -> Option<AggregatedCandle> {
        match &mut self.current {
            None => {
                self.current = Some(AggregatedCandle {
                    open: ctx.open,
                    high: ctx.high,
                    low: ctx.low,
                    close: ctx.close,
                    volume: ctx.volume,
                });
            }
            Some(agg) => {
                if ctx.high > agg.high {
                    agg.high = ctx.high;
                }
                if ctx.low < agg.low {
                    agg.low = ctx.low;
                }
                agg.close = ctx.close;
                agg.volume += ctx.volume;
            }
        }
        self.bars_collected += 1;
        if self.bars_collected == self.every_n {
            self.bars_collected = 0;
            return self.current.take();
        }
        None
    }
}

// ============================================================================
// HtfInput trait
// ============================================================================

/// How an HTF wrapper extracts its inner indicator's input from a closed
/// `AggregatedCandle`. Mirrors the `InputProvider` trait used for
/// primary-stream nodes — the difference is that here the input comes
/// from an already-aggregated candle, not from the per-bar context.
pub trait HtfInput: Copy {
    /// Project the closed HTF candle into the inner indicator's input type.
    fn from_htf(candle: &AggregatedCandle) -> Self;
}

impl HtfInput for f64 {
    fn from_htf(candle: &AggregatedCandle) -> Self {
        candle.close
    }
}

impl HtfInput for Ohlc {
    fn from_htf(candle: &AggregatedCandle) -> Self {
        Ohlc::new(candle.high, candle.low, candle.close)
    }
}

impl HtfInput for Ohlcv {
    fn from_htf(candle: &AggregatedCandle) -> Self {
        Ohlcv::new(candle.high, candle.low, candle.close, candle.volume)
    }
}

// ============================================================================
// HtfIndicatorNode
// ============================================================================

/// Generic HTF wrapper: runs `T` on aggregated HTF bars, forward-fills
/// between closes.
///
/// Has no node input — the wrapper builds its inner input from the
/// per-bar `ExecutionContext` (current candle's OHLCV) via
/// [`HtfInput::from_htf`] when the HTF window closes.
#[derive(Debug, Clone)]
pub struct HtfIndicatorNode<T: Indicator>
where
    T: Hash + Eq,
    T::Input: HtfInput,
    T::Output: OutputToValue,
{
    every_n: usize,
    aggregator: BarAggregator,
    inner: T,
    last: Value,
}

impl<T: Indicator> HtfIndicatorNode<T>
where
    T: Hash + Eq,
    T::Input: HtfInput,
    T::Output: OutputToValue,
{
    /// Construct a wrapper closing an HTF bar every `every_n` primary bars.
    pub fn new(every_n: usize, inner: T) -> Self {
        Self {
            every_n,
            aggregator: BarAggregator::new(every_n),
            inner,
            last: T::Output::to_value(None),
        }
    }

    /// Reference to the inner indicator state.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    fn compute_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        self.every_n.hash(&mut hasher);
        hasher.finish()
    }
}

impl<T: Indicator + Clone + Send + Sync + 'static> Node for HtfIndicatorNode<T>
where
    T: Hash + Eq,
    T::Input: HtfInput,
    T::Output: OutputToValue,
{
    fn signature(&self) -> Option<String> {
        Some(format!(
            "htf:{}:{}:{:x}",
            T::NAME,
            self.every_n,
            self.compute_hash()
        ))
    }

    fn inputs(&self) -> &[NodeId] {
        &[]
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.aggregator.reset();
        self.last = T::Output::to_value(None);
    }

    fn compute(&mut self, ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
        if let Some(candle) = self.aggregator.feed(ctx) {
            let input = T::Input::from_htf(&candle);
            let out = self.inner.next(input);
            self.last = T::Output::to_value(out);
        }
        self.last.clone()
    }

    fn warmup_period(&self) -> usize {
        // Inner needs N HTF bars; that's N * every_n primary bars to produce
        // the first valid output.
        self.inner.warmup_period() * self.every_n
    }

    fn name(&self) -> &str {
        T::NAME
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_indicators::{Atr, Ema, Sma};

    fn ctx(close: f64) -> ExecutionContext {
        ExecutionContext::new(0, close, close + 1.0, close - 1.0, close, 100.0, 0.0)
    }

    #[test]
    fn aggregator_emits_every_n_bars() {
        let mut agg = BarAggregator::new(3);
        assert!(agg.feed(&ctx(10.0)).is_none());
        assert!(agg.feed(&ctx(11.0)).is_none());
        let candle = agg.feed(&ctx(12.0)).expect("third bar closes window");
        assert_eq!(candle.open, 10.0);
        assert_eq!(candle.close, 12.0);
        assert_eq!(candle.high, 13.0);
        assert_eq!(candle.low, 9.0);
        assert_eq!(candle.volume, 300.0);

        // Window starts fresh.
        assert!(agg.feed(&ctx(20.0)).is_none());
    }

    #[test]
    fn htf_sma_advances_only_on_closed_bars() {
        // SMA(2) on HTF candles aggregated every 3 primary bars.
        // After 6 primary bars (= 2 HTF bars) the inner SMA produces its first value.
        let mut node = HtfIndicatorNode::new(3, Sma::new(2));

        // First HTF window: closes at primary bar 3; inner SMA still warming.
        assert!(node.compute(&ctx(10.0), &[]).is_none());
        assert!(node.compute(&ctx(11.0), &[]).is_none());
        assert!(node.compute(&ctx(12.0), &[]).is_none());

        // Forward-fill across remaining primary bars in the second window.
        assert!(node.compute(&ctx(13.0), &[]).is_none());
        assert!(node.compute(&ctx(14.0), &[]).is_none());

        // Second HTF window closes — inner SMA(2) produces its first value
        // = mean of HTF closes [12.0, 15.0] = 13.5.
        let v = node.compute(&ctx(15.0), &[]);
        assert_eq!(v.as_number(), Some(13.5));

        // Forward-fill until the next HTF close.
        assert_eq!(node.compute(&ctx(16.0), &[]).as_number(), Some(13.5));
        assert_eq!(node.compute(&ctx(17.0), &[]).as_number(), Some(13.5));
    }

    #[test]
    fn htf_signature_distinguishes_inner_and_window() {
        let a = HtfIndicatorNode::new(3, Sma::new(20));
        let b = HtfIndicatorNode::new(3, Sma::new(20));
        let c = HtfIndicatorNode::new(3, Sma::new(10));
        let d = HtfIndicatorNode::new(6, Sma::new(20));

        assert_eq!(a.signature(), b.signature());
        assert_ne!(a.signature(), c.signature());
        assert_ne!(a.signature(), d.signature());
    }

    #[test]
    fn htf_warmup_scales_with_window() {
        // SMA(5) on every-3-bars HTF needs 5 HTF bars = 15 primary bars.
        let n = HtfIndicatorNode::new(3, Sma::new(5));
        assert_eq!(n.warmup_period(), 15);
    }

    #[test]
    fn htf_with_ohlc_indicator() {
        // ATR is Ohlc-input; HtfInput<Ohlc> uses high/low/close from the candle.
        // We just verify that the Ohlc dispatch path works end-to-end —
        // values warm up over HTF bars, then ATR produces numbers.
        let mut node = HtfIndicatorNode::new(2, Atr::new(3));
        let mut last = Value::none_number();
        // 8 primary bars = 4 HTF bars; ATR(3) needs warmup of 3.
        for v in [100.0, 101.0, 102.0, 99.0, 105.0, 103.0, 107.0, 110.0] {
            last = node.compute(&ctx(v), &[]);
        }
        assert!(
            last.as_number().is_some(),
            "expected ATR to be live after warmup, got {:?}",
            last
        );
    }

    #[test]
    fn htf_ema_round_trip() {
        // EMA(3) on every-2-bars HTF; after 6 primary bars (= 3 HTF bars)
        // the first value drops out.
        let mut node = HtfIndicatorNode::new(2, Ema::new(3));
        let primary = [10.0, 12.0, 14.0, 16.0, 18.0, 20.0];

        let mut last = Value::none_number();
        for p in primary {
            last = node.compute(&ctx(p), &[]);
        }
        // After 3 closed HTF bars (closes 12, 16, 20), EMA(3) is bootstrapped
        // and produces a value. We don't assert the exact number — that's a
        // test of EMA itself, not the wrapper. Just check it surfaced.
        assert!(last.as_number().is_some(), "expected EMA to be live by now");
    }
}
