//! Risk-based position sizing

use super::{PositionSizer, SizingContext};

/// Position size based on risk per trade
///
/// Calculates position size such that if the stop loss is hit,
/// the loss equals a specified percentage of equity.
#[derive(Debug, Clone, Copy)]
pub struct RiskBased {
    /// Percentage of equity to risk per trade
    risk_pct: f64,
    /// Default stop loss percentage if none provided
    default_stop_pct: f64,
}

impl RiskBased {
    /// Create a new risk-based position sizer
    ///
    /// # Arguments
    /// * `risk_pct` - Percentage of equity to risk per trade (e.g., 1.0 for 1%)
    /// * `default_stop_pct` - Default stop loss percentage if none in context
    pub fn new(risk_pct: f64, default_stop_pct: f64) -> Self {
        Self {
            risk_pct,
            default_stop_pct,
        }
    }

    /// Create with just risk percentage (requires stop loss in context)
    pub fn with_risk(risk_pct: f64) -> Self {
        Self::new(risk_pct, 2.0) // 2% default stop
    }
}

impl PositionSizer for RiskBased {
    fn calculate_size(&self, ctx: &SizingContext) -> f64 {
        // Amount willing to lose on this trade
        let risk_amount = ctx.equity * (self.risk_pct / 100.0);

        // Risk per share (distance to stop loss)
        let risk_per_share = ctx.risk_per_share().unwrap_or_else(|| {
            // If no stop loss provided, use default percentage
            ctx.price * (self.default_stop_pct / 100.0)
        });

        // Avoid division by zero
        if risk_per_share < f64::EPSILON {
            return 0.0;
        }

        // Position size = risk amount / risk per share
        (risk_amount / risk_per_share).floor()
    }

    fn name(&self) -> &'static str {
        "RiskBased"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PositionSide;

    #[test]
    fn test_risk_based_with_stop_loss() {
        let sizer = RiskBased::with_risk(1.0); // Risk 1% per trade

        let ctx = SizingContext::new(
            100_000.0,
            100_000.0,
            100.0, // entry at $100
            PositionSide::Long,
            1000.0,
            0,
        )
        .with_stop_loss(98.0); // stop at $98

        // Risk amount = 1% of 100,000 = $1,000
        // Risk per share = $100 - $98 = $2
        // Position size = $1,000 / $2 = 500 shares
        assert_eq!(sizer.calculate_size(&ctx), 500.0);
    }

    #[test]
    fn test_risk_based_without_stop_loss() {
        let sizer = RiskBased::new(1.0, 2.0); // 1% risk, 2% default stop

        let ctx = SizingContext::new(
            100_000.0,
            100_000.0,
            100.0,
            PositionSide::Long,
            1000.0,
            0,
        );
        // No stop loss provided

        // Risk amount = 1% of 100,000 = $1,000
        // Default risk per share = 2% of $100 = $2
        // Position size = $1,000 / $2 = 500 shares
        assert_eq!(sizer.calculate_size(&ctx), 500.0);
    }

    #[test]
    fn test_risk_based_short() {
        let sizer = RiskBased::with_risk(2.0); // Risk 2% per trade

        let ctx = SizingContext::new(
            50_000.0,
            50_000.0,
            100.0, // entry at $100
            PositionSide::Short,
            1000.0,
            0,
        )
        .with_stop_loss(105.0); // stop at $105 for short

        // Risk amount = 2% of 50,000 = $1,000
        // Risk per share = |100 - 105| = $5
        // Position size = $1,000 / $5 = 200 shares
        assert_eq!(sizer.calculate_size(&ctx), 200.0);
    }
}
