//! Commodity Channel Index stateful implementation

use crate::indicator;
use crate::ring_buffer::RingBuffer;
use crate::{Indicator, Ohlc};

/// Commodity Channel Index with O(n) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Cci {
    #[param(default = 20)]
    period: usize,
    #[param(default = 0.015)]
    constant: f64,

    buffer: RingBuffer,
    tp_values: Vec<f64>,
}

impl Cci {
    pub fn new(period: usize, constant: f64) -> Self {
        Self {
            period,
            constant,
            buffer: RingBuffer::new(period),
            tp_values: Vec::with_capacity(period),
        }
    }

    pub fn period(&self) -> usize { self.period }
    pub fn constant(&self) -> f64 { self.constant }
}

impl Indicator for Cci {
    type Input = Ohlc;
    type Output = f64;
    const NAME: &'static str = "cci";

    fn reset(&mut self) {
        self.buffer.reset();
        self.tp_values.clear();
    }

    fn next(&mut self, input: Ohlc) -> Option<f64> {
        let Ohlc { high, low, close } = input;
        let tp = (high + low + close) / 3.0;
        self.buffer.push(tp);

        if self.tp_values.len() >= self.period {
            self.tp_values.remove(0);
        }
        self.tp_values.push(tp);

        if !self.buffer.is_full() {
            return None;
        }

        let sma = self.buffer.average()?;
        let mad: f64 = self.tp_values.iter().map(|&v| (v - sma).abs()).sum::<f64>()
            / self.period as f64;

        if mad == 0.0 {
            return Some(0.0);
        }

        Some((tp - sma) / (self.constant * mad))
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
    fn test_cci_warmup() {
        let mut cci = Cci::new(3, 0.015);
        assert!(cci.next(ohlc(11.0, 9.0, 10.0)).is_none());
        assert!(cci.next(ohlc(12.0, 10.0, 11.0)).is_none());
        assert!(cci.next(ohlc(13.0, 11.0, 12.0)).is_some());
    }

    #[test]
    fn test_cci_constant_returns_zero() {
        // All identical TPs → MAD = 0 → indicator returns 0
        let mut cci = Cci::new(3, 0.015);
        cci.next(ohlc(10.0, 10.0, 10.0));
        cci.next(ohlc(10.0, 10.0, 10.0));
        let r = cci.next(ohlc(10.0, 10.0, 10.0)).unwrap();
        assert_eq!(r, 0.0);
    }

    #[test]
    fn test_cci_above_average_positive() {
        // Last TP higher than mean → CCI > 0
        let mut cci = Cci::new(3, 0.015);
        cci.next(ohlc(10.0, 9.0, 9.5));
        cci.next(ohlc(10.5, 9.5, 10.0));
        let r = cci.next(ohlc(15.0, 14.0, 14.5)).unwrap();
        assert!(r > 0.0);
    }

    #[test]
    fn test_cci_below_average_negative() {
        let mut cci = Cci::new(3, 0.015);
        cci.next(ohlc(15.0, 14.0, 14.5));
        cci.next(ohlc(14.5, 13.5, 14.0));
        let r = cci.next(ohlc(10.0, 9.0, 9.5)).unwrap();
        assert!(r < 0.0);
    }

    #[test]
    fn test_cci_reset() {
        let mut cci = Cci::new(3, 0.015);
        cci.next(ohlc(10.0, 9.0, 9.5));
        cci.next(ohlc(11.0, 10.0, 10.5));
        cci.next(ohlc(12.0, 11.0, 11.5));
        cci.reset();
        assert!(cci.next(ohlc(10.0, 9.0, 9.5)).is_none());
    }

    #[test]
    fn test_cci_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Cci::new(20, 0.015);
        let b = Cci::new(20, 0.015);
        let c = Cci::new(20, 0.02);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_cci_param_defs() {
        let defs = Cci::param_defs();
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].name, "period");
        assert_eq!(defs[1].name, "constant");
    }

    #[test]
    fn test_cci_from_params() {
        let cci = Cci::from_params(&[ParamValue::Usize(14), ParamValue::F64(0.02)]).unwrap();
        assert_eq!(cci.period(), 14);
        assert!((cci.constant() - 0.02).abs() < 1e-9);
    }

    #[test]
    fn test_cci_warmup_period() {
        assert_eq!(Cci::new(20, 0.015).warmup_period(), 20);
    }
}
