//! # Trdelnik Render — egui backend
//!
//! egui implementation of `trdelnik_render::Renderer`. Provides:
//!
//! - [`EguiPlotRenderer`] — wraps `&mut egui_plot::PlotUi` as a `Renderer<X>`.
//! - [`RightAxisTransform`] — captures the "rescale right-axis values onto
//!   the visible left-axis range" trick that `egui_plot`'s single-axis
//!   model forces on us.
//! - [`ChartThemeAdapter`] — wraps `trdelnik_theme::ChartTheme` as an
//!   `IndicatorTheme`.
//! - [`render_plot`] — convenience entry helper that constructs the
//!   context and dispatches to the plot's `render` method.

pub mod renderer;
pub mod theme;

pub use renderer::{EguiPlotRenderer, RightAxisTransform};
pub use theme::ChartThemeAdapter;

use trdelnik_core::AxisCoordinate;
use trdelnik_render::{Plot, PlotContext};
use trdelnik_theme::ChartTheme;

/// Convenience: render a `Plot` into the given `PlotUi` using a
/// `ChartTheme`. For the typical single-axis case.
pub fn render_plot<X: AxisCoordinate>(
    plot: &dyn Plot<X>,
    plot_ui: &mut egui_plot::PlotUi<'_>,
    bounds: trdelnik_render::Bounds2D,
    theme: &ChartTheme,
) {
    let mut renderer = EguiPlotRenderer::<X>::new(plot_ui, bounds);
    let theme_adapter = ChartThemeAdapter::new(theme);
    let mut ctx = PlotContext::new(&mut renderer, &theme_adapter);
    plot.render(&mut ctx);
}

/// Convenience: render a `Plot` into the given `PlotUi` with a right-axis
/// transform applied to right-axis values.
pub fn render_plot_dual_axis<X: AxisCoordinate>(
    plot: &dyn Plot<X>,
    plot_ui: &mut egui_plot::PlotUi<'_>,
    bounds: trdelnik_render::Bounds2D,
    right: RightAxisTransform,
    theme: &ChartTheme,
) {
    let mut renderer = EguiPlotRenderer::<X>::with_right_axis(plot_ui, bounds, right);
    let theme_adapter = ChartThemeAdapter::new(theme);
    let mut ctx = PlotContext::new(&mut renderer, &theme_adapter);
    plot.render(&mut ctx);
}
