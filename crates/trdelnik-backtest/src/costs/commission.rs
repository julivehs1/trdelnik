//! Commission models

/// Trait for commission calculation
pub trait CommissionModel: Send + Sync {
    /// Calculate commission for a trade
    ///
    /// # Arguments
    /// * `price` - Fill price
    /// * `quantity` - Trade quantity
    ///
    /// # Returns
    /// The commission amount (always positive)
    fn calculate_commission(&self, price: f64, quantity: f64) -> f64;

    /// Name of the model (for logging/display)
    fn name(&self) -> &'static str;
}

/// No commission
#[derive(Debug, Clone, Copy, Default)]
pub struct ZeroCommission;

impl CommissionModel for ZeroCommission {
    fn calculate_commission(&self, _price: f64, _quantity: f64) -> f64 {
        0.0
    }

    fn name(&self) -> &'static str {
        "ZeroCommission"
    }
}

/// Fixed commission per trade
#[derive(Debug, Clone, Copy)]
pub struct FixedCommission {
    /// Fixed amount per trade
    amount: f64,
}

impl FixedCommission {
    /// Create a new fixed commission model
    ///
    /// # Arguments
    /// * `amount` - Fixed commission per trade
    pub fn new(amount: f64) -> Self {
        Self { amount }
    }
}

impl CommissionModel for FixedCommission {
    fn calculate_commission(&self, _price: f64, _quantity: f64) -> f64 {
        self.amount
    }

    fn name(&self) -> &'static str {
        "FixedCommission"
    }
}

/// Percentage-based commission
#[derive(Debug, Clone, Copy)]
pub struct PercentageCommission {
    /// Commission percentage (e.g., 0.05 = 0.05%)
    percent: f64,
}

impl PercentageCommission {
    /// Create a new percentage commission model
    ///
    /// # Arguments
    /// * `percent` - Commission percentage (e.g., 0.05 for 0.05%)
    pub fn new(percent: f64) -> Self {
        Self { percent }
    }
}

impl CommissionModel for PercentageCommission {
    fn calculate_commission(&self, price: f64, quantity: f64) -> f64 {
        price * quantity * (self.percent / 100.0)
    }

    fn name(&self) -> &'static str {
        "PercentageCommission"
    }
}

/// Per-share commission
#[derive(Debug, Clone, Copy)]
pub struct PerShareCommission {
    /// Commission per share
    per_share: f64,
    /// Minimum commission per trade
    minimum: f64,
}

impl PerShareCommission {
    /// Create a new per-share commission model
    ///
    /// # Arguments
    /// * `per_share` - Commission per share
    /// * `minimum` - Minimum commission per trade
    pub fn new(per_share: f64, minimum: f64) -> Self {
        Self { per_share, minimum }
    }
}

impl CommissionModel for PerShareCommission {
    fn calculate_commission(&self, _price: f64, quantity: f64) -> f64 {
        (quantity * self.per_share).max(self.minimum)
    }

    fn name(&self) -> &'static str {
        "PerShareCommission"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_commission() {
        let model = ZeroCommission;
        assert_eq!(model.calculate_commission(100.0, 50.0), 0.0);
    }

    #[test]
    fn test_fixed_commission() {
        let model = FixedCommission::new(10.0);
        assert_eq!(model.calculate_commission(100.0, 50.0), 10.0);
        assert_eq!(model.calculate_commission(50.0, 10.0), 10.0);
    }

    #[test]
    fn test_percentage_commission() {
        let model = PercentageCommission::new(0.1); // 0.1% commission

        // 100 * 50 * 0.001 = 5.0
        assert!((model.calculate_commission(100.0, 50.0) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_per_share_commission() {
        let model = PerShareCommission::new(0.01, 1.0);

        // 100 shares * 0.01 = 1.0 (equals minimum)
        assert_eq!(model.calculate_commission(100.0, 100.0), 1.0);

        // 10 shares * 0.01 = 0.10 < 1.0 minimum -> 1.0
        assert_eq!(model.calculate_commission(100.0, 10.0), 1.0);

        // 200 shares * 0.01 = 2.0 > minimum
        assert_eq!(model.calculate_commission(100.0, 200.0), 2.0);
    }
}
