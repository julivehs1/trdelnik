//! Main backtest configuration

use super::RiskConfig;
use crate::costs::{CommissionModel, SlippageModel, ZeroCommission, ZeroSlippage};
use crate::sizing::{FixedSize, PositionSizer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Main configuration for running a backtest
pub struct BacktestConfig {
    /// Initial capital
    pub initial_capital: f64,
    /// Position sizing strategy
    pub position_sizer: Arc<dyn PositionSizer>,
    /// Slippage model
    pub slippage_model: Arc<dyn SlippageModel>,
    /// Commission model
    pub commission_model: Arc<dyn CommissionModel>,
    /// Allow pyramiding (multiple positions in same direction)
    pub allow_pyramiding: bool,
    /// Maximum number of concurrent positions
    pub max_positions: usize,
    /// Fill orders on close instead of next bar's open
    pub fill_on_close: bool,
    /// Risk management configuration
    pub risk_config: RiskConfig,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            position_sizer: Arc::new(FixedSize::new(1.0)),
            slippage_model: Arc::new(ZeroSlippage),
            commission_model: Arc::new(ZeroCommission),
            allow_pyramiding: false,
            max_positions: 1,
            fill_on_close: false,
            risk_config: RiskConfig::default(),
        }
    }
}

impl BacktestConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration builder
    pub fn builder() -> BacktestConfigBuilder {
        BacktestConfigBuilder::new()
    }
}

impl std::fmt::Debug for BacktestConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BacktestConfig")
            .field("initial_capital", &self.initial_capital)
            .field("allow_pyramiding", &self.allow_pyramiding)
            .field("max_positions", &self.max_positions)
            .field("fill_on_close", &self.fill_on_close)
            .field("risk_config", &self.risk_config)
            .finish()
    }
}

/// Builder for BacktestConfig
pub struct BacktestConfigBuilder {
    config: BacktestConfig,
}

impl BacktestConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            config: BacktestConfig::default(),
        }
    }

    /// Set initial capital
    pub fn initial_capital(mut self, capital: f64) -> Self {
        self.config.initial_capital = capital;
        self
    }

    /// Set position sizer
    pub fn position_sizer<P: PositionSizer + 'static>(mut self, sizer: P) -> Self {
        self.config.position_sizer = Arc::new(sizer);
        self
    }

    /// Set slippage model
    pub fn slippage_model<S: SlippageModel + 'static>(mut self, model: S) -> Self {
        self.config.slippage_model = Arc::new(model);
        self
    }

    /// Set commission model
    pub fn commission_model<C: CommissionModel + 'static>(mut self, model: C) -> Self {
        self.config.commission_model = Arc::new(model);
        self
    }

    /// Allow pyramiding (multiple positions in same direction)
    pub fn allow_pyramiding(mut self, allow: bool) -> Self {
        self.config.allow_pyramiding = allow;
        self
    }

    /// Set maximum concurrent positions
    pub fn max_positions(mut self, max: usize) -> Self {
        self.config.max_positions = max;
        self
    }

    /// Fill orders on close (instead of next bar's open)
    pub fn fill_on_close(mut self, fill_on_close: bool) -> Self {
        self.config.fill_on_close = fill_on_close;
        self
    }

    /// Set risk configuration
    pub fn risk_config(mut self, config: RiskConfig) -> Self {
        self.config.risk_config = config;
        self
    }

    /// Set stop loss percentage
    pub fn stop_loss_pct(mut self, pct: f64) -> Self {
        self.config.risk_config.stop_loss_pct = Some(pct);
        self
    }

    /// Set take profit percentage
    pub fn take_profit_pct(mut self, pct: f64) -> Self {
        self.config.risk_config.take_profit_pct = Some(pct);
        self
    }

    /// Build the configuration
    pub fn build(self) -> BacktestConfig {
        self.config
    }
}

impl Default for BacktestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable version of BacktestConfig for saving/loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfigSnapshot {
    pub initial_capital: f64,
    pub allow_pyramiding: bool,
    pub max_positions: usize,
    pub fill_on_close: bool,
    pub risk_config: RiskConfig,
}

impl From<&BacktestConfig> for BacktestConfigSnapshot {
    fn from(config: &BacktestConfig) -> Self {
        Self {
            initial_capital: config.initial_capital,
            allow_pyramiding: config.allow_pyramiding,
            max_positions: config.max_positions,
            fill_on_close: config.fill_on_close,
            risk_config: config.risk_config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::costs::{PercentageCommission, PercentageSlippage};
    use crate::sizing::PercentOfEquity;

    #[test]
    fn test_builder() {
        let config = BacktestConfig::builder()
            .initial_capital(50_000.0)
            .position_sizer(PercentOfEquity::new(10.0))
            .slippage_model(PercentageSlippage::new(0.1))
            .commission_model(PercentageCommission::new(0.05))
            .allow_pyramiding(true)
            .max_positions(3)
            .stop_loss_pct(2.0)
            .take_profit_pct(6.0)
            .build();

        assert_eq!(config.initial_capital, 50_000.0);
        assert!(config.allow_pyramiding);
        assert_eq!(config.max_positions, 3);
        assert_eq!(config.risk_config.stop_loss_pct, Some(2.0));
        assert_eq!(config.risk_config.take_profit_pct, Some(6.0));
    }

    #[test]
    fn test_default() {
        let config = BacktestConfig::default();
        assert_eq!(config.initial_capital, 100_000.0);
        assert!(!config.allow_pyramiding);
        assert_eq!(config.max_positions, 1);
    }
}
