//! Percentage Price Oscillator stateful implementation

use crate::indicator;
use crate::{Ema, Indicator};

/// PPO output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct PpoValue {
    pub ppo: f64,
    pub signal: f64,
    pub histogram: f64,
}

/// Percentage Price Oscillator with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Ppo {
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

impl Ppo {
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
}

impl Indicator for Ppo {
    type Input = f64;
    type Output = PpoValue;
    const NAME: &'static str = "ppo";

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
    }

    fn next(&mut self, value: f64) -> Option<PpoValue> {
        let fast = self.fast_ema.next(value);
        let slow = self.slow_ema.next(value);

        match (fast, slow) {
            (Some(f), Some(s)) if s != 0.0 => {
                let ppo_line = ((f - s) / s) * 100.0;
                self.signal_ema.next(ppo_line).map(|signal| PpoValue {
                    ppo: ppo_line,
                    signal,
                    histogram: ppo_line - signal,
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
    fn test_ppo_warmup() {
        // slow=3 + signal=2 - 1 = 4 bars to first output
        let mut ppo = Ppo::new(2, 3, 2);
        for _ in 0..3 {
            assert!(ppo.next(100.0).is_none());
        }
        // After signal_ema can produce
        let mut got_value = false;
        for _ in 0..5 {
            if ppo.next(101.0).is_some() {
                got_value = true;
                break;
            }
        }
        assert!(got_value);
    }

    #[test]
    fn test_ppo_constant_values() {
        // All-equal input → fast == slow → ppo line is 0 (and 0/anything = 0)
        let mut ppo = Ppo::new(2, 3, 2);
        let mut last = None;
        for _ in 0..10 {
            last = ppo.next(100.0);
        }
        let v = last.unwrap();
        assert!((v.ppo - 0.0).abs() < 1e-9);
        assert!((v.signal - 0.0).abs() < 1e-9);
        assert!((v.histogram - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_ppo_histogram_relationship() {
        // histogram = ppo - signal must always hold
        let mut ppo = Ppo::new(2, 3, 2);
        let inputs = [100.0, 102.0, 105.0, 110.0, 108.0, 115.0, 120.0, 118.0];
        for v in inputs {
            if let Some(out) = ppo.next(v) {
                assert!((out.histogram - (out.ppo - out.signal)).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_ppo_reset() {
        let mut ppo = Ppo::new(2, 3, 2);
        for v in [100.0, 102.0, 105.0, 110.0, 108.0, 115.0] {
            ppo.next(v);
        }
        ppo.reset();
        assert!(ppo.next(100.0).is_none());
    }

    #[test]
    fn test_ppo_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Ppo::new(12, 26, 9);
        let b = Ppo::new(12, 26, 9);
        let c = Ppo::new(12, 26, 14);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_ppo_param_defs() {
        let defs = Ppo::param_defs();
        assert_eq!(defs.len(), 3);
        assert_eq!(defs[0].name, "fast_period");
        assert_eq!(defs[1].name, "slow_period");
        assert_eq!(defs[2].name, "signal_period");
    }

    #[test]
    fn test_ppo_from_params() {
        let ppo = Ppo::from_params(&[
            ParamValue::Usize(8),
            ParamValue::Usize(21),
            ParamValue::Usize(5),
        ])
        .unwrap();
        assert_eq!(ppo.fast_period(), 8);
        assert_eq!(ppo.slow_period(), 21);
        assert_eq!(ppo.signal_period(), 5);
    }

    #[test]
    fn test_ppo_warmup_period() {
        assert_eq!(Ppo::new(12, 26, 9).warmup_period(), 26 + 9 - 1);
    }
}
