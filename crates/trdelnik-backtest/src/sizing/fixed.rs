//! Fixed position size

use super::{PositionSizer, SizingContext};

/// Fixed position size (always trade a fixed quantity)
#[derive(Debug, Clone, Copy)]
pub struct FixedSize {
    /// Fixed quantity to trade
    quantity: f64,
}

impl FixedSize {
    /// Create a new fixed size position sizer
    pub fn new(quantity: f64) -> Self {
        Self { quantity }
    }
}

impl PositionSizer for FixedSize {
    fn calculate_size(&self, _ctx: &SizingContext) -> f64 {
        self.quantity
    }

    fn name(&self) -> &'static str {
        "FixedSize"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PositionSide;

    #[test]
    fn test_fixed_size() {
        let sizer = FixedSize::new(100.0);
        let ctx = SizingContext::new(10000.0, 10000.0, 50.0, PositionSide::Long, 1000.0, 0);

        assert_eq!(sizer.calculate_size(&ctx), 100.0);
    }
}
