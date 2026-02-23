//! # Trdelnik Drawings
//!
//! Drawing tools for the trdelnik trading chart library.
//! This crate contains no GUI dependencies and can be used in any context.
//!
//! ## Overview
//!
//! This crate provides a modular, extensible system for chart drawings. It follows the same architectural patterns
//! as `trdelnik-indicators`.
//!
//! ## Main Types
//!
//! - [`Drawing`] - The core trait that all drawing types implement
//! - [`DrawingStore`] - Manages all drawings for a chart
//! - [`ToolState`] - State machine for active drawing tools
//! - [`DrawingStyle`] - Style configuration for drawings
//! - [`DrawingOutput`] - Render output (lines, fills, labels)
//!
//! ## Built-in Drawings
//!
//! - [`HorizontalLine`] - Horizontal line at a price level
//! - [`VerticalLine`] - Vertical line at a time
//! - [`TrendLine`] - Line between two points
//! - [`PositionTool`] - Gain/loss measurement tool
//!
//! ## Example
//!
//! ```rust,ignore
//! use trdelnik_drawings::{DrawingStore, ToolState, DrawingTool, ChartPoint};
//! use trdelnik_core::Index;
//!
//! // Create a drawing store and tool state
//! let mut store: DrawingStore<Index> = DrawingStore::new();
//! let mut tool_state: ToolState<Index> = ToolState::new();
//!
//! // Activate a tool
//! tool_state.set_tool(DrawingTool::TrendLine);
//!
//! // Handle clicks
//! tool_state.handle_click(ChartPoint::new(Index(0), 100.0));
//! tool_state.handle_click(ChartPoint::new(Index(10), 110.0));
//!
//! // The tool state will return a complete drawing that can be added to the store
//! ```

pub mod coords;
pub mod drawings;
pub mod output;
pub mod store;
pub mod style;
pub mod tool;
pub mod traits;

// Re-exports - Coordinates
pub use coords::{point_to_line_distance, point_to_segment_distance, Anchor, AnchorPoint, ChartPoint};

// Re-exports - Traits
pub use traits::Drawing;

// Re-exports - Output types
pub use output::{
    DrawingFill, DrawingHandle, DrawingLabel, DrawingLine, DrawingOutput, HandleType,
    HorizontalLineOutput, TextAnchor, VerticalLineOutput,
};

// Re-exports - Style
pub use style::{preset_colors, DrawingStyle, LineStyle, COLOR_PRESETS};

// Re-exports - Store
pub use store::DrawingStore;

// Re-exports - Tool
pub use tool::{DrawingTool, ToolAction, ToolState};

// Re-exports - Drawings
pub use drawings::{HorizontalLine, PositionTool, TrendLine, VerticalLine};
