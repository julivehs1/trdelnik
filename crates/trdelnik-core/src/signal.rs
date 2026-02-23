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
}
