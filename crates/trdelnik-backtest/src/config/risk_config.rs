//! Risk management configuration

use serde::{Deserialize, Serialize};

/// Configuration for trailing stop behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrailingStopConfig {
    /// Enable trailing stop
    pub enabled: bool,
    /// Trailing percentage (distance from high/low water mark)
    pub trail_pct: f64,
    /// Activation threshold - trailing only starts after this profit %
    pub activation_pct: Option<f64>,
}

impl Default for TrailingStopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trail_pct: 2.0,
            activation_pct: None,
        }
    }
}

impl TrailingStopConfig {
    /// Create a new trailing stop config
    pub fn new(trail_pct: f64) -> Self {
        Self {
            enabled: true,
            trail_pct,
            activation_pct: None,
        }
    }

    /// Set activation threshold
    pub fn with_activation(mut self, activation_pct: f64) -> Self {
        self.activation_pct = Some(activation_pct);
        self
    }
}

/// Risk management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Stop loss percentage (from entry price)
    pub stop_loss_pct: Option<f64>,
    /// Take profit percentage (from entry price)
    pub take_profit_pct: Option<f64>,
    /// Maximum drawdown percentage before stopping
    pub max_drawdown_pct: Option<f64>,
    /// Maximum risk per trade as percentage of equity
    pub max_risk_per_trade_pct: Option<f64>,
    /// Trailing stop configuration
    pub trailing_stop: TrailingStopConfig,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            stop_loss_pct: None,
            take_profit_pct: None,
            max_drawdown_pct: None,
            max_risk_per_trade_pct: None,
            trailing_stop: TrailingStopConfig::default(),
        }
    }
}

impl RiskConfig {
    /// Create a new risk config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set stop loss percentage
    pub fn with_stop_loss(mut self, pct: f64) -> Self {
        self.stop_loss_pct = Some(pct);
        self
    }

    /// Set take profit percentage
    pub fn with_take_profit(mut self, pct: f64) -> Self {
        self.take_profit_pct = Some(pct);
        self
    }

    /// Set maximum drawdown percentage
    pub fn with_max_drawdown(mut self, pct: f64) -> Self {
        self.max_drawdown_pct = Some(pct);
        self
    }

    /// Set maximum risk per trade
    pub fn with_max_risk_per_trade(mut self, pct: f64) -> Self {
        self.max_risk_per_trade_pct = Some(pct);
        self
    }

    /// Set trailing stop
    pub fn with_trailing_stop(mut self, config: TrailingStopConfig) -> Self {
        self.trailing_stop = config;
        self
    }

    /// Calculate stop loss price from entry price (for long position)
    pub fn stop_loss_price_long(&self, entry_price: f64) -> Option<f64> {
        self.stop_loss_pct
            .map(|pct| entry_price * (1.0 - pct / 100.0))
    }

    /// Calculate stop loss price from entry price (for short position)
    pub fn stop_loss_price_short(&self, entry_price: f64) -> Option<f64> {
        self.stop_loss_pct
            .map(|pct| entry_price * (1.0 + pct / 100.0))
    }

    /// Calculate take profit price from entry price (for long position)
    pub fn take_profit_price_long(&self, entry_price: f64) -> Option<f64> {
        self.take_profit_pct
            .map(|pct| entry_price * (1.0 + pct / 100.0))
    }

    /// Calculate take profit price from entry price (for short position)
    pub fn take_profit_price_short(&self, entry_price: f64) -> Option<f64> {
        self.take_profit_pct
            .map(|pct| entry_price * (1.0 - pct / 100.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_config_prices() {
        let config = RiskConfig::new()
            .with_stop_loss(2.0)
            .with_take_profit(6.0);

        assert_eq!(config.stop_loss_price_long(100.0), Some(98.0));
        assert_eq!(config.stop_loss_price_short(100.0), Some(102.0));
        assert_eq!(config.take_profit_price_long(100.0), Some(106.0));
        assert_eq!(config.take_profit_price_short(100.0), Some(94.0));
    }

    #[test]
    fn test_trailing_stop_config() {
        let config = TrailingStopConfig::new(3.0).with_activation(1.0);
        assert!(config.enabled);
        assert_eq!(config.trail_pct, 3.0);
        assert_eq!(config.activation_pct, Some(1.0));
    }
}
