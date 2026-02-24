//! trdelnik-optimize Demo
//!
//! This example demonstrates how to use the optimizer to find
//! optimal parameter values for a trading strategy.

use trdelnik_backtest::{BacktestConfig, PercentOfEquity};
use trdelnik_core::{Candle, CandleSeries, Timestamp};
use trdelnik_optimize::{GridSearch, Optimizer, RandomSearch};

fn main() {
    println!("=== trdelnik-optimize Demo ===\n");

    // Create sample price data with a trend and some noise
    let series = create_sample_series();
    println!("Created sample data: {} candles\n", series.len());

    // Define the strategy with parameters
    let source = r#"
        strategy "SMA Crossover"
        param fast_period: int = 10
        param slow_period: int = 26

        let fast = sma(close, fast_period)
        let slow = sma(close, slow_period)

        entry long when crossover(fast, slow)
        entry short when crossunder(fast, slow)

        stop_loss 2%
        take_profit 6%
    "#;

    // Configure backtest
    let config = BacktestConfig::builder()
        .initial_capital(100_000.0)
        .position_sizer(PercentOfEquity::new(10.0))
        .stop_loss_pct(2.0)
        .take_profit_pct(6.0)
        .build();

    // ========================================
    // Grid Search Optimization
    // ========================================
    println!("--- Grid Search Optimization ---");

    let optimizer = Optimizer::new(source)
        .param("fast_period", 5..=15)
        .param_step("slow_period", 20, 40, 5);

    println!("Parameter space: {} combinations", optimizer.total_combinations());

    let result = optimizer
        .backtest_config(config.clone())
        .target(|m| m.sharpe_ratio)
        .run(&series);

    match result {
        Ok(result) => {
            result.print_summary();
            result.print_top_n(5);
        }
        Err(e) => {
            println!("Optimization failed: {}", e);
        }
    }

    println!("\n");

    // ========================================
    // Random Search Optimization
    // ========================================
    println!("--- Random Search Optimization ---");

    let result = Optimizer::new(source)
        .param("fast_period", 3..=20)
        .param("slow_period", 15..=50)
        .method(RandomSearch::new(50).with_seed(42))
        .backtest_config(config.clone())
        .target(|m| m.profit_factor)
        .run(&series);

    match result {
        Ok(result) => {
            result.print_summary();
            result.print_top_n(5);
        }
        Err(e) => {
            println!("Optimization failed: {}", e);
        }
    }

    println!("\n");

    // ========================================
    // Custom Target Function
    // ========================================
    println!("--- Custom Target: Risk-Adjusted Return ---");

    let result = Optimizer::new(source)
        .param("fast_period", 5..=12)
        .param("slow_period", 20..=30)
        .method(GridSearch)
        .backtest_config(config)
        .target(|m| {
            // Custom metric: return adjusted for drawdown
            if m.max_drawdown_pct > 0.0 {
                m.net_profit_pct / m.max_drawdown_pct
            } else {
                m.net_profit_pct
            }
        })
        .run(&series);

    match result {
        Ok(result) => {
            result.print_summary();
            result.print_top_n(3);
        }
        Err(e) => {
            println!("Optimization failed: {}", e);
        }
    }

    println!("\n=== Demo Complete ===");
}

/// Create sample price data with an uptrend and volatility
fn create_sample_series() -> CandleSeries<Timestamp> {
    let mut series = CandleSeries::new();

    // Generate 500 candles with a general uptrend and some cycles
    for i in 0..500 {
        let t = i as f64;

        // Base trend
        let trend = 100.0 + t * 0.1;

        // Add some cycles (simulating market waves)
        let cycle1 = (t * 0.05).sin() * 5.0;
        let cycle2 = (t * 0.02).cos() * 3.0;

        // Add noise
        let noise = ((t * 1.7).sin() + (t * 2.3).cos()) * 1.5;

        let close = trend + cycle1 + cycle2 + noise;
        let open = close - ((t * 0.3).sin() * 0.5);
        let high = close.max(open) + (t * 0.1).sin().abs() * 2.0;
        let low = close.min(open) - (t * 0.2).cos().abs() * 2.0;
        let volume = 10000.0 + (t * 0.1).sin() * 5000.0;

        series.push(Candle::new(
            Timestamp(i as i64 * 3600000), // Hourly candles
            open,
            high,
            low,
            close,
            volume,
        ));
    }

    series
}
