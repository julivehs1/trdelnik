//! Trading signals

use crate::axis::{AxisCoordinate, Timestamp};
use serde::{Deserialize, Serialize};

/// Direction of a trading signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalDirection {
    /// Buy/Long signal
    Buy,
    /// Sell/Short signal
    Sell,
    /// Exit long position
    ExitLong,
    /// Exit short position
    ExitShort,
    /// Neutral/informational signal
    Neutral,
}

impl SignalDirection {
    /// Returns true if this is a buy-side signal
    pub fn is_bullish(&self) -> bool {
        matches!(self, SignalDirection::Buy | SignalDirection::ExitShort)
    }

    /// Returns true if this is a sell-side signal
    pub fn is_bearish(&self) -> bool {
        matches!(self, SignalDirection::Sell | SignalDirection::ExitLong)
    }
}

/// Strength of a trading signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalStrength {
    /// Weak signal
    Weak,
    /// Normal signal
    Normal,
    /// Strong signal
    Strong,
}

impl Default for SignalStrength {
    fn default() -> Self {
        Self::Normal
    }
}

/// A trading signal at a specific position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signal<X: AxisCoordinate = Timestamp> {
    /// X-axis coordinate where the signal occurred
    pub x: X,
    /// Price level of the signal
    pub price: f64,
    /// Direction of the signal
    pub direction: SignalDirection,
    /// Strength of the signal
    pub strength: SignalStrength,
    /// Optional label/annotation
    pub label: Option<String>,
    /// Source/origin of the signal (e.g., "RSI", "MACD crossover")
    pub source: String,
}

impl<X: AxisCoordinate> Signal<X> {
    /// Create a new signal
    pub fn new(x: X, price: f64, direction: SignalDirection, source: impl Into<String>) -> Self {
        Self {
            x,
            price,
            direction,
            strength: SignalStrength::Normal,
            label: None,
            source: source.into(),
        }
    }

    /// Set the signal strength
    pub fn with_strength(mut self, strength: SignalStrength) -> Self {
        self.strength = strength;
        self
    }

    /// Set the signal label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Get the X coordinate as a plot value
    pub fn x_plot_value(&self) -> f64 {
        self.x.to_plot_value()
    }

    /// Map the signal to a different X coordinate type
    pub fn map_x<Y: AxisCoordinate>(self, f: impl FnOnce(X) -> Y) -> Signal<Y> {
        Signal {
            x: f(self.x),
            price: self.price,
            direction: self.direction,
            strength: self.strength,
            label: self.label,
            source: self.source,
        }
    }
}

/// A collection of signals
#[derive(Debug, Clone, Default)]
pub struct SignalSeries<X: AxisCoordinate = Timestamp> {
    signals: Vec<Signal<X>>,
}

impl<X: AxisCoordinate> SignalSeries<X> {
    /// Create a new empty signal series
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
        }
    }

    /// Create from a vector of signals
    pub fn from_signals(signals: Vec<Signal<X>>) -> Self {
        Self { signals }
    }

    /// Add a signal
    pub fn push(&mut self, signal: Signal<X>) {
        self.signals.push(signal);
    }

    /// Get all signals
    pub fn signals(&self) -> &[Signal<X>] {
        &self.signals
    }

    /// Get mutable signals
    pub fn signals_mut(&mut self) -> &mut Vec<Signal<X>> {
        &mut self.signals
    }

    /// Get the number of signals
    pub fn len(&self) -> usize {
        self.signals.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }

    /// Filter signals by direction
    pub fn filter_direction(&self, direction: SignalDirection) -> impl Iterator<Item = &Signal<X>> {
        self.signals.iter().filter(move |s| s.direction == direction)
    }

    /// Filter signals by source
    pub fn filter_source<'a>(&'a self, source: &'a str) -> impl Iterator<Item = &'a Signal<X>> + 'a {
        self.signals.iter().filter(move |s| s.source == source)
    }

    /// Get signals within an X range (as plot values)
    pub fn in_range(&self, x_min: f64, x_max: f64) -> impl Iterator<Item = &Signal<X>> {
        self.signals.iter().filter(move |s| {
            let x = s.x.to_plot_value();
            x >= x_min && x <= x_max
        })
    }

    /// Sort signals by X coordinate
    pub fn sort_by_x(&mut self) {
        self.signals
            .sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Map the series to a different X coordinate type
    pub fn map_x<Y: AxisCoordinate>(self, f: impl Fn(X) -> Y) -> SignalSeries<Y> {
        SignalSeries {
            signals: self.signals.into_iter().map(|s| s.map_x(&f)).collect(),
        }
    }

    /// Iterator over signals
    pub fn iter(&self) -> impl Iterator<Item = &Signal<X>> {
        self.signals.iter()
    }
}

impl<'a, X: AxisCoordinate> IntoIterator for &'a SignalSeries<X> {
    type Item = &'a Signal<X>;
    type IntoIter = std::slice::Iter<'a, Signal<X>>;

    fn into_iter(self) -> Self::IntoIter {
        self.signals.iter()
    }
}

impl<X: AxisCoordinate> IntoIterator for SignalSeries<X> {
    type Item = Signal<X>;
    type IntoIter = std::vec::IntoIter<Signal<X>>;

    fn into_iter(self) -> Self::IntoIter {
        self.signals.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let signal = Signal::new(Timestamp(1000), 100.0, SignalDirection::Buy, "test")
            .with_strength(SignalStrength::Strong)
            .with_label("Entry");

        assert_eq!(signal.x, Timestamp(1000));
        assert_eq!(signal.price, 100.0);
        assert_eq!(signal.direction, SignalDirection::Buy);
        assert_eq!(signal.strength, SignalStrength::Strong);
        assert_eq!(signal.label, Some("Entry".to_string()));
    }

    #[test]
    fn test_signal_direction() {
        assert!(SignalDirection::Buy.is_bullish());
        assert!(!SignalDirection::Buy.is_bearish());
        assert!(SignalDirection::Sell.is_bearish());
        assert!(!SignalDirection::Sell.is_bullish());
    }

    #[test]
    fn test_signal_series() {
        let mut series = SignalSeries::<Timestamp>::new();
        series.push(Signal::new(Timestamp(1000), 100.0, SignalDirection::Buy, "RSI"));
        series.push(Signal::new(Timestamp(2000), 105.0, SignalDirection::Sell, "MACD"));

        assert_eq!(series.len(), 2);
        assert_eq!(series.filter_direction(SignalDirection::Buy).count(), 1);
        assert_eq!(series.filter_source("RSI").count(), 1);
    }

    use crate::axis::Index;

    // ---------- SignalDirection ----------

    #[test]
    fn test_signal_direction_exit_short_is_bullish() {
        assert!(SignalDirection::ExitShort.is_bullish());
        assert!(!SignalDirection::ExitShort.is_bearish());
    }

    #[test]
    fn test_signal_direction_exit_long_is_bearish() {
        assert!(SignalDirection::ExitLong.is_bearish());
        assert!(!SignalDirection::ExitLong.is_bullish());
    }

    #[test]
    fn test_signal_direction_neutral() {
        assert!(!SignalDirection::Neutral.is_bullish());
        assert!(!SignalDirection::Neutral.is_bearish());
    }

    // ---------- SignalStrength ----------

    #[test]
    fn test_signal_strength_default_is_normal() {
        assert_eq!(SignalStrength::default(), SignalStrength::Normal);
    }

    #[test]
    fn test_signal_strength_variants_distinct() {
        assert_ne!(SignalStrength::Weak, SignalStrength::Normal);
        assert_ne!(SignalStrength::Normal, SignalStrength::Strong);
    }

    // ---------- Signal ----------

    #[test]
    fn test_signal_new_defaults() {
        let s = Signal::new(Timestamp(100), 50.0, SignalDirection::Sell, "RSI");
        assert_eq!(s.strength, SignalStrength::Normal);
        assert!(s.label.is_none());
        assert_eq!(s.source, "RSI");
    }

    #[test]
    fn test_signal_x_plot_value() {
        let s = Signal::new(Timestamp(1234), 1.0, SignalDirection::Buy, "x");
        assert!((s.x_plot_value() - 1234.0).abs() < 1e-9);
    }

    #[test]
    fn test_signal_map_x_changes_coordinate_type() {
        let s: Signal<Timestamp> =
            Signal::new(Timestamp(60_000), 10.0, SignalDirection::Buy, "src")
                .with_label("L")
                .with_strength(SignalStrength::Weak);
        let mapped: Signal<Index> = s.map_x(|t| Index((t.0 / 1000) as usize));
        assert_eq!(mapped.x, Index(60));
        assert!((mapped.price - 10.0).abs() < 1e-9);
        assert_eq!(mapped.direction, SignalDirection::Buy);
        assert_eq!(mapped.strength, SignalStrength::Weak);
        assert_eq!(mapped.label.as_deref(), Some("L"));
        assert_eq!(mapped.source, "src");
    }

    // ---------- SignalSeries ----------

    #[test]
    fn test_default_is_empty() {
        let s: SignalSeries<Timestamp> = SignalSeries::default();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_from_signals() {
        let signals = vec![
            Signal::new(Timestamp(1), 1.0, SignalDirection::Buy, "a"),
            Signal::new(Timestamp(2), 2.0, SignalDirection::Sell, "b"),
        ];
        let s = SignalSeries::from_signals(signals);
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn test_signals_accessor_and_mut() {
        let mut s: SignalSeries<Timestamp> = SignalSeries::new();
        s.push(Signal::new(Timestamp(1), 1.0, SignalDirection::Buy, "a"));
        assert_eq!(s.signals().len(), 1);
        s.signals_mut().pop();
        assert!(s.is_empty());
    }

    fn series_with_signals() -> SignalSeries<Timestamp> {
        let mut s: SignalSeries<Timestamp> = SignalSeries::new();
        s.push(Signal::new(Timestamp(1000), 100.0, SignalDirection::Buy, "RSI"));
        s.push(Signal::new(Timestamp(2000), 110.0, SignalDirection::Sell, "MACD"));
        s.push(Signal::new(Timestamp(3000), 120.0, SignalDirection::Buy, "MACD"));
        s
    }

    #[test]
    fn test_filter_direction_zero_match() {
        let s = series_with_signals();
        assert_eq!(s.filter_direction(SignalDirection::Neutral).count(), 0);
    }

    #[test]
    fn test_filter_direction_multiple_matches() {
        let s = series_with_signals();
        assert_eq!(s.filter_direction(SignalDirection::Buy).count(), 2);
    }

    #[test]
    fn test_filter_source() {
        let s = series_with_signals();
        assert_eq!(s.filter_source("MACD").count(), 2);
        assert_eq!(s.filter_source("MISSING").count(), 0);
    }

    #[test]
    fn test_in_range_inclusive_bounds() {
        let s = series_with_signals();
        let count = s.in_range(1000.0, 2500.0).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_in_range_excludes_out_of_window() {
        let s = series_with_signals();
        let count = s.in_range(2500.0, 4000.0).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sort_by_x_orders_ascending() {
        let mut s: SignalSeries<Timestamp> = SignalSeries::new();
        s.push(Signal::new(Timestamp(3000), 0.0, SignalDirection::Buy, "x"));
        s.push(Signal::new(Timestamp(1000), 0.0, SignalDirection::Buy, "x"));
        s.push(Signal::new(Timestamp(2000), 0.0, SignalDirection::Buy, "x"));
        s.sort_by_x();
        let xs: Vec<i64> = s.iter().map(|sig| sig.x.0).collect();
        assert_eq!(xs, vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_map_x_converts_all_signals() {
        let s = series_with_signals();
        let mapped: SignalSeries<Index> =
            s.map_x(|t| Index((t.0 / 1000) as usize));
        let xs: Vec<usize> = mapped.iter().map(|sig| sig.x.0).collect();
        assert_eq!(xs, vec![1, 2, 3]);
    }

    #[test]
    fn test_into_iter_by_ref() {
        let s = series_with_signals();
        assert_eq!((&s).into_iter().count(), 3);
    }

    #[test]
    fn test_into_iter_by_value_consumes() {
        let s = series_with_signals();
        let v: Vec<Signal<Timestamp>> = s.into_iter().collect();
        assert_eq!(v.len(), 3);
    }
}
