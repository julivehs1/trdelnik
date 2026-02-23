//! Core indicator trait and input types.

// ============================================================================
// Input Types
// ============================================================================

/// OHLC (Open-High-Low-Close) candle data input for indicators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ohlc {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Ohlc {
    /// Create a new OHLC input.
    pub fn new(high: f64, low: f64, close: f64) -> Self {
        Self { high, low, close }
    }
}

/// OHLCV (Open-High-Low-Close-Volume) candle data input for indicators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ohlcv {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Ohlcv {
    /// Create a new OHLCV input.
    pub fn new(high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self { high, low, close, volume }
    }
}

// ============================================================================
// Indicator Trait
// ============================================================================

/// Unified trait for all stateful indicators.
///
/// Indicators process data incrementally and maintain internal state.
/// The `Input` type determines what kind of data the indicator needs:
/// - `f64` for price-based indicators (SMA, EMA, RSI, etc.)
/// - `Ohlc` for indicators needing high/low/close (ATR, Stochastic, etc.)
/// - `Ohlcv` for indicators also needing volume (OBV, MFI)
///
/// # CSE (Common Subexpression Elimination)
///
/// Indicators implement `Hash` and `Eq` on their parameters, enabling
/// automatic deduplication in computation graphs. Two indicators with
/// the same type and parameters will produce the same hash.
///
/// # Deriving with `#[indicator]`
///
/// Use the `#[indicator]` macro to automatically derive `Hash`, `Eq`,
/// parameter metadata, and registry registration:
///
/// ```ignore
/// #[indicator(name = "sma", input = f64, output = f64)]
/// pub struct Sma {
///     #[param]
///     period: usize,
///     buffer: RingBuffer,  // non-param fields are internal state
/// }
/// ```
pub trait Indicator: Clone + Send + Sync + 'static {
    /// The input type this indicator processes.
    type Input: Copy;

    /// The output type produced by this indicator.
    type Output: Copy;

    /// Static identifier for this indicator type (e.g., "sma", "ema", "rsi").
    const NAME: &'static str;

    /// Reset the indicator state to initial values.
    fn reset(&mut self);

    /// Process the next input value and return the indicator output.
    ///
    /// Returns `None` during the warmup period.
    fn next(&mut self, input: Self::Input) -> Option<Self::Output>;

    /// The number of bars required before producing valid output.
    fn warmup_period(&self) -> usize;
}
