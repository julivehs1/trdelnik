//! Rate of Change stateful implementation

use crate::indicator;
use crate::ring_buffer::RingBuffer;
use crate::Indicator;

/// Rate of Change with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Roc {
    #[param]
    period: usize,

    buffer: RingBuffer,
}

impl Roc {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            buffer: RingBuffer::new(period + 1),
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Roc {
    type Input = f64;
    type Output = f64;
    const NAME: &'static str = "roc";

    fn reset(&mut self) {
        self.buffer.reset();
    }

    fn next(&mut self, value: f64) -> Option<f64> {
        self.buffer.push(value);
        if !self.buffer.is_full() {
            return None;
        }
        let oldest = self.buffer.iter().next().unwrap();
        if oldest == 0.0 {
            return Some(0.0);
        }
        Some(((value - oldest) / oldest) * 100.0)
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }
}
