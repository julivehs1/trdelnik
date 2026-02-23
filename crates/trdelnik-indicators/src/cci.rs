//! Commodity Channel Index stateful implementation

use crate::indicator;
use crate::ring_buffer::RingBuffer;
use crate::{Indicator, Ohlc};

/// Commodity Channel Index with O(n) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Cci {
    #[param(default = 20)]
    period: usize,
    #[param(default = 0.015)]
    constant: f64,

    buffer: RingBuffer,
    tp_values: Vec<f64>,
}

impl Cci {
    pub fn new(period: usize, constant: f64) -> Self {
        Self {
            period,
            constant,
            buffer: RingBuffer::new(period),
            tp_values: Vec::with_capacity(period),
        }
    }

    pub fn period(&self) -> usize { self.period }
    pub fn constant(&self) -> f64 { self.constant }
}

impl Indicator for Cci {
    type Input = Ohlc;
    type Output = f64;
    const NAME: &'static str = "cci";

    fn reset(&mut self) {
        self.buffer.reset();
        self.tp_values.clear();
    }

    fn next(&mut self, input: Ohlc) -> Option<f64> {
        let Ohlc { high, low, close } = input;
        let tp = (high + low + close) / 3.0;
        self.buffer.push(tp);

        if self.tp_values.len() >= self.period {
            self.tp_values.remove(0);
        }
        self.tp_values.push(tp);

        if !self.buffer.is_full() {
            return None;
        }

        let sma = self.buffer.average()?;
        let mad: f64 = self.tp_values.iter().map(|&v| (v - sma).abs()).sum::<f64>()
            / self.period as f64;

        if mad == 0.0 {
            return Some(0.0);
        }

        Some((tp - sma) / (self.constant * mad))
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
}
