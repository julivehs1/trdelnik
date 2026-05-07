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

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{IndicatorParams, ParamValue};

    #[test]
    fn test_roc_warmup() {
        // period=2 → buffer has size 3, needs 3 values to fill
        let mut roc = Roc::new(2);
        assert!(roc.next(100.0).is_none());
        assert!(roc.next(102.0).is_none());
        assert!(roc.next(110.0).is_some());
    }

    #[test]
    fn test_roc_known_value() {
        // period=2: oldest=100, current=110 → ((110-100)/100)*100 = 10
        let mut roc = Roc::new(2);
        roc.next(100.0);
        roc.next(105.0);
        let result = roc.next(110.0).unwrap();
        assert!((result - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_roc_negative() {
        let mut roc = Roc::new(2);
        roc.next(100.0);
        roc.next(95.0);
        let result = roc.next(90.0).unwrap();
        assert!((result - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn test_roc_zero_oldest_returns_zero() {
        let mut roc = Roc::new(2);
        roc.next(0.0);
        roc.next(50.0);
        let result = roc.next(100.0).unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_roc_constant_values() {
        let mut roc = Roc::new(2);
        roc.next(50.0);
        roc.next(50.0);
        let result = roc.next(50.0).unwrap();
        assert!((result - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_roc_reset() {
        let mut roc = Roc::new(2);
        roc.next(100.0);
        roc.next(105.0);
        roc.next(110.0);
        roc.reset();
        assert!(roc.next(200.0).is_none());
    }

    #[test]
    fn test_roc_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Roc::new(10);
        let b = Roc::new(10);
        let c = Roc::new(14);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_roc_param_defs() {
        let defs = Roc::param_defs();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "period");
    }

    #[test]
    fn test_roc_from_params() {
        let roc = Roc::from_params(&[ParamValue::Usize(12)]).unwrap();
        assert_eq!(roc.period(), 12);
    }

    #[test]
    fn test_roc_warmup_period() {
        assert_eq!(Roc::new(10).warmup_period(), 11);
    }
}
