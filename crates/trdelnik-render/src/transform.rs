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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_new() {
        let b = Bounds2D::new((0.0, 1.0), (10.0, 5.0));
        assert_eq!(b.min, (0.0, 1.0));
        assert_eq!(b.max, (10.0, 5.0));
    }

    #[test]
    fn test_bounds_width() {
        let b = Bounds2D::new((1.0, 0.0), (4.0, 0.0));
        assert!((b.width() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_bounds_height() {
        let b = Bounds2D::new((0.0, -2.0), (0.0, 7.5));
        assert!((b.height() - 9.5).abs() < 1e-9);
    }

    #[test]
    fn test_bounds_zero_extent() {
        let b = Bounds2D::new((5.0, 5.0), (5.0, 5.0));
        assert_eq!(b.width(), 0.0);
        assert_eq!(b.height(), 0.0);
    }

    #[test]
    fn test_bounds_negative_extent_allowed() {
        // Inverted ranges are not normalized — width/height can be negative
        let b = Bounds2D::new((10.0, 10.0), (0.0, 0.0));
        assert!(b.width() < 0.0);
        assert!(b.height() < 0.0);
    }

    #[test]
    fn test_transform_new_carries_bounds() {
        let b = Bounds2D::new((0.0, 0.0), (100.0, 50.0));
        let t = Transform::new(b);
        assert_eq!(t.bounds.min, (0.0, 0.0));
        assert_eq!(t.bounds.max, (100.0, 50.0));
    }

    #[test]
    fn test_transform_copy_clone() {
        let t1 = Transform::new(Bounds2D::new((0.0, 0.0), (1.0, 1.0)));
        let t2 = t1; // Copy
        let t3 = t1.clone();
        assert_eq!(t1.bounds.max, t2.bounds.max);
        assert_eq!(t1.bounds.max, t3.bounds.max);
    }
}
