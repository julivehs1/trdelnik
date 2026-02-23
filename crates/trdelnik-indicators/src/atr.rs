//! Average True Range stateful implementation

use crate::indicator;
use crate::{Indicator, Ohlc};

/// Average True Range with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Atr {
    #[param]
    period: usize,

    prev_close: Option<f64>,
    tr_sum: f64,
    count: usize,
    prev_atr: Option<f64>,
}

impl Atr {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            prev_close: None,
            tr_sum: 0.0,
            count: 0,
            prev_atr: None,
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }

    fn true_range(&self, high: f64, low: f64, prev_close: f64) -> f64 {
        let hl = high - low;
        let hc = (high - prev_close).abs();
        let lc = (low - prev_close).abs();
        hl.max(hc).max(lc)
    }

    pub fn current(&self) -> Option<f64> {
        self.prev_atr
    }
}

impl Indicator for Atr {
    type Input = Ohlc;
    type Output = f64;
    const NAME: &'static str = "atr";

    fn reset(&mut self) {
        self.prev_close = None;
        self.tr_sum = 0.0;
        self.count = 0;
        self.prev_atr = None;
    }

    fn next(&mut self, input: Ohlc) -> Option<f64> {
        let Ohlc { high, low, close } = input;

        let Some(prev_close) = self.prev_close else {
            let tr = high - low;
            self.tr_sum = tr;
            self.count = 1;
            self.prev_close = Some(close);
            if self.period == 1 {
                self.prev_atr = Some(tr);
                return Some(tr);
            }
            return None;
        };

        let tr = self.true_range(high, low, prev_close);
        self.prev_close = Some(close);
        self.count += 1;

        if self.count < self.period {
            self.tr_sum += tr;
            None
        } else if self.count == self.period {
            self.tr_sum += tr;
            let atr = self.tr_sum / self.period as f64;
            self.prev_atr = Some(atr);
            Some(atr)
        } else {
            let prev_atr = self.prev_atr.unwrap();
            let atr = (prev_atr * (self.period - 1) as f64 + tr) / self.period as f64;
            self.prev_atr = Some(atr);
            Some(atr)
        }
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
}
