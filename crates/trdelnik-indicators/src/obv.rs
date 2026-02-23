//! On-Balance Volume stateful implementation

use crate::indicator;
use crate::{Indicator, Ohlcv};

/// On-Balance Volume with O(1) per-bar computation.
/// OBV has no parameters - it's a cumulative indicator.
#[indicator]
#[derive(Debug, Clone)]
pub struct Obv {
    prev_close: Option<f64>,
    obv: f64,
}

impl Obv {
    pub fn new() -> Self {
        Self {
            prev_close: None,
            obv: 0.0,
        }
    }

    pub fn current(&self) -> f64 {
        self.obv
    }
}

impl Default for Obv {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Obv {
    type Input = Ohlcv;
    type Output = f64;
    const NAME: &'static str = "obv";

    fn reset(&mut self) {
        self.prev_close = None;
        self.obv = 0.0;
    }

    fn next(&mut self, input: Ohlcv) -> Option<f64> {
        let Ohlcv { close, volume, .. } = input;

        let Some(prev_close) = self.prev_close else {
            self.prev_close = Some(close);
            self.obv = volume;
            return Some(self.obv);
        };

        if close > prev_close {
            self.obv += volume;
        } else if close < prev_close {
            self.obv -= volume;
        }

        self.prev_close = Some(close);
        Some(self.obv)
    }

    fn warmup_period(&self) -> usize {
        0
    }
}
