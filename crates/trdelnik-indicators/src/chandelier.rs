//! Chandelier Exit stateful implementation

use crate::indicator;
use crate::ring_buffer::MinMaxRingBuffer;
use crate::{Atr, Indicator, Ohlc};

/// Chandelier Exit output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorOutput)]
pub struct ChandelierValue {
    pub long_exit: f64,
    pub short_exit: f64,
}

/// Chandelier Exit with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Chandelier {
    #[param(default = 22)]
    period: usize,
    #[param(default = 3.0)]
    atr_mult: f64,

    high_buffer: MinMaxRingBuffer,
    low_buffer: MinMaxRingBuffer,
    atr: Atr,
}

impl Chandelier {
    pub fn new(period: usize, atr_mult: f64) -> Self {
        Self {
            period,
            atr_mult,
            high_buffer: MinMaxRingBuffer::new(period),
            low_buffer: MinMaxRingBuffer::new(period),
            atr: Atr::new(period),
        }
    }

    pub fn period(&self) -> usize { self.period }
    pub fn atr_mult(&self) -> f64 { self.atr_mult }
}

impl Indicator for Chandelier {
    type Input = Ohlc;
    type Output = ChandelierValue;
    const NAME: &'static str = "chandelier";

    fn reset(&mut self) {
        self.high_buffer.reset();
        self.low_buffer.reset();
        self.atr.reset();
    }

    fn next(&mut self, input: Ohlc) -> Option<ChandelierValue> {
        let Ohlc { high, low, .. } = input;
        self.high_buffer.push(high);
        self.low_buffer.push(low);

        let atr = self.atr.next(input)?;

        if !self.high_buffer.is_full() {
            return None;
        }

        let highest = self.high_buffer.max()?;
        let lowest = self.low_buffer.min()?;

        Some(ChandelierValue {
            long_exit: highest - self.atr_mult * atr,
            short_exit: lowest + self.atr_mult * atr,
        })
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
}
