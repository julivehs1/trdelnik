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

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{IndicatorParams, ParamValue};

    #[test]
    fn test_std_dev_warmup() {
        let mut sd = StdDev::new(3);
        assert!(sd.next(1.0).is_none());
        assert!(sd.next(2.0).is_none());
        assert!(sd.next(3.0).is_some());
    }

    #[test]
    fn test_std_dev_constant_is_zero() {
        let mut sd = StdDev::new(3);
        sd.next(5.0);
        sd.next(5.0);
        let result = sd.next(5.0).unwrap();
        assert!((result - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_std_dev_known_value() {
        // values 2,4,6 → mean=4, var = ((2-4)² + 0 + (6-4)²)/3 = 8/3 → sd = sqrt(8/3) ≈ 1.6329932
        let mut sd = StdDev::new(3);
        sd.next(2.0);
        sd.next(4.0);
        let result = sd.next(6.0).unwrap();
        let expected = (8.0_f64 / 3.0).sqrt();
        assert!((result - expected).abs() < 1e-6, "got {}", result);
    }

    #[test]
    fn test_std_dev_sliding() {
        let mut sd = StdDev::new(3);
        sd.next(1.0);
        sd.next(2.0);
        sd.next(3.0);
        // window slides: 2,3,4 → variance is the same as 1,2,3
        let first = sd.next(4.0).unwrap();
        let mut sd2 = StdDev::new(3);
        sd2.next(1.0);
        sd2.next(2.0);
        let baseline = sd2.next(3.0).unwrap();
        assert!((first - baseline).abs() < 1e-9);
    }

    #[test]
    fn test_std_dev_reset() {
        let mut sd = StdDev::new(3);
        sd.next(1.0);
        sd.next(2.0);
        sd.next(3.0);
        sd.reset();
        assert!(sd.next(10.0).is_none());
    }

    #[test]
    fn test_std_dev_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = StdDev::new(14);
        let b = StdDev::new(14);
        let c = StdDev::new(20);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_std_dev_param_defs() {
        let defs = StdDev::param_defs();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "period");
    }

    #[test]
    fn test_std_dev_from_params() {
        let sd = StdDev::from_params(&[ParamValue::Usize(8)]).unwrap();
        assert_eq!(sd.period(), 8);
    }

    #[test]
    fn test_std_dev_warmup_period() {
        assert_eq!(StdDev::new(5).warmup_period(), 5);
    }
}
