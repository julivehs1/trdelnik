//! Stochastic Oscillator stateful implementation

use crate::indicator;
use crate::ring_buffer::{MinMaxRingBuffer, RingBuffer};
use crate::{Indicator, Ohlc};

/// Stochastic Oscillator output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorOutput)]
pub struct StochasticValue {
    pub k: f64,
    pub d: f64,
}

/// Stochastic Oscillator with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Stochastic {
    #[param(default = 14)]
    k_period: usize,
    #[param(default = 3)]
    d_period: usize,

    high_buffer: MinMaxRingBuffer,
    low_buffer: MinMaxRingBuffer,
    k_buffer: RingBuffer,
}

impl Stochastic {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        Self {
            k_period,
            d_period,
            high_buffer: MinMaxRingBuffer::new(k_period),
            low_buffer: MinMaxRingBuffer::new(k_period),
            k_buffer: RingBuffer::new(d_period),
        }
    }

    pub fn k_period(&self) -> usize { self.k_period }
    pub fn d_period(&self) -> usize { self.d_period }

    pub fn next_k(&mut self, input: Ohlc) -> Option<f64> {
        let Ohlc { high, low, close } = input;
        self.high_buffer.push(high);
        self.low_buffer.push(low);
        if !self.high_buffer.is_full() {
            return None;
        }
        let highest = self.high_buffer.max().unwrap();
        let lowest = self.low_buffer.min().unwrap();
        let range = highest - lowest;
        let k = if range == 0.0 { 50.0 } else { 100.0 * (close - lowest) / range };
        Some(k)
    }
}

impl Indicator for Stochastic {
    type Input = Ohlc;
    type Output = StochasticValue;
    const NAME: &'static str = "stochastic";

    fn reset(&mut self) {
        self.high_buffer.reset();
        self.low_buffer.reset();
        self.k_buffer.reset();
    }

    fn next(&mut self, input: Ohlc) -> Option<StochasticValue> {
        let Ohlc { high, low, close } = input;
        self.high_buffer.push(high);
        self.low_buffer.push(low);
        if !self.high_buffer.is_full() {
            return None;
        }
        let highest = self.high_buffer.max().unwrap();
        let lowest = self.low_buffer.min().unwrap();
        let range = highest - lowest;
        let k = if range == 0.0 { 50.0 } else { 100.0 * (close - lowest) / range };
        self.k_buffer.push(k);
        if !self.k_buffer.is_full() {
            return None;
        }
        Some(StochasticValue { k, d: self.k_buffer.average().unwrap() })
    }

    fn warmup_period(&self) -> usize {
        self.k_period + self.d_period - 1
    }
}
