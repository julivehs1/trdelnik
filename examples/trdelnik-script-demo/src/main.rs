//! TrdelScript Demo Application
//!
//! Demonstrates loading and executing TrdelScript strategies with integrated
//! chart rendering using the new script integration API.
//!
//! Run with: cargo run -p trdelnik-script-demo

use eframe::egui;
use egui_code_editor::{CodeEditor, ColorTheme, Completer, Syntax};
use std::collections::HashMap;
use std::time::Instant;
use trdelnik::{
    generate_sample_data, ChartConfig, ChartData, ChartDataBuilder, ChartTheme, Color, Timeframe,
    Timestamp, TradingChart,
};
use trdelnik_core::CandleSeries;
use trdelnik_data::{create_signals_overlay, plots_to_overlays_with_config, ScriptPlotConfig, SignalStats};
use trdelnik_graph::ExecutionResult;
use trdelnik_script::{compile_with_params, CompiledStrategy, Executor};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("TrdelScript Strategy Demo"),
        ..Default::default()
    };

    eframe::run_native(
        "TrdelScript Demo",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(DemoApp::new()))
        }),
    )
}

/// Built-in example strategies
const STRATEGIES: &[(&str, &str)] = &[
    (
        "SMA Crossover",
        include_str!("../strategies/sma_crossover.trdl"),
    ),
    (
        "Bollinger Breakout",
        include_str!("../strategies/bollinger_breakout.trdl"),
    ),
    (
        "MACD Momentum",
        include_str!("../strategies/macd_momentum.trdl"),
    ),
];

/// TrdelScript syntax definition for the code editor
fn trdelscript_syntax() -> Syntax {
    Syntax::new("trdelscript")
        .with_case_sensitive(true)
        .with_comment("//")
        .with_keywords([
            "strategy", "param", "let", "entry", "exit", "when", "plot",
            "long", "short", "all", "stop_loss", "take_profit", "timeframe",
            "color", "panel", "style", "and", "or", "not",
        ])
        .with_types([
            "int", "float", "bool", "true", "false",
        ])
        .with_special([
            "open", "high", "low", "close", "volume",
            "sma", "ema", "wma", "rsi", "std_dev", "roc",
            "efficiency_ratio", "atr", "cci", "obv", "mfi",
            "bollinger", "macd", "ppo", "stochastic", "keltner", "chandelier",
            "crossover", "crossunder", "cross",
        ])
}

struct DemoApp {
    // Chart data
    series: CandleSeries<Timestamp>,
    chart_data: ChartData<Timestamp>,
    theme: ChartTheme,

    // Strategy
    selected_strategy_idx: usize,
    strategy_source: String,
    compiled_strategy: Option<CompiledStrategy>,
    execution_result: Option<ExecutionResult>,
    compile_error: Option<String>,

    // Execution results
    plot_configs: Vec<ScriptPlotConfig>,
    signal_stats: SignalStats,

    // Parameter overrides
    param_overrides: HashMap<String, f64>,

    // UI state
    num_candles: usize,
    selected_timeframe: Timeframe,
    show_signals: bool,
    show_source: bool,

    // Code editor
    syntax: Syntax,
    completer: Completer,
    last_edit_time: Option<Instant>,
    auto_compile: bool,
}

impl DemoApp {
    fn new() -> Self {
        let timeframe = Timeframe::H1;
        let num_candles = 200;
        let series = generate_sample_data(num_candles, timeframe);

        let syntax = trdelscript_syntax();
        let completer = Completer::new_with_syntax(&syntax).with_user_words();

        let mut app = Self {
            series: series.clone(),
            chart_data: ChartData::new(series),
            theme: ChartTheme::dark(),

            selected_strategy_idx: 0,
            strategy_source: STRATEGIES[0].1.to_string(),
            compiled_strategy: None,
            execution_result: None,
            compile_error: None,

            plot_configs: Vec::new(),
            signal_stats: SignalStats::default(),

            param_overrides: HashMap::new(),

            num_candles,
            selected_timeframe: timeframe,
            show_signals: true,
            show_source: true,

            syntax,
            completer,
            last_edit_time: None,
            auto_compile: true,
        };

        app.compile_and_run();
        app
    }

    fn compile_and_run(&mut self) {
        match compile_with_params(&self.strategy_source, self.param_overrides.clone()) {
            Ok(strategy) => {
                self.compile_error = None;

                // Execute using a cloned graph (Graph is now Clone!)
                let mut executor = Executor::new(strategy.graph.clone());
                let result = executor.process_series(&self.series);

                // Get plot configs from strategy
                self.plot_configs = ScriptPlotConfig::from_strategy(&strategy);

                // Get signal statistics
                self.signal_stats = SignalStats::from_execution(&strategy, &result);

                // Build chart data with script overlays and optionally signals as markers
                let mut overlays =
                    plots_to_overlays_with_config(&strategy, &result, &self.series, Some(&self.plot_configs));

                // Add signals as markers in an overlay if enabled
                if self.show_signals {
                    let signals_overlay = create_signals_overlay(&strategy, &result, &self.series);
                    overlays.push(signals_overlay);
                }

                self.chart_data = ChartDataBuilder::new(self.series.clone())
                    .add_indicator_outputs(overlays)
                    .build();

                self.execution_result = Some(result);
                self.compiled_strategy = Some(strategy);
            }
            Err(err) => {
                self.compile_error = Some(err.to_report_string(&self.strategy_source));
                self.compiled_strategy = None;
                self.execution_result = None;
                self.plot_configs.clear();
                self.signal_stats = SignalStats::default();
                self.chart_data = ChartData::new(self.series.clone());
            }
        }
    }

    /// Rebuild chart data with current plot configs (without recompiling)
    fn rebuild_chart_data(&mut self) {
        if let (Some(strategy), Some(result)) = (&self.compiled_strategy, &self.execution_result) {
            // Build chart data with current plot configs (preserves user colors)
            let mut overlays =
                plots_to_overlays_with_config(strategy, result, &self.series, Some(&self.plot_configs));

            // Add signals as markers if enabled
            if self.show_signals {
                let signals_overlay = create_signals_overlay(strategy, result, &self.series);
                overlays.push(signals_overlay);
            }

            self.chart_data = ChartDataBuilder::new(self.series.clone())
                .add_indicator_outputs(overlays)
                .build();
        }
    }

    fn regenerate_data(&mut self) {
        self.series = generate_sample_data(self.num_candles, self.selected_timeframe);
        self.compile_and_run();
    }

    fn select_strategy(&mut self, idx: usize) {
        self.selected_strategy_idx = idx;
        self.strategy_source = STRATEGIES[idx].1.to_string();
        self.param_overrides.clear();
        self.compile_and_run();
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Debounced auto-compile
        if self.auto_compile {
            if let Some(last_edit) = self.last_edit_time {
                if last_edit.elapsed().as_millis() > 500 {
                    self.last_edit_time = None;
                    self.compile_and_run();
                } else {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
        }

        // Left panel: Strategy selection and info
        egui::SidePanel::left("strategy_panel")
            .min_width(280.0)
            .show(ctx, |ui| {
                ui.heading("TrdelScript Demo");
                ui.separator();

                // Strategy selection
                ui.label("Strategy:");
                egui::ComboBox::from_id_salt("strategy_select")
                    .selected_text(STRATEGIES[self.selected_strategy_idx].0)
                    .show_ui(ui, |ui| {
                        for (idx, (name, _)) in STRATEGIES.iter().enumerate() {
                            if ui
                                .selectable_label(self.selected_strategy_idx == idx, *name)
                                .clicked()
                            {
                                self.select_strategy(idx);
                            }
                        }
                    });

                ui.separator();

                // Strategy info
                if let Some(strategy) = &self.compiled_strategy {
                    if let Some(name) = &strategy.name {
                        ui.label(format!("Name: {}", name));
                    }
                    if let Some(tf) = &strategy.timeframe {
                        ui.label(format!("Timeframe: {}", tf));
                    }

                    ui.label(format!(
                        "Long Entry: {}",
                        if strategy.entry_long.is_some() {
                            "Yes"
                        } else {
                            "No"
                        }
                    ));
                    ui.label(format!(
                        "Short Entry: {}",
                        if strategy.entry_short.is_some() {
                            "Yes"
                        } else {
                            "No"
                        }
                    ));
                    ui.label(format!("Exit Signals: {}", strategy.exit_signals.len()));

                    if let Some(sl) = strategy.stop_loss {
                        ui.label(format!("Stop Loss: {}%", sl));
                    }
                    if let Some(tp) = strategy.take_profit {
                        ui.label(format!("Take Profit: {}%", tp));
                    }
                    ui.label(format!("Plots: {}", strategy.plots.len()));
                }

                ui.separator();

                // Execution results
                ui.heading("Signals");
                ui.colored_label(
                    egui::Color32::GREEN,
                    format!("Long Entries: {}", self.signal_stats.long_entries),
                );
                ui.colored_label(
                    egui::Color32::RED,
                    format!("Short Entries: {}", self.signal_stats.short_entries),
                );
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("Exits: {}", self.signal_stats.exits),
                );

                ui.separator();

                // Plot configuration UI
                if !self.plot_configs.is_empty() {
                    ui.heading("Plot Colors");
                    let mut needs_rebuild = false;

                    for config in &mut self.plot_configs {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut config.visible, "").changed() {
                                needs_rebuild = true;
                            }
                            ui.label(&config.name);

                            // Color picker
                            let mut color_arr = [config.color.r, config.color.g, config.color.b];
                            if ui.color_edit_button_srgb(&mut color_arr).changed() {
                                config.color = Color::rgb(color_arr[0], color_arr[1], color_arr[2]);
                                needs_rebuild = true;
                            }

                            // Reset button
                            if ui
                                .small_button("\u{21BA}")
                                .on_hover_text("Reset to script color")
                                .clicked()
                            {
                                config.color = config.script_color;
                                needs_rebuild = true;
                            }
                        });
                    }

                    if needs_rebuild {
                        self.rebuild_chart_data();
                    }

                    ui.separator();
                }

                // Data settings
                ui.heading("Data");

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

                ui.label("Number of candles:");
                if ui
                    .add(egui::Slider::new(&mut self.num_candles, 50..=500))
                    .changed()
                {
                    self.regenerate_data();
                }

                if ui.button("Generate new data").clicked() {
                    self.regenerate_data();
                }

                ui.separator();

                // Display options
                if ui.checkbox(&mut self.show_signals, "Show signals").changed() {
                    self.rebuild_chart_data();
                }
                ui.checkbox(&mut self.show_source, "Show source code");
                ui.checkbox(&mut self.auto_compile, "Auto-compile");

                ui.separator();

                // Info at bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(format!("Candles: {}", self.series.len()));
                    ui.hyperlink_to(
                        "TrdelScript Docs",
                        "https://github.com/julivehs1/trdelnik",
                    );
                });
            });

        // Right panel: Source code (when enabled)
        if self.show_source {
            egui::SidePanel::right("source_panel")
                .min_width(400.0)
                .show(ctx, |ui| {
                    ui.heading("Strategy Source Code");
                    ui.separator();

                    let output = CodeEditor::default()
                        .id_source("trdelscript_editor")
                        .with_rows(30)
                        .with_fontsize(14.0)
                        .with_theme(ColorTheme::GRUVBOX)
                        .with_syntax(self.syntax.clone())
                        .with_numlines(true)
                        .show_with_completer(ui, &mut self.strategy_source, &mut self.completer);

                    if output.response.changed() {
                        self.last_edit_time = Some(Instant::now());
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Compile & Run").clicked() {
                            self.last_edit_time = None;
                            self.compile_and_run();
                        }
                    });

                    // Error display under editor
                    if let Some(error) = &self.compile_error {
                        ui.add_space(4.0);
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(80, 20, 20, 200))
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.colored_label(egui::Color32::from_rgb(255, 120, 120), "Compilation error:");
                                egui::ScrollArea::vertical()
                                    .max_height(150.0)
                                    .show(ui, |ui| {
                                        ui.monospace(error);
                                    });
                            });
                    }
                });
        }

        // Main chart area
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(self.theme.background.to_egui()))
            .show(ctx, |ui| {
                let config = ChartConfig::default().with_volume(true);

                // Signals are now rendered as markers in chart_data, no with_signals needed
                TradingChart::new(&self.chart_data, &self.theme)
                    .config(config)
                    .show(ui);
            });
    }
}

// Extension trait for egui Color conversion
trait ToEgui {
    fn to_egui(&self) -> egui::Color32;
}

impl ToEgui for Color {
    fn to_egui(&self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(self.r, self.g, self.b, self.a)
    }
}
