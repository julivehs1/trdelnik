//! MACD (Moving Average Convergence Divergence) stateful implementation

use crate::indicator;
use crate::{Ema, Indicator};

/// MACD output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct MacdValue {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

/// MACD with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Macd {
    #[param(default = 12)]
    fast_period: usize,
    #[param(default = 26)]
    slow_period: usize,
    #[param(default = 9)]
    signal_period: usize,

    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
}

impl Macd {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            signal_ema: Ema::new(signal_period),
        }
    }

    pub fn fast_period(&self) -> usize { self.fast_period }
    pub fn slow_period(&self) -> usize { self.slow_period }
    pub fn signal_period(&self) -> usize { self.signal_period }

    pub fn next_macd_line(&mut self, value: f64) -> Option<f64> {
        let fast = self.fast_ema.next(value);
        let slow = self.slow_ema.next(value);
        match (fast, slow) {
            (Some(f), Some(s)) => Some(f - s),
            _ => None,
        }
    }
}

impl Indicator for Macd {
    type Input = f64;
    type Output = MacdValue;
    const NAME: &'static str = "macd";

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
    }

    fn next(&mut self, value: f64) -> Option<MacdValue> {
        let fast = self.fast_ema.next(value);
        let slow = self.slow_ema.next(value);

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let macd_line = f - s;
                self.signal_ema.next(macd_line).map(|signal| MacdValue {
                    macd: macd_line,
                    signal,
                    histogram: macd_line - signal,
                })
            }
            _ => None,
        }
    }

    fn warmup_period(&self) -> usize {
        self.slow_period + self.signal_period - 1
    }
}
