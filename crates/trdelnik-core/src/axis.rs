//! Axis coordinate types for generic X-axis support
//!
//! This module provides the `AxisCoordinate` trait and built-in implementations
//! for timestamps, Solana slots, Ethereum block numbers, and sequential indices.

use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

/// Trait for types that can be used as X-axis coordinates in charts.
///
/// This enables charts to work with different coordinate systems:
/// - Timestamps (milliseconds since Unix epoch)
/// - Solana slots
/// - Ethereum block numbers
/// - Sequential indices
pub trait AxisCoordinate: Copy + Clone + PartialOrd + Display + Debug + Send + Sync + 'static {
    /// Convert to a plot value (f64) for rendering
    fn to_plot_value(&self) -> f64;

    /// Create from a plot value
    fn from_plot_value(value: f64) -> Self;

    /// Format as a label for display
    fn format_label(&self) -> String;

    /// Get the default spacing between coordinates (for candle width calculation)
    fn default_spacing() -> f64;
}

/// Unix timestamp in milliseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub i64);

impl Timestamp {
    /// Create a new timestamp from milliseconds since Unix epoch
    pub fn new(millis: i64) -> Self {
        Self(millis)
    }

    /// Get the raw milliseconds value
    pub fn as_millis(&self) -> i64 {
        self.0
    }

    /// Create from seconds since Unix epoch
    pub fn from_secs(secs: i64) -> Self {
        Self(secs * 1000)
    }

    /// Get as seconds since Unix epoch
    pub fn as_secs(&self) -> i64 {
        self.0 / 1000
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_label())
    }
}

impl AxisCoordinate for Timestamp {
    fn to_plot_value(&self) -> f64 {
        self.0 as f64
    }

    fn from_plot_value(value: f64) -> Self {
        Self(value as i64)
    }

    fn format_label(&self) -> String {
        format_timestamp_ms(self.0)
    }

    fn default_spacing() -> f64 {
        60000.0 // 1 minute in milliseconds
    }
}

impl From<i64> for Timestamp {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<Timestamp> for i64 {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self(0)
    }
}

/// Solana slot number
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Slot(pub u64);

impl Slot {
    /// Create a new slot
    pub fn new(slot: u64) -> Self {
        Self(slot)
    }

    /// Get the raw slot number
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot {}", self.0)
    }
}

impl AxisCoordinate for Slot {
    fn to_plot_value(&self) -> f64 {
        self.0 as f64
    }

    fn from_plot_value(value: f64) -> Self {
        Self(value.max(0.0) as u64)
    }

    fn format_label(&self) -> String {
        format!("#{}", self.0)
    }

    fn default_spacing() -> f64 {
        1.0 // 1 slot
    }
}

impl From<u64> for Slot {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Slot> for u64 {
    fn from(value: Slot) -> Self {
        value.0
    }
}

/// Ethereum block number
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BlockNumber(pub u64);

impl BlockNumber {
    /// Create a new block number
    pub fn new(block: u64) -> Self {
        Self(block)
    }

    /// Get the raw block number
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Display for BlockNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block {}", self.0)
    }
}

impl AxisCoordinate for BlockNumber {
    fn to_plot_value(&self) -> f64 {
        self.0 as f64
    }

    fn from_plot_value(value: f64) -> Self {
        Self(value.max(0.0) as u64)
    }

    fn format_label(&self) -> String {
        format!("#{}", self.0)
    }

    fn default_spacing() -> f64 {
        1.0 // 1 block
    }
}

impl From<u64> for BlockNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<BlockNumber> for u64 {
    fn from(value: BlockNumber) -> Self {
        value.0
    }
}

/// Sequential index (for data without natural X coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Index(pub usize);

impl Index {
    /// Create a new index
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    /// Get the raw index value
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AxisCoordinate for Index {
    fn to_plot_value(&self) -> f64 {
        self.0 as f64
    }

    fn from_plot_value(value: f64) -> Self {
        Self(value.max(0.0) as usize)
    }

    fn format_label(&self) -> String {
        self.0.to_string()
    }

    fn default_spacing() -> f64 {
        1.0
    }
}

impl From<usize> for Index {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<Index> for usize {
    fn from(value: Index) -> Self {
        value.0
    }
}

/// Format a timestamp in milliseconds as a human-readable string
fn format_timestamp_ms(timestamp_ms: i64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let d = UNIX_EPOCH + Duration::from_millis(timestamp_ms as u64);
    let duration = d.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 0;
    for (i, &days) in days_in_month.iter().enumerate() {
        if remaining_days < days as u64 {
            month = i + 1;
            break;
        }
        remaining_days -= days as u64;
    }

    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        year, month, day, hours, minutes
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_conversion() {
        let ts = Timestamp::new(1700000000000);
        let plot_val = ts.to_plot_value();
        let back = Timestamp::from_plot_value(plot_val);
        assert_eq!(ts, back);
    }

    #[test]
    fn test_slot_conversion() {
        let slot = Slot::new(12345678);
        let plot_val = slot.to_plot_value();
        let back = Slot::from_plot_value(plot_val);
        assert_eq!(slot, back);
    }

    #[test]
    fn test_block_number_conversion() {
        let block = BlockNumber::new(18000000);
        let plot_val = block.to_plot_value();
        let back = BlockNumber::from_plot_value(plot_val);
        assert_eq!(block, back);
    }

    #[test]
    fn test_index_conversion() {
        let idx = Index::new(42);
        let plot_val = idx.to_plot_value();
        let back = Index::from_plot_value(plot_val);
        assert_eq!(idx, back);
    }
}
