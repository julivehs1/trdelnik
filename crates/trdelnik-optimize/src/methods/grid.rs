//! Grid search optimization method

use super::OptimizationMethod;
use crate::param::ParamSpace;
use crate::result::ParamSet;
use std::collections::HashMap;

/// Exhaustive grid search over all parameter combinations
#[derive(Debug, Clone, Copy, Default)]
pub struct GridSearch;

impl GridSearch {
    /// Create a new GridSearch optimizer
    pub fn new() -> Self {
        Self
    }
}

impl OptimizationMethod for GridSearch {
    fn name(&self) -> &'static str {
        "Grid Search"
    }

    fn run<F>(&self, space: &ParamSpace, evaluate: F) -> Vec<ParamSet>
    where
        F: Fn(&HashMap<String, f64>) -> Option<ParamSet> + Sync,
    {
        space.iter().filter_map(|params| evaluate(&params)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::param::ParamRange;
    use trdelnik_backtest::PerformanceMetrics;

    fn mock_metrics() -> PerformanceMetrics {
        PerformanceMetrics {
            initial_capital: 100_000.0,
            final_equity: 110_000.0,
            net_profit: 10_000.0,
            net_profit_pct: 10.0,
            total_return: 10.0,
            annualized_return: 10.0,
            total_trades: 50,
            winning_trades: 30,
            losing_trades: 20,
            breakeven_trades: 0,
            win_rate: 60.0,
            loss_rate: 40.0,
            gross_profit: 15_000.0,
            gross_loss: 5_000.0,
            profit_factor: 3.0,
            average_trade: 200.0,
            average_win: 500.0,
            average_loss: -250.0,
            largest_win: 2000.0,
            largest_loss: -1000.0,
            payoff_ratio: 2.0,
            expectancy: 200.0,
            max_drawdown: 5000.0,
            max_drawdown_pct: 5.0,
            average_drawdown: 2000.0,
            sharpe_ratio: 1.5,
            sortino_ratio: 2.0,
            calmar_ratio: 2.0,
            average_duration: 5.0,
            average_win_duration: 4.0,
            average_loss_duration: 6.0,
            longest_win_duration: 10,
            longest_loss_duration: 15,
            max_consecutive_wins: 5,
            max_consecutive_losses: 3,
            total_commission: 100.0,
            total_slippage: 50.0,
            total_costs: 150.0,
            average_mfe: 3.0,
            average_mae: 1.5,
        }
    }

    #[test]
    fn test_grid_search() {
        let mut space = ParamSpace::new();
        space.add("a".to_string(), ParamRange::int(1..=3));
        space.add("b".to_string(), ParamRange::int(10..=11));

        let grid = GridSearch::new();
        let results = grid.run(&space, |params| {
            let score = params["a"] + params["b"];
            Some(ParamSet::new(params.clone(), score, mock_metrics()))
        });

        assert_eq!(results.len(), 6); // 3 * 2 = 6 combinations
    }

    #[test]
    fn test_grid_search_with_failures() {
        let mut space = ParamSpace::new();
        space.add("x".to_string(), ParamRange::int(1..=5));

        let grid = GridSearch::new();
        let results = grid.run(&space, |params| {
            // Only return results for even values
            if params["x"] as i64 % 2 == 0 {
                Some(ParamSet::new(params.clone(), params["x"], mock_metrics()))
            } else {
                None
            }
        });

        assert_eq!(results.len(), 2); // Only x=2 and x=4 succeed
    }
}
