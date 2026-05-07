//! Chande Kroll Stop stateful implementation

use crate::indicator;
use crate::ring_buffer::MinMaxRingBuffer;
use crate::{Atr, Indicator, Ohlc};

/// Chande Kroll Stop output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct ChandeKrollStopValue {
    pub stop_long: f64,
    pub stop_short: f64,
}

/// Chande Kroll Stop with O(1) per-bar computation.
///
/// A trend-following stop indicator based on ATR. Computes two stop levels:
///
/// - `stop_long` – trailing stop for long positions (lowest of first low stops over `q` bars)
/// - `stop_short` – trailing stop for short positions (highest of first high stops over `q` bars)
///
/// When price is above both stops, the trend is up; when below both, the trend is down.
#[indicator]
#[derive(Debug, Clone)]
pub struct ChandeKrollStop {
    /// ATR lookback period
    #[param(default = 10)]
    atr_period: usize,
    /// Stop smoothing period
    #[param(default = 9)]
    stop_period: usize,
    /// ATR multiplier
    #[param(default = 1.0)]
    atr_mult: f64,

    atr: Atr,
    high_buffer: MinMaxRingBuffer,
    low_buffer: MinMaxRingBuffer,
    first_high_stop_buffer: MinMaxRingBuffer,
    first_low_stop_buffer: MinMaxRingBuffer,
    count: usize,
}

impl ChandeKrollStop {
    pub fn new(atr_period: usize, stop_period: usize, atr_mult: f64) -> Self {
        Self {
            atr_period,
            stop_period,
            atr_mult,
            atr: Atr::new(atr_period),
            high_buffer: MinMaxRingBuffer::new(atr_period),
            low_buffer: MinMaxRingBuffer::new(atr_period),
            first_high_stop_buffer: MinMaxRingBuffer::new(stop_period),
            first_low_stop_buffer: MinMaxRingBuffer::new(stop_period),
            count: 0,
        }
    }

    pub fn atr_period(&self) -> usize {
        self.atr_period
    }

    pub fn stop_period(&self) -> usize {
        self.stop_period
    }

    pub fn atr_mult(&self) -> f64 {
        self.atr_mult
    }
}

impl Indicator for ChandeKrollStop {
    type Input = Ohlc;
    type Output = ChandeKrollStopValue;
    const NAME: &'static str = "chande_kroll_stop";

    fn reset(&mut self) {
        self.atr.reset();
        self.high_buffer.reset();
        self.low_buffer.reset();
        self.first_high_stop_buffer.reset();
        self.first_low_stop_buffer.reset();
        self.count = 0;
    }

    fn next(&mut self, input: Ohlc) -> Option<ChandeKrollStopValue> {
        let Ohlc { high, low, .. } = input;

        self.high_buffer.push(high);
        self.low_buffer.push(low);

        let atr = self.atr.next(input)?;
        self.count += 1;

        if !self.high_buffer.is_full() {
            return None;
        }

        // First stops: highest high / lowest low over ATR period ± ATR * mult
        let highest = self.high_buffer.max()?;
        let lowest = self.low_buffer.min()?;
        let first_high_stop = highest - self.atr_mult * atr;
        let first_low_stop = lowest + self.atr_mult * atr;

        self.first_high_stop_buffer.push(first_high_stop);
        self.first_low_stop_buffer.push(first_low_stop);

        if !self.first_high_stop_buffer.is_full() {
            return None;
        }

        // Final stops: highest of first_high_stop / lowest of first_low_stop over stop period
        let stop_short = self.first_high_stop_buffer.max()?;
        let stop_long = self.first_low_stop_buffer.min()?;

        Some(ChandeKrollStopValue {
            stop_long,
            stop_short,
        })
    }

    fn warmup_period(&self) -> usize {
        self.atr_period + self.stop_period - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ohlc(high: f64, low: f64, close: f64) -> Ohlc {
        Ohlc { high, low, close }
    }

    #[test]
    fn test_chande_kroll_warmup() {
        let mut ck = ChandeKrollStop::new(5, 3, 1.0);
        // Warmup = atr_period + stop_period - 1 = 5 + 3 - 1 = 7
        for i in 0..6 {
            assert!(
                ck.next(ohlc(10.0 + i as f64, 9.0 + i as f64, 9.5 + i as f64))
                    .is_none(),
                "Expected None at bar {}",
                i
            );
        }
        assert!(ck
            .next(ohlc(16.0, 15.0, 15.5))
            .is_some());
    }

    #[test]
    fn test_chande_kroll_stop_long_below_short() {
        let mut ck = ChandeKrollStop::new(5, 3, 1.0);
        // Feed enough data
        let bars = [
            ohlc(12.0, 10.0, 11.0),
            ohlc(13.0, 11.0, 12.0),
            ohlc(14.0, 12.0, 13.0),
            ohlc(15.0, 13.0, 14.0),
            ohlc(16.0, 14.0, 15.0),
            ohlc(17.0, 15.0, 16.0),
            ohlc(18.0, 16.0, 17.0),
            ohlc(19.0, 17.0, 18.0),
        ];
        for bar in &bars {
            if let Some(val) = ck.next(*bar) {
                // stop_long should always be <= stop_short in a trending market
                assert!(
                    val.stop_long <= val.stop_short,
                    "stop_long={} > stop_short={}",
                    val.stop_long,
                    val.stop_short
                );
            }
        }
    }

    #[test]
    fn test_chande_kroll_reset() {
        let mut ck = ChandeKrollStop::new(3, 2, 1.0);
        for i in 0..10 {
            ck.next(ohlc(10.0 + i as f64, 9.0 + i as f64, 9.5 + i as f64));
        }
        ck.reset();
        // After reset, should need warmup again
        assert!(ck.next(ohlc(10.0, 9.0, 9.5)).is_none());
    }
}
