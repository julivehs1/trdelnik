//! Value types for the graph execution engine
//!
//! Re-exports from `trdelnik_core`. The `OutputToValue` implementations
//! for indicator value types are now generated via `#[derive(IndicatorOutput)]`.

pub use trdelnik_core::{OutputToValue, Value};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_value_number() {
        let v = Value::number(42.0);
        assert!(v.is_some());
        assert_eq!(v.as_number(), Some(42.0));
        assert_eq!(v.unwrap_number(), 42.0);
    }

    #[test]
    fn test_value_none() {
        let v = Value::none_number();
        assert!(v.is_none());
        assert_eq!(v.as_number(), None);
    }

    #[test]
    fn test_value_bool() {
        let v = Value::bool(true);
        assert!(v.is_some());
        assert_eq!(v.as_bool(), Some(true));
        assert_eq!(v.unwrap_bool(), true);
    }

    #[test]
    fn test_value_from() {
        let v: Value = 42.0.into();
        assert_eq!(v.as_number(), Some(42.0));

        let v: Value = true.into();
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_value_struct() {
        let mut fields = BTreeMap::new();
        fields.insert("upper".to_string(), Value::number(110.0));
        fields.insert("middle".to_string(), Value::number(100.0));
        fields.insert("lower".to_string(), Value::number(90.0));

        let v = Value::structure(fields);
        assert!(v.is_some());

        // Test field extraction
        let upper = v.get_field("upper");
        assert!(upper.is_some());
        assert_eq!(upper.unwrap().as_number(), Some(110.0));

        let middle = v.get_field("middle");
        assert_eq!(middle.unwrap().as_number(), Some(100.0));

        let lower = v.get_field("lower");
        assert_eq!(lower.unwrap().as_number(), Some(90.0));

        // Non-existent field
        assert!(v.get_field("nonexistent").is_none());
    }

    #[test]
    fn test_value_none_struct() {
        let v = Value::none_struct();
        assert!(v.is_none());
        assert!(v.get_field("any").is_none());
    }

    #[test]
    fn test_output_to_value_f64() {
        let v = f64::to_value(Some(42.0));
        assert_eq!(v.as_number(), Some(42.0));

        let v = f64::to_value(None);
        assert!(v.is_none());
    }

    #[test]
    fn test_output_to_value_bollinger() {
        use trdelnik_indicators::BollingerValue;

        let bb = BollingerValue {
            upper: 110.0,
            middle: 100.0,
            lower: 90.0,
        };
        let v = BollingerValue::to_value(Some(bb));

        assert!(v.is_some());
        assert_eq!(v.get_field("upper").unwrap().as_number(), Some(110.0));
        assert_eq!(v.get_field("middle").unwrap().as_number(), Some(100.0));
        assert_eq!(v.get_field("lower").unwrap().as_number(), Some(90.0));

        let v = BollingerValue::to_value(None);
        assert!(v.is_none());
    }

    #[test]
    fn test_output_to_value_macd() {
        use trdelnik_indicators::MacdValue;

        let macd = MacdValue {
            macd: 1.5,
            signal: 1.0,
            histogram: 0.5,
        };
        let v = MacdValue::to_value(Some(macd));

        assert!(v.is_some());
        assert_eq!(v.get_field("macd").unwrap().as_number(), Some(1.5));
        assert_eq!(v.get_field("signal").unwrap().as_number(), Some(1.0));
        assert_eq!(v.get_field("histogram").unwrap().as_number(), Some(0.5));
    }

    #[test]
    fn test_output_to_value_stochastic() {
        use trdelnik_indicators::StochasticValue;

        let stoch = StochasticValue { k: 80.0, d: 75.0 };
        let v = StochasticValue::to_value(Some(stoch));

        assert!(v.is_some());
        assert_eq!(v.get_field("k").unwrap().as_number(), Some(80.0));
        assert_eq!(v.get_field("d").unwrap().as_number(), Some(75.0));
    }
}
