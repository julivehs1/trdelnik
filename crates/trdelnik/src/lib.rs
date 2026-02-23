//! # Trdelnik
//!
//! A comprehensive trading chart library for egui.
//!
//! ## Features
//!
//! - **Candlestick Charts**: Full OHLCV candlestick rendering with customizable colors
//! - **Volume Bars**: Synchronized volume display with color coding
//! - **Technical Indicators**:
//!   - Moving Averages: SMA, EMA, WMA
//!   - Volatility: Bollinger Bands, Keltner Channel, ATR, True Range, Standard Deviation
//!   - Momentum: RSI, MACD, Stochastic, CCI, PPO, ROC, MFI
//!   - Trend: Chandelier Exit, Efficiency Ratio
//!   - Volume: OBV (On-Balance Volume)
//! - **Generic X-Axis**: Support for timestamps, Solana slots, Ethereum blocks, or custom coordinates
//! - **Interactive Features**:
//!   - Zoom and pan
//!   - Crosshair with price/time display
//!   - Linked axis scrolling between panels
//! - **Theming**: Dark, light, blue, midnight themes with persistence support
//! - **Caching**: Pre-computed indicators for optimal render performance
//!
//! ## Quick Start
//!
//! ```rust
//! use trdelnik::{
//!     generate_sample_data, Timeframe, ChartTheme,
//!     ChartDataBuilder, Sma, Ema, Rsi, Macd,
//! };
//!
//! // Create sample data
//! let series = generate_sample_data(100, Timeframe::H1);
//!
//! // Build chart data with indicators (all calculations happen here)
//! let chart_data = ChartDataBuilder::new(series)
//!     .add_overlay(Sma::new(20))
//!     .add_overlay(Ema::new(50))
//!     .add_to_panel("rsi", Rsi::new(14))
//!     .add_to_panel("macd", Macd::default())
//!     .build();
//!
//! let theme = ChartTheme::dark();
//!
//! // In your egui app:
//! // TradingChart::new(&chart_data, &theme).show(ui);
//! ```
//!
//! ## Architecture
//!
//! The library is split into several crates:
//!
//! - `trdelnik-core`: Core types (Candle, CandleSeries, Signal, AxisCoordinate)
//! - `trdelnik-indicators`: Technical indicator calculations
//! - `trdelnik-theme`: GUI-agnostic theme system
//! - `trdelnik-data`: Data layer with caching
//! - `trdelnik-ui`: egui rendering
//! - `trdelnik`: This facade crate (re-exports everything)
//!
//! ## Using with Solana Slots
//!
//! ```rust
//! use trdelnik::{Candle, CandleSeries, Slot, ChartDataBuilder, Sma};
//!
//! // Create candles with slot coordinates
//! let mut series = CandleSeries::<Slot>::new();
//! series.push(Candle::new(Slot::new(100000000), 100.0, 105.0, 98.0, 103.0, 1000.0));
//! series.push(Candle::new(Slot::new(100000001), 103.0, 108.0, 101.0, 106.0, 1500.0));
//!
//! let chart_data = ChartDataBuilder::new(series)
//!     .add_overlay(Sma::new(20))
//!     .build();
//! ```

// Re-export from trdelnik-core
pub use trdelnik_core::{
    AxisCoordinate, BlockNumber, Candle, CandleSeries, Index,
    IndicatorLine, IndicatorOutput, Placement, YAxis, HistogramBar,
    Signal, SignalDirection, SignalSeries, SignalStrength,
    Slot, Timeframe, Timestamp, TimestampCandle,
    generate_sample_data,
};

// Re-export from trdelnik-indicators
pub use trdelnik_indicators::{
    // Moving averages
    Sma, Ema, Wma,
    // Overlay indicators
    Bollinger, BollingerValue,
    Keltner, KeltnerValue,
    Chandelier, ChandelierValue,
    // Panel indicators
    Rsi, Macd, MacdValue,
    Stochastic, StochasticValue,
    Atr, StdDev, Roc,
    EfficiencyRatio, Obv,
    Cci, Ppo, PpoValue,
    Mfi,
    // Stateful indicator trait
    Indicator,
};

// Re-export from trdelnik-theme
pub use trdelnik_theme::{ChartTheme, Color, ThemeError};

// Re-export from trdelnik-data
pub use trdelnik_data::{
    CacheError, ChartData, ChartDataBuilder, ChartIndicator, ComputedIndicators,
    IndicatorKey, IntoChartData,
    Panel, PanelBuilder, PanelConfig,
};

// Re-export from trdelnik-ui
pub use trdelnik_ui::{
    ChartConfig, ChartResponse, PanelRenderInfo, TradingChart,
    ToEguiColor, FromEguiColor,
    to_egui_color, from_egui_color,
    PanelHeightState,
    // Drawing layer
    handle_drawing_input, render_drawings, render_drawings_for_panel, screen_to_data,
    DrawingRenderTheme, ViewBounds,
    // Axis state (for accessing view bounds)
    SharedXState, PanelYState, CursorState,
};

// Re-export from trdelnik-drawings
pub use trdelnik_drawings::{
    // Core types
    ChartPoint, Anchor, AnchorPoint,
    // Trait
    Drawing,
    // Style
    DrawingStyle, LineStyle, preset_colors,
    // Output types
    DrawingOutput, DrawingLine, DrawingFill, DrawingLabel, DrawingHandle,
    TextAnchor, HandleType,
    // Store
    DrawingStore,
    // Tool
    DrawingTool, ToolState, ToolAction,
    // Built-in drawings
    HorizontalLine, VerticalLine, TrendLine, PositionTool,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
