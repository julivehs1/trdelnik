//! Percent of equity position sizing

use super::{PositionSizer, SizingContext};

/// Position size as a percentage of current equity
#[derive(Debug, Clone, Copy)]
pub struct PercentOfEquity {
    /// Percentage of equity to risk (e.g., 10.0 = 10%)
    percent: f64,
}

impl PercentOfEquity {
    /// Create a new percent of equity sizer
    ///
    /// # Arguments
    /// * `percent` - Percentage of equity to use (e.g., 10.0 for 10%)
    pub fn new(percent: f64) -> Self {
        Self { percent }
    }
}

impl PositionSizer for PercentOfEquity {
    fn calculate_size(&self, ctx: &SizingContext) -> f64 {
        let position_value = ctx.equity * (self.percent / 100.0);
        (position_value / ctx.price).floor()
    }

    fn name(&self) -> &'static str {
        "PercentOfEquity"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PositionSide;

    #[test]
    fn test_percent_of_equity() {
        let sizer = PercentOfEquity::new(10.0);
        let ctx = SizingContext::new(
            100_000.0, // equity
            100_000.0, // cash
            50.0,      // price
            PositionSide::Long,
            1000.0, // x as f64
            0,
        );

        // 10% of 100,000 = 10,000
        // 10,000 / 50 = 200 shares
        assert_eq!(sizer.calculate_size(&ctx), 200.0);
    }

    #[test]
    fn test_percent_of_equity_floors() {
        let sizer = PercentOfEquity::new(10.0);
        let ctx = SizingContext::new(
            100_000.0,
            100_000.0,
            33.0, // doesn't divide evenly
            PositionSide::Long,
            1000.0,
            0,
        );

        // 10% of 100,000 = 10,000
        // 10,000 / 33 = 303.03 -> 303 shares (floored)
        assert_eq!(sizer.calculate_size(&ctx), 303.0);
    }

    #[test]
    fn test_percent_of_equity_name() {
        assert_eq!(PercentOfEquity::new(10.0).name(), "PercentOfEquity");
    }
}
