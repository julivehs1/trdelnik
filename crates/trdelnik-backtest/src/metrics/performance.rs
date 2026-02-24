//! Performance metrics calculation

use crate::models::Trade;
use crate::result::EquityCurve;
use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;

/// Comprehensive performance metrics for a backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // Capital metrics
    /// Initial capital
    pub initial_capital: f64,
    /// Final equity
    pub final_equity: f64,
    /// Net profit/loss
    pub net_profit: f64,
    /// Net profit/loss percentage
    pub net_profit_pct: f64,
    /// Total return (same as net_profit_pct)
    pub total_return: f64,
    /// Annualized return (assuming 252 trading days)
    pub annualized_return: f64,

    // Trade counts
    /// Total number of trades
    pub total_trades: usize,
    /// Number of winning trades
    pub winning_trades: usize,
    /// Number of losing trades
    pub losing_trades: usize,
    /// Number of breakeven trades
    pub breakeven_trades: usize,

    // Win rate
    /// Win rate percentage
    pub win_rate: f64,
    /// Loss rate percentage
    pub loss_rate: f64,

    // P&L metrics
    /// Gross profit
    pub gross_profit: f64,
    /// Gross loss
    pub gross_loss: f64,
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: f64,
    /// Average trade P&L
    pub average_trade: f64,
    /// Average winning trade
    pub average_win: f64,
    /// Average losing trade
    pub average_loss: f64,
    /// Largest winning trade
    pub largest_win: f64,
    /// Largest losing trade
    pub largest_loss: f64,
    /// Payoff ratio (average win / average loss)
    pub payoff_ratio: f64,
    /// Expectancy per trade
    pub expectancy: f64,

    // Drawdown metrics
    /// Maximum drawdown amount
    pub max_drawdown: f64,
    /// Maximum drawdown percentage
    pub max_drawdown_pct: f64,
    /// Average drawdown
    pub average_drawdown: f64,

    // Risk-adjusted metrics
    /// Sharpe ratio (assuming risk-free rate of 0)
    pub sharpe_ratio: f64,
    /// Sortino ratio (using downside deviation)
    pub sortino_ratio: f64,
    /// Calmar ratio (annualized return / max drawdown)
    pub calmar_ratio: f64,

    // Duration metrics
    /// Average trade duration in bars
    pub average_duration: f64,
    /// Average winning trade duration
    pub average_win_duration: f64,
    /// Average losing trade duration
    pub average_loss_duration: f64,
    /// Longest winning trade duration
    pub longest_win_duration: usize,
    /// Longest losing trade duration
    pub longest_loss_duration: usize,

    // Streak metrics
    /// Maximum consecutive wins
    pub max_consecutive_wins: usize,
    /// Maximum consecutive losses
    pub max_consecutive_losses: usize,

    // Cost metrics
    /// Total commission paid
    pub total_commission: f64,
    /// Total slippage paid
    pub total_slippage: f64,
    /// Total costs (commission + slippage)
    pub total_costs: f64,

    // MFE/MAE metrics
    /// Average MFE (Maximum Favorable Excursion)
    pub average_mfe: f64,
    /// Average MAE (Maximum Adverse Excursion)
    pub average_mae: f64,
}

impl PerformanceMetrics {
    /// Calculate metrics from trades and equity curve
    pub fn calculate<X: AxisCoordinate>(
        initial_capital: f64,
        final_equity: f64,
        trades: &[Trade<X>],
        equity_curve: &EquityCurve<X>,
        total_commission: f64,
        total_slippage: f64,
    ) -> Self {
        let total_trades = trades.len();

        // Separate winners and losers
        let winners: Vec<_> = trades.iter().filter(|t| t.net_pnl > 0.0).collect();
        let losers: Vec<_> = trades.iter().filter(|t| t.net_pnl < 0.0).collect();
        let breakeven: Vec<_> = trades
            .iter()
            .filter(|t| t.net_pnl.abs() < f64::EPSILON)
            .collect();

        let winning_trades = winners.len();
        let losing_trades = losers.len();
        let breakeven_trades = breakeven.len();

        // Win/loss rates
        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };
        let loss_rate = if total_trades > 0 {
            (losing_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        // Gross profit/loss
        let gross_profit: f64 = winners.iter().map(|t| t.net_pnl).sum();
        let gross_loss: f64 = losers.iter().map(|t| t.net_pnl.abs()).sum();

        // Profit factor
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        // Average metrics
        let average_trade = if total_trades > 0 {
            trades.iter().map(|t| t.net_pnl).sum::<f64>() / total_trades as f64
        } else {
            0.0
        };

        let average_win = if winning_trades > 0 {
            gross_profit / winning_trades as f64
        } else {
            0.0
        };

        let average_loss = if losing_trades > 0 {
            -gross_loss / losing_trades as f64
        } else {
            0.0
        };

        // Largest win/loss
        let largest_win = winners
            .iter()
            .map(|t| t.net_pnl)
            .fold(0.0, f64::max);
        let largest_loss = losers
            .iter()
            .map(|t| t.net_pnl)
            .fold(0.0, f64::min);

        // Payoff ratio
        let payoff_ratio = if average_loss.abs() > f64::EPSILON {
            average_win / average_loss.abs()
        } else {
            0.0
        };

        // Expectancy
        let expectancy = (win_rate / 100.0 * average_win) + ((1.0 - win_rate / 100.0) * average_loss);

        // Capital metrics
        let net_profit = final_equity - initial_capital;
        let net_profit_pct = (net_profit / initial_capital) * 100.0;
        let total_return = net_profit_pct;

        // Duration metrics
        let durations: Vec<_> = trades.iter().map(|t| t.duration_bars()).collect();
        let average_duration = if !durations.is_empty() {
            durations.iter().sum::<usize>() as f64 / durations.len() as f64
        } else {
            0.0
        };

        let win_durations: Vec<_> = winners.iter().map(|t| t.duration_bars()).collect();
        let average_win_duration = if !win_durations.is_empty() {
            win_durations.iter().sum::<usize>() as f64 / win_durations.len() as f64
        } else {
            0.0
        };

        let loss_durations: Vec<_> = losers.iter().map(|t| t.duration_bars()).collect();
        let average_loss_duration = if !loss_durations.is_empty() {
            loss_durations.iter().sum::<usize>() as f64 / loss_durations.len() as f64
        } else {
            0.0
        };

        let longest_win_duration = win_durations.iter().copied().max().unwrap_or(0);
        let longest_loss_duration = loss_durations.iter().copied().max().unwrap_or(0);

        // Consecutive streaks
        let (max_consecutive_wins, max_consecutive_losses) = calculate_streaks(trades);

        // Drawdown metrics
        let dd_info = equity_curve.max_drawdown_info();
        let max_drawdown = dd_info.as_ref().map(|d| d.max_drawdown).unwrap_or(0.0);
        let max_drawdown_pct = dd_info.as_ref().map(|d| d.max_drawdown_pct).unwrap_or(0.0);
        let average_drawdown = equity_curve.average_drawdown();

        // Risk-adjusted metrics
        let returns = equity_curve.period_returns();
        let annualized_return = calculate_annualized_return(&returns, 252.0);
        let sharpe_ratio = calculate_sharpe_ratio(&returns, 0.0, 252.0);
        let sortino_ratio = calculate_sortino_ratio(&returns, 0.0, 252.0);
        let calmar_ratio = if max_drawdown_pct > 0.0 {
            annualized_return / max_drawdown_pct
        } else {
            0.0
        };

        // MFE/MAE
        let average_mfe = if total_trades > 0 {
            trades.iter().map(|t| t.mfe).sum::<f64>() / total_trades as f64
        } else {
            0.0
        };

        let average_mae = if total_trades > 0 {
            trades.iter().map(|t| t.mae).sum::<f64>() / total_trades as f64
        } else {
            0.0
        };

        // Total costs
        let total_costs = total_commission + total_slippage;

        Self {
            initial_capital,
            final_equity,
            net_profit,
            net_profit_pct,
            total_return,
            annualized_return,
            total_trades,
            winning_trades,
            losing_trades,
            breakeven_trades,
            win_rate,
            loss_rate,
            gross_profit,
            gross_loss,
            profit_factor,
            average_trade,
            average_win,
            average_loss,
            largest_win,
            largest_loss,
            payoff_ratio,
            expectancy,
            max_drawdown,
            max_drawdown_pct,
            average_drawdown,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            average_duration,
            average_win_duration,
            average_loss_duration,
            longest_win_duration,
            longest_loss_duration,
            max_consecutive_wins,
            max_consecutive_losses,
            total_commission,
            total_slippage,
            total_costs,
            average_mfe,
            average_mae,
        }
    }
}

/// Calculate consecutive win/loss streaks
fn calculate_streaks<X: AxisCoordinate>(trades: &[Trade<X>]) -> (usize, usize) {
    let mut max_wins = 0;
    let mut max_losses = 0;
    let mut current_wins = 0;
    let mut current_losses = 0;

    for trade in trades {
        if trade.is_winner() {
            current_wins += 1;
            current_losses = 0;
            max_wins = max_wins.max(current_wins);
        } else if trade.is_loser() {
            current_losses += 1;
            current_wins = 0;
            max_losses = max_losses.max(current_losses);
        } else {
            // Breakeven - reset both
            current_wins = 0;
            current_losses = 0;
        }
    }

    (max_wins, max_losses)
}

/// Calculate annualized return from period returns
fn calculate_annualized_return(returns: &[f64], periods_per_year: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let total_return: f64 = returns.iter().map(|r| 1.0 + r).product::<f64>() - 1.0;
    let n_periods = returns.len() as f64;
    let years = n_periods / periods_per_year;

    if years > 0.0 {
        ((1.0 + total_return).powf(1.0 / years) - 1.0) * 100.0
    } else {
        0.0
    }
}

/// Calculate Sharpe ratio
fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64, periods_per_year: f64) -> f64 {
    if returns.len() < 2 {
        return 0.0;
    }

    let excess_returns: Vec<f64> = returns
        .iter()
        .map(|r| r - (risk_free_rate / periods_per_year))
        .collect();

    let mean = excess_returns.iter().sum::<f64>() / excess_returns.len() as f64;
    let variance = excess_returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / (excess_returns.len() - 1) as f64;
    let std_dev = variance.sqrt();

    if std_dev > 0.0 {
        (mean / std_dev) * periods_per_year.sqrt()
    } else {
        0.0
    }
}

/// Calculate Sortino ratio (using downside deviation)
fn calculate_sortino_ratio(returns: &[f64], target_return: f64, periods_per_year: f64) -> f64 {
    if returns.len() < 2 {
        return 0.0;
    }

    let target_per_period = target_return / periods_per_year;
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;

    // Downside deviation - only negative deviations from target
    let downside_returns: Vec<f64> = returns
        .iter()
        .filter(|&&r| r < target_per_period)
        .map(|&r| (r - target_per_period).powi(2))
        .collect();

    if downside_returns.is_empty() {
        return f64::INFINITY;
    }

    let downside_variance = downside_returns.iter().sum::<f64>() / downside_returns.len() as f64;
    let downside_dev = downside_variance.sqrt();

    if downside_dev > 0.0 {
        ((mean - target_per_period) / downside_dev) * periods_per_year.sqrt()
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaks() {
        use crate::models::PositionSide;
        use trdelnik_core::Timestamp;
        use uuid::Uuid;

        let trades = vec![
            // 3 winners
            Trade::new(
                Uuid::new_v4(),
                PositionSide::Long,
                100.0,
                Timestamp::new(1000),
                0,
                0.0,
                0.0,
                110.0,
                Timestamp::new(2000),
                1,
                0.0,
                0.0,
                crate::models::ExitReason::Signal,
                10.0,
                110.0,
                100.0,
            ),
            Trade::new(
                Uuid::new_v4(),
                PositionSide::Long,
                100.0,
                Timestamp::new(3000),
                2,
                0.0,
                0.0,
                105.0,
                Timestamp::new(4000),
                3,
                0.0,
                0.0,
                crate::models::ExitReason::Signal,
                10.0,
                105.0,
                100.0,
            ),
            Trade::new(
                Uuid::new_v4(),
                PositionSide::Long,
                100.0,
                Timestamp::new(5000),
                4,
                0.0,
                0.0,
                108.0,
                Timestamp::new(6000),
                5,
                0.0,
                0.0,
                crate::models::ExitReason::Signal,
                10.0,
                108.0,
                100.0,
            ),
            // 2 losers
            Trade::new(
                Uuid::new_v4(),
                PositionSide::Long,
                100.0,
                Timestamp::new(7000),
                6,
                0.0,
                0.0,
                95.0,
                Timestamp::new(8000),
                7,
                0.0,
                0.0,
                crate::models::ExitReason::StopLoss,
                10.0,
                100.0,
                95.0,
            ),
            Trade::new(
                Uuid::new_v4(),
                PositionSide::Long,
                100.0,
                Timestamp::new(9000),
                8,
                0.0,
                0.0,
                92.0,
                Timestamp::new(10000),
                9,
                0.0,
                0.0,
                crate::models::ExitReason::StopLoss,
                10.0,
                100.0,
                92.0,
            ),
        ];

        let (max_wins, max_losses) = calculate_streaks(&trades);
        assert_eq!(max_wins, 3);
        assert_eq!(max_losses, 2);
    }

    #[test]
    fn test_sharpe_ratio() {
        // Test with varying positive returns
        let returns = vec![0.01, 0.02, 0.01, 0.015, 0.01];
        let sharpe = calculate_sharpe_ratio(&returns, 0.0, 252.0);
        // Should be positive since average return is positive
        assert!(sharpe > 0.0);

        // Test with negative returns
        let neg_returns = vec![-0.01, -0.02, -0.01, -0.015, -0.01];
        let neg_sharpe = calculate_sharpe_ratio(&neg_returns, 0.0, 252.0);
        // Should be negative
        assert!(neg_sharpe < 0.0);

        // Test with too few samples
        let short_returns = vec![0.01];
        let short_sharpe = calculate_sharpe_ratio(&short_returns, 0.0, 252.0);
        assert_eq!(short_sharpe, 0.0);
    }
}
