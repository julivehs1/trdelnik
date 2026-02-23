//! Timeframe definitions for candle charts

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Timeframe for candles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timeframe {
    /// 1 second
    S1,
    /// 5 seconds
    S5,
    /// 15 seconds
    S15,
    /// 30 seconds
    S30,
    /// 1 minute
    M1,
    /// 3 minutes
    M3,
    /// 5 minutes
    M5,
    /// 15 minutes
    M15,
    /// 30 minutes
    M30,
    /// 1 hour
    H1,
    /// 2 hours
    H2,
    /// 4 hours
    H4,
    /// 6 hours
    H6,
    /// 12 hours
    H12,
    /// 1 day
    D1,
    /// 1 week
    W1,
    /// 1 month
    Mo1,
}

impl Timeframe {
    /// Returns the duration of this timeframe
    pub fn duration(&self) -> Duration {
        match self {
            Timeframe::S1 => Duration::from_secs(1),
            Timeframe::S5 => Duration::from_secs(5),
            Timeframe::S15 => Duration::from_secs(15),
            Timeframe::S30 => Duration::from_secs(30),
            Timeframe::M1 => Duration::from_secs(60),
            Timeframe::M3 => Duration::from_secs(180),
            Timeframe::M5 => Duration::from_secs(300),
            Timeframe::M15 => Duration::from_secs(900),
            Timeframe::M30 => Duration::from_secs(1800),
            Timeframe::H1 => Duration::from_secs(3600),
            Timeframe::H2 => Duration::from_secs(7200),
            Timeframe::H4 => Duration::from_secs(14400),
            Timeframe::H6 => Duration::from_secs(21600),
            Timeframe::H12 => Duration::from_secs(43200),
            Timeframe::D1 => Duration::from_secs(86400),
            Timeframe::W1 => Duration::from_secs(604800),
            Timeframe::Mo1 => Duration::from_secs(2592000), // 30 days approximation
        }
    }

    /// Returns the timeframe as milliseconds
    pub fn as_millis(&self) -> i64 {
        self.duration().as_millis() as i64
    }

    /// Returns a human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Timeframe::S1 => "1s",
            Timeframe::S5 => "5s",
            Timeframe::S15 => "15s",
            Timeframe::S30 => "30s",
            Timeframe::M1 => "1m",
            Timeframe::M3 => "3m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::M30 => "30m",
            Timeframe::H1 => "1h",
            Timeframe::H2 => "2h",
            Timeframe::H4 => "4h",
            Timeframe::H6 => "6h",
            Timeframe::H12 => "12h",
            Timeframe::D1 => "1D",
            Timeframe::W1 => "1W",
            Timeframe::Mo1 => "1M",
        }
    }

    /// Get all available timeframes
    pub fn all() -> &'static [Timeframe] {
        &[
            Timeframe::S1,
            Timeframe::S5,
            Timeframe::S15,
            Timeframe::S30,
            Timeframe::M1,
            Timeframe::M3,
            Timeframe::M5,
            Timeframe::M15,
            Timeframe::M30,
            Timeframe::H1,
            Timeframe::H2,
            Timeframe::H4,
            Timeframe::H6,
            Timeframe::H12,
            Timeframe::D1,
            Timeframe::W1,
            Timeframe::Mo1,
        ]
    }
}

impl Default for Timeframe {
    fn default() -> Self {
        Self::H1
    }
}

impl std::fmt::Display for Timeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeframe_duration() {
        assert_eq!(Timeframe::M1.duration().as_secs(), 60);
        assert_eq!(Timeframe::H1.duration().as_secs(), 3600);
        assert_eq!(Timeframe::D1.duration().as_secs(), 86400);
    }

    #[test]
    fn test_timeframe_as_millis() {
        assert_eq!(Timeframe::M1.as_millis(), 60000);
        assert_eq!(Timeframe::H1.as_millis(), 3600000);
    }
}
