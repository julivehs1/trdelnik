//! Random search optimization method

use super::OptimizationMethod;
use crate::param::ParamSpace;
use crate::result::ParamSet;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;

/// Random sampling search over parameter combinations
#[derive(Debug, Clone)]
pub struct RandomSearch {
    /// Number of random samples to evaluate
    pub samples: usize,
    /// Optional seed for reproducibility
    pub seed: Option<u64>,
}

impl RandomSearch {
    /// Create a new RandomSearch with the specified number of samples
    pub fn new(samples: usize) -> Self {
        Self {
            samples,
            seed: None,
        }
    }

    /// Set a seed for reproducible results
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
}

impl OptimizationMethod for RandomSearch {
    fn name(&self) -> &'static str {
        "Random Search"
    }

    fn run<F>(&self, space: &ParamSpace, evaluate: F) -> Vec<ParamSet>
    where
        F: Fn(&HashMap<String, f64>) -> Option<ParamSet> + Sync,
    {
        let mut rng: StdRng = match self.seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        space
            .sample(self.samples, &mut rng)
            .into_iter()
            .filter_map(|params| evaluate(&params))
            .collect()
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
    fn test_random_search() {
        let mut space = ParamSpace::new();
        space.add("x".to_string(), ParamRange::int(1..=100));

        let random = RandomSearch::new(10).with_seed(42);
        let results = random.run(&space, |params| {
            Some(ParamSet::new(params.clone(), params["x"], mock_metrics()))
        });

        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_random_search_reproducibility() {
        let mut space = ParamSpace::new();
        space.add("x".to_string(), ParamRange::int(1..=100));

        let random1 = RandomSearch::new(5).with_seed(42);
        let random2 = RandomSearch::new(5).with_seed(42);

        let eval = |params: &HashMap<String, f64>| {
            Some(ParamSet::new(params.clone(), params["x"], mock_metrics()))
        };

        let results1 = random1.run(&space, &eval);
        let results2 = random2.run(&space, &eval);

        // Same seed should produce same results
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.params["x"], r2.params["x"]);
        }
    }

    #[test]
    fn test_random_search_with_failures() {
        let mut space = ParamSpace::new();
        space.add("x".to_string(), ParamRange::int(1..=10));

        let random = RandomSearch::new(20).with_seed(123);
        let results = random.run(&space, |params| {
            // Only succeed for values > 5
            if params["x"] > 5.0 {
                Some(ParamSet::new(params.clone(), params["x"], mock_metrics()))
            } else {
                None
            }
        });

        // Should have fewer than 20 results due to failures
        assert!(results.len() < 20);
        // All results should have x > 5
        for r in &results {
            assert!(r.params["x"] > 5.0);
        }
    }
}
