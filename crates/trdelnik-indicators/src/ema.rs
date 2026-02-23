//! Exponential Moving Average stateful implementation

use crate::indicator;
use crate::Indicator;

/// Exponential Moving Average with O(1) per-bar computation.
///
/// Uses the standard EMA formula: EMA = (Value - EMA_prev) * multiplier + EMA_prev
/// The first EMA value is calculated as an SMA.
#[indicator]
#[derive(Debug, Clone)]
pub struct Ema {
    #[param]
    period: usize,

    multiplier: f64,
    sum: f64,
    count: usize,
    prev_ema: Option<f64>,
}

impl Ema {
    /// Create a new EMA state with the given period
    pub fn new(period: usize) -> Self {
        Self {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            sum: 0.0,
            count: 0,
            prev_ema: None,
        }
    }

    /// Get the period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Get the smoothing multiplier
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    /// Get the current EMA value (if available)
    pub fn current(&self) -> Option<f64> {
        self.prev_ema
    }
}

impl Indicator for Ema {
    type Input = f64;
    type Output = f64;
    const NAME: &'static str = "ema";

    fn reset(&mut self) {
        self.sum = 0.0;
        self.count = 0;
        self.prev_ema = None;
    }

    fn next(&mut self, value: f64) -> Option<f64> {
        self.count += 1;

        if self.count < self.period {
            // Accumulating for initial SMA
            self.sum += value;
            None
        } else if self.count == self.period {
            // First EMA value is SMA
            self.sum += value;
            let sma = self.sum / self.period as f64;
            self.prev_ema = Some(sma);
            Some(sma)
        } else {
            // Standard EMA calculation
            let prev = self.prev_ema.unwrap();
            let ema = (value - prev) * self.multiplier + prev;
            self.prev_ema = Some(ema);
            Some(ema)
        }
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_ema_warmup() {
        let mut ema = Ema::new(3);

        assert!(ema.next(1.0).is_none());
        assert!(ema.next(2.0).is_none());
        assert_eq!(ema.next(3.0), Some(2.0)); // SMA(1,2,3) = 2
    }

    #[test]
    fn test_ema_calculation() {
        let mut ema = Ema::new(3);

        ema.next(1.0);
        ema.next(2.0);
        let first = ema.next(3.0).unwrap(); // 2.0
        assert_eq!(first, 2.0);

        // EMA = (4 - 2) * 0.5 + 2 = 3
        // Multiplier = 2 / (3 + 1) = 0.5
        let second = ema.next(4.0).unwrap();
        assert_eq!(second, 3.0);

        // EMA = (5 - 3) * 0.5 + 3 = 4
        let third = ema.next(5.0).unwrap();
        assert_eq!(third, 4.0);
    }

    #[test]
    fn test_ema_reset() {
        let mut ema = Ema::new(3);

        ema.next(1.0);
        ema.next(2.0);
        ema.next(3.0);

        ema.reset();

        assert!(ema.next(10.0).is_none());
    }

    #[test]
    fn test_ema_multiplier() {
        let ema = Ema::new(12);
        let expected = 2.0 / 13.0;
        assert!((ema.multiplier() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_ema_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let ema1 = Ema::new(12);
        let ema2 = Ema::new(12);
        let ema3 = Ema::new(26);

        assert_eq!(ema1, ema2);
        assert_ne!(ema1, ema3);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        ema1.hash(&mut hasher1);
        ema2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
