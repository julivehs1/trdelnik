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

    #[test]
    fn test_fixed_size_name() {
        assert_eq!(FixedSize::new(1.0).name(), "FixedSize");
    }

    #[test]
    fn test_fixed_size_ignores_context() {
        // Same fixed sizer, very different contexts → same output.
        let sizer = FixedSize::new(7.5);
        let small_ctx = SizingContext::new(1.0, 1.0, 1.0, PositionSide::Long, 0.0, 0);
        let big_ctx = SizingContext::new(1e9, 1e9, 1e9, PositionSide::Short, 0.0, 999);
        assert_eq!(sizer.calculate_size(&small_ctx), 7.5);
        assert_eq!(sizer.calculate_size(&big_ctx), 7.5);
    }
}
