//! Main trading chart widget

use std::collections::HashMap;

use egui::{Color32, Rect, Ui};

use trdelnik_core::{AxisCoordinate, Candle, Timestamp};
use trdelnik_data::ChartData;
use trdelnik_theme::ChartTheme;

use crate::config::ChartConfig;
use crate::panel_state::PanelHeightState;
use crate::panels::{render_main_chart, render_panel_container};
use crate::resize_handle::render_resize_handle_dual;

/// Render info for a single panel (rect and bounds)
#[derive(Debug, Clone, Copy)]
pub struct PanelRenderInfo {
    /// The inner rect of the panel (data area)
    pub rect: Rect,
    /// X bounds (shared across all panels)
    pub x_bounds: (f64, f64),
    /// Y bounds for this panel
    pub y_bounds: (f64, f64),
}

/// Response from the trading chart
#[derive(Debug, Clone)]
pub struct ChartResponse<X: AxisCoordinate = Timestamp> {
    /// Whether the chart is hovered
    pub hovered: bool,
    /// Index of clicked candle
    pub clicked_candle: Option<usize>,
    /// Current hovered candle data
    pub hovered_candle: Option<Candle<X>>,
    /// The rect of the main chart (data area, for drawing overlays)
    pub main_rect: Option<egui::Rect>,
    /// Current X-axis view bounds (after pan/zoom)
    pub x_bounds: Option<(f64, f64)>,
    /// Current Y-axis view bounds for main chart (after pan/zoom)
    pub y_bounds: Option<(f64, f64)>,
    /// All panel render info (keyed by panel_id, "main" for main chart)
    pub panels: HashMap<String, PanelRenderInfo>,
}

impl<X: AxisCoordinate> Default for ChartResponse<X> {
    fn default() -> Self {
        Self {
            hovered: false,
            clicked_candle: None,
            hovered_candle: None,
            main_rect: None,
            x_bounds: None,
            y_bounds: None,
            panels: HashMap::new(),
        }
    }
}

/// Panel info for tracking panel order and constraints
struct PanelInfo {
    id: String,
    min_height: f32,
    max_height: f32,
    default_height: f32,
}

/// The main trading chart widget
///
/// This widget renders pre-computed ChartData - it does NO calculations!
/// All indicator computations happen in the data layer before rendering.
pub struct TradingChart<'a, X: AxisCoordinate = Timestamp> {
    /// The chart data (with pre-computed indicators)
    data: &'a ChartData<X>,
    /// Chart configuration
    config: ChartConfig,
    /// Theme
    theme: &'a ChartTheme,
    /// Chart ID for state management
    id: egui::Id,
}

impl<'a, X: AxisCoordinate> TradingChart<'a, X> {
    /// Create a new trading chart
    pub fn new(data: &'a ChartData<X>, theme: &'a ChartTheme) -> Self {
        Self {
            data,
            config: ChartConfig::default(),
            theme,
            id: egui::Id::new("trading_chart"),
        }
    }

    /// Set the chart configuration
    pub fn config(mut self, config: ChartConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the chart ID
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id = egui::Id::new(id);
        self
    }

    /// Show the chart
    pub fn show(self, ui: &mut Ui) -> ChartResponse<X> {
        let mut response = ChartResponse::<X>::default();

        if self.data.is_empty() {
            ui.label("No data to display");
            return response;
        }

        let available_height = ui.available_height();

        // Load panel height state
        let panel_state_id = self.id.with("panel_heights");
        let mut panel_height_state = PanelHeightState::load(ui.ctx(), panel_state_id);

        // Collect all panels in order with their constraints
        let new_panels = self.data.panel_containers();

        let default_sub_panel_height = 100.0_f32;
        let resize_handle_height = 6.0_f32;
        let x_axis_height = 25.0_f32;

        // Build ordered list of all panels (main first, then sub-panels)
        let mut all_panels: Vec<PanelInfo> = Vec::new();

        // Main panel
        all_panels.push(PanelInfo {
            id: "main".to_string(),
            min_height: 200.0,
            max_height: available_height - 100.0, // Leave room for at least one sub-panel
            default_height: 400.0,
        });

        // Volume panel
        if self.config.show_volume {
            all_panels.push(PanelInfo {
                id: "volume".to_string(),
                min_height: 50.0,
                max_height: 300.0,
                default_height: default_sub_panel_height,
            });
        }

        // Indicator panels
        for panel in new_panels {
            all_panels.push(PanelInfo {
                id: panel.id().to_string(),
                min_height: panel.config.min_height,
                max_height: panel.config.max_height,
                default_height: panel.config.default_height,
            });
        }

        let sub_panel_count = all_panels.len() - 1; // Exclude main
        let total_resize_handles = resize_handle_height * sub_panel_count as f32;
        let fixed_overhead = total_resize_handles + x_axis_height;

        // Calculate current total height of all panels
        let total_panel_heights: f32 = all_panels
            .iter()
            .map(|p| panel_height_state.get_height(&p.id, p.default_height))
            .sum();

        // Available height for panels (excluding handles and x-axis)
        let height_for_panels = available_height - fixed_overhead;

        // If total doesn't match available, scale proportionally (initial layout or window resize)
        // But DON'T scale while actively resizing - that causes jumping
        if !panel_height_state.is_resizing() && (total_panel_heights - height_for_panels).abs() > 1.0 {
            let scale = height_for_panels / total_panel_heights;
            for panel in &all_panels {
                let current = panel_height_state.get_height(&panel.id, panel.default_height);
                let scaled = (current * scale).clamp(panel.min_height, panel.max_height);
                panel_height_state.set_height(&panel.id, scaled);
            }
        }

        // Get heights for rendering
        let main_height = panel_height_state.get_height("main", 400.0);

        // Determine which panel shows X-axis
        let has_sub_panels = sub_panel_count > 0;
        let main_shows_x = !has_sub_panels;

        // Handle colors
        let handle_color = Color32::from_gray(80);
        let hover_color = Color32::from_gray(140);

        // Render main chart
        let main_result = render_main_chart(
            ui,
            self.data,
            self.theme,
            &self.config,
            self.id,
            main_height,
            main_shows_x,
        );

        // Populate response with main chart info
        response.hovered = main_result.hovered;
        response.main_rect = Some(main_result.inner_rect);
        response.x_bounds = Some(main_result.x_bounds);
        response.y_bounds = Some(main_result.y_bounds);

        // Add main panel to panels map
        response.panels.insert(
            "main".to_string(),
            PanelRenderInfo {
                rect: main_result.inner_rect,
                x_bounds: main_result.x_bounds,
                y_bounds: main_result.y_bounds,
            },
        );

        // Render sub-panels with resize handles
        let x_spacing = self.data.x_spacing();
        let x_range = self.data.x_range().unwrap_or((0.0, 1.0));
        let total_items = sub_panel_count;
        let mut current_index = 0;

        // Volume panel
        if self.config.show_volume {
            let volume_panel = self.data.create_volume_panel(
                self.theme.volume_bullish,
                self.theme.volume_bearish,
            );
            let is_last = current_index == total_items - 1;
            let height = panel_height_state.get_height("volume", default_sub_panel_height);

            // Get the panel above this one
            let panel_above_id = if current_index == 0 {
                "main"
            } else {
                &all_panels[current_index].id
            };
            let panel_above = &all_panels[current_index]; // 0 = main

            render_resize_handle_dual(
                ui,
                panel_above_id,
                "volume",
                &mut panel_height_state,
                panel_above.min_height,
                panel_above.max_height,
                50.0,
                300.0,
                handle_color,
                hover_color,
            );

            let panel_result = render_panel_container(
                ui,
                &volume_panel,
                self.theme,
                &self.config,
                self.id,
                x_spacing,
                height,
                is_last,
                x_range,
            );

            // Add volume panel to panels map
            response.panels.insert(
                "volume".to_string(),
                PanelRenderInfo {
                    rect: panel_result.inner_rect,
                    x_bounds: panel_result.x_bounds,
                    y_bounds: panel_result.y_bounds,
                },
            );

            current_index += 1;
        }

        // Indicator panels
        for panel in new_panels {
            let is_last = current_index == total_items - 1;
            let panel_id = panel.id();
            let height = panel_height_state.get_height(panel_id, panel.config.default_height);

            // Get panel above (index in all_panels is current_index + 1 because main is at 0)
            let panel_above = &all_panels[current_index]; // The panel rendered before this one

            render_resize_handle_dual(
                ui,
                &panel_above.id,
                panel_id,
                &mut panel_height_state,
                panel_above.min_height,
                panel_above.max_height,
                panel.config.min_height,
                panel.config.max_height,
                handle_color,
                hover_color,
            );

            let panel_result = render_panel_container(
                ui,
                panel,
                self.theme,
                &self.config,
                self.id,
                x_spacing,
                height,
                is_last,
                x_range,
            );

            // Add panel to panels map
            response.panels.insert(
                panel_id.to_string(),
                PanelRenderInfo {
                    rect: panel_result.inner_rect,
                    x_bounds: panel_result.x_bounds,
                    y_bounds: panel_result.y_bounds,
                },
            );

            current_index += 1;
        }

        // Store panel height state
        panel_height_state.store(ui.ctx(), panel_state_id);

        response
    }
}
