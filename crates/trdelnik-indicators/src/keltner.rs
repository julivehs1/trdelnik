//! Keltner Channel stateful implementation

use crate::indicator;
use crate::{Atr, Ema, Indicator, Ohlc};

/// Keltner Channel output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct KeltnerValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

/// Keltner Channel with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Keltner {
    #[param(default = 20)]
    ema_period: usize,
    #[param(default = 10)]
    atr_period: usize,
    #[param(default = 2.0)]
    atr_mult: f64,

    ema: Ema,
    atr: Atr,
}

impl Keltner {
    pub fn new(ema_period: usize, atr_period: usize, atr_mult: f64) -> Self {
        Self {
            ema_period,
            atr_period,
            atr_mult,
            ema: Ema::new(ema_period),
            atr: Atr::new(atr_period),
        }
    }

    pub fn ema_period(&self) -> usize { self.ema_period }
    pub fn atr_period(&self) -> usize { self.atr_period }
    pub fn atr_mult(&self) -> f64 { self.atr_mult }
}

impl Indicator for Keltner {
    type Input = Ohlc;
    type Output = KeltnerValue;
    const NAME: &'static str = "keltner";

    fn reset(&mut self) {
        self.ema.reset();
        self.atr.reset();
    }

    fn next(&mut self, input: Ohlc) -> Option<KeltnerValue> {
        let Ohlc { close, .. } = input;
        let middle = self.ema.next(close);
        let atr = self.atr.next(input);

        match (middle, atr) {
            (Some(m), Some(a)) => Some(KeltnerValue {
                upper: m + self.atr_mult * a,
                middle: m,
                lower: m - self.atr_mult * a,
            }),
            _ => None,
        }
    }

    fn warmup_period(&self) -> usize {
        self.ema_period.max(self.atr_period)
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
    fn test_keltner_warmup() {
        let mut k = Keltner::new(3, 3, 2.0);
        // Need both EMA and ATR to be ready (period bars)
        assert!(k.next(ohlc(11.0, 9.0, 10.0)).is_none());
        let _ = k.next(ohlc(12.0, 10.0, 11.0));
        let _ = k.next(ohlc(13.0, 11.0, 12.0));
        // After enough bars some Some(_) must arrive
        let mut got = false;
        for v in [13.5, 14.0, 14.5, 15.0] {
            if k.next(ohlc(v, v - 1.0, v - 0.5)).is_some() {
                got = true;
                break;
            }
        }
        assert!(got);
    }

    #[test]
    fn test_keltner_band_relationship_with_volatility() {
        // With non-zero ATR, upper > middle > lower
        let mut k = Keltner::new(3, 3, 2.0);
        let inputs = [
            ohlc(11.0, 9.0, 10.0),
            ohlc(13.0, 10.0, 12.0),
            ohlc(15.0, 11.0, 13.0),
            ohlc(16.0, 12.0, 14.0),
            ohlc(17.0, 13.0, 15.0),
            ohlc(18.0, 14.0, 16.0),
        ];
        let mut last = None;
        for i in inputs {
            last = k.next(i);
        }
        let v = last.expect("Keltner should have produced a value");
        assert!(v.upper > v.middle, "upper {} must be > middle {}", v.upper, v.middle);
        assert!(v.middle > v.lower, "middle {} must be > lower {}", v.middle, v.lower);
        // Symmetry: middle is centred between upper and lower
        let center = (v.upper + v.lower) / 2.0;
        assert!((center - v.middle).abs() < 1e-9);
    }

    #[test]
    fn test_keltner_constant_collapses_bands() {
        // Zero range → ATR = 0 → upper = middle = lower
        let mut k = Keltner::new(3, 3, 2.0);
        let mut last = None;
        for _ in 0..6 {
            last = k.next(ohlc(10.0, 10.0, 10.0));
        }
        let v = last.unwrap();
        assert!((v.upper - v.middle).abs() < 1e-9);
        assert!((v.middle - v.lower).abs() < 1e-9);
    }

    #[test]
    fn test_keltner_reset() {
        let mut k = Keltner::new(3, 3, 2.0);
        for _ in 0..5 {
            k.next(ohlc(11.0, 9.0, 10.0));
        }
        k.reset();
        assert!(k.next(ohlc(11.0, 9.0, 10.0)).is_none());
    }

    #[test]
    fn test_keltner_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Keltner::new(20, 10, 2.0);
        let b = Keltner::new(20, 10, 2.0);
        let c = Keltner::new(20, 10, 2.5);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_keltner_param_defs() {
        let defs = Keltner::param_defs();
        assert_eq!(defs.len(), 3);
        assert_eq!(defs[0].name, "ema_period");
        assert_eq!(defs[1].name, "atr_period");
        assert_eq!(defs[2].name, "atr_mult");
    }

    #[test]
    fn test_keltner_from_params() {
        let k = Keltner::from_params(&[
            ParamValue::Usize(15),
            ParamValue::Usize(8),
            ParamValue::F64(1.5),
        ])
        .unwrap();
        assert_eq!(k.ema_period(), 15);
        assert_eq!(k.atr_period(), 8);
        assert!((k.atr_mult() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_keltner_warmup_period() {
        assert_eq!(Keltner::new(20, 10, 2.0).warmup_period(), 20);
        assert_eq!(Keltner::new(5, 14, 2.0).warmup_period(), 14);
    }
}
