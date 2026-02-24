//! Position types for tracking open trades

use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;
use uuid::Uuid;

/// Direction of a position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionSide {
    /// Long position (buy low, sell high)
    Long,
    /// Short position (sell high, buy low)
    Short,
}

impl PositionSide {
    /// Get the opposite side
    #[inline]
    pub fn opposite(&self) -> Self {
        match self {
            PositionSide::Long => PositionSide::Short,
            PositionSide::Short => PositionSide::Long,
        }
    }

    /// Get the direction multiplier (+1 for long, -1 for short)
    #[inline]
    pub fn direction(&self) -> f64 {
        match self {
            PositionSide::Long => 1.0,
            PositionSide::Short => -1.0,
        }
    }

    /// Returns true if this is a long position
    #[inline]
    pub fn is_long(&self) -> bool {
        matches!(self, PositionSide::Long)
    }

    /// Returns true if this is a short position
    #[inline]
    pub fn is_short(&self) -> bool {
        matches!(self, PositionSide::Short)
    }
}

impl std::fmt::Display for PositionSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionSide::Long => write!(f, "Long"),
            PositionSide::Short => write!(f, "Short"),
        }
    }
}

/// An open position in the backtest
#[derive(Debug, Clone)]
pub struct Position<X: AxisCoordinate> {
    /// Unique identifier for this position
    pub id: Uuid,
    /// Position side (long or short)
    pub side: PositionSide,
    /// Entry price
    pub entry_price: f64,
    /// Position size (quantity)
    pub quantity: f64,
    /// X coordinate at entry (timestamp, slot, etc.)
    pub entry_x: X,
    /// Bar index at entry
    pub entry_bar: usize,
    /// Stop loss price (if set)
    pub stop_loss: Option<f64>,
    /// Take profit price (if set)
    pub take_profit: Option<f64>,
    /// Highest price seen since entry (for trailing stop)
    pub high_water_mark: f64,
    /// Lowest price seen since entry (for trailing stop)
    pub low_water_mark: f64,
    /// Commission paid on entry
    pub entry_commission: f64,
    /// Slippage on entry
    pub entry_slippage: f64,
}

impl<X: AxisCoordinate> Position<X> {
    /// Create a new position
    pub fn new(
        side: PositionSide,
        entry_price: f64,
        quantity: f64,
        entry_x: X,
        entry_bar: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            side,
            entry_price,
            quantity,
            entry_x,
            entry_bar,
            stop_loss: None,
            take_profit: None,
            high_water_mark: entry_price,
            low_water_mark: entry_price,
            entry_commission: 0.0,
            entry_slippage: 0.0,
        }
    }

    /// Set stop loss price
    pub fn with_stop_loss(mut self, price: f64) -> Self {
        self.stop_loss = Some(price);
        self
    }

    /// Set take profit price
    pub fn with_take_profit(mut self, price: f64) -> Self {
        self.take_profit = Some(price);
        self
    }

    /// Set entry costs
    pub fn with_entry_costs(mut self, commission: f64, slippage: f64) -> Self {
        self.entry_commission = commission;
        self.entry_slippage = slippage;
        self
    }

    /// Update water marks with current price
    pub fn update_water_marks(&mut self, high: f64, low: f64) {
        if high > self.high_water_mark {
            self.high_water_mark = high;
        }
        if low < self.low_water_mark {
            self.low_water_mark = low;
        }
    }

    /// Calculate unrealized P&L at given price (before exit costs)
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let price_diff = current_price - self.entry_price;
        price_diff * self.quantity * self.side.direction()
    }

    /// Calculate unrealized P&L percentage
    pub fn unrealized_pnl_pct(&self, current_price: f64) -> f64 {
        let price_diff = current_price - self.entry_price;
        (price_diff / self.entry_price) * 100.0 * self.side.direction()
    }

    /// Check if stop loss was hit
    pub fn is_stop_loss_hit(&self, low: f64, high: f64) -> bool {
        if let Some(sl) = self.stop_loss {
            match self.side {
                PositionSide::Long => low <= sl,
                PositionSide::Short => high >= sl,
            }
        } else {
            false
        }
    }

    /// Check if take profit was hit
    pub fn is_take_profit_hit(&self, low: f64, high: f64) -> bool {
        if let Some(tp) = self.take_profit {
            match self.side {
                PositionSide::Long => high >= tp,
                PositionSide::Short => low <= tp,
            }
        } else {
            false
        }
    }

    /// Get the position's notional value at entry
    pub fn notional_value(&self) -> f64 {
        self.entry_price * self.quantity
    }

    /// Calculate the trailing stop price based on current water marks
    pub fn calculate_trailing_stop(&self, trail_pct: f64) -> f64 {
        match self.side {
            PositionSide::Long => self.high_water_mark * (1.0 - trail_pct / 100.0),
            PositionSide::Short => self.low_water_mark * (1.0 + trail_pct / 100.0),
        }
    }

    /// Check if trailing stop was hit
    pub fn is_trailing_stop_hit(&self, low: f64, high: f64, trail_pct: f64) -> bool {
        let trail_price = self.calculate_trailing_stop(trail_pct);
        match self.side {
            PositionSide::Long => low <= trail_price,
            PositionSide::Short => high >= trail_price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::Timestamp;

    #[test]
    fn test_position_side() {
        assert_eq!(PositionSide::Long.opposite(), PositionSide::Short);
        assert_eq!(PositionSide::Short.opposite(), PositionSide::Long);
        assert_eq!(PositionSide::Long.direction(), 1.0);
        assert_eq!(PositionSide::Short.direction(), -1.0);
    }

    #[test]
    fn test_position_pnl_long() {
        let pos = Position::new(
            PositionSide::Long,
            100.0,
            10.0,
            Timestamp::new(1000),
            0,
        );

        assert_eq!(pos.unrealized_pnl(110.0), 100.0);
        assert_eq!(pos.unrealized_pnl(90.0), -100.0);
        assert_eq!(pos.unrealized_pnl_pct(110.0), 10.0);
    }

    #[test]
    fn test_position_pnl_short() {
        let pos = Position::new(
            PositionSide::Short,
            100.0,
            10.0,
            Timestamp::new(1000),
            0,
        );

        assert_eq!(pos.unrealized_pnl(90.0), 100.0);
        assert_eq!(pos.unrealized_pnl(110.0), -100.0);
    }

    #[test]
    fn test_stop_loss_detection() {
        let pos = Position::new(
            PositionSide::Long,
            100.0,
            10.0,
            Timestamp::new(1000),
            0,
        )
        .with_stop_loss(95.0);

        assert!(pos.is_stop_loss_hit(94.0, 102.0));
        assert!(!pos.is_stop_loss_hit(96.0, 102.0));
    }

    #[test]
    fn test_take_profit_detection() {
        let pos = Position::new(
            PositionSide::Long,
            100.0,
            10.0,
            Timestamp::new(1000),
            0,
        )
        .with_take_profit(110.0);

        assert!(pos.is_take_profit_hit(99.0, 112.0));
        assert!(!pos.is_take_profit_hit(99.0, 108.0));
    }

    #[test]
    fn test_trailing_stop() {
        let mut pos = Position::new(
            PositionSide::Long,
            100.0,
            10.0,
            Timestamp::new(1000),
            0,
        );

        // Price goes up
        pos.update_water_marks(120.0, 98.0);
        assert_eq!(pos.high_water_mark, 120.0);

        // Trailing stop at 5% would be at 114
        assert_eq!(pos.calculate_trailing_stop(5.0), 114.0);
        assert!(pos.is_trailing_stop_hit(113.0, 115.0, 5.0));
        assert!(!pos.is_trailing_stop_hit(115.0, 118.0, 5.0));
    }
}
