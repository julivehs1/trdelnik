//! Stochastic Oscillator stateful implementation

use crate::indicator;
use crate::ring_buffer::{MinMaxRingBuffer, RingBuffer};
use crate::{Indicator, Ohlc};

/// Stochastic Oscillator output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorValue)]
pub struct StochasticValue {
    pub k: f64,
    pub d: f64,
}

/// Stochastic Oscillator with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Stochastic {
    #[param(default = 14)]
    k_period: usize,
    #[param(default = 3)]
    d_period: usize,

    high_buffer: MinMaxRingBuffer,
    low_buffer: MinMaxRingBuffer,
    k_buffer: RingBuffer,
}

impl Stochastic {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        Self {
            k_period,
            d_period,
            high_buffer: MinMaxRingBuffer::new(k_period),
            low_buffer: MinMaxRingBuffer::new(k_period),
            k_buffer: RingBuffer::new(d_period),
        }
    }

    pub fn k_period(&self) -> usize { self.k_period }
    pub fn d_period(&self) -> usize { self.d_period }

    pub fn next_k(&mut self, input: Ohlc) -> Option<f64> {
        let Ohlc { high, low, close } = input;
        self.high_buffer.push(high);
        self.low_buffer.push(low);
        if !self.high_buffer.is_full() {
            return None;
        }
        let highest = self.high_buffer.max().unwrap();
        let lowest = self.low_buffer.min().unwrap();
        let range = highest - lowest;
        let k = if range == 0.0 { 50.0 } else { 100.0 * (close - lowest) / range };
        Some(k)
    }
}

impl Indicator for Stochastic {
    type Input = Ohlc;
    type Output = StochasticValue;
    const NAME: &'static str = "stochastic";

    fn reset(&mut self) {
        self.high_buffer.reset();
        self.low_buffer.reset();
        self.k_buffer.reset();
    }

    fn next(&mut self, input: Ohlc) -> Option<StochasticValue> {
        let Ohlc { high, low, close } = input;
        self.high_buffer.push(high);
        self.low_buffer.push(low);
        if !self.high_buffer.is_full() {
            return None;
        }
        let highest = self.high_buffer.max().unwrap();
        let lowest = self.low_buffer.min().unwrap();
        let range = highest - lowest;
        let k = if range == 0.0 { 50.0 } else { 100.0 * (close - lowest) / range };
        self.k_buffer.push(k);
        if !self.k_buffer.is_full() {
            return None;
        }
        Some(StochasticValue { k, d: self.k_buffer.average().unwrap() })
    }

    fn warmup_period(&self) -> usize {
        self.k_period + self.d_period - 1
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
    fn test_stochastic_warmup() {
        let mut s = Stochastic::new(3, 2);
        // Need k_period = 3 highs/lows, then d_period = 2 %K values
        assert!(s.next(ohlc(11.0, 9.0, 10.0)).is_none());
        assert!(s.next(ohlc(12.0, 10.0, 11.0)).is_none());
        assert!(s.next(ohlc(13.0, 11.0, 12.0)).is_none()); // first %K but not enough %K for %D
        let v = s.next(ohlc(14.0, 12.0, 13.0));
        assert!(v.is_some());
    }

    #[test]
    fn test_stochastic_k_at_top_of_range() {
        // Close == highest → %K == 100
        let mut s = Stochastic::new(3, 2);
        s.next(ohlc(10.0, 5.0, 7.0));
        s.next(ohlc(11.0, 6.0, 8.0));
        s.next(ohlc(12.0, 7.0, 12.0)); // close = high = 12, lowest low = 5 → %K = 100
        let v = s.next(ohlc(13.0, 8.0, 13.0)).unwrap();
        // After this, %K = 100 again. %D is average of last two %K.
        assert!(v.k > 99.0);
    }

    #[test]
    fn test_stochastic_k_at_bottom_of_range() {
        // Close == lowest → %K == 0
        let mut s = Stochastic::new(3, 2);
        s.next(ohlc(20.0, 10.0, 15.0));
        s.next(ohlc(21.0, 11.0, 14.0));
        s.next(ohlc(22.0, 9.0, 9.0)); // close = low = 9, highest high = 22 → %K close to 0
        let v = s.next(ohlc(23.0, 8.0, 8.0)).unwrap();
        assert!(v.k < 1.0);
    }

    #[test]
    fn test_stochastic_zero_range_returns_50() {
        // All H == L → range = 0 → %K = 50
        let mut s = Stochastic::new(3, 2);
        for _ in 0..4 {
            s.next(ohlc(10.0, 10.0, 10.0));
        }
        let v = s.next(ohlc(10.0, 10.0, 10.0)).unwrap();
        assert!((v.k - 50.0).abs() < 1e-9);
        assert!((v.d - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_stochastic_k_in_range_0_100() {
        let mut s = Stochastic::new(3, 2);
        for (h, l, c) in [
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.5),
            (14.0, 12.0, 13.5),
            (15.0, 13.0, 14.0),
            (15.5, 13.5, 14.5),
        ] {
            if let Some(v) = s.next(ohlc(h, l, c)) {
                assert!((0.0..=100.0).contains(&v.k), "k out of range: {}", v.k);
                assert!((0.0..=100.0).contains(&v.d), "d out of range: {}", v.d);
            }
        }
    }

    #[test]
    fn test_stochastic_d_is_avg_of_last_k() {
        // Manually verify: %D = avg of last d_period %K values
        let mut s = Stochastic::new(3, 2);
        let inputs = [
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.5),
            (14.0, 12.0, 13.5),
        ];
        let mut last_two_k = Vec::new();
        let mut last_d = None;
        for (h, l, c) in inputs {
            if let Some(v) = s.next(ohlc(h, l, c)) {
                last_two_k.push(v.k);
                if last_two_k.len() > 2 {
                    last_two_k.remove(0);
                }
                last_d = Some(v.d);
                if last_two_k.len() == 2 {
                    let expected = (last_two_k[0] + last_two_k[1]) / 2.0;
                    assert!((v.d - expected).abs() < 1e-9);
                }
            }
        }
        assert!(last_d.is_some());
    }

    #[test]
    fn test_stochastic_next_k_helper_does_not_consume_d_buffer() {
        // next_k advances the H/L buffers without producing %D; sanity check.
        let mut s = Stochastic::new(3, 2);
        s.next_k(ohlc(11.0, 9.0, 10.0));
        s.next_k(ohlc(12.0, 10.0, 11.0));
        let k = s.next_k(ohlc(13.0, 11.0, 12.0));
        assert!(k.is_some());
    }

    #[test]
    fn test_stochastic_reset() {
        let mut s = Stochastic::new(3, 2);
        for (h, l, c) in [
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
        ] {
            s.next(ohlc(h, l, c));
        }
        s.reset();
        assert!(s.next(ohlc(11.0, 9.0, 10.0)).is_none());
    }

    #[test]
    fn test_stochastic_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Stochastic::new(14, 3);
        let b = Stochastic::new(14, 3);
        let c = Stochastic::new(14, 5);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_stochastic_param_defs() {
        let defs = Stochastic::param_defs();
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].name, "k_period");
        assert_eq!(defs[1].name, "d_period");
    }

    #[test]
    fn test_stochastic_from_params() {
        let s = Stochastic::from_params(&[ParamValue::Usize(14), ParamValue::Usize(3)]).unwrap();
        assert_eq!(s.k_period(), 14);
        assert_eq!(s.d_period(), 3);
    }

    #[test]
    fn test_stochastic_warmup_period() {
        assert_eq!(Stochastic::new(14, 3).warmup_period(), 14 + 3 - 1);
    }
}
