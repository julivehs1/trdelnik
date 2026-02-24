//! Backtest state tracking

use crate::models::{Position, PositionSide, Trade};
use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;

/// A point on the equity curve
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EquityPoint<X: AxisCoordinate> {
    /// X coordinate
    pub x: X,
    /// Bar index
    pub bar_index: usize,
    /// Equity value
    pub equity: f64,
    /// Cash balance
    pub cash: f64,
    /// Number of open positions
    pub open_positions: usize,
    /// Drawdown from peak
    pub drawdown: f64,
    /// Drawdown percentage
    pub drawdown_pct: f64,
}

impl<X: AxisCoordinate> EquityPoint<X> {
    /// Create a new equity point
    pub fn new(
        x: X,
        bar_index: usize,
        equity: f64,
        cash: f64,
        open_positions: usize,
        peak_equity: f64,
    ) -> Self {
        let drawdown = peak_equity - equity;
        let drawdown_pct = if peak_equity > 0.0 {
            (drawdown / peak_equity) * 100.0
        } else {
            0.0
        };

        Self {
            x,
            bar_index,
            equity,
            cash,
            open_positions,
            drawdown,
            drawdown_pct,
        }
    }
}

/// Internal state maintained during backtesting
pub struct BacktestState<X: AxisCoordinate> {
    /// Current equity (cash + position values)
    pub equity: f64,
    /// Available cash
    pub cash: f64,
    /// Open positions
    pub positions: Vec<Position<X>>,
    /// Completed trades
    pub trades: Vec<Trade<X>>,
    /// Equity curve
    pub equity_curve: Vec<EquityPoint<X>>,
    /// Peak equity (for drawdown calculation)
    pub peak_equity: f64,
    /// Maximum drawdown seen
    pub max_drawdown: f64,
    /// Maximum drawdown percentage
    pub max_drawdown_pct: f64,
    /// Total commission paid
    pub total_commission: f64,
    /// Total slippage paid
    pub total_slippage: f64,
    /// Current bar index
    pub current_bar: usize,
}

impl<X: AxisCoordinate> BacktestState<X> {
    /// Create a new backtest state with initial capital
    pub fn new(initial_capital: f64) -> Self {
        Self {
            equity: initial_capital,
            cash: initial_capital,
            positions: Vec::new(),
            trades: Vec::new(),
            equity_curve: Vec::new(),
            peak_equity: initial_capital,
            max_drawdown: 0.0,
            max_drawdown_pct: 0.0,
            total_commission: 0.0,
            total_slippage: 0.0,
            current_bar: 0,
        }
    }

    /// Update equity based on current positions and price
    pub fn update_equity(&mut self, current_price: f64) {
        let position_value: f64 = self
            .positions
            .iter()
            .map(|p| p.unrealized_pnl(current_price) + p.notional_value())
            .sum();

        self.equity = self.cash + position_value;

        // Update peak and drawdown
        if self.equity > self.peak_equity {
            self.peak_equity = self.equity;
        }

        let drawdown = self.peak_equity - self.equity;
        let drawdown_pct = if self.peak_equity > 0.0 {
            (drawdown / self.peak_equity) * 100.0
        } else {
            0.0
        };

        if drawdown > self.max_drawdown {
            self.max_drawdown = drawdown;
        }
        if drawdown_pct > self.max_drawdown_pct {
            self.max_drawdown_pct = drawdown_pct;
        }
    }

    /// Record current state to equity curve
    pub fn record_equity_point(&mut self, x: X) {
        let point = EquityPoint::new(
            x,
            self.current_bar,
            self.equity,
            self.cash,
            self.positions.len(),
            self.peak_equity,
        );
        self.equity_curve.push(point);
    }

    /// Count positions by side
    pub fn count_positions(&self, side: PositionSide) -> usize {
        self.positions.iter().filter(|p| p.side == side).count()
    }

    /// Check if there are any open positions
    pub fn has_positions(&self) -> bool {
        !self.positions.is_empty()
    }

    /// Check if there are positions of a specific side
    pub fn has_position_side(&self, side: PositionSide) -> bool {
        self.positions.iter().any(|p| p.side == side)
    }

    /// Get total position value for a side
    pub fn position_value(&self, side: PositionSide, current_price: f64) -> f64 {
        self.positions
            .iter()
            .filter(|p| p.side == side)
            .map(|p| p.unrealized_pnl(current_price) + p.notional_value())
            .sum()
    }

    /// Net profit/loss
    pub fn net_pnl(&self, initial_capital: f64) -> f64 {
        self.equity - initial_capital
    }

    /// Net profit/loss percentage
    pub fn net_pnl_pct(&self, initial_capital: f64) -> f64 {
        ((self.equity - initial_capital) / initial_capital) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::Timestamp;

    #[test]
    fn test_state_creation() {
        let state: BacktestState<Timestamp> = BacktestState::new(100_000.0);
        assert_eq!(state.equity, 100_000.0);
        assert_eq!(state.cash, 100_000.0);
        assert!(state.positions.is_empty());
        assert!(state.trades.is_empty());
    }

    #[test]
    fn test_equity_point() {
        let point = EquityPoint::new(Timestamp::new(1000), 0, 95_000.0, 95_000.0, 0, 100_000.0);

        assert_eq!(point.drawdown, 5_000.0);
        assert!((point.drawdown_pct - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_state_with_position() {
        let mut state: BacktestState<Timestamp> = BacktestState::new(100_000.0);

        // Add a position
        state.positions.push(Position::new(
            PositionSide::Long,
            100.0,
            100.0,
            Timestamp::new(1000),
            0,
        ));
        state.cash -= 10_000.0; // Cost of position

        // Update equity at higher price
        state.update_equity(110.0);
        // Position value = 100 shares * $110 = $11,000
        // Cash = $90,000
        // Equity = $90,000 + $11,000 = $101,000
        assert_eq!(state.equity, 101_000.0);

        assert!(state.has_positions());
        assert!(state.has_position_side(PositionSide::Long));
        assert!(!state.has_position_side(PositionSide::Short));
    }
}
