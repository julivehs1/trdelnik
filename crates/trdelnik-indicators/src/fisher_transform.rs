//! Fisher Transform stateful implementation

use crate::indicator;
use crate::ring_buffer::MinMaxRingBuffer;
use crate::Indicator;

/// Fisher Transform output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct FisherTransformValue {
    pub fisher: f64,
    pub trigger: f64,
}

/// Fisher Transform with O(1) per-bar computation.
///
/// Converts prices into a Gaussian normal distribution, producing sharper
/// turning point signals. The output oscillates around zero.
///
/// - `fisher` is the current Fisher Transform value
/// - `trigger` is the previous Fisher Transform value (signal line)
#[indicator]
#[derive(Debug, Clone)]
pub struct FisherTransform {
    #[param(default = 9)]
    period: usize,

    buffer: MinMaxRingBuffer,
    prev_norm: f64,
    prev_fisher: f64,
    count: usize,
}

impl FisherTransform {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            buffer: MinMaxRingBuffer::new(period),
            prev_norm: 0.0,
            prev_fisher: 0.0,
            count: 0,
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for FisherTransform {
    type Input = f64;
    type Output = FisherTransformValue;
    const NAME: &'static str = "fisher_transform";

    fn reset(&mut self) {
        self.buffer.reset();
        self.prev_norm = 0.0;
        self.prev_fisher = 0.0;
        self.count = 0;
    }

    fn next(&mut self, value: f64) -> Option<FisherTransformValue> {
        self.buffer.push(value);
        self.count += 1;

        if !self.buffer.is_full() {
            return None;
        }

        let highest = self.buffer.max()?;
        let lowest = self.buffer.min()?;

        // Normalize to [0, 1] range (TradingView/Ehlers formula)
        let normalized = if (highest - lowest).abs() < f64::EPSILON {
            0.5
        } else {
            (value - lowest) / (highest - lowest)
        };

        // Smoothed value: 0.66 * (normalized - 0.5) + 0.67 * prev
        let v = 0.66 * (normalized - 0.5) + 0.67 * self.prev_norm;
        let v = v.clamp(-0.999, 0.999);

        // Fisher Transform
        let fisher = 0.5 * ((1.0 + v) / (1.0 - v)).ln() + 0.5 * self.prev_fisher;

        let trigger = self.prev_fisher;

        self.prev_norm = v;
        self.prev_fisher = fisher;

        Some(FisherTransformValue { fisher, trigger })
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fisher_transform_warmup() {
        let mut ft = FisherTransform::new(5);
        for _ in 0..4 {
            assert!(ft.next(10.0).is_none());
        }
        assert!(ft.next(10.0).is_some());
    }

    #[test]
    fn test_fisher_transform_trending_up() {
        let mut ft = FisherTransform::new(5);
        let prices = [10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0];
        let mut last_fisher = f64::NEG_INFINITY;
        for &p in &prices {
            if let Some(val) = ft.next(p) {
                // In an uptrend, fisher should generally be positive
                assert!(val.fisher > 0.0, "fisher={}", val.fisher);
                last_fisher = val.fisher;
            }
        }
        assert!(last_fisher > 0.0);
    }

    #[test]
    fn test_fisher_transform_flat() {
        let mut ft = FisherTransform::new(5);
        let prices = [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0];
        for &p in &prices {
            if let Some(val) = ft.next(p) {
                // Flat prices should produce values near zero
                assert!(val.fisher.abs() < 1.0, "fisher={}", val.fisher);
            }
        }
    }

    #[test]
    fn test_fisher_transform_reset() {
        let mut ft = FisherTransform::new(3);
        ft.next(10.0);
        ft.next(11.0);
        ft.next(12.0);
        ft.reset();
        // After reset, should need warmup again
        assert!(ft.next(10.0).is_none());
        assert!(ft.next(11.0).is_none());
        assert!(ft.next(12.0).is_some());
    }

    #[test]
    fn test_fisher_trigger_is_previous() {
        let mut ft = FisherTransform::new(3);
        ft.next(10.0);
        ft.next(11.0);
        let first = ft.next(12.0).unwrap();
        let second = ft.next(13.0).unwrap();
        assert!((second.trigger - first.fisher).abs() < 1e-10);
    }
}
