//! Optimization methods

mod grid;
mod random;

pub use grid::GridSearch;
pub use random::RandomSearch;

use crate::param::ParamSpace;
use crate::result::ParamSet;
use std::collections::HashMap;

/// Trait for optimization methods
pub trait OptimizationMethod: Send + Sync {
    /// Name of the optimization method
    fn name(&self) -> &'static str;

    /// Run the optimization over the parameter space
    ///
    /// The `evaluate` function takes a parameter combination and returns
    /// an optional ParamSet. It returns None if the evaluation failed
    /// (e.g., backtest produced invalid results).
    fn run<F>(&self, space: &ParamSpace, evaluate: F) -> Vec<ParamSet>
    where
        F: Fn(&HashMap<String, f64>) -> Option<ParamSet> + Sync;
}
