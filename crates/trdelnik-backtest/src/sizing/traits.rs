//! Position sizing trait and context

use crate::models::PositionSide;

/// Context provided to position sizers for calculating position size
///
/// This is a simplified context that doesn't use generics to allow
/// `PositionSizer` to be dyn-compatible.
#[derive(Debug, Clone)]
pub struct SizingContext {
    /// Current equity
    pub equity: f64,
    /// Available cash
    pub cash: f64,
    /// Entry price
    pub price: f64,
    /// Position side
    pub side: PositionSide,
    /// Stop loss price (if known)
    pub stop_loss: Option<f64>,
    /// ATR value at entry (if available)
    pub atr: Option<f64>,
    /// X coordinate as plot value
    pub x: f64,
    /// Bar index
    pub bar_index: usize,
    /// Number of current open positions
    pub open_position_count: usize,
}

impl SizingContext {
    /// Create a new sizing context
    pub fn new(
        equity: f64,
        cash: f64,
        price: f64,
        side: PositionSide,
        x: f64,
        bar_index: usize,
    ) -> Self {
        Self {
            equity,
            cash,
            price,
            side,
            stop_loss: None,
            atr: None,
            x,
            bar_index,
            open_position_count: 0,
        }
    }

    /// Set stop loss
    pub fn with_stop_loss(mut self, stop_loss: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    /// Set ATR
    pub fn with_atr(mut self, atr: f64) -> Self {
        self.atr = Some(atr);
        self
    }

    /// Set open position count
    pub fn with_open_positions(mut self, count: usize) -> Self {
        self.open_position_count = count;
        self
    }

    /// Calculate risk per share based on stop loss
    pub fn risk_per_share(&self) -> Option<f64> {
        self.stop_loss.map(|sl| (self.price - sl).abs())
    }
}

/// Trait for position sizing strategies
pub trait PositionSizer: Send + Sync {
    /// Calculate the position size (quantity) for a trade
    fn calculate_size(&self, ctx: &SizingContext) -> f64;

    /// Name of the sizer (for logging/display)
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> SizingContext {
        SizingContext::new(10_000.0, 10_000.0, 100.0, PositionSide::Long, 0.0, 0)
    }

    #[test]
    fn test_with_atr_sets_field() {
        let c = ctx().with_atr(2.5);
        assert_eq!(c.atr, Some(2.5));
    }

    #[test]
    fn test_with_open_positions_sets_field() {
        let c = ctx().with_open_positions(5);
        assert_eq!(c.open_position_count, 5);
    }

    #[test]
    fn test_risk_per_share_some_when_stop_loss_set() {
        let c = ctx().with_stop_loss(95.0);
        // 100 - 95 = 5
        assert_eq!(c.risk_per_share(), Some(5.0));
    }

    #[test]
    fn test_risk_per_share_none_when_no_stop_loss() {
        assert!(ctx().risk_per_share().is_none());
    }

    #[test]
    fn test_risk_per_share_uses_absolute_value() {
        // For shorts, stop_loss > price → risk should still be positive.
        let c = SizingContext::new(10_000.0, 10_000.0, 100.0, PositionSide::Short, 0.0, 0)
            .with_stop_loss(105.0);
        assert_eq!(c.risk_per_share(), Some(5.0));
    }
}
