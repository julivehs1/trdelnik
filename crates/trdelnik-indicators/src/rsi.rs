//! Relative Strength Index stateful implementation

use crate::indicator;
use crate::Indicator;

/// Relative Strength Index with O(1) per-bar computation.
#[indicator]
#[derive(Debug, Clone)]
pub struct Rsi {
    #[param]
    period: usize,

    prev_value: Option<f64>,
    gain_sum: f64,
    loss_sum: f64,
    avg_gain: f64,
    avg_loss: f64,
    change_count: usize,
}

impl Rsi {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            prev_value: None,
            gain_sum: 0.0,
            loss_sum: 0.0,
            avg_gain: 0.0,
            avg_loss: 0.0,
            change_count: 0,
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }

    fn calculate_rsi(&self) -> f64 {
        if self.avg_loss == 0.0 {
            100.0
        } else {
            let rs = self.avg_gain / self.avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        }
    }
}

impl Indicator for Rsi {
    type Input = f64;
    type Output = f64;
    const NAME: &'static str = "rsi";

    fn reset(&mut self) {
        self.prev_value = None;
        self.gain_sum = 0.0;
        self.loss_sum = 0.0;
        self.avg_gain = 0.0;
        self.avg_loss = 0.0;
        self.change_count = 0;
    }

    fn next(&mut self, value: f64) -> Option<f64> {
        let Some(prev) = self.prev_value else {
            self.prev_value = Some(value);
            return None;
        };

        let change = value - prev;
        self.prev_value = Some(value);

        let (gain, loss) = if change > 0.0 { (change, 0.0) } else { (0.0, -change) };
        self.change_count += 1;

        if self.change_count < self.period {
            self.gain_sum += gain;
            self.loss_sum += loss;
            None
        } else if self.change_count == self.period {
            self.gain_sum += gain;
            self.loss_sum += loss;
            self.avg_gain = self.gain_sum / self.period as f64;
            self.avg_loss = self.loss_sum / self.period as f64;
            Some(self.calculate_rsi())
        } else {
            self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain) / self.period as f64;
            self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss) / self.period as f64;
            Some(self.calculate_rsi())
        }
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }
}
