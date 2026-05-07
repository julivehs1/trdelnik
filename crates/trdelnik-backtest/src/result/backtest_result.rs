//! Backtest result container

use crate::engine::EquityPoint;
use crate::metrics::PerformanceMetrics;
use crate::models::Trade;
use crate::result::EquityCurve;
use trdelnik_core::AxisCoordinate;

/// Complete backtest result data
#[derive(Debug, Clone)]
pub struct BacktestResultData<X: AxisCoordinate> {
    /// Initial capital
    pub initial_capital: f64,
    /// Final equity
    pub final_equity: f64,
    /// All completed trades
    pub trades: Vec<Trade<X>>,
    /// Equity curve
    pub equity_curve: EquityCurve<X>,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
}

impl<X: AxisCoordinate> BacktestResultData<X> {
    /// Create a new backtest result
    pub fn new(
        initial_capital: f64,
        final_equity: f64,
        trades: Vec<Trade<X>>,
        equity_points: Vec<EquityPoint<X>>,
        total_commission: f64,
        total_slippage: f64,
    ) -> Self {
        let equity_curve = EquityCurve::new(equity_points);
        let metrics = PerformanceMetrics::calculate(
            initial_capital,
            final_equity,
            &trades,
            &equity_curve,
            total_commission,
            total_slippage,
        );

        Self {
            initial_capital,
            final_equity,
            trades,
            equity_curve,
            metrics,
        }
    }

    /// Get net profit/loss
    pub fn net_profit(&self) -> f64 {
        self.final_equity - self.initial_capital
    }

    /// Get net profit/loss percentage
    pub fn net_profit_pct(&self) -> f64 {
        ((self.final_equity - self.initial_capital) / self.initial_capital) * 100.0
    }

    /// Get number of trades
    pub fn num_trades(&self) -> usize {
        self.trades.len()
    }

    /// Get winning trades
    pub fn winning_trades(&self) -> Vec<&Trade<X>> {
        self.trades.iter().filter(|t| t.is_winner()).collect()
    }

    /// Get losing trades
    pub fn losing_trades(&self) -> Vec<&Trade<X>> {
        self.trades.iter().filter(|t| t.is_loser()).collect()
    }

    /// Get trades by exit reason
    pub fn trades_by_exit_reason(
        &self,
        reason: crate::models::ExitReason,
    ) -> Vec<&Trade<X>> {
        self.trades
            .iter()
            .filter(|t| t.exit_reason == reason)
            .collect()
    }

    /// Get trades by side
    pub fn trades_by_side(&self, side: crate::models::PositionSide) -> Vec<&Trade<X>> {
        self.trades.iter().filter(|t| t.side == side).collect()
    }

    /// Print a summary to stdout
    pub fn print_summary(&self) {
        println!("=== Backtest Results ===");
        println!();
        println!("Capital:");
        println!("  Initial:     ${:.2}", self.initial_capital);
        println!("  Final:       ${:.2}", self.final_equity);
        println!("  Net P&L:     ${:.2} ({:.2}%)", self.net_profit(), self.net_profit_pct());
        println!();
        println!("Trades:");
        println!("  Total:       {}", self.metrics.total_trades);
        println!("  Winners:     {} ({:.1}%)", self.metrics.winning_trades, self.metrics.win_rate);
        println!("  Losers:      {} ({:.1}%)", self.metrics.losing_trades, self.metrics.loss_rate);
        println!("  Breakeven:   {}", self.metrics.breakeven_trades);
        println!();
        println!("P&L:");
        println!("  Gross Profit: ${:.2}", self.metrics.gross_profit);
        println!("  Gross Loss:   ${:.2}", self.metrics.gross_loss);
        println!("  Profit Factor: {:.2}", self.metrics.profit_factor);
        println!("  Avg Trade:     ${:.2}", self.metrics.average_trade);
        println!("  Avg Win:       ${:.2}", self.metrics.average_win);
        println!("  Avg Loss:      ${:.2}", self.metrics.average_loss);
        println!("  Largest Win:   ${:.2}", self.metrics.largest_win);
        println!("  Largest Loss:  ${:.2}", self.metrics.largest_loss);
        println!("  Expectancy:    ${:.2}", self.metrics.expectancy);
        println!();
        println!("Risk:");
        println!("  Max Drawdown:    ${:.2} ({:.2}%)", self.metrics.max_drawdown, self.metrics.max_drawdown_pct);
        println!("  Sharpe Ratio:    {:.2}", self.metrics.sharpe_ratio);
        println!("  Sortino Ratio:   {:.2}", self.metrics.sortino_ratio);
        println!("  Calmar Ratio:    {:.2}", self.metrics.calmar_ratio);
        println!();
        println!("Duration:");
        println!("  Avg Trade Duration:  {:.1} bars", self.metrics.average_duration);
        println!("  Avg Win Duration:    {:.1} bars", self.metrics.average_win_duration);
        println!("  Avg Loss Duration:   {:.1} bars", self.metrics.average_loss_duration);
        println!();
        println!("Streaks:");
        println!("  Max Consecutive Wins:   {}", self.metrics.max_consecutive_wins);
        println!("  Max Consecutive Losses: {}", self.metrics.max_consecutive_losses);
        println!();
        println!("Costs:");
        println!("  Total Commission: ${:.2}", self.metrics.total_commission);
        println!("  Total Slippage:   ${:.2}", self.metrics.total_slippage);
        println!("  Total Costs:      ${:.2}", self.metrics.total_costs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EquityPoint;
    use crate::models::{ExitReason, PositionSide, Trade};
    use trdelnik_core::Timestamp;
    use uuid::Uuid;

    #[test]
    fn test_backtest_result() {
        let trades = vec![Trade::new(
            Uuid::new_v4(),
            PositionSide::Long,
            100.0,
            Timestamp::new(1000),
            0,
            1.0,
            0.5,
            110.0,
            Timestamp::new(5000),
            4,
            1.0,
            0.5,
            ExitReason::Signal,
            100.0,
            112.0,
            98.0,
        )];

        let equity_points = vec![
            EquityPoint::new(Timestamp::new(1000), 0, 100_000.0, 90_000.0, 1, 100_000.0),
            EquityPoint::new(Timestamp::new(2000), 1, 102_000.0, 90_000.0, 1, 102_000.0),
            EquityPoint::new(Timestamp::new(3000), 2, 105_000.0, 90_000.0, 1, 105_000.0),
            EquityPoint::new(Timestamp::new(4000), 3, 108_000.0, 90_000.0, 1, 108_000.0),
            EquityPoint::new(Timestamp::new(5000), 4, 110_997.0, 110_997.0, 0, 110_997.0),
        ];

        let result = BacktestResultData::new(
            100_000.0,
            110_997.0,
            trades,
            equity_points,
            2.0,
            1.0,
        );

        assert_eq!(result.num_trades(), 1);
        assert!((result.net_profit() - 10_997.0).abs() < 1.0);
        assert_eq!(result.winning_trades().len(), 1);
        assert_eq!(result.losing_trades().len(), 0);
    }

    fn make_trade(side: PositionSide, exit_reason: ExitReason, exit_price: f64) -> Trade<Timestamp> {
        Trade::new(
            Uuid::new_v4(),
            side,
            100.0,
            Timestamp::new(1000),
            0,
            0.0,
            0.0,
            exit_price,
            Timestamp::new(2000),
            1,
            0.0,
            0.0,
            exit_reason,
            10.0,
            exit_price.max(100.0),
            exit_price.min(100.0),
        )
    }

    fn mixed_result() -> BacktestResultData<Timestamp> {
        let trades = vec![
            make_trade(PositionSide::Long, ExitReason::Signal, 110.0),     // +100 (winner)
            make_trade(PositionSide::Long, ExitReason::StopLoss, 95.0),    // -50  (loser)
            make_trade(PositionSide::Short, ExitReason::TakeProfit, 90.0), // +100 (winner)
            make_trade(PositionSide::Short, ExitReason::Signal, 105.0),    // -50  (loser)
            make_trade(PositionSide::Long, ExitReason::EndOfData, 100.0),  //   0  (breakeven)
        ];
        let equity_points = vec![EquityPoint::new(
            Timestamp::new(1000),
            0,
            100_000.0,
            100_000.0,
            0,
            100_000.0,
        )];
        BacktestResultData::new(100_000.0, 100_100.0, trades, equity_points, 0.0, 0.0)
    }

    #[test]
    fn test_net_profit_negative_when_final_below_initial() {
        let r = BacktestResultData::new(
            100_000.0,
            90_000.0,
            vec![],
            vec![EquityPoint::new(Timestamp::new(0), 0, 100_000.0, 100_000.0, 0, 100_000.0)],
            0.0,
            0.0,
        );
        assert!((r.net_profit() - (-10_000.0)).abs() < 1e-9);
        assert!((r.net_profit_pct() - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn test_winning_and_losing_trades_split_correctly() {
        let r = mixed_result();
        assert_eq!(r.winning_trades().len(), 2);
        assert_eq!(r.losing_trades().len(), 2);
    }

    #[test]
    fn test_trades_by_exit_reason_filters_by_reason() {
        let r = mixed_result();
        assert_eq!(r.trades_by_exit_reason(ExitReason::Signal).len(), 2);
        assert_eq!(r.trades_by_exit_reason(ExitReason::StopLoss).len(), 1);
        assert_eq!(r.trades_by_exit_reason(ExitReason::TakeProfit).len(), 1);
        assert_eq!(r.trades_by_exit_reason(ExitReason::EndOfData).len(), 1);
        // Reasons that don't appear at all yield empty vec
        assert!(r.trades_by_exit_reason(ExitReason::TrailingStop).is_empty());
    }

    #[test]
    fn test_trades_by_side_filters_by_side() {
        let r = mixed_result();
        assert_eq!(r.trades_by_side(PositionSide::Long).len(), 3);
        assert_eq!(r.trades_by_side(PositionSide::Short).len(), 2);
    }

    #[test]
    fn test_print_summary_runs_without_panicking() {
        // Smoke test — print_summary writes to stdout but must not panic.
        let r = mixed_result();
        r.print_summary();
    }
}
