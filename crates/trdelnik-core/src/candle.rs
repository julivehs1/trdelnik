//! Candle (OHLCV) data structures

use crate::axis::{AxisCoordinate, Timestamp};
use serde::{Deserialize, Serialize};

/// A single OHLCV (Open, High, Low, Close, Volume) candle with a generic X coordinate
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Candle<X: AxisCoordinate = Timestamp> {
    /// X-axis coordinate (timestamp, slot, block number, etc.)
    pub x: X,
    /// Opening price
    pub open: f64,
    /// Highest price
    pub high: f64,
    /// Lowest price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Trading volume
    pub volume: f64,
}

impl<X: AxisCoordinate> Candle<X> {
    /// Create a new candle
    pub fn new(x: X, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            x,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Returns true if this is a bullish (green) candle
    #[inline]
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }

    /// Returns true if this is a bearish (red) candle
    #[inline]
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Returns the body size (absolute difference between open and close)
    #[inline]
    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Returns the full range (high - low)
    #[inline]
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Returns the upper wick size
    #[inline]
    pub fn upper_wick(&self) -> f64 {
        self.high - self.open.max(self.close)
    }

    /// Returns the lower wick size
    #[inline]
    pub fn lower_wick(&self) -> f64 {
        self.open.min(self.close) - self.low
    }

    /// Get the X coordinate as a plot value
    #[inline]
    pub fn x_plot_value(&self) -> f64 {
        self.x.to_plot_value()
    }

    /// Map the candle to a different X coordinate type
    pub fn map_x<Y: AxisCoordinate>(self, f: impl FnOnce(X) -> Y) -> Candle<Y> {
        Candle {
            x: f(self.x),
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
        }
    }
}

/// Type alias for timestamp-based candles (most common use case)
pub type TimestampCandle = Candle<Timestamp>;

impl Candle<Timestamp> {
    /// Create a new candle from a Unix timestamp in milliseconds
    pub fn from_timestamp(
        timestamp_ms: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        Self::new(Timestamp(timestamp_ms), open, high, low, close, volume)
    }

    /// Get the timestamp as milliseconds
    pub fn timestamp(&self) -> i64 {
        self.x.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::Slot;

    #[test]
    fn test_candle_bullish_bearish() {
        let bullish = Candle::new(Timestamp(0), 100.0, 110.0, 95.0, 105.0, 1000.0);
        assert!(bullish.is_bullish());
        assert!(!bullish.is_bearish());

        let bearish = Candle::new(Timestamp(0), 105.0, 110.0, 95.0, 100.0, 1000.0);
        assert!(!bearish.is_bullish());
        assert!(bearish.is_bearish());
    }

    #[test]
    fn test_candle_metrics() {
        let candle = Candle::new(Timestamp(0), 100.0, 120.0, 90.0, 110.0, 1000.0);
        assert_eq!(candle.body_size(), 10.0);
        assert_eq!(candle.range(), 30.0);
        assert_eq!(candle.upper_wick(), 10.0);
        assert_eq!(candle.lower_wick(), 10.0);
    }

    #[test]
    fn test_candle_map_x() {
        let ts_candle = Candle::new(Timestamp(1000), 100.0, 110.0, 95.0, 105.0, 1000.0);
        let slot_candle = ts_candle.map_x(|ts| Slot::new(ts.0 as u64 / 400));

        assert_eq!(slot_candle.x, Slot::new(2));
        assert_eq!(slot_candle.open, 100.0);
    }

    #[test]
    fn test_x_plot_value_for_timestamp() {
        let c = Candle::new(Timestamp(123_456), 1.0, 2.0, 0.5, 1.5, 100.0);
        assert!((c.x_plot_value() - 123_456.0).abs() < 1e-9);
    }

    #[test]
    fn test_from_timestamp_constructs_timestamp_candle() {
        let c = Candle::from_timestamp(60_000, 100.0, 105.0, 99.0, 102.0, 1_000.0);
        assert_eq!(c.x, Timestamp(60_000));
        assert_eq!(c.open, 100.0);
        assert_eq!(c.high, 105.0);
        assert_eq!(c.low, 99.0);
        assert_eq!(c.close, 102.0);
        assert_eq!(c.volume, 1_000.0);
    }

    #[test]
    fn test_timestamp_helper_returns_raw_millis() {
        let c = Candle::from_timestamp(99_999, 1.0, 2.0, 0.5, 1.5, 0.0);
        assert_eq!(c.timestamp(), 99_999);
    }

    #[test]
    fn test_timestamp_candle_type_alias_compiles() {
        let c: TimestampCandle = Candle::from_timestamp(0, 1.0, 2.0, 0.0, 1.0, 0.0);
        assert!(c.is_bullish());
    }

    #[test]
    fn test_doji_is_bullish() {
        // close == open is treated as bullish (>= comparison)
        let c = Candle::new(Timestamp(0), 100.0, 101.0, 99.0, 100.0, 1.0);
        assert!(c.is_bullish());
        assert!(!c.is_bearish());
        assert_eq!(c.body_size(), 0.0);
    }

    #[test]
    fn test_wicks_for_bearish_candle() {
        // open=110, close=100, high=115, low=98 → upper wick = 5, lower wick = 2
        let c = Candle::new(Timestamp(0), 110.0, 115.0, 98.0, 100.0, 1.0);
        assert!((c.upper_wick() - 5.0).abs() < 1e-9);
        assert!((c.lower_wick() - 2.0).abs() < 1e-9);
    }
}
