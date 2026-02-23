//! Panel height state management
//!
//! Manages the heights of resizable panels and tracks resize interactions.

use std::collections::HashMap;

use egui::{Context, Id};

/// State for managing panel heights
#[derive(Debug, Clone, Default)]
pub struct PanelHeightState {
    /// Current heights for each panel by ID
    pub heights: HashMap<String, f32>,
    /// Panel ID currently being resized (if any)
    pub resizing: Option<String>,
    /// Y position where resize started
    pub resize_start_y: Option<f32>,
    /// Height when resize started
    pub resize_start_height: Option<f32>,
    /// Start heights for dual resize (above, below)
    resize_start_heights: Option<(f32, f32)>,
}

impl PanelHeightState {
    /// Create a new panel height state
    pub fn new() -> Self {
        Self::default()
    }

    /// Load panel height state from egui context
    pub fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|d| d.get_temp(id).unwrap_or_default())
    }

    /// Store panel height state to egui context
    pub fn store(&self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self.clone()));
    }

    /// Get the height for a panel, or return the default
    pub fn get_height(&self, panel_id: &str, default_height: f32) -> f32 {
        self.heights
            .get(panel_id)
            .copied()
            .unwrap_or(default_height)
    }

    /// Set the height for a panel
    pub fn set_height(&mut self, panel_id: impl Into<String>, height: f32) {
        self.heights.insert(panel_id.into(), height);
    }

    /// Start resizing a panel
    pub fn start_resize(&mut self, panel_id: impl Into<String>, start_y: f32, current_height: f32) {
        self.resizing = Some(panel_id.into());
        self.resize_start_y = Some(start_y);
        self.resize_start_height = Some(current_height);
    }

    /// Update resize in progress
    ///
    /// Returns the new height if resizing is active.
    /// The handle is positioned ABOVE the panel, so dragging down (positive delta)
    /// should DECREASE the panel height (top boundary moves down).
    pub fn update_resize(&mut self, current_y: f32, min_height: f32, max_height: f32) -> Option<f32> {
        if let (Some(panel_id), Some(start_y), Some(start_height)) = (
            self.resizing.as_ref(),
            self.resize_start_y,
            self.resize_start_height,
        ) {
            let delta = current_y - start_y;
            // Negate delta: drag down = smaller panel, drag up = larger panel
            let new_height = (start_height - delta).clamp(min_height, max_height);
            self.heights.insert(panel_id.clone(), new_height);
            Some(new_height)
        } else {
            None
        }
    }

    /// End resizing
    pub fn end_resize(&mut self) {
        self.resizing = None;
        self.resize_start_y = None;
        self.resize_start_height = None;
        self.resize_start_heights = None;
    }

    /// Check if currently resizing
    pub fn is_resizing(&self) -> bool {
        self.resizing.is_some()
    }

    /// Check if a specific panel is being resized
    pub fn is_resizing_panel(&self, panel_id: &str) -> bool {
        self.resizing.as_deref() == Some(panel_id)
    }

    /// Check if either of two panels is being resized (for dual-panel handles)
    pub fn is_resizing_either(&self, panel_above: &str, panel_below: &str) -> bool {
        match self.resizing.as_deref() {
            Some(id) => id == panel_above || id == panel_below,
            None => false,
        }
    }

    /// Start resizing two adjacent panels
    pub fn start_resize_dual(
        &mut self,
        panel_above: impl Into<String>,
        panel_below: impl Into<String>,
        start_y: f32,
        height_above: f32,
        height_below: f32,
    ) {
        let above = panel_above.into();
        let below = panel_below.into();
        // Store both panel IDs joined with a separator
        self.resizing = Some(format!("{}|{}", above, below));
        self.resize_start_y = Some(start_y);
        // Store combined height as reference
        self.resize_start_height = Some(height_above + height_below);
        // Store individual start heights separately (not in the heights map)
        self.resize_start_heights = Some((height_above, height_below));
    }

    /// Update resize for two adjacent panels
    /// Dragging down increases the panel above, decreases the panel below
    pub fn update_resize_dual(
        &mut self,
        current_y: f32,
        min_height_above: f32,
        max_height_above: f32,
        min_height_below: f32,
        max_height_below: f32,
    ) -> Option<(f32, f32)> {
        let (panel_above, panel_below) = self.get_dual_panels()?;
        let start_y = self.resize_start_y?;
        let (start_height_above, start_height_below) = self.resize_start_heights?;

        let delta = current_y - start_y;

        // Drag down = panel above grows, panel below shrinks
        let mut new_height_above = start_height_above + delta;
        let mut new_height_below = start_height_below - delta;

        // Clamp both heights
        new_height_above = new_height_above.clamp(min_height_above, max_height_above);
        new_height_below = new_height_below.clamp(min_height_below, max_height_below);

        // Recalculate to ensure constraints are respected
        let total = start_height_above + start_height_below;
        if (new_height_above + new_height_below - total).abs() > 0.1 {
            // Adjust based on which hit a limit
            if new_height_above == min_height_above || new_height_above == max_height_above {
                new_height_below = total - new_height_above;
            } else {
                new_height_above = total - new_height_below;
            }
        }

        self.heights.insert(panel_above, new_height_above);
        self.heights.insert(panel_below, new_height_below);

        Some((new_height_above, new_height_below))
    }

    /// Get the two panel IDs being resized
    fn get_dual_panels(&self) -> Option<(String, String)> {
        let resizing = self.resizing.as_ref()?;
        let parts: Vec<&str> = resizing.split('|').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// Check if a dual resize is in progress involving these panels
    pub fn is_resizing_dual(&self, panel_above: &str, panel_below: &str) -> bool {
        if let Some((above, below)) = self.get_dual_panels() {
            above == panel_above && below == panel_below
        } else {
            false
        }
    }

    /// Apply a resize delta to two adjacent panels
    /// This is simpler than the start/update/end pattern - just applies the delta directly
    /// Positive delta = panel above grows, panel below shrinks
    pub fn apply_resize_delta(
        &mut self,
        panel_above: &str,
        panel_below: &str,
        delta: f32,
        min_height_above: f32,
        max_height_above: f32,
        min_height_below: f32,
        max_height_below: f32,
    ) {
        let current_above = self.get_height(panel_above, 100.0);
        let current_below = self.get_height(panel_below, 100.0);

        // Calculate how much we can actually move based on constraints
        let effective_delta = if delta > 0.0 {
            // Dragging down: above grows, below shrinks
            // Limited by: above's max, below's min
            let max_grow_above = max_height_above - current_above;
            let max_shrink_below = current_below - min_height_below;
            delta.min(max_grow_above).min(max_shrink_below)
        } else {
            // Dragging up: above shrinks, below grows
            // Limited by: above's min, below's max
            let max_shrink_above = current_above - min_height_above;
            let max_grow_below = max_height_below - current_below;
            // delta is negative, so we need to handle signs carefully
            let max_negative_delta = -max_shrink_above.min(max_grow_below);
            delta.max(max_negative_delta)
        };

        let new_above = current_above + effective_delta;
        let new_below = current_below - effective_delta;

        // Mark as resizing (for is_resizing() check)
        self.resizing = Some(format!("{}|{}", panel_above, panel_below));

        self.heights.insert(panel_above.to_string(), new_above);
        self.heights.insert(panel_below.to_string(), new_below);
    }
}
