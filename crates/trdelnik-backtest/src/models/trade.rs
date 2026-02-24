//! Trade types for completed trades

use super::PositionSide;
use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;
use uuid::Uuid;

/// Reason why a trade was exited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExitReason {
    /// Exit signal from the strategy
    Signal,
    /// Stop loss was hit
    StopLoss,
    /// Take profit was hit
    TakeProfit,
    /// Trailing stop was hit
    TrailingStop,
    /// Risk limit was exceeded
    RiskLimit,
    /// End of data (forced close)
    EndOfData,
}

impl std::fmt::Display for ExitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitReason::Signal => write!(f, "Signal"),
            ExitReason::StopLoss => write!(f, "Stop Loss"),
            ExitReason::TakeProfit => write!(f, "Take Profit"),
            ExitReason::TrailingStop => write!(f, "Trailing Stop"),
            ExitReason::RiskLimit => write!(f, "Risk Limit"),
            ExitReason::EndOfData => write!(f, "End of Data"),
        }
    }
}

/// A completed trade
#[derive(Debug, Clone)]
pub struct Trade<X: AxisCoordinate> {
    /// Unique identifier
    pub id: Uuid,
    /// Trade side (long or short)
    pub side: PositionSide,

    // Entry details
    /// Entry price
    pub entry_price: f64,
    /// X coordinate at entry
    pub entry_x: X,
    /// Bar index at entry
    pub entry_bar: usize,
    /// Commission paid on entry
    pub entry_commission: f64,
    /// Slippage on entry
    pub entry_slippage: f64,

    // Exit details
    /// Exit price
    pub exit_price: f64,
    /// X coordinate at exit
    pub exit_x: X,
    /// Bar index at exit
    pub exit_bar: usize,
    /// Commission paid on exit
    pub exit_commission: f64,
    /// Slippage on exit
    pub exit_slippage: f64,
    /// Reason for exit
    pub exit_reason: ExitReason,

    // Trade metrics
    /// Position size (quantity)
    pub quantity: f64,
    /// Gross P&L (before costs)
    pub gross_pnl: f64,
    /// Net P&L (after costs)
    pub net_pnl: f64,
    /// Maximum Favorable Excursion (best unrealized P&L)
    pub mfe: f64,
    /// Maximum Adverse Excursion (worst unrealized P&L)
    pub mae: f64,
}

impl<X: AxisCoordinate> Trade<X> {
    /// Create a new trade from entry/exit details
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        side: PositionSide,
        entry_price: f64,
        entry_x: X,
        entry_bar: usize,
        entry_commission: f64,
        entry_slippage: f64,
        exit_price: f64,
        exit_x: X,
        exit_bar: usize,
        exit_commission: f64,
        exit_slippage: f64,
        exit_reason: ExitReason,
        quantity: f64,
        high_water_mark: f64,
        low_water_mark: f64,
    ) -> Self {
        // Calculate gross P&L
        let price_diff = exit_price - entry_price;
        let gross_pnl = price_diff * quantity * side.direction();

        // Calculate net P&L
        let total_commission = entry_commission + exit_commission;
        let total_slippage = entry_slippage + exit_slippage;
        let net_pnl = gross_pnl - total_commission - total_slippage;

        // Calculate MFE/MAE
        let (mfe, mae) = match side {
            PositionSide::Long => {
                let mfe_price_diff = high_water_mark - entry_price;
                let mae_price_diff = low_water_mark - entry_price;
                (mfe_price_diff * quantity, mae_price_diff * quantity)
            }
            PositionSide::Short => {
                let mfe_price_diff = entry_price - low_water_mark;
                let mae_price_diff = entry_price - high_water_mark;
                (mfe_price_diff * quantity, mae_price_diff * quantity)
            }
        };

        Self {
            id,
            side,
            entry_price,
            entry_x,
            entry_bar,
            entry_commission,
            entry_slippage,
            exit_price,
            exit_x,
            exit_bar,
            exit_commission,
            exit_slippage,
            exit_reason,
            quantity,
            gross_pnl,
            net_pnl,
            mfe,
            mae,
        }
    }

    /// Total commission paid (entry + exit)
    pub fn total_commission(&self) -> f64 {
        self.entry_commission + self.exit_commission
    }

    /// Total slippage (entry + exit)
    pub fn total_slippage(&self) -> f64 {
        self.entry_slippage + self.exit_slippage
    }

    /// Return as a percentage of entry value
    pub fn return_pct(&self) -> f64 {
        (self.net_pnl / (self.entry_price * self.quantity)) * 100.0
    }

    /// Gross return as a percentage of entry value
    pub fn gross_return_pct(&self) -> f64 {
        (self.gross_pnl / (self.entry_price * self.quantity)) * 100.0
    }

    /// Duration in bars
    pub fn duration_bars(&self) -> usize {
        self.exit_bar - self.entry_bar
    }

    /// Whether this trade was profitable
    pub fn is_winner(&self) -> bool {
        self.net_pnl > 0.0
    }

    /// Whether this trade was a loss
    pub fn is_loser(&self) -> bool {
        self.net_pnl < 0.0
    }

    /// Notional value at entry
    pub fn notional_value(&self) -> f64 {
        self.entry_price * self.quantity
    }

    /// Risk/Reward ratio (MFE / MAE absolute values)
    pub fn risk_reward_ratio(&self) -> Option<f64> {
        if self.mae.abs() > f64::EPSILON {
            Some(self.mfe / self.mae.abs())
        } else {
            None
        }
    }

    /// How much of the MFE was captured (net_pnl / mfe)
    pub fn mfe_capture_ratio(&self) -> Option<f64> {
        if self.mfe > f64::EPSILON {
            Some(self.net_pnl / self.mfe)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::Timestamp;

    #[test]
    fn test_trade_long_winner() {
        let trade = Trade::new(
            Uuid::new_v4(),
            PositionSide::Long,
            100.0,                  // entry price
            Timestamp::new(1000),   // entry x
            0,                      // entry bar
            1.0,                    // entry commission
            0.5,                    // entry slippage
            110.0,                  // exit price
            Timestamp::new(5000),   // exit x
            4,                      // exit bar
            1.0,                    // exit commission
            0.5,                    // exit slippage
            ExitReason::Signal,     // exit reason
            10.0,                   // quantity
            115.0,                  // high water mark
            98.0,                   // low water mark
        );

        assert_eq!(trade.gross_pnl, 100.0);
        assert_eq!(trade.net_pnl, 97.0); // 100 - 2 commission - 1 slippage
        assert!(trade.is_winner());
        assert!(!trade.is_loser());
        assert_eq!(trade.duration_bars(), 4);
        assert_eq!(trade.mfe, 150.0); // (115 - 100) * 10
        assert_eq!(trade.mae, -20.0); // (98 - 100) * 10
    }

    #[test]
    fn test_trade_short_winner() {
        let trade = Trade::new(
            Uuid::new_v4(),
            PositionSide::Short,
            100.0,                  // entry price
            Timestamp::new(1000),
            0,
            1.0,
            0.5,
            90.0,                   // exit price (lower = profit for short)
            Timestamp::new(5000),
            4,
            1.0,
            0.5,
            ExitReason::Signal,
            10.0,
            105.0,                  // high water mark
            85.0,                   // low water mark
        );

        assert_eq!(trade.gross_pnl, 100.0);
        assert!(trade.is_winner());
        assert_eq!(trade.mfe, 150.0); // (100 - 85) * 10
        assert_eq!(trade.mae, -50.0); // (100 - 105) * 10
    }

    #[test]
    fn test_trade_long_loser() {
        let trade = Trade::new(
            Uuid::new_v4(),
            PositionSide::Long,
            100.0,
            Timestamp::new(1000),
            0,
            1.0,
            0.5,
            95.0,                   // exit price (lower = loss for long)
            Timestamp::new(3000),
            2,
            1.0,
            0.5,
            ExitReason::StopLoss,
            10.0,
            102.0,
            94.0,
        );

        assert_eq!(trade.gross_pnl, -50.0);
        assert!(trade.is_loser());
    }

    #[test]
    fn test_exit_reason_display() {
        assert_eq!(format!("{}", ExitReason::StopLoss), "Stop Loss");
        assert_eq!(format!("{}", ExitReason::TakeProfit), "Take Profit");
        assert_eq!(format!("{}", ExitReason::TrailingStop), "Trailing Stop");
    }
}
