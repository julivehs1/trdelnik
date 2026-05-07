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

impl Clone for BacktestConfig {
    fn clone(&self) -> Self {
        Self {
            initial_capital: self.initial_capital,
            position_sizer: self.position_sizer.clone(),
            slippage_model: self.slippage_model.clone(),
            commission_model: self.commission_model.clone(),
            allow_pyramiding: self.allow_pyramiding,
            max_positions: self.max_positions,
            fill_on_close: self.fill_on_close,
            risk_config: self.risk_config.clone(),
        }
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

    use crate::costs::FixedCommission;
    use crate::sizing::FixedSize;

    #[test]
    fn test_new_matches_default() {
        let a = BacktestConfig::new();
        let b = BacktestConfig::default();
        assert_eq!(a.initial_capital, b.initial_capital);
        assert_eq!(a.max_positions, b.max_positions);
        assert_eq!(a.allow_pyramiding, b.allow_pyramiding);
        assert_eq!(a.fill_on_close, b.fill_on_close);
    }

    #[test]
    fn test_builder_default_is_new() {
        let b1 = BacktestConfigBuilder::default().build();
        let b2 = BacktestConfigBuilder::new().build();
        assert_eq!(b1.initial_capital, b2.initial_capital);
    }

    #[test]
    fn test_builder_slippage_and_commission_models() {
        let cfg = BacktestConfig::builder()
            .slippage_model(PercentageSlippage::new(0.5))
            .commission_model(FixedCommission::new(2.5))
            .build();
        // We verify the models stuck by asking them what they'd do.
        let s = cfg
            .slippage_model
            .adjusted_price(100.0, crate::models::PositionSide::Long, true);
        assert!(s > 100.0);
        let c = cfg.commission_model.calculate_commission(100.0, 10.0);
        assert!((c - 2.5).abs() < 1e-9);
    }

    #[test]
    fn test_builder_position_sizer() {
        let cfg = BacktestConfig::builder()
            .position_sizer(FixedSize::new(7.0))
            .build();
        let ctx = crate::sizing::SizingContext::new(
            10_000.0,
            10_000.0,
            100.0,
            crate::models::PositionSide::Long,
            0.0,
            0,
        );
        assert!((cfg.position_sizer.calculate_size(&ctx) - 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_builder_fill_on_close_and_max_positions() {
        let cfg = BacktestConfig::builder()
            .fill_on_close(true)
            .max_positions(5)
            .build();
        assert!(cfg.fill_on_close);
        assert_eq!(cfg.max_positions, 5);
    }

    #[test]
    fn test_builder_risk_config_replaces_whole_struct() {
        let custom = RiskConfig::new()
            .with_stop_loss(3.0)
            .with_take_profit(9.0)
            .with_max_drawdown(15.0);
        let cfg = BacktestConfig::builder().risk_config(custom).build();
        assert_eq!(cfg.risk_config.stop_loss_pct, Some(3.0));
        assert_eq!(cfg.risk_config.take_profit_pct, Some(9.0));
        assert_eq!(cfg.risk_config.max_drawdown_pct, Some(15.0));
    }

    #[test]
    fn test_clone_preserves_fields() {
        let cfg = BacktestConfig::builder()
            .initial_capital(42_000.0)
            .max_positions(7)
            .build();
        let cloned = cfg.clone();
        assert_eq!(cloned.initial_capital, 42_000.0);
        assert_eq!(cloned.max_positions, 7);
    }

    #[test]
    fn test_debug_includes_initial_capital() {
        let cfg = BacktestConfig::default();
        let s = format!("{:?}", cfg);
        assert!(s.contains("BacktestConfig"));
        assert!(s.contains("initial_capital"));
    }

    // ---------- Snapshot ----------

    #[test]
    fn test_snapshot_from_config_preserves_serializable_fields() {
        let cfg = BacktestConfig::builder()
            .initial_capital(50_000.0)
            .allow_pyramiding(true)
            .max_positions(3)
            .fill_on_close(true)
            .stop_loss_pct(2.5)
            .build();
        let snap = BacktestConfigSnapshot::from(&cfg);
        assert_eq!(snap.initial_capital, 50_000.0);
        assert!(snap.allow_pyramiding);
        assert_eq!(snap.max_positions, 3);
        assert!(snap.fill_on_close);
        assert_eq!(snap.risk_config.stop_loss_pct, Some(2.5));
    }

    #[test]
    fn test_snapshot_clone_and_debug() {
        let cfg = BacktestConfig::builder().initial_capital(99_000.0).build();
        let snap = BacktestConfigSnapshot::from(&cfg);
        let cloned = snap.clone();
        assert_eq!(cloned.initial_capital, snap.initial_capital);
        let dbg = format!("{:?}", snap);
        assert!(dbg.contains("BacktestConfigSnapshot"));
    }
}
