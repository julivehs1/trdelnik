//! Standard Deviation stateful implementation

use crate::indicator;
use crate::ring_buffer::RingBuffer;
use crate::Indicator;

/// Standard Deviation with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct StdDev {
    #[param]
    period: usize,

    buffer: RingBuffer,
}

impl StdDev {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            buffer: RingBuffer::new(period),
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for StdDev {
    type Input = f64;
    type Output = f64;
    const NAME: &'static str = "std_dev";

    fn reset(&mut self) {
        self.buffer.reset();
    }

    fn next(&mut self, value: f64) -> Option<f64> {
        self.buffer.push(value);
        if self.buffer.is_full() {
            self.buffer.std_dev()
        } else {
            None
        }
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
}
