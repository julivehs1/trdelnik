//! Percentage Price Oscillator stateful implementation

use crate::indicator;
use crate::{Ema, Indicator};

/// PPO output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorOutput)]
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
