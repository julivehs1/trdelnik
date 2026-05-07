//! Plot ↔ screen coordinate transform.
//!
//! Most plot primitives accept logical (plot-space) coordinates and rely
//! on the renderer's bound transform to translate to screen pixels. The
//! `Transform` struct exposes the current view bounds and screen rect for
//! plots that need to do their own pixel-level work.

/// 2D rectangle of inclusive bounds.
#[derive(Debug, Clone, Copy)]
pub struct Bounds2D {
    /// Inclusive minimum (x, y).
    pub min: (f64, f64),
    /// Inclusive maximum (x, y).
    pub max: (f64, f64),
}

impl Bounds2D {
    /// Construct from raw min/max pairs.
    pub fn new(min: (f64, f64), max: (f64, f64)) -> Self {
        Self { min, max }
    }

    /// Width on the X axis.
    pub fn width(&self) -> f64 {
        self.max.0 - self.min.0
    }

    /// Height on the Y axis.
    pub fn height(&self) -> f64 {
        self.max.1 - self.min.1
    }
}

/// Read-only view of the current plot transform. Plots use this when they
/// need to know what's visible (for clipping or LOD decisions).
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Current visible bounds in plot space.
    pub bounds: Bounds2D,
}

impl Transform {
    /// New transform from plot-space bounds.
    pub fn new(bounds: Bounds2D) -> Self {
        Self { bounds }
    }
}
