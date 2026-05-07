//! Chandelier Exit stateful implementation

use crate::indicator;
use crate::ring_buffer::MinMaxRingBuffer;
use crate::{Atr, Indicator, Ohlc};

/// Chandelier Exit output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct ChandelierValue {
    pub long_exit: f64,
    pub short_exit: f64,
}

/// Chandelier Exit with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Chandelier {
    #[param(default = 22)]
    period: usize,
    #[param(default = 3.0)]
    atr_mult: f64,

    high_buffer: MinMaxRingBuffer,
    low_buffer: MinMaxRingBuffer,
    atr: Atr,
}

impl Chandelier {
    pub fn new(period: usize, atr_mult: f64) -> Self {
        Self {
            period,
            atr_mult,
            high_buffer: MinMaxRingBuffer::new(period),
            low_buffer: MinMaxRingBuffer::new(period),
            atr: Atr::new(period),
        }
    }

    pub fn period(&self) -> usize { self.period }
    pub fn atr_mult(&self) -> f64 { self.atr_mult }
}

impl Indicator for Chandelier {
    type Input = Ohlc;
    type Output = ChandelierValue;
    const NAME: &'static str = "chandelier";

    fn reset(&mut self) {
        self.high_buffer.reset();
        self.low_buffer.reset();
        self.atr.reset();
    }

    fn next(&mut self, input: Ohlc) -> Option<ChandelierValue> {
        let Ohlc { high, low, .. } = input;
        self.high_buffer.push(high);
        self.low_buffer.push(low);

        let atr = self.atr.next(input)?;

        if !self.high_buffer.is_full() {
            return None;
        }

        let highest = self.high_buffer.max()?;
        let lowest = self.low_buffer.min()?;

        Some(ChandelierValue {
            long_exit: highest - self.atr_mult * atr,
            short_exit: lowest + self.atr_mult * atr,
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

    fn ohlc(h: f64, l: f64, c: f64) -> Ohlc {
        Ohlc::new(h, l, c)
    }

    #[test]
    fn test_chandelier_warmup() {
        let mut ch = Chandelier::new(3, 3.0);
        assert!(ch.next(ohlc(11.0, 9.0, 10.0)).is_none());
        assert!(ch.next(ohlc(12.0, 10.0, 11.0)).is_none());
        // After period bars (and ATR ready), should have value
        let r = ch.next(ohlc(13.0, 11.0, 12.0));
        assert!(r.is_some());
    }

    #[test]
    fn test_chandelier_exit_relationships() {
        // long_exit < highest, short_exit > lowest, given non-zero ATR
        let mut ch = Chandelier::new(3, 3.0);
        let inputs = [
            ohlc(11.0, 9.0, 10.0),
            ohlc(13.0, 10.0, 12.0),
            ohlc(15.0, 11.0, 13.0),
            ohlc(16.0, 12.0, 14.0),
        ];
        let mut last = None;
        for i in inputs {
            last = ch.next(i);
        }
        let v = last.unwrap();
        // Long exit should be below the recent highest high (16.0).
        assert!(v.long_exit < 16.0);
        // Short exit should be above the recent lowest low (10.0).
        assert!(v.short_exit > 10.0);
    }

    #[test]
    fn test_chandelier_constant_collapses() {
        // Zero range bars → ATR = 0 → exits equal high/low
        let mut ch = Chandelier::new(3, 3.0);
        let mut last = None;
        for _ in 0..5 {
            last = ch.next(ohlc(10.0, 10.0, 10.0));
        }
        let v = last.unwrap();
        assert!((v.long_exit - 10.0).abs() < 1e-9);
        assert!((v.short_exit - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_chandelier_reset() {
        let mut ch = Chandelier::new(3, 3.0);
        for v in [
            ohlc(11.0, 9.0, 10.0),
            ohlc(12.0, 10.0, 11.0),
            ohlc(13.0, 11.0, 12.0),
        ] {
            ch.next(v);
        }
        ch.reset();
        assert!(ch.next(ohlc(11.0, 9.0, 10.0)).is_none());
    }

    #[test]
    fn test_chandelier_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Chandelier::new(22, 3.0);
        let b = Chandelier::new(22, 3.0);
        let c = Chandelier::new(22, 2.5);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_chandelier_param_defs() {
        let defs = Chandelier::param_defs();
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].name, "period");
        assert_eq!(defs[1].name, "atr_mult");
    }

    #[test]
    fn test_chandelier_from_params() {
        let ch = Chandelier::from_params(&[ParamValue::Usize(14), ParamValue::F64(2.5)]).unwrap();
        assert_eq!(ch.period(), 14);
        assert!((ch.atr_mult() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn test_chandelier_warmup_period() {
        assert_eq!(Chandelier::new(22, 3.0).warmup_period(), 22);
    }
}
