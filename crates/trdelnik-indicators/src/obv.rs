//! On-Balance Volume stateful implementation

use crate::indicator;
use crate::{Indicator, Ohlcv};

/// On-Balance Volume with O(1) per-bar computation.
/// OBV has no parameters - it's a cumulative indicator.
#[indicator]
#[derive(Debug, Clone)]
pub struct Obv {
    prev_close: Option<f64>,
    obv: f64,
}

impl Obv {
    pub fn new() -> Self {
        Self {
            prev_close: None,
            obv: 0.0,
        }
    }

    pub fn current(&self) -> f64 {
        self.obv
    }
}

impl Default for Obv {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Obv {
    type Input = Ohlcv;
    type Output = f64;
    const NAME: &'static str = "obv";

    fn reset(&mut self) {
        self.prev_close = None;
        self.obv = 0.0;
    }

    fn next(&mut self, input: Ohlcv) -> Option<f64> {
        let Ohlcv { close, volume, .. } = input;

        let Some(prev_close) = self.prev_close else {
            self.prev_close = Some(close);
            self.obv = volume;
            return Some(self.obv);
        };

        if close > prev_close {
            self.obv += volume;
        } else if close < prev_close {
            self.obv -= volume;
        }

        self.prev_close = Some(close);
        Some(self.obv)
    }

    fn warmup_period(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::IndicatorParams;

    fn ohlcv(close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(close, close, close, volume)
    }

    #[test]
    fn test_obv_default() {
        let a = Obv::default();
        let b = Obv::new();
        assert_eq!(a.current(), b.current());
        assert_eq!(a.current(), 0.0);
    }

    #[test]
    fn test_obv_first_value_sets_initial() {
        let mut obv = Obv::new();
        // First bar always returns Some(volume)
        let result = obv.next(ohlcv(100.0, 500.0));
        assert_eq!(result, Some(500.0));
        assert_eq!(obv.current(), 500.0);
    }

    #[test]
    fn test_obv_close_up_adds_volume() {
        let mut obv = Obv::new();
        obv.next(ohlcv(100.0, 1000.0));
        let r = obv.next(ohlcv(101.0, 200.0)).unwrap();
        assert!((r - 1200.0).abs() < 1e-9);
    }

    #[test]
    fn test_obv_close_down_subtracts_volume() {
        let mut obv = Obv::new();
        obv.next(ohlcv(100.0, 1000.0));
        let r = obv.next(ohlcv(99.0, 200.0)).unwrap();
        assert!((r - 800.0).abs() < 1e-9);
    }

    #[test]
    fn test_obv_close_unchanged_no_change() {
        let mut obv = Obv::new();
        obv.next(ohlcv(100.0, 1000.0));
        let r = obv.next(ohlcv(100.0, 999.0)).unwrap();
        assert!((r - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn test_obv_sequence() {
        let mut obv = Obv::new();
        obv.next(ohlcv(10.0, 100.0)); // 100
        obv.next(ohlcv(11.0, 50.0));  // 150
        obv.next(ohlcv(10.5, 30.0));  // 120
        obv.next(ohlcv(10.5, 25.0));  // 120
        let r = obv.next(ohlcv(12.0, 40.0)).unwrap();
        assert!((r - 160.0).abs() < 1e-9);
    }

    #[test]
    fn test_obv_reset() {
        let mut obv = Obv::new();
        obv.next(ohlcv(100.0, 1000.0));
        obv.next(ohlcv(101.0, 500.0));
        obv.reset();
        assert_eq!(obv.current(), 0.0);
        // After reset, next call again seeds with the volume
        let r = obv.next(ohlcv(50.0, 77.0));
        assert_eq!(r, Some(77.0));
    }

    #[test]
    fn test_obv_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Obv::new();
        let b = Obv::new();
        assert_eq!(a, b);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_obv_no_params() {
        let defs = Obv::param_defs();
        assert!(defs.is_empty());
        let _ = Obv::from_params(&[]).unwrap();
    }

    #[test]
    fn test_obv_warmup_zero() {
        assert_eq!(Obv::new().warmup_period(), 0);
    }
}
