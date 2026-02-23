//! Trdelnik Demo Application
//!
//! Run with: cargo run -p trdelnik-demo

use eframe::egui;
use trdelnik::{
    generate_sample_data, ChartConfig, ChartData, ChartDataBuilder,
    ChartTheme, Timeframe, Timestamp, TradingChart,
    // Moving averages & overlays
    Sma, Ema, Bollinger, Keltner, Chandelier,
    // Panel indicators
    Rsi, Macd, Atr, StdDev, Roc,
    EfficiencyRatio, Obv, Cci, Ppo, Mfi,
    // Drawing tools
    DrawingStore, ToolState, DrawingTool, DrawingRenderTheme, ViewBounds,
    render_drawings_for_panel,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Trdelnik - Trading Chart Demo"),
        ..Default::default()
    };

    eframe::run_native(
        "Trdelnik Demo",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(DemoApp::new()))
        }),
    )
}

struct DemoApp {
    chart_data: ChartData<Timestamp>,
    theme: ChartTheme,
    selected_theme: ThemeSelection,
    selected_timeframe: Timeframe,
    num_candles: usize,
    show_volume: bool,

    // Overlay indicators
    show_sma: bool,
    sma_period: usize,
    show_ema: bool,
    ema_period: usize,
    show_bollinger: bool,
    bollinger_period: usize,
    show_keltner: bool,
    keltner_period: usize,
    show_chandelier: bool,
    chandelier_period: usize,

    // Panel indicators
    show_rsi: bool,
    rsi_period: usize,
    show_macd: bool,
    show_atr: bool,
    atr_period: usize,
    show_stddev: bool,
    stddev_period: usize,
    show_roc: bool,
    roc_period: usize,
    show_efficiency: bool,
    efficiency_period: usize,
    show_obv: bool,
    show_cci: bool,
    cci_period: usize,
    show_ppo: bool,
    show_mfi: bool,
    mfi_period: usize,

    // Drawing state
    drawing_store: DrawingStore<Timestamp>,
    tool_state: ToolState<Timestamp>,
    drawing_theme: DrawingRenderTheme,
    /// Current panel being drawn on
    active_panel: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ThemeSelection {
    Dark,
    Light,
    Blue,
    Midnight,
}

impl DemoApp {
    fn new() -> Self {
        let timeframe = Timeframe::H1;
        let num_candles = 200;
        let series = generate_sample_data(num_candles, timeframe);

        let mut app = Self {
            chart_data: ChartData::new(series),
            theme: ChartTheme::dark(),
            selected_theme: ThemeSelection::Dark,
            selected_timeframe: timeframe,
            num_candles,
            show_volume: true,

            // Overlay indicators
            show_sma: true,
            sma_period: 20,
            show_ema: true,
            ema_period: 50,
            show_bollinger: false,
            bollinger_period: 20,
            show_keltner: false,
            keltner_period: 20,
            show_chandelier: false,
            chandelier_period: 22,

            // Panel indicators
            show_rsi: true,
            rsi_period: 14,
            show_macd: false,
            show_atr: false,
            atr_period: 14,
            show_stddev: false,
            stddev_period: 20,
            show_roc: false,
            roc_period: 12,
            show_efficiency: false,
            efficiency_period: 10,
            show_obv: false,
            show_cci: false,
            cci_period: 20,
            show_ppo: false,
            show_mfi: false,
            mfi_period: 14,

            // Initialize drawing state
            drawing_store: DrawingStore::new(),
            tool_state: ToolState::new(),
            drawing_theme: DrawingRenderTheme::default(),
            active_panel: "main".to_string(),
        };
        app.rebuild_chart_data();
        app
    }

    fn regenerate_data(&mut self) {
        let series = generate_sample_data(self.num_candles, self.selected_timeframe);
        self.chart_data = ChartData::new(series);
        self.rebuild_chart_data();
    }

    fn rebuild_chart_data(&mut self) {
        // Get the series from existing chart data
        let series = std::mem::take(self.chart_data.series_mut());

        // Rebuild with selected indicators using new API
        let mut builder = ChartDataBuilder::new(series);

        // Overlay indicators
        if self.show_sma {
            builder = builder.add_overlay(Sma::new(self.sma_period));
        }
        if self.show_ema {
            builder = builder.add_overlay(Ema::new(self.ema_period));
        }
        if self.show_bollinger {
            builder = builder.add_overlay(Bollinger::new(self.bollinger_period, 2.0));
        }
        if self.show_keltner {
            builder = builder.add_overlay(Keltner::new(self.keltner_period, self.keltner_period, 2.0));
        }
        if self.show_chandelier {
            builder = builder.add_overlay(Chandelier::new(self.chandelier_period, 3.0));
        }

        // Panel indicators
        if self.show_rsi {
            builder = builder.add_to_panel("rsi", Rsi::new(self.rsi_period));
        }
        if self.show_macd {
            builder = builder.add_to_panel("macd", Macd::default());
        }
        if self.show_atr {
            builder = builder.add_to_panel("atr", Atr::new(self.atr_period));
        }
        if self.show_stddev {
            builder = builder.add_to_panel("stddev", StdDev::new(self.stddev_period));
        }
        if self.show_roc {
            builder = builder.add_to_panel("roc", Roc::new(self.roc_period));
        }
        if self.show_efficiency {
            builder = builder.add_to_panel("efficiency", EfficiencyRatio::new(self.efficiency_period));
        }
        if self.show_obv {
            builder = builder.add_to_panel("obv", Obv::new());
        }
        if self.show_cci {
            builder = builder.add_to_panel("cci", Cci::new(self.cci_period, 0.015));
        }
        if self.show_ppo {
            builder = builder.add_to_panel("ppo", Ppo::default());
        }
        if self.show_mfi {
            builder = builder.add_to_panel("mfi", Mfi::new(self.mfi_period));
        }

        self.chart_data = builder.build();
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut needs_rebuild = false;

        // Side panel for controls
        egui::SidePanel::left("controls")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Trdelnik Demo");
                ui.separator();

                // Theme selection
                ui.label("Theme:");
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.selected_theme == ThemeSelection::Dark, "Dark")
                        .clicked()
                    {
                        self.selected_theme = ThemeSelection::Dark;
                        self.theme = ChartTheme::dark();
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                    if ui
                        .selectable_label(self.selected_theme == ThemeSelection::Light, "Light")
                        .clicked()
                    {
                        self.selected_theme = ThemeSelection::Light;
                        self.theme = ChartTheme::light();
                        ctx.set_visuals(egui::Visuals::light());
                    }
                });
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.selected_theme == ThemeSelection::Blue, "Blue")
                        .clicked()
                    {
                        self.selected_theme = ThemeSelection::Blue;
                        self.theme = ChartTheme::blue();
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                    if ui
                        .selectable_label(self.selected_theme == ThemeSelection::Midnight, "Midnight")
                        .clicked()
                    {
                        self.selected_theme = ThemeSelection::Midnight;
                        self.theme = ChartTheme::midnight();
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                });

                ui.separator();

                // Drawing Tools
                ui.heading("Drawing Tools");
                ui.horizontal_wrapped(|ui| {
                    let current_tool = self.tool_state.tool();

                    if ui.selectable_label(current_tool == DrawingTool::None, "Pointer").clicked() {
                        self.tool_state.set_tool(DrawingTool::None);
                    }
                    if ui.selectable_label(current_tool == DrawingTool::Crosshair, "Crosshair").clicked() {
                        self.tool_state.set_tool(DrawingTool::Crosshair);
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    let current_tool = self.tool_state.tool();

                    if ui.selectable_label(current_tool == DrawingTool::TrendLine, "Trend").clicked() {
                        self.tool_state.set_tool(DrawingTool::TrendLine);
                    }
                    if ui.selectable_label(current_tool == DrawingTool::Ray, "Ray").clicked() {
                        self.tool_state.set_tool(DrawingTool::Ray);
                    }
                    if ui.selectable_label(current_tool == DrawingTool::HorizontalLine, "H-Line").clicked() {
                        self.tool_state.set_tool(DrawingTool::HorizontalLine);
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    let current_tool = self.tool_state.tool();

                    if ui.selectable_label(current_tool == DrawingTool::VerticalLine, "V-Line").clicked() {
                        self.tool_state.set_tool(DrawingTool::VerticalLine);
                    }
                    if ui.selectable_label(current_tool == DrawingTool::Position, "Position").clicked() {
                        self.tool_state.set_tool(DrawingTool::Position);
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Clear All").clicked() {
                        self.drawing_store.clear();
                    }
                    ui.label(format!("{} drawings", self.drawing_store.len()));
                });

                if self.tool_state.is_drawing() {
                    ui.colored_label(egui::Color32::YELLOW, "Click to place point...");
                }

                ui.separator();

                // Timeframe selection
                ui.label("Timeframe:");
                egui::ComboBox::from_id_salt("timeframe")
                    .selected_text(self.selected_timeframe.label())
                    .show_ui(ui, |ui| {
                        let timeframes = [
                            Timeframe::M1,
                            Timeframe::M5,
                            Timeframe::M15,
                            Timeframe::M30,
                            Timeframe::H1,
                            Timeframe::H4,
                            Timeframe::D1,
                            Timeframe::W1,
                        ];
                        for tf in timeframes {
                            if ui
                                .selectable_label(self.selected_timeframe == tf, tf.label())
                                .clicked()
                            {
                                self.selected_timeframe = tf;
                                self.regenerate_data();
                            }
                        }
                    });

                ui.separator();

                // Number of candles
                ui.label("Number of candles:");
                if ui
                    .add(egui::Slider::new(&mut self.num_candles, 50..=500))
                    .changed()
                {
                    self.regenerate_data();
                }

                if ui.button("Regenerate Data").clicked() {
                    self.regenerate_data();
                }

                ui.separator();

                // Overlay indicators
                ui.heading("Overlays");

                if ui.checkbox(&mut self.show_sma, "SMA").changed() {
                    needs_rebuild = true;
                }
                if self.show_sma {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.sma_period).range(2..=200))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_ema, "EMA").changed() {
                    needs_rebuild = true;
                }
                if self.show_ema {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.ema_period).range(2..=200))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_bollinger, "Bollinger Bands").changed() {
                    needs_rebuild = true;
                }
                if self.show_bollinger {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.bollinger_period).range(2..=100))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_keltner, "Keltner Channel").changed() {
                    needs_rebuild = true;
                }
                if self.show_keltner {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.keltner_period).range(2..=100))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_chandelier, "Chandelier Exit").changed() {
                    needs_rebuild = true;
                }
                if self.show_chandelier {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.chandelier_period).range(2..=100))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                ui.separator();

                // Panel indicators
                ui.heading("Panels");

                ui.checkbox(&mut self.show_volume, "Volume");

                if ui.checkbox(&mut self.show_rsi, "RSI").changed() {
                    needs_rebuild = true;
                }
                if self.show_rsi {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.rsi_period).range(2..=50))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_macd, "MACD").changed() {
                    needs_rebuild = true;
                }

                if ui.checkbox(&mut self.show_ppo, "PPO").changed() {
                    needs_rebuild = true;
                }

                if ui.checkbox(&mut self.show_atr, "ATR").changed() {
                    needs_rebuild = true;
                }
                if self.show_atr {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.atr_period).range(2..=50))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_stddev, "Std Deviation").changed() {
                    needs_rebuild = true;
                }
                if self.show_stddev {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.stddev_period).range(2..=100))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_roc, "Rate of Change").changed() {
                    needs_rebuild = true;
                }
                if self.show_roc {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.roc_period).range(1..=50))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_efficiency, "Efficiency Ratio").changed() {
                    needs_rebuild = true;
                }
                if self.show_efficiency {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.efficiency_period).range(2..=50))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_obv, "OBV").changed() {
                    needs_rebuild = true;
                }

                if ui.checkbox(&mut self.show_cci, "CCI").changed() {
                    needs_rebuild = true;
                }
                if self.show_cci {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.cci_period).range(2..=50))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                if ui.checkbox(&mut self.show_mfi, "Money Flow Index").changed() {
                    needs_rebuild = true;
                }
                if self.show_mfi {
                    ui.horizontal(|ui| {
                        ui.label("Period:");
                        if ui
                            .add(egui::DragValue::new(&mut self.mfi_period).range(2..=50))
                            .changed()
                        {
                            needs_rebuild = true;
                        }
                    });
                }

                ui.separator();

                // Info
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(format!("Candles: {}", self.chart_data.len()));
                    if let Some((low, high)) = self.chart_data.price_range() {
                        ui.label(format!("Price range: {:.2} - {:.2}", low, high));
                    }
                    ui.label(format!("Overlays: {}", self.chart_data.overlay_indicators().len()));
                    ui.label(format!("Panels: {}", self.chart_data.panel_containers().len()));
                    ui.hyperlink_to("GitHub", "https://github.com/julivehs1/trdelnik");
                    ui.label(format!("v{}", trdelnik::VERSION));
                });
            });

        // Rebuild chart data if needed
        if needs_rebuild {
            self.rebuild_chart_data();
        }

        // Main chart area
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(self.theme.background.to_egui()))
            .show(ctx, |ui| {
                let config = ChartConfig::default().with_volume(self.show_volume);

                // Show the trading chart - this handles all pan/zoom
                let response = TradingChart::new(&self.chart_data, &self.theme)
                    .config(config)
                    .show(ui);

                // Detect which panel the user is clicking on and handle drawing input
                let hover_pos = ui.input(|i| i.pointer.hover_pos());

                // Find which panel is being hovered/clicked
                let mut hovered_panel: Option<(String, egui::Rect, ViewBounds)> = None;
                for (panel_id, panel_info) in &response.panels {
                    if let Some(pos) = hover_pos {
                        if panel_info.rect.contains(pos) {
                            let (x_min, x_max) = panel_info.x_bounds;
                            let (y_min, y_max) = panel_info.y_bounds;
                            if x_max > x_min && y_max > y_min {
                                hovered_panel = Some((
                                    panel_id.clone(),
                                    panel_info.rect,
                                    ViewBounds::new(x_min, x_max, y_min, y_max),
                                ));
                                break;
                            }
                        }
                    }
                }

                // Handle drawing input for the hovered panel
                if let Some((panel_id, panel_rect, bounds)) = &hovered_panel {
                    // Update active panel when starting to draw
                    if self.tool_state.tool().is_drawing_tool() {
                        self.active_panel = panel_id.clone();

                        // Create an invisible layer on top for drawing input
                        let drawing_response = ui.interact(
                            *panel_rect,
                            egui::Id::new("drawing_layer").with(panel_id),
                            egui::Sense::click(),
                        );

                        // Handle drawing input - but we need to manually handle adding to the right panel
                        if drawing_response.clicked() {
                            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                                if panel_rect.contains(pointer_pos) {
                                    let data_point = trdelnik::screen_to_data(pointer_pos, *panel_rect, *bounds);

                                    match self.tool_state.handle_click(data_point.clone()) {
                                        trdelnik::ToolAction::Complete(drawing) => {
                                            // Add to the active panel
                                            self.drawing_store.add_boxed_to_panel(drawing, &self.active_panel);
                                        }
                                        trdelnik::ToolAction::Continue => {}
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // Update preview point for mouse position
                        if let Some(pos) = hover_pos {
                            if panel_rect.contains(pos) {
                                let data_point: trdelnik::ChartPoint<Timestamp> =
                                    trdelnik::screen_to_data(pos, *panel_rect, *bounds);
                                self.tool_state.handle_move(data_point);
                            }
                        }

                        // Show crosshair cursor
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    } else {
                        // In pointer mode, handle selection on click
                        if ui.input(|i| i.pointer.primary_clicked()) {
                            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                if panel_rect.contains(pos) {
                                    let data_point: trdelnik::ChartPoint<Timestamp> =
                                        trdelnik::screen_to_data(pos, *panel_rect, *bounds);

                                    let tolerance = 5.0 * (bounds.x_max - bounds.x_min) / panel_rect.width() as f64;

                                    if let Some(id) = self.drawing_store.hit_test(&data_point, tolerance) {
                                        self.drawing_store.select(Some(id));
                                    } else {
                                        self.drawing_store.deselect();
                                    }
                                }
                            }
                        }
                    }
                }

                // Handle escape and delete regardless of tool
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.tool_state.handle_escape();
                    self.drawing_store.deselect();
                }
                if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                    if let Some(selected) = self.drawing_store.selected() {
                        self.drawing_store.remove(selected);
                    }
                }

                // Render drawings on each panel
                for (panel_id, panel_info) in &response.panels {
                    let (x_min, x_max) = panel_info.x_bounds;
                    let (y_min, y_max) = panel_info.y_bounds;
                    if x_max > x_min && y_max > y_min {
                        let bounds = ViewBounds::new(x_min, x_max, y_min, y_max);
                        render_drawings_for_panel(
                            ui.painter(),
                            panel_info.rect,
                            &self.drawing_store,
                            &self.tool_state,
                            bounds,
                            &self.drawing_theme,
                            panel_id,
                        );
                    }
                }
            });
    }
}

// Extension trait for egui color conversion (needed in demo)
trait ToEgui {
    fn to_egui(&self) -> egui::Color32;
}

impl ToEgui for trdelnik::Color {
    fn to_egui(&self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(self.r, self.g, self.b, self.a)
    }
}
