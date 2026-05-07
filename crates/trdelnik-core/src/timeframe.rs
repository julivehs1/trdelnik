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

    #[test]
    fn test_timeframe_durations_for_all_variants() {
        let cases = [
            (Timeframe::S1, 1u64),
            (Timeframe::S5, 5),
            (Timeframe::S15, 15),
            (Timeframe::S30, 30),
            (Timeframe::M1, 60),
            (Timeframe::M3, 180),
            (Timeframe::M5, 300),
            (Timeframe::M15, 900),
            (Timeframe::M30, 1_800),
            (Timeframe::H1, 3_600),
            (Timeframe::H2, 7_200),
            (Timeframe::H4, 14_400),
            (Timeframe::H6, 21_600),
            (Timeframe::H12, 43_200),
            (Timeframe::D1, 86_400),
            (Timeframe::W1, 604_800),
            (Timeframe::Mo1, 2_592_000),
        ];
        for (tf, secs) in cases {
            assert_eq!(tf.duration().as_secs(), secs, "{:?}", tf);
            assert_eq!(tf.as_millis() as u64, secs * 1000, "{:?}", tf);
        }
    }

    #[test]
    fn test_timeframe_label_for_all_variants() {
        assert_eq!(Timeframe::S1.label(), "1s");
        assert_eq!(Timeframe::S5.label(), "5s");
        assert_eq!(Timeframe::S15.label(), "15s");
        assert_eq!(Timeframe::S30.label(), "30s");
        assert_eq!(Timeframe::M1.label(), "1m");
        assert_eq!(Timeframe::M3.label(), "3m");
        assert_eq!(Timeframe::M5.label(), "5m");
        assert_eq!(Timeframe::M15.label(), "15m");
        assert_eq!(Timeframe::M30.label(), "30m");
        assert_eq!(Timeframe::H1.label(), "1h");
        assert_eq!(Timeframe::H2.label(), "2h");
        assert_eq!(Timeframe::H4.label(), "4h");
        assert_eq!(Timeframe::H6.label(), "6h");
        assert_eq!(Timeframe::H12.label(), "12h");
        assert_eq!(Timeframe::D1.label(), "1D");
        assert_eq!(Timeframe::W1.label(), "1W");
        assert_eq!(Timeframe::Mo1.label(), "1M");
    }

    #[test]
    fn test_timeframe_all_returns_every_variant_once() {
        let all = Timeframe::all();
        assert_eq!(all.len(), 17);
        // Ensure no duplicates
        for (i, &a) in all.iter().enumerate() {
            for &b in &all[i + 1..] {
                assert_ne!(a, b, "duplicate variant in all(): {:?}", a);
            }
        }
    }

    #[test]
    fn test_timeframe_default_is_h1() {
        assert_eq!(Timeframe::default(), Timeframe::H1);
    }

    #[test]
    fn test_timeframe_display_matches_label() {
        for &tf in Timeframe::all() {
            assert_eq!(format!("{}", tf), tf.label());
        }
    }

    #[test]
    fn test_timeframe_durations_strictly_increase_in_all() {
        // The order in `all()` should be ascending by duration
        let all = Timeframe::all();
        for w in all.windows(2) {
            assert!(
                w[0].duration() < w[1].duration(),
                "{:?} not < {:?}",
                w[0],
                w[1]
            );
        }
    }
}
