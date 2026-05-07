//! # Trdelnik Render
//!
//! Backend-agnostic rendering layer for trdelnik. Defines:
//!
//! - [`Plot`] — the extension point for visualisations (lines, histograms,
//!   custom heatmaps, volume profiles, …).
//! - [`StandardPlot`] — built-in `Plot` covering the classical "lines +
//!   optional histogram" shape used by every indicator.
//! - [`Renderer`] — the immediate-mode primitives a backend implements
//!   (egui, Canvas, wgpu, …).
//! - [`PlotContext`] — the bundle of `Renderer + IndicatorTheme` passed
//!   into `Plot::render`.
//! - [`Transform`], [`LineStyle`], [`Stroke`], [`IndicatorTheme`] —
//!   supporting types.
//! - [`HLine`] / [`HLineStyle`] — horizontal reference lines (panel-level).
//!
//! Concrete backends live in their own crates (e.g. `trdelnik-render-egui`).

pub mod plot;
pub mod renderer;
pub mod style;
pub mod theme;
pub mod transform;

pub use plot::{
    aggregate_ranges, y_range_with_padding, HLine, HLineStyle, Plot, PlotContext, StandardPlot,
};
pub use renderer::Renderer;
pub use style::{LineStyle, Stroke};
pub use theme::IndicatorTheme;
pub use transform::{Bounds2D, Transform};
