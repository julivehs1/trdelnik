//! Built-in computation graph nodes
//!
//! This module provides nodes for building indicator computation graphs.
//!
//! ## Factory Functions (Recommended)
//!
//! Use factory functions for a clean, concise API:
//!
//! ```ignore
//! use trdelnik_graph::nodes::*;
//!
//! let mut graph = Graph::new();
//! let close = graph.add_node(Box::new(CloseNode::new()));
//!
//! // Single-output indicators
//! let sma20 = graph.add_node(sma(close, 20));
//! let rsi14 = graph.add_node(rsi(close, 14));
//!
//! // Multi-output indicators with field extraction
//! let (boll, middle, upper, lower) = bollinger_with_fields(&mut graph, close, 20, 2.0);
//! ```
//!
//! ## Generic Wrappers
//!
//! For custom indicators implementing `StatefulIndicator`, `OhlcIndicator`, or
//! `OhlcvIndicator`, use the generic wrappers directly. Indicators now provide
//! their own metadata (NAME, param_signature), so no additional parameters are needed:
//!
//! ```ignore
//! use trdelnik_graph::nodes::StatefulNode;
//! use my_indicators::MyCustomState;
//!
//! // MyCustomState must implement StatefulIndicator with NAME and param_signature()
//! let node = StatefulNode::new(input, MyCustomState::new(params));
//! ```

pub mod arithmetic;
pub mod comparison;
pub mod control;
pub mod data;
pub mod factories;
pub mod field;
pub mod generic;
pub mod htf;
pub mod htf_factories;
pub mod pattern;

// Re-export all nodes
pub use arithmetic::*;
pub use comparison::*;
pub use control::{LagNode, SelectNode};
pub use data::*;
pub use pattern::{BarsSinceNode, CountWhenNode};

// Re-export generic types
pub use field::*;
pub use generic::*;
pub use htf::{AggregatedCandle, BarAggregator, HtfIndicatorNode, HtfInput};

// Re-export factory functions as primary API
pub use factories::*;
pub use htf_factories::{htf_atr, htf_ema, htf_rsi, htf_sma};
