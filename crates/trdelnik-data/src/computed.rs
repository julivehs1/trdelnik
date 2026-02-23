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
}
