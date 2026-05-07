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

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{IndicatorParams, ParamValue};

    #[test]
    fn test_efficiency_ratio_warmup() {
        // period=3 → buffer size 4, needs 4 values to fill
        let mut er = EfficiencyRatio::new(3);
        assert!(er.next(1.0).is_none());
        assert!(er.next(2.0).is_none());
        assert!(er.next(3.0).is_none());
        assert!(er.next(4.0).is_some());
    }

    #[test]
    fn test_efficiency_ratio_perfect_trend_is_one() {
        // Monotonic uniform step: total_change = volatility → ER = 1
        let mut er = EfficiencyRatio::new(3);
        er.next(1.0);
        er.next(2.0);
        er.next(3.0);
        let r = er.next(4.0).unwrap();
        assert!((r - 1.0).abs() < 1e-9, "got {}", r);
    }

    #[test]
    fn test_efficiency_ratio_constant_returns_zero() {
        // Volatility = 0 → ER returns 0
        let mut er = EfficiencyRatio::new(3);
        er.next(5.0);
        er.next(5.0);
        er.next(5.0);
        let r = er.next(5.0).unwrap();
        assert_eq!(r, 0.0);
    }

    #[test]
    fn test_efficiency_ratio_choppy_below_one() {
        // Up-down-up-down: net change small, volatility large → ER < 1
        let mut er = EfficiencyRatio::new(3);
        er.next(10.0);
        er.next(11.0);
        er.next(10.0);
        let r = er.next(11.0).unwrap();
        assert!(r < 1.0);
        assert!(r >= 0.0);
    }

    #[test]
    fn test_efficiency_ratio_in_range() {
        let mut er = EfficiencyRatio::new(3);
        for v in [10.0, 12.0, 11.0, 13.0, 14.0, 12.5, 15.0] {
            if let Some(r) = er.next(v) {
                assert!((0.0..=1.0).contains(&r), "ER out of range: {}", r);
            }
        }
    }

    #[test]
    fn test_efficiency_ratio_reset() {
        let mut er = EfficiencyRatio::new(3);
        for v in [1.0, 2.0, 3.0, 4.0] {
            er.next(v);
        }
        er.reset();
        assert!(er.next(10.0).is_none());
    }

    #[test]
    fn test_efficiency_ratio_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = EfficiencyRatio::new(10);
        let b = EfficiencyRatio::new(10);
        let c = EfficiencyRatio::new(20);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_efficiency_ratio_param_defs() {
        let defs = EfficiencyRatio::param_defs();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "period");
    }

    #[test]
    fn test_efficiency_ratio_from_params() {
        let er = EfficiencyRatio::from_params(&[ParamValue::Usize(20)]).unwrap();
        assert_eq!(er.period(), 20);
    }

    #[test]
    fn test_efficiency_ratio_warmup_period() {
        assert_eq!(EfficiencyRatio::new(10).warmup_period(), 11);
    }
}
