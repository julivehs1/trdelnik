//! Money Flow Index stateful implementation

use crate::indicator;
use crate::{Indicator, Ohlcv};

/// Money Flow Index with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Mfi {
    #[param(default = 14)]
    period: usize,

    prev_tp: Option<f64>,
    positive_mf_sum: f64,
    negative_mf_sum: f64,
    avg_positive_mf: f64,
    avg_negative_mf: f64,
    change_count: usize,
}

impl Mfi {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            prev_tp: None,
            positive_mf_sum: 0.0,
            negative_mf_sum: 0.0,
            avg_positive_mf: 0.0,
            avg_negative_mf: 0.0,
            change_count: 0,
        }
    }

    pub fn period(&self) -> usize { self.period }

    fn calculate_mfi(&self) -> f64 {
        if self.avg_negative_mf == 0.0 {
            100.0
        } else {
            let mf_ratio = self.avg_positive_mf / self.avg_negative_mf;
            100.0 - (100.0 / (1.0 + mf_ratio))
        }
    }
}

impl Indicator for Mfi {
    type Input = Ohlcv;
    type Output = f64;
    const NAME: &'static str = "mfi";

    fn reset(&mut self) {
        self.prev_tp = None;
        self.positive_mf_sum = 0.0;
        self.negative_mf_sum = 0.0;
        self.avg_positive_mf = 0.0;
        self.avg_negative_mf = 0.0;
        self.change_count = 0;
    }

    fn next(&mut self, input: Ohlcv) -> Option<f64> {
        let Ohlcv { high, low, close, volume } = input;
        let tp = (high + low + close) / 3.0;
        let raw_mf = tp * volume;

        let Some(prev_tp) = self.prev_tp else {
            self.prev_tp = Some(tp);
            return None;
        };

        self.prev_tp = Some(tp);

        let (positive_mf, negative_mf) = if tp > prev_tp {
            (raw_mf, 0.0)
        } else if tp < prev_tp {
            (0.0, raw_mf)
        } else {
            (0.0, 0.0)
        };

        self.change_count += 1;

        if self.change_count < self.period {
            self.positive_mf_sum += positive_mf;
            self.negative_mf_sum += negative_mf;
            None
        } else if self.change_count == self.period {
            self.positive_mf_sum += positive_mf;
            self.negative_mf_sum += negative_mf;
            self.avg_positive_mf = self.positive_mf_sum / self.period as f64;
            self.avg_negative_mf = self.negative_mf_sum / self.period as f64;
            Some(self.calculate_mfi())
        } else {
            self.avg_positive_mf = (self.avg_positive_mf * (self.period - 1) as f64 + positive_mf)
                / self.period as f64;
            self.avg_negative_mf = (self.avg_negative_mf * (self.period - 1) as f64 + negative_mf)
                / self.period as f64;
            Some(self.calculate_mfi())
        }
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{IndicatorParams, ParamValue};

    fn ohlcv(c: f64, v: f64) -> Ohlcv {
        Ohlcv::new(c, c, c, v)
    }

    #[test]
    fn test_mfi_warmup() {
        // period=3 → needs 4 bars total (1 to seed prev_tp + 3 changes)
        let mut mfi = Mfi::new(3);
        assert!(mfi.next(ohlcv(10.0, 100.0)).is_none()); // seed
        assert!(mfi.next(ohlcv(11.0, 100.0)).is_none()); // change 1
        assert!(mfi.next(ohlcv(12.0, 100.0)).is_none()); // change 2
        assert!(mfi.next(ohlcv(13.0, 100.0)).is_some()); // change 3 → first value
    }

    #[test]
    fn test_mfi_all_rising_is_100() {
        // All positive money flow, no negative → MFI = 100
        let mut mfi = Mfi::new(3);
        mfi.next(ohlcv(10.0, 100.0));
        mfi.next(ohlcv(11.0, 100.0));
        mfi.next(ohlcv(12.0, 100.0));
        let r = mfi.next(ohlcv(13.0, 100.0)).unwrap();
        assert!((r - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_mfi_all_falling_is_zero() {
        // All negative money flow, no positive → MFI = 0
        let mut mfi = Mfi::new(3);
        mfi.next(ohlcv(13.0, 100.0));
        mfi.next(ohlcv(12.0, 100.0));
        mfi.next(ohlcv(11.0, 100.0));
        let r = mfi.next(ohlcv(10.0, 100.0)).unwrap();
        assert!((r - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_mfi_in_range() {
        let mut mfi = Mfi::new(3);
        for (c, v) in [(10.0, 100.0), (11.0, 50.0), (10.5, 80.0), (12.0, 60.0), (11.5, 70.0)] {
            if let Some(r) = mfi.next(ohlcv(c, v)) {
                assert!((0.0..=100.0).contains(&r), "MFI out of range: {}", r);
            }
        }
    }

    #[test]
    fn test_mfi_reset() {
        let mut mfi = Mfi::new(3);
        for (c, v) in [(10.0, 100.0), (11.0, 100.0), (12.0, 100.0), (13.0, 100.0)] {
            mfi.next(ohlcv(c, v));
        }
        mfi.reset();
        // After reset, first call again seeds, returns None
        assert!(mfi.next(ohlcv(10.0, 100.0)).is_none());
    }

    #[test]
    fn test_mfi_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Mfi::new(14);
        let b = Mfi::new(14);
        let c = Mfi::new(20);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_mfi_param_defs() {
        let defs = Mfi::param_defs();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "period");
    }

    #[test]
    fn test_mfi_from_params() {
        let mfi = Mfi::from_params(&[ParamValue::Usize(10)]).unwrap();
        assert_eq!(mfi.period(), 10);
    }

    #[test]
    fn test_mfi_warmup_period() {
        assert_eq!(Mfi::new(14).warmup_period(), 15);
    }
}
