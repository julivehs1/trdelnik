//! Optimization result types

use std::collections::HashMap;
use std::time::Duration;
use trdelnik_backtest::PerformanceMetrics;

/// A single parameter combination with its evaluation results
#[derive(Debug, Clone)]
pub struct ParamSet {
    /// The parameter values
    pub params: HashMap<String, f64>,
    /// The optimization score (higher is better)
    pub score: f64,
    /// Full performance metrics from backtesting
    pub metrics: PerformanceMetrics,
}

impl ParamSet {
    /// Create a new ParamSet
    pub fn new(params: HashMap<String, f64>, score: f64, metrics: PerformanceMetrics) -> Self {
        Self {
            params,
            score,
            metrics,
        }
    }

    /// Format parameters as a string
    pub fn params_string(&self) -> String {
        let mut pairs: Vec<_> = self.params.iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));
        pairs
            .iter()
            .map(|(k, v)| {
                if v.fract() == 0.0 {
                    format!("{}={}", k, **v as i64)
                } else {
                    format!("{}={:.2}", k, v)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Result of an optimization run
#[derive(Debug)]
pub struct OptimizationResult {
    /// Best parameter set found
    pub best: ParamSet,
    /// All evaluated parameter sets, sorted by score (descending)
    pub all: Vec<ParamSet>,
    /// Total number of parameter sets evaluated
    pub total_evaluated: usize,
    /// Time elapsed during optimization
    pub elapsed: Duration,
    /// Name of the optimization method used
    pub method: String,
}

impl OptimizationResult {
    /// Get the top N results
    pub fn top_n(&self, n: usize) -> &[ParamSet] {
        &self.all[..n.min(self.all.len())]
    }

    /// Print a summary of the optimization results
    pub fn print_summary(&self) {
        println!("\n=== Optimization Results ===");
        println!("Method: {}", self.method);
        println!("Evaluated: {} combinations", self.total_evaluated);
        println!("Time: {:.2}s", self.elapsed.as_secs_f64());
        println!();
        println!("=== Best Parameters ===");
        for (name, value) in &self.best.params {
            if value.fract() == 0.0 {
                println!("  {}: {}", name, *value as i64);
            } else {
                println!("  {}: {:.2}", name, value);
            }
        }
        println!();
        println!("=== Metrics ===");
        println!("  Score:         {:.4}", self.best.score);
        println!("  Sharpe Ratio:  {:.2}", self.best.metrics.sharpe_ratio);
        println!("  Profit Factor: {:.2}", self.best.metrics.profit_factor);
        println!("  Win Rate:      {:.1}%", self.best.metrics.win_rate);
        println!("  Max Drawdown:  {:.2}%", self.best.metrics.max_drawdown_pct);
        println!("  Total Trades:  {}", self.best.metrics.total_trades);
    }

    /// Print top N results
    pub fn print_top_n(&self, n: usize) {
        println!("\n=== Top {} Results ===", n);
        for (i, result) in self.top_n(n).iter().enumerate() {
            println!(
                "  {}. {} -> Score: {:.4}",
                i + 1,
                result.params_string(),
                result.score
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_param_set_string() {
        let mut params = HashMap::new();
        params.insert("fast".to_string(), 10.0);
        params.insert("slow".to_string(), 26.0);
        params.insert("ratio".to_string(), 1.5);

        let ps = ParamSet::new(params, 1.5, mock_metrics());
        let s = ps.params_string();

        assert!(s.contains("fast=10"));
        assert!(s.contains("slow=26"));
        assert!(s.contains("ratio=1.50"));
    }

    #[test]
    fn test_optimization_result_top_n() {
        let best = ParamSet::new(HashMap::new(), 2.0, mock_metrics());
        let mut all = vec![
            ParamSet::new(HashMap::new(), 2.0, mock_metrics()),
            ParamSet::new(HashMap::new(), 1.5, mock_metrics()),
            ParamSet::new(HashMap::new(), 1.0, mock_metrics()),
        ];
        all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        let result = OptimizationResult {
            best,
            all,
            total_evaluated: 3,
            elapsed: Duration::from_secs(1),
            method: "Test".to_string(),
        };

        assert_eq!(result.top_n(2).len(), 2);
        assert_eq!(result.top_n(10).len(), 3); // Only 3 available
    }

    fn build_result_with(params: HashMap<String, f64>) -> OptimizationResult {
        let best = ParamSet::new(params.clone(), 1.5, mock_metrics());
        let all = vec![
            best.clone(),
            ParamSet::new(params, 1.0, mock_metrics()),
        ];
        OptimizationResult {
            best,
            all,
            total_evaluated: 2,
            elapsed: Duration::from_secs(2),
            method: "Test".to_string(),
        }
    }

    #[test]
    fn test_print_summary_runs_without_panicking() {
        let mut p = HashMap::new();
        p.insert("fast".to_string(), 10.0);
        p.insert("ratio".to_string(), 1.25);
        let r = build_result_with(p);
        // Smoke: prints to stdout. Must not panic.
        r.print_summary();
    }

    #[test]
    fn test_print_top_n_runs_without_panicking() {
        let mut p = HashMap::new();
        p.insert("k".to_string(), 7.0);
        let r = build_result_with(p);
        r.print_top_n(2);
    }

    #[test]
    fn test_top_n_with_zero_returns_empty_slice() {
        let r = build_result_with(HashMap::new());
        assert!(r.top_n(0).is_empty());
    }

    #[test]
    fn test_param_set_string_only_integers() {
        let mut p = HashMap::new();
        p.insert("a".to_string(), 1.0);
        p.insert("b".to_string(), 2.0);
        let ps = ParamSet::new(p, 1.0, mock_metrics());
        let s = ps.params_string();
        // Both are whole-number floats → formatted as integers (no decimals).
        assert!(s.contains("a=1"));
        assert!(s.contains("b=2"));
        assert!(!s.contains("a=1."), "expected integer formatting, got: {}", s);
    }
}
