//! Equity curve analysis

use crate::engine::EquityPoint;
use serde::{Deserialize, Serialize};
use trdelnik_core::AxisCoordinate;

/// Information about maximum drawdown
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DrawdownInfo<X: AxisCoordinate> {
    /// Maximum drawdown amount
    pub max_drawdown: f64,
    /// Maximum drawdown percentage
    pub max_drawdown_pct: f64,
    /// X coordinate at drawdown peak
    pub peak_x: X,
    /// X coordinate at drawdown trough
    pub trough_x: X,
    /// Bar index at peak
    pub peak_bar: usize,
    /// Bar index at trough
    pub trough_bar: usize,
    /// Peak equity value
    pub peak_equity: f64,
    /// Trough equity value
    pub trough_equity: f64,
}

/// Equity curve with analysis methods
#[derive(Debug, Clone)]
pub struct EquityCurve<X: AxisCoordinate> {
    points: Vec<EquityPoint<X>>,
}

impl<X: AxisCoordinate> EquityCurve<X> {
    /// Create a new equity curve from points
    pub fn new(points: Vec<EquityPoint<X>>) -> Self {
        Self { points }
    }

    /// Get the equity points
    pub fn points(&self) -> &[EquityPoint<X>] {
        &self.points
    }

    /// Number of points
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Get equity values as a vector
    pub fn equity_values(&self) -> Vec<f64> {
        self.points.iter().map(|p| p.equity).collect()
    }

    /// Get X values as plot values
    pub fn x_values(&self) -> Vec<f64> {
        self.points.iter().map(|p| p.x.to_plot_value()).collect()
    }

    /// Get cash values
    pub fn cash_values(&self) -> Vec<f64> {
        self.points.iter().map(|p| p.cash).collect()
    }

    /// Get drawdown values
    pub fn drawdown_values(&self) -> Vec<f64> {
        self.points.iter().map(|p| p.drawdown).collect()
    }

    /// Get drawdown percentage values
    pub fn drawdown_pct_values(&self) -> Vec<f64> {
        self.points.iter().map(|p| p.drawdown_pct).collect()
    }

    /// Calculate period returns (bar-to-bar)
    pub fn period_returns(&self) -> Vec<f64> {
        if self.points.len() < 2 {
            return Vec::new();
        }

        self.points
            .windows(2)
            .map(|w| {
                let prev = w[0].equity;
                let curr = w[1].equity;
                if prev > 0.0 {
                    (curr - prev) / prev
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// Calculate logarithmic returns
    pub fn log_returns(&self) -> Vec<f64> {
        if self.points.len() < 2 {
            return Vec::new();
        }

        self.points
            .windows(2)
            .map(|w| {
                let prev = w[0].equity;
                let curr = w[1].equity;
                if prev > 0.0 && curr > 0.0 {
                    (curr / prev).ln()
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// Get detailed drawdown information
    pub fn max_drawdown_info(&self) -> Option<DrawdownInfo<X>> {
        if self.points.is_empty() {
            return None;
        }

        let mut max_dd = 0.0;
        let mut max_dd_pct = 0.0;
        let mut peak_idx = 0;
        let mut trough_idx = 0;
        let mut running_peak_idx = 0;
        let mut running_peak = self.points[0].equity;

        for (i, point) in self.points.iter().enumerate() {
            if point.equity > running_peak {
                running_peak = point.equity;
                running_peak_idx = i;
            }

            let dd = running_peak - point.equity;
            let dd_pct = if running_peak > 0.0 {
                (dd / running_peak) * 100.0
            } else {
                0.0
            };

            if dd > max_dd {
                max_dd = dd;
                max_dd_pct = dd_pct;
                peak_idx = running_peak_idx;
                trough_idx = i;
            }
        }

        let peak = &self.points[peak_idx];
        let trough = &self.points[trough_idx];

        Some(DrawdownInfo {
            max_drawdown: max_dd,
            max_drawdown_pct: max_dd_pct,
            peak_x: peak.x,
            trough_x: trough.x,
            peak_bar: peak.bar_index,
            trough_bar: trough.bar_index,
            peak_equity: peak.equity,
            trough_equity: trough.equity,
        })
    }

    /// Calculate the maximum equity (high water mark)
    pub fn max_equity(&self) -> Option<f64> {
        self.points.iter().map(|p| p.equity).reduce(f64::max)
    }

    /// Calculate the minimum equity
    pub fn min_equity(&self) -> Option<f64> {
        self.points.iter().map(|p| p.equity).reduce(f64::min)
    }

    /// Get the final equity
    pub fn final_equity(&self) -> Option<f64> {
        self.points.last().map(|p| p.equity)
    }

    /// Get the initial equity
    pub fn initial_equity(&self) -> Option<f64> {
        self.points.first().map(|p| p.equity)
    }

    /// Calculate total return
    pub fn total_return(&self) -> Option<f64> {
        let initial = self.initial_equity()?;
        let final_eq = self.final_equity()?;
        if initial > 0.0 {
            Some(final_eq - initial)
        } else {
            None
        }
    }

    /// Calculate total return percentage
    pub fn total_return_pct(&self) -> Option<f64> {
        let initial = self.initial_equity()?;
        let final_eq = self.final_equity()?;
        if initial > 0.0 {
            Some(((final_eq - initial) / initial) * 100.0)
        } else {
            None
        }
    }

    /// Calculate underwater curve (drawdown from running peak)
    pub fn underwater(&self) -> Vec<f64> {
        if self.points.is_empty() {
            return Vec::new();
        }

        let mut running_peak = self.points[0].equity;
        self.points
            .iter()
            .map(|p| {
                if p.equity > running_peak {
                    running_peak = p.equity;
                }
                -((running_peak - p.equity) / running_peak) * 100.0
            })
            .collect()
    }

    /// Get number of bars spent in drawdown
    pub fn bars_in_drawdown(&self) -> usize {
        self.points.iter().filter(|p| p.drawdown > 0.0).count()
    }

    /// Calculate average drawdown
    pub fn average_drawdown(&self) -> f64 {
        let drawdowns: Vec<f64> = self
            .points
            .iter()
            .filter(|p| p.drawdown > 0.0)
            .map(|p| p.drawdown)
            .collect();

        if drawdowns.is_empty() {
            0.0
        } else {
            drawdowns.iter().sum::<f64>() / drawdowns.len() as f64
        }
    }
}

impl<X: AxisCoordinate> From<Vec<EquityPoint<X>>> for EquityCurve<X> {
    fn from(points: Vec<EquityPoint<X>>) -> Self {
        Self::new(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::Timestamp;

    fn create_test_curve() -> EquityCurve<Timestamp> {
        let points = vec![
            EquityPoint::new(Timestamp::new(1000), 0, 100_000.0, 100_000.0, 0, 100_000.0),
            EquityPoint::new(Timestamp::new(2000), 1, 105_000.0, 100_000.0, 1, 105_000.0),
            EquityPoint::new(Timestamp::new(3000), 2, 102_000.0, 100_000.0, 1, 105_000.0),
            EquityPoint::new(Timestamp::new(4000), 3, 108_000.0, 100_000.0, 1, 108_000.0),
            EquityPoint::new(Timestamp::new(5000), 4, 95_000.0, 100_000.0, 0, 108_000.0),
            EquityPoint::new(Timestamp::new(6000), 5, 110_000.0, 110_000.0, 0, 110_000.0),
        ];
        EquityCurve::new(points)
    }

    #[test]
    fn test_equity_curve_basics() {
        let curve = create_test_curve();
        assert_eq!(curve.len(), 6);
        assert_eq!(curve.initial_equity(), Some(100_000.0));
        assert_eq!(curve.final_equity(), Some(110_000.0));
        assert_eq!(curve.max_equity(), Some(110_000.0));
        assert_eq!(curve.min_equity(), Some(95_000.0));
    }

    #[test]
    fn test_total_return() {
        let curve = create_test_curve();
        assert_eq!(curve.total_return(), Some(10_000.0));
        assert!((curve.total_return_pct().unwrap() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_period_returns() {
        let curve = create_test_curve();
        let returns = curve.period_returns();
        assert_eq!(returns.len(), 5);
        // First return: (105000 - 100000) / 100000 = 0.05
        assert!((returns[0] - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_max_drawdown_info() {
        let curve = create_test_curve();
        let dd_info = curve.max_drawdown_info().unwrap();

        // Max drawdown is from 108000 to 95000 = 13000
        assert!((dd_info.max_drawdown - 13_000.0).abs() < 1.0);
        assert!((dd_info.max_drawdown_pct - 12.037).abs() < 0.1);
        assert_eq!(dd_info.peak_equity, 108_000.0);
        assert_eq!(dd_info.trough_equity, 95_000.0);
    }

    fn empty_curve() -> EquityCurve<Timestamp> {
        EquityCurve::new(vec![])
    }

    fn single_point_curve() -> EquityCurve<Timestamp> {
        EquityCurve::new(vec![EquityPoint::new(
            Timestamp::new(1000),
            0,
            100_000.0,
            100_000.0,
            0,
            100_000.0,
        )])
    }

    // ---------- Empty / edge cases ----------

    #[test]
    fn test_is_empty_for_no_points() {
        assert!(empty_curve().is_empty());
    }

    #[test]
    fn test_initial_final_min_max_none_on_empty() {
        let c = empty_curve();
        assert!(c.initial_equity().is_none());
        assert!(c.final_equity().is_none());
        assert!(c.min_equity().is_none());
        assert!(c.max_equity().is_none());
    }

    #[test]
    fn test_total_return_none_on_empty() {
        let c = empty_curve();
        assert!(c.total_return().is_none());
        assert!(c.total_return_pct().is_none());
    }

    #[test]
    fn test_max_drawdown_info_none_on_empty() {
        let c = empty_curve();
        assert!(c.max_drawdown_info().is_none());
    }

    #[test]
    fn test_period_returns_empty_when_under_two_points() {
        assert!(empty_curve().period_returns().is_empty());
        assert!(single_point_curve().period_returns().is_empty());
    }

    #[test]
    fn test_log_returns_empty_when_under_two_points() {
        assert!(empty_curve().log_returns().is_empty());
        assert!(single_point_curve().log_returns().is_empty());
    }

    #[test]
    fn test_underwater_empty_for_empty_curve() {
        assert!(empty_curve().underwater().is_empty());
    }

    // ---------- Accessors ----------

    #[test]
    fn test_points_accessor() {
        let c = create_test_curve();
        assert_eq!(c.points().len(), 6);
        assert_eq!(c.points()[0].equity, 100_000.0);
    }

    #[test]
    fn test_equity_values() {
        let v = create_test_curve().equity_values();
        assert_eq!(v.len(), 6);
        assert_eq!(v[0], 100_000.0);
        assert_eq!(v[5], 110_000.0);
    }

    #[test]
    fn test_x_values_returns_plot_values() {
        let v = create_test_curve().x_values();
        assert_eq!(v.len(), 6);
        // Timestamp::to_plot_value() returns the raw millis as f64.
        assert!((v[0] - 1000.0).abs() < 1e-9);
        assert!((v[5] - 6000.0).abs() < 1e-9);
    }

    #[test]
    fn test_cash_values() {
        let v = create_test_curve().cash_values();
        assert_eq!(v.len(), 6);
        assert_eq!(v[0], 100_000.0);
    }

    #[test]
    fn test_drawdown_and_drawdown_pct_values() {
        let c = create_test_curve();
        let dd = c.drawdown_values();
        let dd_pct = c.drawdown_pct_values();
        assert_eq!(dd.len(), 6);
        assert_eq!(dd_pct.len(), 6);
        // First point: peak == equity, dd = 0
        assert_eq!(dd[0], 0.0);
        assert_eq!(dd_pct[0], 0.0);
        // The trough at i=4: equity 95k vs peak 108k → dd = 13k
        assert!((dd[4] - 13_000.0).abs() < 1e-9);
        assert!((dd_pct[4] - 12.037).abs() < 0.1);
    }

    // ---------- Returns / underwater ----------

    #[test]
    fn test_log_returns_known_values() {
        let c = create_test_curve();
        let lr = c.log_returns();
        assert_eq!(lr.len(), 5);
        // ln(105_000 / 100_000) ≈ 0.04879
        assert!((lr[0] - (105_000.0_f64 / 100_000.0).ln()).abs() < 1e-9);
    }

    #[test]
    fn test_log_returns_zero_when_prev_or_curr_nonpositive() {
        // Construct a curve where one equity is zero — log_returns must
        // emit 0.0 instead of -inf or panicking.
        let points = vec![
            EquityPoint::new(Timestamp::new(0), 0, 100.0, 100.0, 0, 100.0),
            EquityPoint::new(Timestamp::new(1000), 1, 0.0, 0.0, 0, 100.0),
            EquityPoint::new(Timestamp::new(2000), 2, 50.0, 50.0, 0, 100.0),
        ];
        let lr = EquityCurve::new(points).log_returns();
        assert_eq!(lr.len(), 2);
        // 100 → 0 has curr <= 0
        assert_eq!(lr[0], 0.0);
        // 0 → 50 has prev <= 0
        assert_eq!(lr[1], 0.0);
    }

    #[test]
    fn test_underwater_running_peak() {
        let c = create_test_curve();
        let uw = c.underwater();
        assert_eq!(uw.len(), 6);
        // First bar: peak == equity → underwater = 0
        assert!((uw[0] - 0.0).abs() < 1e-9);
        // Trough at i=4: -((108k - 95k) / 108k) * 100 ≈ -12.037
        assert!((uw[4] - (-12.037)).abs() < 0.1);
        // After recovery to a new high at i=5, underwater is 0 again
        assert!((uw[5] - 0.0).abs() < 1e-9);
    }

    // ---------- Drawdown statistics ----------

    #[test]
    fn test_bars_in_drawdown_counts_nonzero_drawdowns() {
        let c = create_test_curve();
        // The points where running_peak > equity: i=2 (102k vs 105k peak)
        // and i=4 (95k vs 108k peak). i=0 and others are at peak.
        let n = c.bars_in_drawdown();
        assert!(n >= 2);
    }

    #[test]
    fn test_average_drawdown_excludes_zero_drawdown_points() {
        let c = create_test_curve();
        let avg = c.average_drawdown();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_average_drawdown_zero_when_no_drawdown() {
        let points = vec![
            EquityPoint::new(Timestamp::new(0), 0, 100.0, 100.0, 0, 100.0),
            EquityPoint::new(Timestamp::new(1000), 1, 200.0, 200.0, 0, 200.0),
            EquityPoint::new(Timestamp::new(2000), 2, 300.0, 300.0, 0, 300.0),
        ];
        let c = EquityCurve::new(points);
        assert_eq!(c.average_drawdown(), 0.0);
        assert_eq!(c.bars_in_drawdown(), 0);
    }

    // ---------- Period returns edge case ----------

    #[test]
    fn test_period_returns_zero_when_prev_nonpositive() {
        let points = vec![
            EquityPoint::new(Timestamp::new(0), 0, 0.0, 0.0, 0, 100.0),
            EquityPoint::new(Timestamp::new(1000), 1, 50.0, 50.0, 0, 100.0),
        ];
        let pr = EquityCurve::new(points).period_returns();
        assert_eq!(pr, vec![0.0]);
    }

    // ---------- From impl ----------

    #[test]
    fn test_from_vec_constructs_equivalent_curve() {
        let points = vec![EquityPoint::new(
            Timestamp::new(1000),
            0,
            100.0,
            100.0,
            0,
            100.0,
        )];
        let from_curve: EquityCurve<Timestamp> = points.clone().into();
        let new_curve = EquityCurve::new(points);
        assert_eq!(from_curve.len(), new_curve.len());
        assert_eq!(from_curve.initial_equity(), new_curve.initial_equity());
    }

    // ---------- Total return when initial equity zero ----------

    #[test]
    fn test_total_return_none_when_initial_zero() {
        let points = vec![
            EquityPoint::new(Timestamp::new(0), 0, 0.0, 0.0, 0, 0.0),
            EquityPoint::new(Timestamp::new(1000), 1, 100.0, 100.0, 0, 100.0),
        ];
        let c = EquityCurve::new(points);
        assert!(c.total_return().is_none());
        assert!(c.total_return_pct().is_none());
    }
}
