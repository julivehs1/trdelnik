//! Indicator registry for dynamic indicator creation.
//!
//! The registry collects all indicators registered via the `#[indicator]` macro
//! at compile time and provides runtime lookup by name.
//!
//! # Example
//!
//! ```rust,ignore
//! use trdelnik_graph::registry::IndicatorRegistry;
//!
//! let registry = IndicatorRegistry::new();
//!
//! // List all available indicators
//! for info in registry.iter() {
//!     println!("{}: {:?}", info.name, info.params());
//! }
//!
//! // Create an indicator dynamically
//! let params = vec![ParamValue::Usize(20)];
//! if let Some(node) = registry.create_node("sma", Some(close_id), &params) {
//!     let node_id = graph.add_node(node);
//! }
//! ```

use std::collections::HashMap;
use trdelnik_indicators::{all_indicators, IndicatorMeta, ParamDef, ParamValue};

use crate::node::{BoxedNode, NodeId};

/// Registry of all available indicators.
///
/// Built at runtime from compile-time registrations via `inventory`.
pub struct IndicatorRegistry {
    by_name: HashMap<&'static str, &'static IndicatorMeta>,
}

impl IndicatorRegistry {
    /// Create a new registry with all registered indicators.
    pub fn new() -> Self {
        let by_name: HashMap<_, _> = all_indicators()
            .map(|meta| (meta.name, meta))
            .collect();

        Self { by_name }
    }

    /// Get indicator metadata by name.
    pub fn get(&self, name: &str) -> Option<&'static IndicatorMeta> {
        self.by_name.get(name).copied()
    }

    /// Check if an indicator exists.
    pub fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// Get the number of registered indicators.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// Iterate over all registered indicators.
    pub fn iter(&self) -> impl Iterator<Item = &'static IndicatorMeta> + '_ {
        self.by_name.values().copied()
    }

    /// Get all indicator names.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.by_name.keys().copied()
    }

    /// Get parameter definitions for an indicator.
    pub fn params(&self, name: &str) -> Option<&'static [ParamDef]> {
        self.get(name).map(|meta| meta.params())
    }

    /// Create a node for the given indicator.
    ///
    /// - `name`: Indicator name (e.g., "sma", "ema", "atr")
    /// - `input`: Input node ID (required for f64-input indicators like SMA)
    /// - `params`: Parameter values matching the indicator's param_defs
    ///
    /// Returns `None` if the indicator doesn't exist or params are invalid.
    pub fn create_node(
        &self,
        name: &str,
        input: Option<NodeId>,
        params: &[ParamValue],
    ) -> Option<BoxedNode> {
        let meta = self.get(name)?;

        // Create the indicator instance
        let indicator = meta.create(params).ok()?;

        // Create the appropriate node based on input type
        create_node_for_indicator(name, input, indicator, meta.input_type())
    }
}

impl Default for IndicatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a node wrapper for the indicator.
///
/// This function handles the type-erasure: we know the input type from metadata
/// but need to create the appropriate `IndicatorNode` variant.
fn create_node_for_indicator(
    name: &str,
    input: Option<NodeId>,
    indicator: Box<dyn std::any::Any + Send + Sync>,
    input_type: &str,
) -> Option<BoxedNode> {
    use crate::nodes::generic::IndicatorNode;
    use trdelnik_indicators::*;

    // Match on indicator name and create the appropriate node
    // This is necessary because we need to downcast to the concrete type
    match name {
        "sma" => {
            let ind = indicator.downcast::<Sma>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "ema" => {
            let ind = indicator.downcast::<Ema>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "wma" => {
            let ind = indicator.downcast::<Wma>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "rsi" => {
            let ind = indicator.downcast::<Rsi>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "std_dev" => {
            let ind = indicator.downcast::<StdDev>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "roc" => {
            let ind = indicator.downcast::<Roc>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "efficiency_ratio" => {
            let ind = indicator.downcast::<EfficiencyRatio>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "bollinger" => {
            let ind = indicator.downcast::<Bollinger>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "macd" => {
            let ind = indicator.downcast::<Macd>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        "ppo" => {
            let ind = indicator.downcast::<Ppo>().ok()?;
            let input = input?;
            Some(Box::new(IndicatorNode::with_input(input, *ind)))
        }
        // OHLC-input indicators (no node input needed)
        "atr" => {
            let ind = indicator.downcast::<Atr>().ok()?;
            Some(Box::new(IndicatorNode::from_context(*ind)))
        }
        "stochastic" => {
            let ind = indicator.downcast::<Stochastic>().ok()?;
            Some(Box::new(IndicatorNode::from_context(*ind)))
        }
        "cci" => {
            let ind = indicator.downcast::<Cci>().ok()?;
            Some(Box::new(IndicatorNode::from_context(*ind)))
        }
        "keltner" => {
            let ind = indicator.downcast::<Keltner>().ok()?;
            Some(Box::new(IndicatorNode::from_context(*ind)))
        }
        "chandelier" => {
            let ind = indicator.downcast::<Chandelier>().ok()?;
            Some(Box::new(IndicatorNode::from_context(*ind)))
        }
        // OHLCV-input indicators
        "obv" => {
            let ind = indicator.downcast::<Obv>().ok()?;
            Some(Box::new(IndicatorNode::from_context(*ind)))
        }
        "mfi" => {
            let ind = indicator.downcast::<Mfi>().ok()?;
            Some(Box::new(IndicatorNode::from_context(*ind)))
        }
        _ => {
            // Unknown indicator - check input type for fallback
            eprintln!("Unknown indicator: {} (input_type: {})", name, input_type);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_contains_indicators() {
        let registry = IndicatorRegistry::new();

        // Should have some indicators registered
        assert!(registry.len() > 0);

        // Check for some known indicators
        assert!(registry.contains("sma"));
        assert!(registry.contains("ema"));
        assert!(registry.contains("rsi"));
        assert!(registry.contains("atr"));
    }

    #[test]
    fn test_registry_get_params() {
        let registry = IndicatorRegistry::new();

        let sma_params = registry.params("sma").unwrap();
        assert_eq!(sma_params.len(), 1);
        assert_eq!(sma_params[0].name, "period");

        let bollinger_params = registry.params("bollinger").unwrap();
        assert_eq!(bollinger_params.len(), 2);
        assert_eq!(bollinger_params[0].name, "period");
        assert_eq!(bollinger_params[1].name, "std_dev_mult");
    }

    #[test]
    fn test_registry_create_node_sma() {
        let registry = IndicatorRegistry::new();
        let close_id = NodeId(0);

        let node = registry.create_node("sma", Some(close_id), &[ParamValue::Usize(20)]);
        assert!(node.is_some());

        let node = node.unwrap();
        assert_eq!(node.name(), "sma");
    }

    #[test]
    fn test_registry_create_node_atr() {
        let registry = IndicatorRegistry::new();

        // ATR doesn't need an input node (uses OHLC from context)
        let node = registry.create_node("atr", None, &[ParamValue::Usize(14)]);
        assert!(node.is_some());

        let node = node.unwrap();
        assert_eq!(node.name(), "atr");
    }

    #[test]
    fn test_registry_create_node_bollinger() {
        let registry = IndicatorRegistry::new();
        let close_id = NodeId(0);

        let node = registry.create_node(
            "bollinger",
            Some(close_id),
            &[ParamValue::Usize(20), ParamValue::F64(2.0)],
        );
        assert!(node.is_some());
    }

    #[test]
    fn test_registry_create_node_invalid() {
        let registry = IndicatorRegistry::new();

        // Non-existent indicator
        let node = registry.create_node("nonexistent", None, &[]);
        assert!(node.is_none());

        // Wrong params
        let node = registry.create_node("sma", Some(NodeId(0)), &[ParamValue::F64(20.0)]);
        assert!(node.is_none());
    }

    #[test]
    fn test_registry_iter() {
        let registry = IndicatorRegistry::new();

        let names: Vec<_> = registry.names().collect();

        assert!(names.contains(&"sma"));
        assert!(names.contains(&"ema"));
        assert!(names.contains(&"macd"));
    }

    // ---------- Default + introspection ----------

    #[test]
    fn test_default_matches_new() {
        let r = IndicatorRegistry::default();
        assert!(r.contains("sma"));
        assert!(!r.is_empty());
    }

    #[test]
    fn test_get_returns_meta_with_correct_name() {
        let r = IndicatorRegistry::new();
        let meta = r.get("rsi").unwrap();
        assert_eq!(meta.name, "rsi");
    }

    #[test]
    fn test_get_unknown_returns_none() {
        let r = IndicatorRegistry::new();
        assert!(r.get("not_a_real_indicator").is_none());
    }

    #[test]
    fn test_params_unknown_returns_none() {
        let r = IndicatorRegistry::new();
        assert!(r.params("does_not_exist").is_none());
    }

    #[test]
    fn test_iter_yields_all_known_meta() {
        let r = IndicatorRegistry::new();
        let count = r.iter().count();
        assert_eq!(count, r.len());
    }

    // ---------- create_node for every covered indicator branch ----------

    #[test]
    fn test_create_node_ema_with_input() {
        let r = IndicatorRegistry::new();
        let n = r.create_node("ema", Some(NodeId(0)), &[ParamValue::Usize(10)]);
        assert!(n.is_some());
        assert_eq!(n.unwrap().name(), "ema");
    }

    #[test]
    fn test_create_node_wma_with_input() {
        let r = IndicatorRegistry::new();
        let n = r.create_node("wma", Some(NodeId(0)), &[ParamValue::Usize(7)]);
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_rsi_with_input() {
        let r = IndicatorRegistry::new();
        let n = r.create_node("rsi", Some(NodeId(0)), &[ParamValue::Usize(14)]);
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_std_dev_with_input() {
        let n = IndicatorRegistry::new()
            .create_node("std_dev", Some(NodeId(0)), &[ParamValue::Usize(5)]);
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_roc_with_input() {
        let n = IndicatorRegistry::new()
            .create_node("roc", Some(NodeId(0)), &[ParamValue::Usize(3)]);
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_efficiency_ratio_with_input() {
        let n = IndicatorRegistry::new()
            .create_node("efficiency_ratio", Some(NodeId(0)), &[ParamValue::Usize(10)]);
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_macd_with_input() {
        let n = IndicatorRegistry::new().create_node(
            "macd",
            Some(NodeId(0)),
            &[ParamValue::Usize(12), ParamValue::Usize(26), ParamValue::Usize(9)],
        );
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_ppo_with_input() {
        let n = IndicatorRegistry::new().create_node(
            "ppo",
            Some(NodeId(0)),
            &[ParamValue::Usize(12), ParamValue::Usize(26), ParamValue::Usize(9)],
        );
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_stochastic_no_input() {
        let n = IndicatorRegistry::new().create_node(
            "stochastic",
            None,
            &[ParamValue::Usize(14), ParamValue::Usize(3)],
        );
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_cci_no_input() {
        let n = IndicatorRegistry::new().create_node(
            "cci",
            None,
            &[ParamValue::Usize(20), ParamValue::F64(0.015)],
        );
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_keltner_no_input() {
        let n = IndicatorRegistry::new().create_node(
            "keltner",
            None,
            &[
                ParamValue::Usize(20),
                ParamValue::Usize(10),
                ParamValue::F64(2.0),
            ],
        );
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_chandelier_no_input() {
        let n = IndicatorRegistry::new().create_node(
            "chandelier",
            None,
            &[ParamValue::Usize(22), ParamValue::F64(3.0)],
        );
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_obv_no_input() {
        let n = IndicatorRegistry::new().create_node("obv", None, &[]);
        assert!(n.is_some());
    }

    #[test]
    fn test_create_node_mfi_no_input() {
        let n = IndicatorRegistry::new()
            .create_node("mfi", None, &[ParamValue::Usize(14)]);
        assert!(n.is_some());
    }

    // ---------- Missing-input edge case (f64-input indicator without `input`) ----------

    #[test]
    fn test_create_node_sma_without_input_returns_none() {
        let n = IndicatorRegistry::new()
            .create_node("sma", None, &[ParamValue::Usize(20)]);
        assert!(n.is_none());
    }
}
