//! Backtesting engine for trdelnik trading library
//!
//! This crate provides a complete backtesting framework that integrates with
//! `trdelnik-script` (for strategy compilation) and `trdelnik-graph` (for execution).
//!
//! # Example
//!
//! ```ignore
//! use trdelnik_backtest::{Backtester, BacktestConfig, PercentOfEquity};
//! use trdelnik_script::compile;
//!
//! let strategy = compile(r#"
//!     strategy "SMA Cross"
//!     let fast = sma(close, 12)
//!     let slow = sma(close, 26)
//!     entry long when crossover(fast, slow)
//!     entry short when crossunder(fast, slow)
//!     stop_loss 2%
//!     take_profit 6%
//! "#)?;
//!
//! let config = BacktestConfig::builder()
//!     .initial_capital(100_000.0)
//!     .position_sizer(PercentOfEquity::new(10.0))
//!     .build();
//!
//! let result = Backtester::new(config).run(&strategy, &series)?;
//!
//! println!("Net P&L: ${:.2}", result.metrics.net_profit);
//! println!("Sharpe: {:.2}", result.metrics.sharpe_ratio);
//! println!("Max DD: {:.2}%", result.metrics.max_drawdown_pct);
//! ```
//!
//! # Features
//!
//! - Multiple position sizing strategies (fixed, percent of equity, risk-based)
//! - Configurable slippage and commission models
//! - Stop loss, take profit, and trailing stop support
//! - Comprehensive performance metrics (30+ indicators)
//! - Equity curve analysis and drawdown tracking
//! - MFE/MAE tracking for trade optimization
//! - Support for generic X-axis coordinates (timestamps, slots, blocks, indices)

pub mod config;
pub mod costs;
pub mod engine;
pub mod error;
pub mod metrics;
pub mod models;
pub mod result;
pub mod sizing;

// Re-export main types for convenient access
pub use config::{
    BacktestConfig, BacktestConfigBuilder, BacktestConfigSnapshot, RiskConfig, TrailingStopConfig,
};
pub use costs::{
    CommissionModel, FixedCommission, PerShareCommission, PercentageCommission, PercentageSlippage,
    SlippageModel, ZeroCommission, ZeroSlippage,
};
pub use engine::{BacktestState, Backtester, EquityPoint};
pub use error::{BacktestError, BacktestResult};
pub use metrics::PerformanceMetrics;
pub use models::{ExitReason, Position, PositionSide, Trade};
pub use result::{BacktestResultData, DrawdownInfo, EquityCurve};
pub use sizing::{FixedSize, PercentOfEquity, PositionSizer, RiskBased, SizingContext};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_backtest_flow() {
        // This test validates the basic flow works without running a full strategy
        let config = BacktestConfig::builder()
            .initial_capital(100_000.0)
            .position_sizer(PercentOfEquity::new(10.0))
            .stop_loss_pct(2.0)
            .take_profit_pct(6.0)
            .build();

        assert_eq!(config.initial_capital, 100_000.0);
        assert!(config.risk_config.stop_loss_pct.is_some());
        assert!(config.risk_config.take_profit_pct.is_some());
    }

    #[test]
    fn test_position_side_properties() {
        assert_eq!(PositionSide::Long.direction(), 1.0);
        assert_eq!(PositionSide::Short.direction(), -1.0);
        assert_eq!(PositionSide::Long.opposite(), PositionSide::Short);
        assert_eq!(PositionSide::Short.opposite(), PositionSide::Long);
    }

    #[test]
    fn test_risk_config() {
        let config = RiskConfig::new()
            .with_stop_loss(2.0)
            .with_take_profit(6.0)
            .with_max_drawdown(10.0);

        assert_eq!(config.stop_loss_pct, Some(2.0));
        assert_eq!(config.take_profit_pct, Some(6.0));
        assert_eq!(config.max_drawdown_pct, Some(10.0));

        assert_eq!(config.stop_loss_price_long(100.0), Some(98.0));
        assert_eq!(config.take_profit_price_long(100.0), Some(106.0));
    }

    #[test]
    fn test_position_sizers() {
        let ctx = SizingContext::new(100_000.0, 100_000.0, 100.0, PositionSide::Long, 0.0, 0);

        // Fixed size
        let fixed = FixedSize::new(50.0);
        assert_eq!(fixed.calculate_size(&ctx), 50.0);

        // Percent of equity (10% of 100k = 10k, 10k / 100 = 100 shares)
        let pct = PercentOfEquity::new(10.0);
        assert_eq!(pct.calculate_size(&ctx), 100.0);

        // Risk-based
        let risk = RiskBased::with_risk(1.0);
        let ctx_with_stop =
            SizingContext::new(100_000.0, 100_000.0, 100.0, PositionSide::Long, 0.0, 0)
                .with_stop_loss(98.0);
        // Risk amount = 1% of 100k = 1k
        // Risk per share = $2
        // Size = 1k / 2 = 500
        assert_eq!(risk.calculate_size(&ctx_with_stop), 500.0);
    }

    #[test]
    fn test_slippage_models() {
        // Zero slippage
        let zero = ZeroSlippage;
        assert_eq!(
            zero.adjusted_price(100.0, PositionSide::Long, true),
            100.0
        );

        // Percentage slippage
        let pct = PercentageSlippage::new(0.1);
        let adjusted = pct.adjusted_price(100.0, PositionSide::Long, true);
        assert!((adjusted - 100.1).abs() < 0.01);
    }

    #[test]
    fn test_commission_models() {
        // Zero commission
        let zero = ZeroCommission;
        assert_eq!(zero.calculate_commission(100.0, 50.0), 0.0);

        // Fixed commission
        let fixed = FixedCommission::new(10.0);
        assert_eq!(fixed.calculate_commission(100.0, 50.0), 10.0);

        // Percentage commission
        let pct = PercentageCommission::new(0.1);
        let comm = pct.calculate_commission(100.0, 50.0);
        assert!((comm - 5.0).abs() < 0.01);
    }
}
