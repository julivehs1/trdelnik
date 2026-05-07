//! # Trdelnik
//!
//! A comprehensive trading chart library for egui.
//!
//! ## Quick Start
//!
//! ```rust
//! use trdelnik::{
//!     generate_sample_data, Timeframe, ChartTheme,
//!     ChartBuilder, Sma, Ema, Rsi, Macd,
//! };
//!
//! let series = generate_sample_data(100, Timeframe::H1);
//!
//! let chart_data = ChartBuilder::new(series)
//!     .overlay(Sma::new(20))
//!     .overlay(Ema::new(50))
//!     .panel("rsi", |p| {
//!         p.plot(Rsi::new(14));
//!         p.hline(30.0);
//!         p.hline(70.0);
//!         p.ylim(0.0, 100.0);
//!     })
//!     .panel("macd", |p| {
//!         p.plot(Macd::default());
//!         p.hline(0.0);
//!     })
//!     .build();
//!
//! let theme = ChartTheme::dark();
//! // TradingChart::new(&chart_data, &theme).show(ui);
//! ```

// Re-export from trdelnik-core
pub use trdelnik_core::{
    AxisCoordinate, BlockNumber, Candle, CandleSeries, Index,
    HistogramBar, IndicatorLine, YAxis,
    Signal, SignalDirection, SignalSeries, SignalStrength,
    Slot, Timeframe, Timestamp, TimestampCandle,
    generate_sample_data,
};

// Re-export from trdelnik-render
// `LineStyle`/`Stroke` are intentionally not re-exported because
// `trdelnik_drawings::LineStyle` is the more visible type for end users.
pub use trdelnik_render::{
    HLine, HLineStyle, IndicatorTheme, Plot, PlotContext, Renderer, StandardPlot, Transform,
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
    FisherTransform, FisherTransformValue,
    ChandeKrollStop, ChandeKrollStopValue,
    // Traits
    Indicator, Plottable,
};

// Re-export from trdelnik-theme
pub use trdelnik_theme::{ChartTheme, Color, ThemeError};

// Re-export from trdelnik-data
pub use trdelnik_data::{
    CacheError, ChartBuilder, ChartData, ComputedIndicators,
    IndicatorKey, IntoChartData,
    Panel, PanelConfig, PanelHandle,
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
