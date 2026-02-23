//! Value types for the graph execution engine
//!
//! This module defines the `Value` enum that flows through computation graphs,
//! and the `OutputToValue` trait for converting indicator outputs to `Value`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// A value that can flow through the computation graph.
///
/// Values can be numbers, booleans, or structs (for multi-output indicators).
/// During warmup periods, values may be `None` to indicate that the indicator
/// doesn't have enough data yet to produce a valid result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// A numeric value (or None during warmup)
    Number(Option<f64>),
    /// A boolean value (or None during warmup)
    Bool(Option<bool>),
    /// A struct value for multi-output indicators (e.g., Bollinger bands)
    /// Uses Arc for cheap cloning, BTreeMap for deterministic field order
    #[serde(skip)]
    Struct(Option<Arc<BTreeMap<String, Value>>>),
}

impl Value {
    /// Create a numeric value
    #[inline]
    pub fn number(n: f64) -> Self {
        Value::Number(Some(n))
    }

    /// Create an empty numeric value (for warmup periods)
    #[inline]
    pub fn none_number() -> Self {
        Value::Number(None)
    }

    /// Create a boolean value
    #[inline]
    pub fn bool(b: bool) -> Self {
        Value::Bool(Some(b))
    }

    /// Create an empty boolean value (for warmup periods)
    #[inline]
    pub fn none_bool() -> Self {
        Value::Bool(None)
    }

    /// Create a struct value from a BTreeMap of fields
    #[inline]
    pub fn structure(fields: BTreeMap<String, Value>) -> Self {
        Value::Struct(Some(Arc::new(fields)))
    }

    /// Create an empty struct value (for warmup periods)
    #[inline]
    pub fn none_struct() -> Self {
        Value::Struct(None)
    }

    /// Check if the value is Some (not in warmup)
    #[inline]
    pub fn is_some(&self) -> bool {
        match self {
            Value::Number(n) => n.is_some(),
            Value::Bool(b) => b.is_some(),
            Value::Struct(s) => s.is_some(),
        }
    }

    /// Check if the value is None (in warmup)
    #[inline]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Try to get the numeric value
    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => *n,
            Value::Bool(_) | Value::Struct(_) => None,
        }
    }

    /// Try to get the boolean value
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => *b,
            Value::Number(_) | Value::Struct(_) => None,
        }
    }

    /// Try to get a field from a struct value
    #[inline]
    pub fn get_field(&self, name: &str) -> Option<Value> {
        match self {
            Value::Struct(Some(fields)) => fields.get(name).cloned(),
            _ => None,
        }
    }

    /// Get the numeric value, panicking if not a number
    #[inline]
    pub fn unwrap_number(&self) -> f64 {
        self.as_number().expect("Expected a numeric value")
    }

    /// Get the boolean value, panicking if not a boolean
    #[inline]
    pub fn unwrap_bool(&self) -> bool {
        self.as_bool().expect("Expected a boolean value")
    }

    /// Convert to Option<f64>, returning None for booleans and structs
    #[inline]
    pub fn to_option_f64(&self) -> Option<f64> {
        self.as_number()
    }
}

/// Trait for converting indicator output types to Value.
///
/// This enables generic node wrappers to convert any indicator output
/// to the appropriate Value variant.
///
/// # Derive macro
///
/// For structs with f64 fields, use `#[derive(IndicatorOutput)]`:
///
/// ```ignore
/// use trdelnik_indicator_derive::IndicatorOutput;
///
/// #[derive(IndicatorOutput)]
/// pub struct MyValue {
///     pub upper: f64,
///     pub lower: f64,
/// }
/// ```
pub trait OutputToValue: Copy {
    /// Convert an optional output value to a Value
    fn to_value(opt: Option<Self>) -> Value;
}

// Implementation for f64 (single-output indicators)
impl OutputToValue for f64 {
    fn to_value(opt: Option<Self>) -> Value {
        Value::Number(opt)
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Number(None)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::number(n)
    }
}

impl From<Option<f64>> for Value {
    fn from(n: Option<f64>) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::bool(b)
    }
}

impl From<Option<bool>> for Value {
    fn from(b: Option<bool>) -> Self {
        Value::Bool(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
