//! MACD (Moving Average Convergence Divergence) stateful implementation

use crate::indicator;
use crate::{Ema, Indicator};

/// MACD output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct MacdValue {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

/// MACD with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Macd {
    #[param(default = 12)]
    fast_period: usize,
    #[param(default = 26)]
    slow_period: usize,
    #[param(default = 9)]
    signal_period: usize,

    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
}

impl Macd {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            signal_ema: Ema::new(signal_period),
        }
    }

    pub fn fast_period(&self) -> usize { self.fast_period }
    pub fn slow_period(&self) -> usize { self.slow_period }
    pub fn signal_period(&self) -> usize { self.signal_period }

    pub fn next_macd_line(&mut self, value: f64) -> Option<f64> {
        let fast = self.fast_ema.next(value);
        let slow = self.slow_ema.next(value);
        match (fast, slow) {
            (Some(f), Some(s)) => Some(f - s),
            _ => None,
        }
    }
}

impl Indicator for Macd {
    type Input = f64;
    type Output = MacdValue;
    const NAME: &'static str = "macd";

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
    }

    fn next(&mut self, value: f64) -> Option<MacdValue> {
        let fast = self.fast_ema.next(value);
        let slow = self.slow_ema.next(value);

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let macd_line = f - s;
                self.signal_ema.next(macd_line).map(|signal| MacdValue {
                    macd: macd_line,
                    signal,
                    histogram: macd_line - signal,
                })
            }
            _ => None,
        }
    }

    fn warmup_period(&self) -> usize {
        self.slow_period + self.signal_period - 1
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{IndicatorParams, ParamValue};

    #[test]
    fn test_macd_warmup_returns_none() {
        let mut macd = Macd::new(2, 3, 2);
        for _ in 0..3 {
            assert!(macd.next(100.0).is_none());
        }
    }

    #[test]
    fn test_macd_eventually_produces_value() {
        let mut macd = Macd::new(2, 3, 2);
        let mut got = false;
        for v in [100.0, 102.0, 105.0, 110.0, 108.0, 115.0, 120.0, 118.0, 125.0] {
            if macd.next(v).is_some() {
                got = true;
                break;
            }
        }
        assert!(got);
    }

    #[test]
    fn test_macd_constant_input_yields_zero_components() {
        // Constant input → fast == slow → MACD line == 0 → signal == 0 → histogram == 0
        let mut macd = Macd::new(2, 3, 2);
        let mut last = None;
        for _ in 0..15 {
            last = macd.next(100.0);
        }
        let v = last.unwrap();
        assert!((v.macd - 0.0).abs() < 1e-9);
        assert!((v.signal - 0.0).abs() < 1e-9);
        assert!((v.histogram - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_macd_histogram_relationship_holds() {
        let mut macd = Macd::new(2, 3, 2);
        let inputs = [100.0, 102.0, 105.0, 110.0, 108.0, 115.0, 120.0, 118.0];
        for v in inputs {
            if let Some(out) = macd.next(v) {
                assert!((out.histogram - (out.macd - out.signal)).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_macd_next_macd_line_helper() {
        // Helper exists separately; mirrors the diff between fast and slow EMAs.
        let mut macd = Macd::new(2, 3, 2);
        let inputs = [100.0, 102.0, 105.0, 110.0, 108.0, 115.0];
        let mut got = false;
        for v in inputs {
            if let Some(line) = macd.next_macd_line(v) {
                // line is a finite number
                assert!(line.is_finite());
                got = true;
            }
        }
        assert!(got);
    }

    #[test]
    fn test_macd_reset() {
        let mut macd = Macd::new(2, 3, 2);
        for v in [100.0, 102.0, 105.0, 110.0, 108.0, 115.0] {
            macd.next(v);
        }
        macd.reset();
        assert!(macd.next(100.0).is_none());
    }

    #[test]
    fn test_macd_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Macd::new(12, 26, 9);
        let b = Macd::new(12, 26, 9);
        let c = Macd::new(12, 26, 14);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_macd_param_defs() {
        let defs = Macd::param_defs();
        assert_eq!(defs.len(), 3);
        assert_eq!(defs[0].name, "fast_period");
        assert_eq!(defs[1].name, "slow_period");
        assert_eq!(defs[2].name, "signal_period");
    }

    #[test]
    fn test_macd_from_params() {
        let macd = Macd::from_params(&[
            ParamValue::Usize(8),
            ParamValue::Usize(21),
            ParamValue::Usize(5),
        ])
        .unwrap();
        assert_eq!(macd.fast_period(), 8);
        assert_eq!(macd.slow_period(), 21);
        assert_eq!(macd.signal_period(), 5);
    }

    #[test]
    fn test_macd_warmup_period() {
        assert_eq!(Macd::new(12, 26, 9).warmup_period(), 26 + 9 - 1);
    }
}
