//! Keltner Channel stateful implementation

use crate::indicator;
use crate::{Atr, Ema, Indicator, Ohlc};

/// Keltner Channel output values
#[derive(Debug, Clone, Copy, PartialEq, crate::IndicatorOutput)]
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
