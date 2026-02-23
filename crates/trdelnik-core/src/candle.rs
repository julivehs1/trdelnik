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
}
