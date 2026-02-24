//! Slippage models

use crate::models::PositionSide;

/// Trait for slippage calculation
pub trait SlippageModel: Send + Sync {
    /// Calculate slippage amount
    ///
    /// # Arguments
    /// * `price` - Order price
    /// * `quantity` - Order quantity
    /// * `side` - Position side
    /// * `is_entry` - True if this is an entry order
    ///
    /// # Returns
    /// The total slippage cost (always positive)
    fn calculate_slippage(
        &self,
        price: f64,
        quantity: f64,
        side: PositionSide,
        is_entry: bool,
    ) -> f64;

    /// Calculate the adjusted fill price after slippage
    fn adjusted_price(&self, price: f64, side: PositionSide, is_entry: bool) -> f64;

    /// Name of the model (for logging/display)
    fn name(&self) -> &'static str;
}

/// No slippage
#[derive(Debug, Clone, Copy, Default)]
pub struct ZeroSlippage;

impl SlippageModel for ZeroSlippage {
    fn calculate_slippage(
        &self,
        _price: f64,
        _quantity: f64,
        _side: PositionSide,
        _is_entry: bool,
    ) -> f64 {
        0.0
    }

    fn adjusted_price(&self, price: f64, _side: PositionSide, _is_entry: bool) -> f64 {
        price
    }

    fn name(&self) -> &'static str {
        "ZeroSlippage"
    }
}

/// Percentage-based slippage
#[derive(Debug, Clone, Copy)]
pub struct PercentageSlippage {
    /// Slippage percentage (e.g., 0.1 = 0.1%)
    percent: f64,
}

impl PercentageSlippage {
    /// Create a new percentage slippage model
    ///
    /// # Arguments
    /// * `percent` - Slippage percentage (e.g., 0.1 for 0.1%)
    pub fn new(percent: f64) -> Self {
        Self { percent }
    }
}

impl SlippageModel for PercentageSlippage {
    fn calculate_slippage(
        &self,
        price: f64,
        quantity: f64,
        _side: PositionSide,
        _is_entry: bool,
    ) -> f64 {
        price * quantity * (self.percent / 100.0)
    }

    fn adjusted_price(&self, price: f64, side: PositionSide, is_entry: bool) -> f64 {
        let adjustment = price * (self.percent / 100.0);

        // Entry buys get worse prices (higher for long, lower for short)
        // Exit sells get worse prices (lower for long, higher for short)
        match (side, is_entry) {
            (PositionSide::Long, true) => price + adjustment,   // Buying: pay more
            (PositionSide::Long, false) => price - adjustment,  // Selling: receive less
            (PositionSide::Short, true) => price - adjustment,  // Selling short: receive less
            (PositionSide::Short, false) => price + adjustment, // Buying to cover: pay more
        }
    }

    fn name(&self) -> &'static str {
        "PercentageSlippage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_slippage() {
        let model = ZeroSlippage;
        assert_eq!(
            model.calculate_slippage(100.0, 10.0, PositionSide::Long, true),
            0.0
        );
        assert_eq!(
            model.adjusted_price(100.0, PositionSide::Long, true),
            100.0
        );
    }

    #[test]
    fn test_percentage_slippage() {
        let model = PercentageSlippage::new(0.1); // 0.1% slippage

        // Slippage cost
        let slippage = model.calculate_slippage(100.0, 10.0, PositionSide::Long, true);
        assert!((slippage - 1.0).abs() < 0.001); // 100 * 10 * 0.001 = 1.0

        // Long entry: price goes up
        assert!((model.adjusted_price(100.0, PositionSide::Long, true) - 100.1).abs() < 0.001);

        // Long exit: price goes down
        assert!((model.adjusted_price(100.0, PositionSide::Long, false) - 99.9).abs() < 0.001);

        // Short entry: price goes down
        assert!((model.adjusted_price(100.0, PositionSide::Short, true) - 99.9).abs() < 0.001);

        // Short exit: price goes up
        assert!((model.adjusted_price(100.0, PositionSide::Short, false) - 100.1).abs() < 0.001);
    }
}
