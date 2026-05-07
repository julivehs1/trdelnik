//! Backend-agnostic immediate-mode rendering primitives.

use trdelnik_core::{AxisCoordinate, Color, MarkerShape, YAxis};

use crate::style::Stroke;
use crate::transform::Transform;

/// Immediate-mode primitives every backend implements.
///
/// All coordinates are in plot space (x in `X` plot-value units, y as f64).
/// The backend handles the projection to screen pixels.
pub trait Renderer<X: AxisCoordinate> {
    /// Current view transform (visible bounds, etc.).
    fn transform(&self) -> Transform;

    /// Draw a polyline through the given (x, y) points. Points with `y =
    /// None` create gaps.
    fn draw_polyline(&mut self, name: &str, points: &[(X, Option<f64>)], stroke: Stroke);

    /// Draw a vertical bar from `(x, 0)` to `(x, value)` (or
    /// `(x, base)` to `(x, value)` if `base` is set).
    fn draw_bar(&mut self, x: X, base: f64, value: f64, width: f64, color: Color);

    /// Draw a batched bar series sharing the same colour.
    fn draw_bars(&mut self, name: &str, bars: &[(X, f64)], width: f64, color: Color);

    /// Draw a horizontal reference line spanning the visible X range.
    fn draw_hline(&mut self, level: f64, stroke: Stroke);

    /// Draw a marker shape at a plot-space point.
    fn draw_marker(&mut self, x: X, y: f64, shape: MarkerShape, color: Color);

    /// Draw a closed filled polygon (e.g. for area fills, custom shapes).
    fn draw_polygon(&mut self, points: &[(X, f64)], fill: Color);

    /// Draw text anchored at a plot-space point.
    fn draw_text(&mut self, x: X, y: f64, text: &str, color: Color);

    /// Whether the panel currently has both axes (some renderers transform
    /// right-axis values onto the left axis for combined drawing).
    fn has_right_axis(&self) -> bool {
        false
    }

    /// Map a value on the given axis into the renderer's drawing space.
    /// Default: identity. Backends with axis-coupling override this.
    fn map_axis_value(&self, axis: YAxis, value: f64) -> f64 {
        let _ = axis;
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::Bounds2D;
    use trdelnik_core::Index;

    /// Minimal `Renderer` impl that only implements the required methods,
    /// so we can verify `has_right_axis()` and `map_axis_value()` defaults.
    struct MinimalRenderer;

    impl Renderer<Index> for MinimalRenderer {
        fn transform(&self) -> Transform {
            Transform::new(Bounds2D::new((0.0, 0.0), (1.0, 1.0)))
        }
        fn draw_polyline(&mut self, _: &str, _: &[(Index, Option<f64>)], _: Stroke) {}
        fn draw_bar(&mut self, _: Index, _: f64, _: f64, _: f64, _: Color) {}
        fn draw_bars(&mut self, _: &str, _: &[(Index, f64)], _: f64, _: Color) {}
        fn draw_hline(&mut self, _: f64, _: Stroke) {}
        fn draw_marker(&mut self, _: Index, _: f64, _: MarkerShape, _: Color) {}
        fn draw_polygon(&mut self, _: &[(Index, f64)], _: Color) {}
        fn draw_text(&mut self, _: Index, _: f64, _: &str, _: Color) {}
        // Intentionally NOT overriding has_right_axis or map_axis_value —
        // the defaults from the trait should apply.
    }

    #[test]
    fn test_default_has_right_axis_is_false() {
        let r = MinimalRenderer;
        assert!(!Renderer::<Index>::has_right_axis(&r));
    }

    #[test]
    fn test_default_map_axis_value_is_identity_for_left() {
        let r = MinimalRenderer;
        assert_eq!(Renderer::<Index>::map_axis_value(&r, YAxis::Left, 42.0), 42.0);
    }

    #[test]
    fn test_default_map_axis_value_is_identity_for_right() {
        let r = MinimalRenderer;
        assert_eq!(Renderer::<Index>::map_axis_value(&r, YAxis::Right, -7.5), -7.5);
    }
}
