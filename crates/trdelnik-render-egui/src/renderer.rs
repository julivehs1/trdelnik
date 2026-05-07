//! `Renderer` implementation backed by `egui_plot::PlotUi`.

use std::collections::HashMap;
use std::marker::PhantomData;

use egui::Color32;
use egui_plot::{Bar, BarChart, HLine, Line, MarkerShape as EguiMarkerShape, PlotPoints, Points, Polygon, Text};

use trdelnik_core::{AxisCoordinate, Color, MarkerShape, YAxis};
use trdelnik_render::{Bounds2D, LineStyle, Renderer, Stroke, Transform};

/// Maps right-axis values onto the left-axis range for combined plotting.
///
/// `egui_plot` only exposes a single Y axis, so a panel that wants to draw
/// on the "right axis" must manually rescale its values into the visible
/// left-axis range. Both UI sites already do this; we capture it here so
/// `Renderer::map_axis_value` can apply it uniformly.
#[derive(Debug, Clone, Copy)]
pub struct RightAxisTransform {
    /// Right-axis min visible value.
    pub right_min: f64,
    /// Right-axis max visible value.
    pub right_max: f64,
    /// Left-axis min visible value.
    pub left_min: f64,
    /// Left-axis max visible value.
    pub left_max: f64,
}

impl RightAxisTransform {
    /// Apply the transform to a value on the right axis.
    pub fn apply(&self, value: f64) -> f64 {
        let right_range = self.right_max - self.right_min;
        if right_range.abs() < f64::EPSILON {
            return value;
        }
        let normalised = (value - self.right_min) / right_range;
        self.left_min + normalised * (self.left_max - self.left_min)
    }
}

/// `egui_plot`-backed renderer.
///
/// Held by `&mut` for the duration of one `plot.show(...)` closure call.
/// Constructed via [`EguiPlotRenderer::new`] / [`EguiPlotRenderer::with_right_axis`].
pub struct EguiPlotRenderer<'a, 'p, X: AxisCoordinate> {
    plot_ui: &'a mut egui_plot::PlotUi<'p>,
    bounds: Bounds2D,
    right_axis: Option<RightAxisTransform>,
    _marker: PhantomData<X>,
}

impl<'a, 'p, X: AxisCoordinate> EguiPlotRenderer<'a, 'p, X> {
    /// Single-axis renderer.
    pub fn new(plot_ui: &'a mut egui_plot::PlotUi<'p>, bounds: Bounds2D) -> Self {
        Self {
            plot_ui,
            bounds,
            right_axis: None,
            _marker: PhantomData,
        }
    }

    /// Renderer with a right-axis transform.
    pub fn with_right_axis(
        plot_ui: &'a mut egui_plot::PlotUi<'p>,
        bounds: Bounds2D,
        right: RightAxisTransform,
    ) -> Self {
        Self {
            plot_ui,
            bounds,
            right_axis: Some(right),
            _marker: PhantomData,
        }
    }
}

fn to_egui(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

impl<'a, 'p, X: AxisCoordinate> Renderer<X> for EguiPlotRenderer<'a, 'p, X> {
    fn transform(&self) -> Transform {
        Transform::new(self.bounds)
    }

    fn draw_polyline(&mut self, name: &str, points: &[(X, Option<f64>)], stroke: Stroke) {
        let pts: Vec<[f64; 2]> = points
            .iter()
            .filter_map(|(x, y)| y.map(|v| [x.to_plot_value(), v]))
            .collect();
        let plot_points: PlotPoints = pts.into();
        let mut line = Line::new(name.to_string(), plot_points).color(to_egui(stroke.color));
        if let LineStyle::Dashed { length } = stroke.style {
            line = line.style(egui_plot::LineStyle::Dashed {
                length: length.max(1.0),
            });
        }
        self.plot_ui.line(line);
    }

    fn draw_bar(&mut self, x: X, base: f64, value: f64, width: f64, color: Color) {
        let bar = Bar::new(x.to_plot_value(), value - base)
            .base_offset(base)
            .width(width);
        self.plot_ui
            .bar_chart(BarChart::new("bar", vec![bar]).color(to_egui(color)));
    }

    fn draw_bars(&mut self, name: &str, bars: &[(X, f64)], width: f64, color: Color) {
        let collected: Vec<Bar> = bars
            .iter()
            .map(|(x, v)| Bar::new(x.to_plot_value(), *v).width(width))
            .collect();
        if collected.is_empty() {
            return;
        }
        self.plot_ui
            .bar_chart(BarChart::new(name.to_string(), collected).color(to_egui(color)));
    }

    fn draw_hline(&mut self, level: f64, stroke: Stroke) {
        let mut hline = HLine::new("ref", level).color(to_egui(stroke.color));
        if let LineStyle::Dashed { length } = stroke.style {
            hline = hline.style(egui_plot::LineStyle::Dashed {
                length: length.max(1.0),
            });
        }
        self.plot_ui.hline(hline);
    }

    fn draw_marker(&mut self, x: X, y: f64, shape: MarkerShape, color: Color) {
        let egui_shape = match shape {
            MarkerShape::ArrowUp => EguiMarkerShape::Up,
            MarkerShape::ArrowDown => EguiMarkerShape::Down,
            MarkerShape::Cross => EguiMarkerShape::Cross,
            MarkerShape::Circle => EguiMarkerShape::Circle,
            MarkerShape::Square => EguiMarkerShape::Square,
        };
        self.plot_ui.points(
            Points::new("marker", vec![[x.to_plot_value(), y]])
                .shape(egui_shape)
                .filled(true)
                .radius(6.0)
                .color(to_egui(color)),
        );
    }

    fn draw_polygon(&mut self, points: &[(X, f64)], fill: Color) {
        let pts: Vec<[f64; 2]> = points
            .iter()
            .map(|(x, y)| [x.to_plot_value(), *y])
            .collect();
        if pts.is_empty() {
            return;
        }
        let plot_points: PlotPoints = pts.into();
        self.plot_ui
            .polygon(Polygon::new("polygon", plot_points).fill_color(to_egui(fill)));
    }

    fn draw_text(&mut self, x: X, y: f64, text: &str, color: Color) {
        self.plot_ui.text(
            Text::new("text", egui_plot::PlotPoint::new(x.to_plot_value(), y), text.to_string())
                .color(to_egui(color)),
        );
    }

    fn has_right_axis(&self) -> bool {
        self.right_axis.is_some()
    }

    fn map_axis_value(&self, axis: YAxis, value: f64) -> f64 {
        match (axis, &self.right_axis) {
            (YAxis::Right, Some(t)) => t.apply(value),
            _ => value,
        }
    }
}

/// Stable type used for batching bars by colour key in the rendering loop.
pub type BarBatchMap<X> = HashMap<[u8; 4], Vec<(X, f64)>>;
