//! State structures for axis scaling and view management

use egui::Id;

/// Combined view state that tracks both plot bounds and axis scaling
#[derive(Clone, Debug)]
pub struct ViewState {
    /// Current view bounds (may be modified by plot zoom/pan or axis drag)
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    /// Whether state has been initialized
    pub initialized: bool,
    /// Y-axis drag state
    pub y_dragging: bool,
    pub y_drag_start_y: Option<f32>,
    pub y_drag_start_bounds: Option<(f64, f64)>,
    /// X-axis drag state
    pub x_dragging: bool,
    pub x_drag_start_x: Option<f32>,
    pub x_drag_start_bounds: Option<(f64, f64)>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
            initialized: false,
            y_dragging: false,
            y_drag_start_y: None,
            y_drag_start_bounds: None,
            x_dragging: false,
            x_drag_start_x: None,
            x_drag_start_bounds: None,
        }
    }
}

impl ViewState {
    /// Load state from egui memory
    pub fn load(ctx: &egui::Context, id: Id) -> Self {
        ctx.data(|d| d.get_temp::<Self>(id)).unwrap_or_default()
    }

    /// Store state to egui memory
    pub fn store(self, ctx: &egui::Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    /// Initialize with data bounds if not already initialized
    pub fn init_if_needed(&mut self, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
        if !self.initialized {
            self.x_min = x_min;
            self.x_max = x_max;
            self.y_min = y_min;
            self.y_max = y_max;
            self.initialized = true;
        }
    }

    /// Update bounds from plot's actual transform (called after plot interaction)
    pub fn update_from_plot(&mut self, plot_x_min: f64, plot_x_max: f64, plot_y_min: f64, plot_y_max: f64) {
        // Only update if not currently dragging an axis
        if !self.y_dragging && !self.x_dragging {
            self.x_min = plot_x_min;
            self.x_max = plot_x_max;
            self.y_min = plot_y_min;
            self.y_max = plot_y_max;
        }
    }

    /// Reset to data bounds
    pub fn reset(&mut self, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
        self.x_min = x_min;
        self.x_max = x_max;
        self.y_min = y_min;
        self.y_max = y_max;
    }

    /// Scale Y-axis around center
    pub fn scale_y(&mut self, factor: f64) {
        let center = (self.y_min + self.y_max) / 2.0;
        let half_range = (self.y_max - self.y_min) / 2.0;
        let new_half_range = half_range * factor;
        self.y_min = center - new_half_range;
        self.y_max = center + new_half_range;
    }

    /// Scale X-axis around center
    pub fn scale_x(&mut self, factor: f64) {
        let center = (self.x_min + self.x_max) / 2.0;
        let half_range = (self.x_max - self.x_min) / 2.0;
        let new_half_range = half_range * factor;
        self.x_min = center - new_half_range;
        self.x_max = center + new_half_range;
    }
}

/// Shared X-axis view state (synchronized across all panels)
#[derive(Clone, Debug, Default)]
pub struct SharedXState {
    pub x_min: f64,
    pub x_max: f64,
    pub initialized: bool,
    pub dragging: bool,
    pub drag_start_x: Option<f32>,
    pub drag_start_bounds: Option<(f64, f64)>,
}

impl SharedXState {
    pub fn load(ctx: &egui::Context, id: Id) -> Self {
        ctx.data(|d| d.get_temp::<Self>(id)).unwrap_or_default()
    }

    pub fn store(self, ctx: &egui::Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    pub fn init_if_needed(&mut self, x_min: f64, x_max: f64) {
        if !self.initialized {
            self.x_min = x_min;
            self.x_max = x_max;
            self.initialized = true;
        }
    }

    pub fn update_from_plot(&mut self, plot_x_min: f64, plot_x_max: f64) {
        if !self.dragging {
            self.x_min = plot_x_min;
            self.x_max = plot_x_max;
        }
    }

    pub fn scale_x(&mut self, factor: f64) {
        let center = (self.x_min + self.x_max) / 2.0;
        let half_range = (self.x_max - self.x_min) / 2.0;
        let new_half_range = half_range * factor;
        self.x_min = center - new_half_range;
        self.x_max = center + new_half_range;
    }
}

/// Cursor state for tracking mouse position in data coordinates
#[derive(Clone, Debug, Default)]
pub struct CursorState {
    /// Cursor X position in data coordinates (time axis)
    pub x: Option<f64>,
    /// Cursor Y position in data coordinates (price axis)
    pub y: Option<f64>,
    /// Whether the cursor is currently over a panel
    pub active: bool,
}

impl CursorState {
    pub fn load(ctx: &egui::Context, id: Id) -> Self {
        ctx.data(|d| d.get_temp::<Self>(id)).unwrap_or_default()
    }

    pub fn store(self, ctx: &egui::Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    /// Update cursor position from screen coordinates
    pub fn update_from_screen(
        &mut self,
        screen_pos: Option<egui::Pos2>,
        plot_rect: egui::Rect,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
    ) {
        if let Some(pos) = screen_pos {
            if plot_rect.contains(pos) {
                // Convert screen to data coordinates
                let rel_x = (pos.x - plot_rect.left()) / plot_rect.width();
                let rel_y = (pos.y - plot_rect.top()) / plot_rect.height();

                self.x = Some(x_min + rel_x as f64 * (x_max - x_min));
                // Y is inverted (screen Y goes down, data Y goes up)
                self.y = Some(y_max - rel_y as f64 * (y_max - y_min));
                self.active = true;
            } else {
                self.active = false;
            }
        } else {
            self.active = false;
        }
    }

    /// Clear cursor position when leaving the chart area
    pub fn clear(&mut self) {
        self.x = None;
        self.y = None;
        self.active = false;
    }
}

/// Y-axis view state per panel
#[derive(Clone, Debug, Default)]
pub struct PanelYState {
    pub y_min: f64,
    pub y_max: f64,
    pub initialized: bool,
    pub dragging: bool,
    pub drag_start_y: Option<f32>,
    pub drag_start_bounds: Option<(f64, f64)>,
}

impl PanelYState {
    pub fn load(ctx: &egui::Context, id: Id) -> Self {
        ctx.data(|d| d.get_temp::<Self>(id)).unwrap_or_default()
    }

    pub fn store(self, ctx: &egui::Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    pub fn init_if_needed(&mut self, y_min: f64, y_max: f64) {
        if !self.initialized {
            self.y_min = y_min;
            self.y_max = y_max;
            self.initialized = true;
        }
    }

    pub fn update_from_plot(&mut self, plot_y_min: f64, plot_y_max: f64) {
        if !self.dragging {
            self.y_min = plot_y_min;
            self.y_max = plot_y_max;
        }
    }

    pub fn scale_y(&mut self, factor: f64) {
        let center = (self.y_min + self.y_max) / 2.0;
        let half_range = (self.y_max - self.y_min) / 2.0;
        let new_half_range = half_range * factor;
        self.y_min = center - new_half_range;
        self.y_max = center + new_half_range;
    }

    pub fn reset(&mut self, y_min: f64, y_max: f64) {
        self.y_min = y_min;
        self.y_max = y_max;
    }
}
