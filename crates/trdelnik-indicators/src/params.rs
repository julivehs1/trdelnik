//! Parameter types for dynamic indicator construction.
//!
//! This module provides the types needed for runtime indicator creation
//! from configuration or scripts.

use crate::Indicator;

/// Parameter type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    /// Unsigned integer (usize)
    Usize,
    /// 64-bit float
    F64,
    /// 64-bit signed integer
    I64,
    /// Boolean
    Bool,
}

/// Dynamic parameter value
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Usize(usize),
    F64(f64),
    I64(i64),
    Bool(bool),
}

impl ParamValue {
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            ParamValue::Usize(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ParamValue::F64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ParamValue::I64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ParamValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn param_type(&self) -> ParamType {
        match self {
            ParamValue::Usize(_) => ParamType::Usize,
            ParamValue::F64(_) => ParamType::F64,
            ParamValue::I64(_) => ParamType::I64,
            ParamValue::Bool(_) => ParamType::Bool,
        }
    }
}

/// Parameter definition for an indicator
#[derive(Debug, Clone)]
pub struct ParamDef {
    /// Parameter name
    pub name: &'static str,
    /// Parameter type
    pub param_type: ParamType,
    /// Default value (if any)
    pub default: Option<ParamValue>,
}

impl ParamDef {
    pub const fn usize(name: &'static str) -> Self {
        Self { name, param_type: ParamType::Usize, default: None }
    }

    pub const fn f64(name: &'static str) -> Self {
        Self { name, param_type: ParamType::F64, default: None }
    }

    pub const fn usize_with_default(name: &'static str, default: usize) -> Self {
        Self { name, param_type: ParamType::Usize, default: Some(ParamValue::Usize(default)) }
    }

    pub const fn f64_with_default(name: &'static str, default: f64) -> Self {
        Self { name, param_type: ParamType::F64, default: Some(ParamValue::F64(default)) }
    }
}

/// Trait for indicators with parameter metadata and dynamic construction.
///
/// This trait is automatically implemented by the `#[indicator]` macro.
/// It extends `Indicator` with parameter introspection and runtime construction.
pub trait IndicatorParams: Indicator + Sized {
    /// Get the parameter definitions for this indicator
    fn param_defs() -> &'static [ParamDef];

    /// Construct the indicator from dynamic parameter values
    fn from_params(params: &[ParamValue]) -> Result<Self, String>;
}

/// Type-erased indicator metadata for registry.
///
/// All fields are function pointers or static strings to allow const initialization.
pub struct IndicatorMeta {
    /// Indicator name
    pub name: &'static str,
    /// Function to get parameter definitions
    pub params_fn: fn() -> &'static [ParamDef],
    /// Function to get input type name
    pub input_type_fn: fn() -> &'static str,
    /// Function to get output type name
    pub output_type_fn: fn() -> &'static str,
    /// Factory function
    pub factory: fn(&[ParamValue]) -> Result<Box<dyn std::any::Any + Send + Sync>, String>,
}

impl IndicatorMeta {
    /// Create metadata for an indicator type (const-compatible)
    pub const fn of<T: IndicatorParams>() -> Self {
        Self {
            name: T::NAME,
            params_fn: T::param_defs,
            input_type_fn: || std::any::type_name::<T::Input>(),
            output_type_fn: || std::any::type_name::<T::Output>(),
            factory: |params| {
                T::from_params(params).map(|v| Box::new(v) as Box<dyn std::any::Any + Send + Sync>)
            },
        }
    }

    /// Get parameter definitions
    pub fn params(&self) -> &'static [ParamDef] {
        (self.params_fn)()
    }

    /// Get input type name
    pub fn input_type(&self) -> &'static str {
        (self.input_type_fn)()
    }

    /// Get output type name
    pub fn output_type(&self) -> &'static str {
        (self.output_type_fn)()
    }

    /// Create an indicator instance
    pub fn create(&self, params: &[ParamValue]) -> Result<Box<dyn std::any::Any + Send + Sync>, String> {
        (self.factory)(params)
    }

    /// Create and downcast to specific type
    pub fn create_typed<T: IndicatorParams + 'static>(&self, params: &[ParamValue]) -> Result<T, String> {
        let boxed = self.create(params)?;
        boxed.downcast::<T>()
            .map(|b| *b)
            .map_err(|_| format!("type mismatch: expected {}", std::any::type_name::<T>()))
    }
}

inventory::collect!(IndicatorMeta);

/// Get all registered indicators
pub fn all_indicators() -> impl Iterator<Item = &'static IndicatorMeta> {
    inventory::iter::<IndicatorMeta>.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_value_conversions() {
        let v = ParamValue::Usize(42);
        assert_eq!(v.as_usize(), Some(42));
        assert_eq!(v.as_f64(), None);

        let v = ParamValue::F64(3.14);
        assert_eq!(v.as_f64(), Some(3.14));
        assert_eq!(v.as_usize(), None);
    }

    use crate::Sma;

    #[test]
    fn test_param_value_i64_and_bool_conversions() {
        let v = ParamValue::I64(-7);
        assert_eq!(v.as_i64(), Some(-7));
        assert_eq!(v.as_usize(), None);
        assert_eq!(v.as_f64(), None);
        assert_eq!(v.as_bool(), None);

        let v = ParamValue::Bool(true);
        assert_eq!(v.as_bool(), Some(true));
        assert_eq!(v.as_i64(), None);
    }

    #[test]
    fn test_param_value_param_type_for_each_variant() {
        assert_eq!(ParamValue::Usize(0).param_type(), ParamType::Usize);
        assert_eq!(ParamValue::F64(0.0).param_type(), ParamType::F64);
        assert_eq!(ParamValue::I64(0).param_type(), ParamType::I64);
        assert_eq!(ParamValue::Bool(false).param_type(), ParamType::Bool);
    }

    #[test]
    fn test_param_def_usize_no_default() {
        let d = ParamDef::usize("period");
        assert_eq!(d.name, "period");
        assert_eq!(d.param_type, ParamType::Usize);
        assert!(d.default.is_none());
    }

    #[test]
    fn test_param_def_f64_no_default() {
        let d = ParamDef::f64("mult");
        assert_eq!(d.name, "mult");
        assert_eq!(d.param_type, ParamType::F64);
        assert!(d.default.is_none());
    }

    #[test]
    fn test_param_def_usize_with_default() {
        let d = ParamDef::usize_with_default("period", 14);
        assert_eq!(d.default, Some(ParamValue::Usize(14)));
    }

    #[test]
    fn test_param_def_f64_with_default() {
        let d = ParamDef::f64_with_default("mult", 2.5);
        assert_eq!(d.default, Some(ParamValue::F64(2.5)));
    }

    #[test]
    fn test_param_type_eq_and_distinct() {
        assert_eq!(ParamType::Usize, ParamType::Usize);
        assert_ne!(ParamType::Usize, ParamType::F64);
        assert_ne!(ParamType::I64, ParamType::Bool);
    }

    // ---------- IndicatorMeta ----------

    const SMA_META: IndicatorMeta = IndicatorMeta::of::<Sma>();

    #[test]
    fn test_indicator_meta_name_and_params() {
        assert_eq!(SMA_META.name, "sma");
        let defs = SMA_META.params();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "period");
    }

    #[test]
    fn test_indicator_meta_input_output_type() {
        assert!(SMA_META.input_type().contains("f64"));
        assert!(SMA_META.output_type().contains("f64"));
    }

    #[test]
    fn test_indicator_meta_create_succeeds() {
        let boxed = SMA_META.create(&[ParamValue::Usize(10)]).unwrap();
        // Boxed Any value lands here; we don't downcast in this assertion,
        // we just check the factory accepted the params.
        let _ = boxed;
    }

    #[test]
    fn test_indicator_meta_create_typed_returns_concrete() {
        let sma: Sma = SMA_META.create_typed(&[ParamValue::Usize(20)]).unwrap();
        assert_eq!(sma.period(), 20);
    }

    #[test]
    fn test_indicator_meta_create_propagates_factory_error() {
        // SMA expects exactly one Usize param — a wrong-arity call must error.
        let err = SMA_META.create(&[]);
        assert!(err.is_err());
    }

    // ---------- inventory ----------

    #[test]
    fn test_all_indicators_includes_sma() {
        let names: Vec<&str> = all_indicators().map(|m| m.name).collect();
        assert!(names.contains(&"sma"), "expected sma in registry, got: {:?}", names);
    }

    #[test]
    fn test_all_indicators_yields_unique_names() {
        let names: Vec<&str> = all_indicators().map(|m| m.name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        let mut deduped = sorted.clone();
        deduped.dedup();
        assert_eq!(sorted, deduped, "indicator registry contains duplicates");
    }
}
