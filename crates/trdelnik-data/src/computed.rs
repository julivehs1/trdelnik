//! Computed indicator values with caching

use std::any::Any;
use std::collections::HashMap;
use std::fmt;

/// Error type for cache operations
#[derive(Debug, Clone)]
pub enum CacheError {
    /// The cached value has a different type than requested
    TypeMismatch {
        /// The key that had the type mismatch
        key: IndicatorKey,
        /// The expected type name
        expected: &'static str,
    },
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::TypeMismatch { key, expected } => {
                write!(
                    f,
                    "Type mismatch in cache for key '{}:{}': expected {}",
                    key.indicator_type, key.params, expected
                )
            }
        }
    }
}

impl std::error::Error for CacheError {}

/// Key for identifying cached indicator values
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IndicatorKey {
    /// Type of indicator (e.g., "sma", "ema", "rsi")
    pub indicator_type: String,
    /// Unique identifier (e.g., period, or combined parameters)
    pub params: String,
}

impl IndicatorKey {
    /// Create a new indicator key
    pub fn new(indicator_type: impl Into<String>, params: impl Into<String>) -> Self {
        Self {
            indicator_type: indicator_type.into(),
            params: params.into(),
        }
    }

    /// Create a key for SMA
    pub fn sma(period: usize) -> Self {
        Self::new("sma", period.to_string())
    }

    /// Create a key for EMA
    pub fn ema(period: usize) -> Self {
        Self::new("ema", period.to_string())
    }

    /// Create a key for RSI
    pub fn rsi(period: usize) -> Self {
        Self::new("rsi", period.to_string())
    }

    /// Create a key for MACD
    pub fn macd(fast: usize, slow: usize, signal: usize) -> Self {
        Self::new("macd", format!("{}_{}_{}", fast, slow, signal))
    }

    /// Create a key for Bollinger Bands
    pub fn bollinger(period: usize, std_dev: f64) -> Self {
        Self::new("bollinger", format!("{}_{}", period, std_dev))
    }
}

/// Cache for computed indicator values
#[derive(Debug, Default)]
pub struct ComputedIndicators {
    cache: HashMap<IndicatorKey, Box<dyn Any + Send + Sync>>,
    /// Version number that increments when underlying data changes
    version: u64,
}

impl ComputedIndicators {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Invalidate the cache (increment version and clear)
    pub fn invalidate(&mut self) {
        self.version += 1;
        self.cache.clear();
    }

    /// Check if the cache contains a specific indicator
    pub fn contains(&self, key: &IndicatorKey) -> bool {
        self.cache.contains_key(key)
    }

    /// Get a cached value
    pub fn get<T: 'static + Clone + Send + Sync>(&self, key: &IndicatorKey) -> Option<&Vec<Option<T>>> {
        self.cache
            .get(key)
            .and_then(|v| v.downcast_ref::<Vec<Option<T>>>())
    }

    /// Insert a value into the cache
    pub fn insert<T: 'static + Clone + Send + Sync>(&mut self, key: IndicatorKey, values: Vec<Option<T>>) {
        self.cache.insert(key, Box::new(values));
    }

    /// Remove a value from the cache
    pub fn remove(&mut self, key: &IndicatorKey) -> bool {
        self.cache.remove(key).is_some()
    }

    /// Get or compute a value
    ///
    /// Returns a reference to the cached value, computing it if not present.
    /// Returns an error if the cached value has a different type than requested.
    pub fn get_or_insert_with<T, F>(
        &mut self,
        key: IndicatorKey,
        compute: F,
    ) -> Result<&Vec<Option<T>>, CacheError>
    where
        T: 'static + Clone + Send + Sync,
        F: FnOnce() -> Vec<Option<T>>,
    {
        if !self.cache.contains_key(&key) {
            let values = compute();
            self.cache.insert(key.clone(), Box::new(values));
        }
        self.cache
            .get(&key)
            .and_then(|v| v.downcast_ref::<Vec<Option<T>>>())
            .ok_or_else(|| CacheError::TypeMismatch {
                key,
                expected: std::any::type_name::<T>(),
            })
    }

    /// Clear all cached values
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get the number of cached indicators
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator_key() {
        let key1 = IndicatorKey::sma(20);
        let key2 = IndicatorKey::sma(20);
        let key3 = IndicatorKey::sma(50);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_insert_get() {
        let mut cache = ComputedIndicators::new();
        let values: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), None, Some(4.0)];

        cache.insert(IndicatorKey::sma(20), values.clone());

        let retrieved = cache.get::<f64>(&IndicatorKey::sma(20)).unwrap();
        assert_eq!(retrieved, &values);
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = ComputedIndicators::new();
        cache.insert(IndicatorKey::sma(20), vec![Some(1.0)]);

        assert_eq!(cache.version(), 0);
        cache.invalidate();
        assert_eq!(cache.version(), 1);
        assert!(cache.is_empty());
    }

    // ---------- IndicatorKey constructors ----------

    #[test]
    fn test_key_ema() {
        let k = IndicatorKey::ema(50);
        assert_eq!(k.indicator_type, "ema");
        assert_eq!(k.params, "50");
    }

    #[test]
    fn test_key_rsi() {
        let k = IndicatorKey::rsi(14);
        assert_eq!(k.indicator_type, "rsi");
        assert_eq!(k.params, "14");
    }

    #[test]
    fn test_key_macd_combines_three_periods() {
        let k = IndicatorKey::macd(12, 26, 9);
        assert_eq!(k.indicator_type, "macd");
        assert_eq!(k.params, "12_26_9");
    }

    #[test]
    fn test_key_bollinger_combines_period_and_dev() {
        let k = IndicatorKey::bollinger(20, 2.0);
        assert_eq!(k.indicator_type, "bollinger");
        assert_eq!(k.params, "20_2");
    }

    #[test]
    fn test_key_new_with_custom_strings() {
        let k = IndicatorKey::new("custom", "p=10,m=2.0");
        assert_eq!(k.indicator_type, "custom");
        assert_eq!(k.params, "p=10,m=2.0");
    }

    #[test]
    fn test_keys_with_different_indicators_inequal() {
        let a = IndicatorKey::sma(20);
        let b = IndicatorKey::ema(20);
        assert_ne!(a, b);
    }

    // ---------- Cache management ----------

    #[test]
    fn test_default_is_empty() {
        let cache = ComputedIndicators::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.version(), 0);
    }

    #[test]
    fn test_contains_after_insert() {
        let mut cache = ComputedIndicators::new();
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(1.0)]);
        assert!(cache.contains(&IndicatorKey::sma(20)));
        assert!(!cache.contains(&IndicatorKey::sma(50)));
    }

    #[test]
    fn test_remove_returns_true_when_present() {
        let mut cache = ComputedIndicators::new();
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(1.0)]);
        assert!(cache.remove(&IndicatorKey::sma(20)));
        assert!(!cache.contains(&IndicatorKey::sma(20)));
    }

    #[test]
    fn test_remove_returns_false_when_missing() {
        let mut cache = ComputedIndicators::new();
        assert!(!cache.remove(&IndicatorKey::sma(20)));
    }

    #[test]
    fn test_clear_empties_cache_without_bumping_version() {
        let mut cache = ComputedIndicators::new();
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(1.0)]);
        let v0 = cache.version();
        cache.clear();
        assert!(cache.is_empty());
        // clear() does not change the version
        assert_eq!(cache.version(), v0);
    }

    #[test]
    fn test_len_counts_entries() {
        let mut cache = ComputedIndicators::new();
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(1.0)]);
        cache.insert::<f64>(IndicatorKey::ema(50), vec![Some(2.0)]);
        assert_eq!(cache.len(), 2);
    }

    // ---------- get_or_insert_with ----------

    #[test]
    fn test_get_or_insert_with_computes_on_miss() {
        let mut cache = ComputedIndicators::new();
        let mut called = 0;
        let result = cache
            .get_or_insert_with::<f64, _>(IndicatorKey::sma(20), || {
                called += 1;
                vec![Some(1.0), Some(2.0)]
            })
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(called, 1);
    }

    #[test]
    fn test_get_or_insert_with_skips_compute_on_hit() {
        let mut cache = ComputedIndicators::new();
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(99.0)]);
        let mut called = 0;
        let result = cache
            .get_or_insert_with::<f64, _>(IndicatorKey::sma(20), || {
                called += 1;
                vec![Some(0.0)]
            })
            .unwrap();
        assert_eq!(result, &vec![Some(99.0)]);
        assert_eq!(called, 0);
    }

    #[test]
    fn test_get_or_insert_with_type_mismatch_returns_error() {
        let mut cache = ComputedIndicators::new();
        // Insert as Vec<Option<f64>>
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(1.0)]);
        // Try to read as Vec<Option<i64>> via get_or_insert_with
        let err = cache
            .get_or_insert_with::<i64, _>(IndicatorKey::sma(20), || vec![Some(1)])
            .unwrap_err();
        match err {
            CacheError::TypeMismatch { key, expected } => {
                assert_eq!(key.indicator_type, "sma");
                assert!(expected.contains("i64"));
            }
        }
    }

    // ---------- get with type mismatch ----------

    #[test]
    fn test_get_returns_none_on_type_mismatch() {
        let mut cache = ComputedIndicators::new();
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(1.0)]);
        let result: Option<&Vec<Option<i64>>> = cache.get::<i64>(&IndicatorKey::sma(20));
        assert!(result.is_none());
    }

    #[test]
    fn test_get_returns_none_for_missing_key() {
        let cache = ComputedIndicators::new();
        let result: Option<&Vec<Option<f64>>> = cache.get(&IndicatorKey::sma(20));
        assert!(result.is_none());
    }

    // ---------- CacheError display ----------

    #[test]
    fn test_cache_error_display_includes_key_and_type() {
        let err = CacheError::TypeMismatch {
            key: IndicatorKey::sma(20),
            expected: "i64",
        };
        let msg = format!("{}", err);
        assert!(msg.contains("sma"));
        assert!(msg.contains("20"));
        assert!(msg.contains("i64"));
    }

    #[test]
    fn test_cache_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(CacheError::TypeMismatch {
            key: IndicatorKey::sma(20),
            expected: "f64",
        });
        // Must be possible to format and inspect via the Error trait
        assert!(format!("{}", err).contains("Type mismatch"));
    }

    // ---------- Version semantics ----------

    #[test]
    fn test_invalidate_clears_and_increments_version_repeatedly() {
        let mut cache = ComputedIndicators::new();
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(1.0)]);
        cache.invalidate();
        cache.insert::<f64>(IndicatorKey::sma(20), vec![Some(2.0)]);
        cache.invalidate();
        assert_eq!(cache.version(), 2);
        assert!(cache.is_empty());
    }
}
