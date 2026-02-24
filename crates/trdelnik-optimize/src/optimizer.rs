//! Optimizer builder

use crate::error::{OptimizeError, OptimizeResult};
use crate::methods::{GridSearch, OptimizationMethod};
use crate::param::{ParamRange, ParamSpace};
use crate::result::{OptimizationResult, ParamSet};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::Instant;
use trdelnik_backtest::{BacktestConfig, Backtester, PerformanceMetrics};
use trdelnik_core::{AxisCoordinate, CandleSeries};
use trdelnik_script::{lex, parse, Compiler};

/// Builder for running strategy optimizations
pub struct Optimizer<'a, M: OptimizationMethod = GridSearch> {
    source: &'a str,
    space: ParamSpace,
    method: M,
    config: BacktestConfig,
    target: Arc<dyn Fn(&PerformanceMetrics) -> f64 + Send + Sync>,
}

impl<'a> Optimizer<'a, GridSearch> {
    /// Create a new optimizer for the given TrdelScript source
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            space: ParamSpace::default(),
            method: GridSearch,
            config: BacktestConfig::default(),
            target: Arc::new(|m| m.sharpe_ratio),
        }
    }
}

impl<'a, M: OptimizationMethod> Optimizer<'a, M> {
    /// Add an integer parameter range with step=1
    pub fn param(mut self, name: &str, range: RangeInclusive<i64>) -> Self {
        self.space.add(name.to_string(), ParamRange::int(range));
        self
    }

    /// Add an integer parameter range with custom step
    pub fn param_step(mut self, name: &str, min: i64, max: i64, step: i64) -> Self {
        self.space
            .add(name.to_string(), ParamRange::int_step(min, max, step));
        self
    }

    /// Add a floating-point parameter range
    pub fn param_f64(mut self, name: &str, min: f64, max: f64, step: f64) -> Self {
        self.space
            .add(name.to_string(), ParamRange::float(min, max, step));
        self
    }

    /// Set the optimization method
    pub fn method<N: OptimizationMethod>(self, method: N) -> Optimizer<'a, N> {
        Optimizer {
            source: self.source,
            space: self.space,
            method,
            config: self.config,
            target: self.target,
        }
    }

    /// Set the backtest configuration
    pub fn backtest_config(mut self, config: BacktestConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the optimization target (metric to maximize)
    pub fn target<F>(mut self, f: F) -> Self
    where
        F: Fn(&PerformanceMetrics) -> f64 + Send + Sync + 'static,
    {
        self.target = Arc::new(f);
        self
    }

    /// Get the total number of parameter combinations
    pub fn total_combinations(&self) -> usize {
        self.space.total_combinations()
    }

    /// Run the optimization
    pub fn run<X: AxisCoordinate>(
        self,
        series: &CandleSeries<X>,
    ) -> OptimizeResult<OptimizationResult> {
        if self.space.is_empty() {
            return Err(OptimizeError::EmptyParamSpace);
        }

        let start = Instant::now();

        // Parse once (lex + parse)
        let tokens = lex(self.source).map_err(|e| {
            OptimizeError::Parse(format!("Lexer error: {}", e))
        })?;
        let script = parse(&tokens).map_err(|e| {
            OptimizeError::Parse(format!("Parser error: {:?}", e))
        })?;

        // Clone what we need for the closure
        let target = self.target.clone();

        // Evaluation function
        let evaluate = |params: &HashMap<String, f64>| -> Option<ParamSet> {
            // Compile with these parameters
            let strategy = Compiler::compile_with_params(&script, params.clone()).ok()?;

            // Run backtest
            let backtester = Backtester::new(self.config.clone());
            let result = backtester.run(&strategy, series).ok()?;

            // Calculate score
            let score = target(&result.metrics);

            // Skip invalid scores
            if !score.is_finite() {
                return None;
            }

            Some(ParamSet::new(params.clone(), score, result.metrics))
        };

        // Run the optimization method
        let mut results = self.method.run(&self.space, evaluate);

        if results.is_empty() {
            return Err(OptimizeError::NoValidResults);
        }

        // Sort by score (descending)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
        });

        let best = results[0].clone();
        let total = results.len();

        Ok(OptimizationResult {
            best,
            all: results,
            total_evaluated: total,
            elapsed: start.elapsed(),
            method: self.method.name().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_backtest::PercentOfEquity;
    use trdelnik_core::{Candle, Timestamp};

    fn create_test_series() -> CandleSeries<Timestamp> {
        let mut series = CandleSeries::new();
        // Create a series with a clear trend for testing
        let base_prices: Vec<f64> = (0..100)
            .map(|i| {
                let trend = 100.0 + (i as f64 * 0.5);
                let noise = (i as f64 * 0.7).sin() * 2.0;
                trend + noise
            })
            .collect();

        for (i, &price) in base_prices.iter().enumerate() {
            series.push(Candle::new(
                Timestamp(i as i64 * 3600000),
                price - 0.5,
                price + 1.0,
                price - 1.0,
                price,
                1000.0 + (i as f64 * 10.0),
            ));
        }
        series
    }

    #[test]
    fn test_optimizer_builder() {
        let source = r#"
            param fast_period: int = 5
            param slow_period: int = 10
            let fast = sma(close, fast_period)
            let slow = sma(close, slow_period)
            entry long when crossover(fast, slow)
            entry short when crossunder(fast, slow)
        "#;

        let optimizer = Optimizer::new(source)
            .param("fast_period", 3..=5)
            .param("slow_period", 8..=10);

        assert_eq!(optimizer.total_combinations(), 9); // 3 * 3
    }

    #[test]
    fn test_optimizer_run() {
        let source = r#"
            param fast_period: int = 5
            param slow_period: int = 10
            let fast = sma(close, fast_period)
            let slow = sma(close, slow_period)
            entry long when crossover(fast, slow)
            entry short when crossunder(fast, slow)
        "#;

        let series = create_test_series();
        let config = BacktestConfig::builder()
            .initial_capital(100_000.0)
            .position_sizer(PercentOfEquity::new(10.0))
            .build();

        let result = Optimizer::new(source)
            .param("fast_period", 3..=5)
            .param("slow_period", 10..=12)
            .backtest_config(config)
            .target(|m| m.net_profit_pct)
            .run(&series);

        // Should succeed
        assert!(result.is_ok());
        let result = result.unwrap();

        // Should have evaluated all combinations
        assert!(result.total_evaluated > 0);
        assert_eq!(result.method, "Grid Search");

        // Best should be the first in sorted results
        assert_eq!(result.best.score, result.all[0].score);
    }

    #[test]
    fn test_empty_space_error() {
        let source = "let x = sma(close, 10)";
        let series = create_test_series();

        let result = Optimizer::new(source).run(&series);

        assert!(matches!(result, Err(OptimizeError::EmptyParamSpace)));
    }
}
