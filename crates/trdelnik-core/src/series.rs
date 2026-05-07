//! Candle series (collection of candles)

use crate::axis::{AxisCoordinate, Timestamp};
use crate::candle::Candle;
use crate::timeframe::Timeframe;
use std::time::{SystemTime, UNIX_EPOCH};

/// A series of candles with a generic X coordinate
#[derive(Debug, Clone, Default)]
pub struct CandleSeries<X: AxisCoordinate = Timestamp> {
    candles: Vec<Candle<X>>,
    timeframe: Option<Timeframe>,
}

impl<X: AxisCoordinate> CandleSeries<X> {
    /// Create a new empty candle series
    pub fn new() -> Self {
        Self {
            candles: Vec::new(),
            timeframe: None,
        }
    }

    /// Create a candle series with a specific timeframe
    pub fn with_timeframe(timeframe: Timeframe) -> Self {
        Self {
            candles: Vec::new(),
            timeframe: Some(timeframe),
        }
    }

    /// Create from a vector of candles
    pub fn from_candles(candles: Vec<Candle<X>>) -> Self {
        Self {
            candles,
            timeframe: None,
        }
    }

    /// Create from candles with a timeframe
    pub fn from_candles_with_timeframe(candles: Vec<Candle<X>>, timeframe: Timeframe) -> Self {
        Self {
            candles,
            timeframe: Some(timeframe),
        }
    }

    /// Add a candle to the series
    pub fn push(&mut self, candle: Candle<X>) {
        self.candles.push(candle);
    }

    /// Get the candles
    pub fn candles(&self) -> &[Candle<X>] {
        &self.candles
    }

    /// Get mutable candles
    pub fn candles_mut(&mut self) -> &mut Vec<Candle<X>> {
        &mut self.candles
    }

    /// Get the timeframe
    pub fn timeframe(&self) -> Option<Timeframe> {
        self.timeframe
    }

    /// Set the timeframe
    pub fn set_timeframe(&mut self, timeframe: Timeframe) {
        self.timeframe = Some(timeframe);
    }

    /// Get the number of candles
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    /// Get a candle by index
    pub fn get(&self, index: usize) -> Option<&Candle<X>> {
        self.candles.get(index)
    }

    /// Get the last candle
    pub fn last(&self) -> Option<&Candle<X>> {
        self.candles.last()
    }

    /// Get the first candle
    pub fn first(&self) -> Option<&Candle<X>> {
        self.candles.first()
    }

    /// Get the price range (min low, max high)
    pub fn price_range(&self) -> Option<(f64, f64)> {
        if self.candles.is_empty() {
            return None;
        }
        let min = self
            .candles
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min);
        let max = self
            .candles
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);
        Some((min, max))
    }

    /// Get the volume range (0, max volume)
    pub fn volume_range(&self) -> Option<(f64, f64)> {
        if self.candles.is_empty() {
            return None;
        }
        let max = self.candles.iter().map(|c| c.volume).fold(0.0, f64::max);
        Some((0.0, max))
    }

    /// Get the X range (first x, last x) as plot values
    pub fn x_range(&self) -> Option<(f64, f64)> {
        if self.candles.is_empty() {
            return None;
        }
        let first = self.candles.first().map(|c| c.x.to_plot_value())?;
        let last = self.candles.last().map(|c| c.x.to_plot_value())?;
        Some((first, last))
    }

    /// Get closing prices as a vector
    pub fn closes(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.close).collect()
    }

    /// Get opening prices as a vector
    pub fn opens(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.open).collect()
    }

    /// Get high prices as a vector
    pub fn highs(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.high).collect()
    }

    /// Get low prices as a vector
    pub fn lows(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.low).collect()
    }

    /// Get volumes as a vector
    pub fn volumes(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.volume).collect()
    }

    /// Get X coordinates as plot values
    pub fn x_values(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.x.to_plot_value()).collect()
    }

    /// Iterator over candles
    pub fn iter(&self) -> impl Iterator<Item = &Candle<X>> {
        self.candles.iter()
    }

    /// Sort candles by X coordinate
    pub fn sort_by_x(&mut self) {
        self.candles
            .sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Map the series to a different X coordinate type
    pub fn map_x<Y: AxisCoordinate>(self, f: impl Fn(X) -> Y) -> CandleSeries<Y> {
        CandleSeries {
            candles: self.candles.into_iter().map(|c| c.map_x(&f)).collect(),
            timeframe: self.timeframe,
        }
    }

    /// Calculate the spacing between candles (for rendering).
    ///
    /// The spacing is determined in the following order:
    /// 1. If a timeframe is set, returns the timeframe duration in milliseconds
    /// 2. If at least 2 candles exist, calculates spacing from the first two candles
    ///    (assumes candles are sorted by X coordinate)
    /// 3. Otherwise, returns the default spacing for the coordinate type
    ///
    /// # Note
    ///
    /// When relying on the calculation from the first two candles, ensure the series
    /// is sorted by X coordinate (call `sort_by_x()`) or set a timeframe explicitly.
    pub fn x_spacing(&self) -> f64 {
        if let Some(tf) = self.timeframe {
            tf.as_millis() as f64
        } else if self.candles.len() >= 2 {
            let x0 = self.candles[0].x.to_plot_value();
            let x1 = self.candles[1].x.to_plot_value();
            (x1 - x0).abs()
        } else {
            X::default_spacing()
        }
    }
}

impl<'a, X: AxisCoordinate> IntoIterator for &'a CandleSeries<X> {
    type Item = &'a Candle<X>;
    type IntoIter = std::slice::Iter<'a, Candle<X>>;

    fn into_iter(self) -> Self::IntoIter {
        self.candles.iter()
    }
}

impl<X: AxisCoordinate> IntoIterator for CandleSeries<X> {
    type Item = Candle<X>;
    type IntoIter = std::vec::IntoIter<Candle<X>>;

    fn into_iter(self) -> Self::IntoIter {
        self.candles.into_iter()
    }
}

// Timestamp-specific implementations
impl CandleSeries<Timestamp> {
    /// Get the time range (first timestamp, last timestamp) in milliseconds
    pub fn time_range(&self) -> Option<(i64, i64)> {
        if self.candles.is_empty() {
            return None;
        }
        let first = self.candles.first().map(|c| c.x.0)?;
        let last = self.candles.last().map(|c| c.x.0)?;
        Some((first, last))
    }

    /// Get timestamps as a vector of milliseconds
    pub fn timestamps(&self) -> Vec<i64> {
        self.candles.iter().map(|c| c.x.0).collect()
    }

    /// Sort candles by timestamp
    pub fn sort_by_time(&mut self) {
        self.candles.sort_by_key(|c| c.x.0);
    }
}

/// Generate sample data for testing/demo purposes
pub fn generate_sample_data(num_candles: usize, timeframe: Timeframe) -> CandleSeries<Timestamp> {
    let mut series = CandleSeries::with_timeframe(timeframe);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let interval = timeframe.as_millis();
    let start_time = now - (num_candles as i64 * interval);

    let mut price = 100.0;
    let mut rng_state: u64 = 12345;

    for i in 0..num_candles {
        // Simple pseudo-random number generator
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let random = ((rng_state >> 16) & 0x7FFF) as f64 / 32767.0;

        // Add some trend and volatility
        let trend = (i as f64 * 0.01).sin() * 5.0;
        let volatility = 2.0 + (i as f64 * 0.05).sin().abs() * 3.0;

        let change = (random - 0.5) * volatility + trend * 0.1;
        let open = price;
        price += change;
        let close = price;

        let high = open.max(close) + random * volatility * 0.5;
        let low = open.min(close) - random * volatility * 0.5;

        let volume = 1000.0 + random * 5000.0 + (i as f64 * 0.02).sin().abs() * 3000.0;

        let timestamp = Timestamp(start_time + (i as i64 * interval));

        series.push(Candle::new(timestamp, open, high, low, close, volume));
    }

    series
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_basics() {
        let mut series = CandleSeries::<Timestamp>::new();
        assert!(series.is_empty());

        series.push(Candle::new(Timestamp(1000), 100.0, 110.0, 95.0, 105.0, 1000.0));
        series.push(Candle::new(Timestamp(2000), 105.0, 115.0, 100.0, 110.0, 1500.0));

        assert_eq!(series.len(), 2);
        assert!(!series.is_empty());
    }

    #[test]
    fn test_series_ranges() {
        let mut series = CandleSeries::<Timestamp>::new();
        series.push(Candle::new(Timestamp(1000), 100.0, 110.0, 95.0, 105.0, 1000.0));
        series.push(Candle::new(Timestamp(2000), 105.0, 120.0, 90.0, 115.0, 2000.0));

        let (low, high) = series.price_range().unwrap();
        assert_eq!(low, 90.0);
        assert_eq!(high, 120.0);

        let (_, vol_max) = series.volume_range().unwrap();
        assert_eq!(vol_max, 2000.0);
    }

    #[test]
    fn test_generate_sample_data() {
        let series = generate_sample_data(100, Timeframe::H1);
        assert_eq!(series.len(), 100);
        assert_eq!(series.timeframe(), Some(Timeframe::H1));
    }

    use crate::axis::Index;

    fn ts(t: i64, o: f64, h: f64, l: f64, c: f64, v: f64) -> Candle<Timestamp> {
        Candle::new(Timestamp(t), o, h, l, c, v)
    }

    fn populated() -> CandleSeries<Timestamp> {
        let mut s = CandleSeries::<Timestamp>::new();
        s.push(ts(1000, 10.0, 12.0, 9.0, 11.0, 100.0));
        s.push(ts(2000, 11.0, 14.0, 10.0, 13.0, 200.0));
        s.push(ts(3000, 13.0, 15.0, 12.0, 14.0, 150.0));
        s
    }

    #[test]
    fn test_default_is_empty() {
        let s: CandleSeries<Timestamp> = CandleSeries::default();
        assert!(s.is_empty());
        assert!(s.timeframe().is_none());
    }

    #[test]
    fn test_with_timeframe_sets_timeframe() {
        let s: CandleSeries<Timestamp> = CandleSeries::with_timeframe(Timeframe::M5);
        assert_eq!(s.timeframe(), Some(Timeframe::M5));
        assert!(s.is_empty());
    }

    #[test]
    fn test_from_candles_no_timeframe() {
        let candles = vec![ts(1000, 1.0, 2.0, 0.5, 1.5, 10.0)];
        let s = CandleSeries::from_candles(candles);
        assert_eq!(s.len(), 1);
        assert!(s.timeframe().is_none());
    }

    #[test]
    fn test_from_candles_with_timeframe_sets_both() {
        let candles = vec![ts(1000, 1.0, 2.0, 0.5, 1.5, 10.0)];
        let s = CandleSeries::from_candles_with_timeframe(candles, Timeframe::H4);
        assert_eq!(s.len(), 1);
        assert_eq!(s.timeframe(), Some(Timeframe::H4));
    }

    #[test]
    fn test_set_timeframe_overwrites() {
        let mut s: CandleSeries<Timestamp> = CandleSeries::new();
        s.set_timeframe(Timeframe::M1);
        assert_eq!(s.timeframe(), Some(Timeframe::M1));
        s.set_timeframe(Timeframe::H1);
        assert_eq!(s.timeframe(), Some(Timeframe::H1));
    }

    #[test]
    fn test_first_last_get_with_data() {
        let s = populated();
        assert_eq!(s.first().unwrap().x, Timestamp(1000));
        assert_eq!(s.last().unwrap().x, Timestamp(3000));
        assert_eq!(s.get(1).unwrap().x, Timestamp(2000));
        assert!(s.get(99).is_none());
    }

    #[test]
    fn test_first_last_empty() {
        let s: CandleSeries<Timestamp> = CandleSeries::new();
        assert!(s.first().is_none());
        assert!(s.last().is_none());
    }

    #[test]
    fn test_price_volume_x_range_empty_returns_none() {
        let s: CandleSeries<Timestamp> = CandleSeries::new();
        assert!(s.price_range().is_none());
        assert!(s.volume_range().is_none());
        assert!(s.x_range().is_none());
    }

    #[test]
    fn test_x_range_uses_first_and_last_plot_values() {
        let s = populated();
        let (a, b) = s.x_range().unwrap();
        assert!((a - 1000.0).abs() < 1e-9);
        assert!((b - 3000.0).abs() < 1e-9);
    }

    #[test]
    fn test_volume_range_lower_bound_is_zero() {
        let s = populated();
        let (lo, hi) = s.volume_range().unwrap();
        assert_eq!(lo, 0.0);
        assert!((hi - 200.0).abs() < 1e-9);
    }

    #[test]
    fn test_extracted_columns() {
        let s = populated();
        assert_eq!(s.opens(), vec![10.0, 11.0, 13.0]);
        assert_eq!(s.highs(), vec![12.0, 14.0, 15.0]);
        assert_eq!(s.lows(), vec![9.0, 10.0, 12.0]);
        assert_eq!(s.closes(), vec![11.0, 13.0, 14.0]);
        assert_eq!(s.volumes(), vec![100.0, 200.0, 150.0]);
        assert_eq!(s.x_values(), vec![1000.0, 2000.0, 3000.0]);
    }

    #[test]
    fn test_iter_yields_all_candles() {
        let s = populated();
        let xs: Vec<i64> = s.iter().map(|c| c.x.0).collect();
        assert_eq!(xs, vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_into_iter_by_ref() {
        let s = populated();
        let count = (&s).into_iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_into_iter_by_value_consumes() {
        let s = populated();
        let candles: Vec<Candle<Timestamp>> = s.into_iter().collect();
        assert_eq!(candles.len(), 3);
    }

    #[test]
    fn test_candles_mut_can_be_modified() {
        let mut s = populated();
        s.candles_mut().pop();
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn test_sort_by_x_orders_ascending() {
        let mut s: CandleSeries<Timestamp> = CandleSeries::new();
        s.push(ts(3000, 1.0, 2.0, 0.5, 1.5, 10.0));
        s.push(ts(1000, 1.0, 2.0, 0.5, 1.5, 10.0));
        s.push(ts(2000, 1.0, 2.0, 0.5, 1.5, 10.0));
        s.sort_by_x();
        let xs: Vec<i64> = s.iter().map(|c| c.x.0).collect();
        assert_eq!(xs, vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_x_spacing_uses_timeframe_when_set() {
        let mut s: CandleSeries<Timestamp> = CandleSeries::with_timeframe(Timeframe::M1);
        // Add candles whose actual spacing differs from the timeframe — timeframe wins
        s.push(ts(1000, 1.0, 2.0, 0.5, 1.5, 10.0));
        s.push(ts(99_999_000, 1.0, 2.0, 0.5, 1.5, 10.0));
        assert_eq!(s.x_spacing(), 60_000.0);
    }

    #[test]
    fn test_x_spacing_falls_back_to_first_two_candles() {
        let mut s: CandleSeries<Timestamp> = CandleSeries::new();
        s.push(ts(1000, 1.0, 2.0, 0.5, 1.5, 10.0));
        s.push(ts(4000, 1.0, 2.0, 0.5, 1.5, 10.0));
        assert!((s.x_spacing() - 3000.0).abs() < 1e-9);
    }

    #[test]
    fn test_x_spacing_uses_default_when_only_one_candle() {
        let mut s: CandleSeries<Timestamp> = CandleSeries::new();
        s.push(ts(1000, 1.0, 2.0, 0.5, 1.5, 10.0));
        // Only one candle and no timeframe → use the type's default spacing
        assert!((s.x_spacing() - Timestamp::default_spacing()).abs() < 1e-9);
    }

    #[test]
    fn test_x_spacing_empty_uses_default() {
        let s: CandleSeries<Timestamp> = CandleSeries::new();
        assert!((s.x_spacing() - Timestamp::default_spacing()).abs() < 1e-9);
    }

    #[test]
    fn test_map_x_converts_coordinate_type() {
        let s = populated();
        let mapped: CandleSeries<Index> =
            s.map_x(|t| Index((t.0 / 1000) as usize));
        let xs: Vec<usize> = mapped.iter().map(|c| c.x.0).collect();
        assert_eq!(xs, vec![1, 2, 3]);
    }

    // ---------- Timestamp-specific helpers ----------

    #[test]
    fn test_time_range_empty() {
        let s: CandleSeries<Timestamp> = CandleSeries::new();
        assert!(s.time_range().is_none());
    }

    #[test]
    fn test_time_range_first_and_last() {
        let s = populated();
        assert_eq!(s.time_range(), Some((1000, 3000)));
    }

    #[test]
    fn test_timestamps_returns_raw_millis() {
        let s = populated();
        assert_eq!(s.timestamps(), vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_sort_by_time_orders_by_timestamp() {
        let mut s: CandleSeries<Timestamp> = CandleSeries::new();
        s.push(ts(3000, 1.0, 2.0, 0.5, 1.5, 10.0));
        s.push(ts(1000, 1.0, 2.0, 0.5, 1.5, 10.0));
        s.sort_by_time();
        assert_eq!(s.timestamps(), vec![1000, 3000]);
    }

    #[test]
    fn test_generate_sample_data_zero_candles() {
        let s = generate_sample_data(0, Timeframe::M1);
        assert!(s.is_empty());
        assert_eq!(s.timeframe(), Some(Timeframe::M1));
    }
}
