//! Bollinger Bands stateful implementation

use crate::indicator;
use crate::ring_buffer::RingBuffer;
use crate::Indicator;

/// Bollinger Bands output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct BollingerValue {
    /// Upper band (middle + std_dev_mult * std_dev)
    pub upper: f64,
    /// Middle band (SMA)
    pub middle: f64,
    /// Lower band (middle - std_dev_mult * std_dev)
    pub lower: f64,
}

/// Bollinger Bands with O(1) per-bar computation.
///
/// Uses a ring buffer with running sum and sum of squares for
/// constant-time SMA and standard deviation calculation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Bollinger {
    #[param(default = 20)]
    period: usize,
    #[param(default = 2.0)]
    std_dev_mult: f64,

    buffer: RingBuffer,
}

impl Bollinger {
    /// Create new Bollinger Bands state
    pub fn new(period: usize, std_dev_mult: f64) -> Self {
        Self {
            period,
            std_dev_mult,
            buffer: RingBuffer::new(period),
        }
    }

    /// Get the period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Get the standard deviation multiplier
    pub fn std_dev_mult(&self) -> f64 {
        self.std_dev_mult
    }
}

impl Indicator for Bollinger {
    type Input = f64;
    type Output = BollingerValue;
    const NAME: &'static str = "bollinger";

    fn reset(&mut self) {
        self.buffer.reset();
    }

    fn next(&mut self, value: f64) -> Option<BollingerValue> {
        self.buffer.push(value);

        if !self.buffer.is_full() {
            return None;
        }

        let middle = self.buffer.average()?;
        let std_dev = self.buffer.std_dev()?;

        Some(BollingerValue {
            upper: middle + self.std_dev_mult * std_dev,
            middle,
            lower: middle - self.std_dev_mult * std_dev,
        })
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
    fn test_bollinger_warmup() {
        let mut bb = Bollinger::new(3, 2.0);

        assert!(bb.next(100.0).is_none());
        assert!(bb.next(101.0).is_none());
        assert!(bb.next(102.0).is_some());
    }

    #[test]
    fn test_bollinger_middle_is_sma() {
        let mut bb = Bollinger::new(3, 2.0);

        bb.next(1.0);
        bb.next(2.0);
        let result = bb.next(3.0).unwrap();

        // Middle should be SMA = (1 + 2 + 3) / 3 = 2
        assert_eq!(result.middle, 2.0);
    }

    #[test]
    fn test_bollinger_bands_relationship() {
        let mut bb = Bollinger::new(3, 2.0);

        // Use values with some variance
        bb.next(100.0);
        bb.next(110.0);
        let result = bb.next(105.0).unwrap();

        // Upper > Middle > Lower
        assert!(result.upper > result.middle);
        assert!(result.middle > result.lower);
    }

    #[test]
    fn test_bollinger_constant_values() {
        let mut bb = Bollinger::new(3, 2.0);

        bb.next(100.0);
        bb.next(100.0);
        let result = bb.next(100.0).unwrap();

        // With constant values, all bands should be equal
        assert_eq!(result.upper, 100.0);
        assert_eq!(result.middle, 100.0);
        assert_eq!(result.lower, 100.0);
    }

    #[test]
    fn test_bollinger_reset() {
        let mut bb = Bollinger::new(3, 2.0);

        bb.next(100.0);
        bb.next(101.0);
        bb.next(102.0);

        bb.reset();

        assert!(bb.next(100.0).is_none());
    }

    #[test]
    fn test_bollinger_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let bb1 = Bollinger::new(20, 2.0);
        let bb2 = Bollinger::new(20, 2.0);
        let bb3 = Bollinger::new(20, 2.5);

        assert_eq!(bb1, bb2);
        assert_ne!(bb1, bb3);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        bb1.hash(&mut hasher1);
        bb2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_bollinger_param_defs() {
        let defs = Bollinger::param_defs();
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].name, "period");
        assert_eq!(defs[1].name, "std_dev_mult");
    }

    #[test]
    fn test_bollinger_from_params() {
        let bb = Bollinger::from_params(&[ParamValue::Usize(10), ParamValue::F64(3.0)]).unwrap();
        assert_eq!(bb.period(), 10);
        assert_eq!(bb.std_dev_mult(), 3.0);
    }
}
