//! # trdelnik-optimize
//!
//! Strategy optimization for the trdelnik trading library.
//!
//! This crate provides tools for finding optimal parameter values for trading strategies
//! written in TrdelScript. It supports multiple optimization methods including grid search
//! and random search.
//!
//! ## Example
//!
//! ```ignore
//! use trdelnik_optimize::{Optimizer, GridSearch, RandomSearch};
//! use trdelnik_backtest::BacktestConfig;
//!
//! let source = r#"
//!     strategy "SMA Cross"
//!     param fast_period: int = 10
//!     param slow_period: int = 26
//!     let fast = sma(close, fast_period)
//!     let slow = sma(close, slow_period)
//!     entry long when crossover(fast, slow)
//!     entry short when crossunder(fast, slow)
//!     stop_loss 2%
//! "#;
//!
//! let result = Optimizer::new(source)
//!     .param("fast_period", 5..=50)              // Int range
//!     .param_f64("stop_loss", 1.0, 5.0, 0.5)     // Float with step
//!     .method(GridSearch)                         // or RandomSearch::new(1000)
//!     .backtest_config(config)
//!     .target(|m| m.sharpe_ratio)                // Optimization target
//!     .run(&series)?;
//!
//! // Results
//! println!("Best: {:?}", result.best.params);
//! println!("Sharpe: {:.2}", result.best.metrics.sharpe_ratio);
//!
//! for r in result.top_n(10) {
//!     println!("{} -> {:.2}", r.params_string(), r.score);
//! }
//! ```
//!
//! ## Optimization Methods
//!
//! - **GridSearch**: Exhaustively evaluates all parameter combinations. Best for small
//!   parameter spaces where you want to guarantee finding the global optimum.
//!
//! - **RandomSearch**: Randomly samples from the parameter space. More efficient for
//!   large spaces and can often find good solutions with fewer evaluations.
//!
//! ## Parameter Types
//!
//! - Integer ranges: `.param("name", min..=max)` or `.param_step("name", min, max, step)`
//! - Float ranges: `.param_f64("name", min, max, step)`

pub mod error;
pub mod methods;
pub mod optimizer;
pub mod param;
pub mod result;

// Re-export main types
pub use error::{OptimizeError, OptimizeResult};
pub use methods::{GridSearch, OptimizationMethod, RandomSearch};
pub use optimizer::Optimizer;
pub use param::{ParamRange, ParamSpace};
pub use result::{OptimizationResult, ParamSet};
