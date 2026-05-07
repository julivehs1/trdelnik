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

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{IndicatorParams, ParamValue};

    #[test]
    fn test_wma_warmup() {
        let mut wma = Wma::new(3);
        assert!(wma.next(1.0).is_none());
        assert!(wma.next(2.0).is_none());
        assert!(wma.next(3.0).is_some());
    }

    #[test]
    fn test_wma_known_value() {
        let mut wma = Wma::new(3);
        wma.next(1.0);
        wma.next(2.0);
        // (1*1 + 2*2 + 3*3) / (1+2+3) = 14/6
        let result = wma.next(3.0).unwrap();
        assert!((result - 14.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_wma_sliding() {
        let mut wma = Wma::new(3);
        wma.next(1.0);
        wma.next(2.0);
        wma.next(3.0);
        // After pushing 4: buffer = [2, 3, 4] → (2 + 6 + 12)/6 = 20/6
        let result = wma.next(4.0).unwrap();
        assert!((result - 20.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_wma_constant_values() {
        let mut wma = Wma::new(3);
        wma.next(5.0);
        wma.next(5.0);
        let result = wma.next(5.0).unwrap();
        assert!((result - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_wma_reset() {
        let mut wma = Wma::new(3);
        wma.next(1.0);
        wma.next(2.0);
        wma.next(3.0);
        wma.reset();
        assert!(wma.next(10.0).is_none());
    }

    #[test]
    fn test_wma_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Wma::new(20);
        let b = Wma::new(20);
        let c = Wma::new(10);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_wma_param_defs() {
        let defs = Wma::param_defs();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "period");
    }

    #[test]
    fn test_wma_from_params() {
        let wma = Wma::from_params(&[ParamValue::Usize(15)]).unwrap();
        assert_eq!(wma.period(), 15);
    }

    #[test]
    fn test_wma_warmup_period() {
        assert_eq!(Wma::new(7).warmup_period(), 7);
    }
}
