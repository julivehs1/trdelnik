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
}
