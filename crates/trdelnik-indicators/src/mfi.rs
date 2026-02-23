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
