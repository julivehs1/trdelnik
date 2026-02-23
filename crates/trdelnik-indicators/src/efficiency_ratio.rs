//! Efficiency Ratio stateful implementation

use crate::indicator;
use crate::ring_buffer::RingBuffer;
use crate::Indicator;

/// Kaufman's Efficiency Ratio with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct EfficiencyRatio {
    #[param(default = 10)]
    period: usize,

    buffer: RingBuffer,
    change_sum: f64,
    prev_value: Option<f64>,
    count: usize,
}

impl EfficiencyRatio {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            buffer: RingBuffer::new(period + 1),
            change_sum: 0.0,
            prev_value: None,
            count: 0,
        }
    }

    pub fn period(&self) -> usize { self.period }
}

impl Indicator for EfficiencyRatio {
    type Input = f64;
    type Output = f64;
    const NAME: &'static str = "efficiency_ratio";

    fn reset(&mut self) {
        self.buffer.reset();
        self.change_sum = 0.0;
        self.prev_value = None;
        self.count = 0;
    }

    fn next(&mut self, value: f64) -> Option<f64> {
        self.buffer.push(value);
        self.count += 1;

        if let Some(prev) = self.prev_value {
            self.change_sum += (value - prev).abs();
        }
        self.prev_value = Some(value);

        if !self.buffer.is_full() {
            return None;
        }

        let oldest = self.buffer.iter().next().unwrap();
        let total_change = (value - oldest).abs();

        let volatility: f64 = self.buffer.iter()
            .collect::<Vec<_>>()
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .sum();

        if volatility == 0.0 {
            return Some(0.0);
        }

        Some(total_change / volatility)
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }
}
