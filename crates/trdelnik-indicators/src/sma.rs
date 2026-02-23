//! Simple Moving Average stateful implementation

use crate::indicator;
use crate::ring_buffer::RingBuffer;
use crate::Indicator;

/// Simple Moving Average with O(1) per-bar computation.
///
/// Maintains a ring buffer with running sum for constant-time average calculation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Sma {
    #[param]
    period: usize,

    buffer: RingBuffer,
}

impl Sma {
    /// Create a new SMA state with the given period
    pub fn new(period: usize) -> Self {
        Self {
            period,
            buffer: RingBuffer::new(period),
        }
    }

    /// Get the period
    pub fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Sma {
    type Input = f64;
    type Output = f64;
    const NAME: &'static str = "sma";

    fn reset(&mut self) {
        self.buffer.reset();
    }

    fn next(&mut self, value: f64) -> Option<f64> {
        self.buffer.push(value);
        if self.buffer.is_full() {
            self.buffer.average()
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
    fn test_sma_warmup() {
        let mut sma = Sma::new(3);

        assert!(sma.next(1.0).is_none());
        assert!(sma.next(2.0).is_none());
        assert_eq!(sma.next(3.0), Some(2.0)); // (1+2+3)/3
    }

    #[test]
    fn test_sma_sliding() {
        let mut sma = Sma::new(3);

        sma.next(1.0);
        sma.next(2.0);
        sma.next(3.0);

        assert_eq!(sma.next(4.0), Some(3.0)); // (2+3+4)/3
        assert_eq!(sma.next(5.0), Some(4.0)); // (3+4+5)/3
    }

    #[test]
    fn test_sma_reset() {
        let mut sma = Sma::new(3);

        sma.next(1.0);
        sma.next(2.0);
        sma.next(3.0);

        sma.reset();

        assert!(sma.next(10.0).is_none());
    }

    #[test]
    fn test_sma_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let sma1 = Sma::new(20);
        let sma2 = Sma::new(20);
        let sma3 = Sma::new(10);

        assert_eq!(sma1, sma2);
        assert_ne!(sma1, sma3);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        sma1.hash(&mut hasher1);
        sma2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_sma_param_defs() {
        let defs = Sma::param_defs();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "period");
    }

    #[test]
    fn test_sma_from_params() {
        let sma = Sma::from_params(&[ParamValue::Usize(20)]).unwrap();
        assert_eq!(sma.period(), 20);
    }
}
