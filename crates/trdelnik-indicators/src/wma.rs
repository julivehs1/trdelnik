//! Weighted Moving Average stateful implementation

use crate::indicator;
use crate::ring_buffer::RingBuffer;
use crate::Indicator;

/// Weighted Moving Average with O(n) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Wma {
    #[param]
    period: usize,

    buffer: RingBuffer,
    weight_sum: f64,
}

impl Wma {
    pub fn new(period: usize) -> Self {
        let weight_sum: f64 = (1..=period).map(|x| x as f64).sum();
        Self {
            period,
            buffer: RingBuffer::new(period),
            weight_sum,
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Wma {
    type Input = f64;
    type Output = f64;
    const NAME: &'static str = "wma";

    fn reset(&mut self) {
        self.buffer.reset();
    }

    fn next(&mut self, value: f64) -> Option<f64> {
        self.buffer.push(value);
        if !self.buffer.is_full() {
            return None;
        }
        let weighted_sum: f64 = self.buffer.iter().enumerate()
            .map(|(i, v)| v * (i + 1) as f64).sum();
        Some(weighted_sum / self.weight_sum)
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
}
