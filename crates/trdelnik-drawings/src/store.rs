//! Drawing store for managing all drawings on a chart
//!
//! The `DrawingStore` manages the collection of drawings, their styles,
//! visibility, and selection state.

use crate::{ChartPoint, Drawing, DrawingStyle};
use std::collections::HashMap;
use std::sync::OnceLock;
use trdelnik_core::AxisCoordinate;
use uuid::Uuid;

/// Returns a static reference to a default DrawingStyle.
/// Used as a fallback when a drawing's style is not found in the store.
fn default_drawing_style() -> &'static DrawingStyle {
    static DEFAULT: OnceLock<DrawingStyle> = OnceLock::new();
    DEFAULT.get_or_init(DrawingStyle::default)
}

/// Manages all drawings for a chart
#[derive(Clone)]
pub struct DrawingStore<X: AxisCoordinate> {
    /// All drawings, ordered by z-index (first = bottom)
    drawings: Vec<Box<dyn Drawing<X>>>,
    /// Style per drawing
    styles: HashMap<Uuid, DrawingStyle>,
    /// Visibility per drawing
    visibility: HashMap<Uuid, bool>,
    /// Locked drawings (not editable)
    locked: HashMap<Uuid, bool>,
    /// Panel ID per drawing (None = main chart, Some("rsi") = RSI panel)
    panel_ids: HashMap<Uuid, String>,
    /// Currently selected drawing
    selected: Option<Uuid>,
}

impl<X: AxisCoordinate> Default for DrawingStore<X> {
    fn default() -> Self {
        Self::new()
    }
}

impl<X: AxisCoordinate> DrawingStore<X> {
    /// Create a new empty drawing store
    pub fn new() -> Self {
        Self {
            drawings: Vec::new(),
            styles: HashMap::new(),
            visibility: HashMap::new(),
            locked: HashMap::new(),
            panel_ids: HashMap::new(),
            selected: None,
        }
    }

    /// Add a drawing to the store (defaults to main panel)
    pub fn add<D: Drawing<X> + 'static>(&mut self, drawing: D) -> Uuid {
        self.add_to_panel(drawing, "main")
    }

    /// Add a drawing to a specific panel
    pub fn add_to_panel<D: Drawing<X> + 'static>(
        &mut self,
        drawing: D,
        panel_id: &str,
    ) -> Uuid {
        let id = drawing.id();
        self.drawings.push(Box::new(drawing));
        self.styles.insert(id, DrawingStyle::default());
        self.visibility.insert(id, true);
        self.panel_ids.insert(id, panel_id.to_string());
        id
    }

    /// Add a drawing with a custom style
    pub fn add_with_style<D: Drawing<X> + 'static>(
        &mut self,
        drawing: D,
        style: DrawingStyle,
    ) -> Uuid {
        let id = drawing.id();
        self.drawings.push(Box::new(drawing));
        self.styles.insert(id, style);
        self.visibility.insert(id, true);
        self.panel_ids.insert(id, "main".to_string());
        id
    }

    /// Add a boxed drawing (defaults to main panel)
    pub fn add_boxed(&mut self, drawing: Box<dyn Drawing<X>>) -> Uuid {
        self.add_boxed_to_panel(drawing, "main")
    }

    /// Add a boxed drawing to a specific panel
    pub fn add_boxed_to_panel(&mut self, drawing: Box<dyn Drawing<X>>, panel_id: &str) -> Uuid {
        let id = drawing.id();
        self.drawings.push(drawing);
        self.styles.insert(id, DrawingStyle::default());
        self.visibility.insert(id, true);
        self.panel_ids.insert(id, panel_id.to_string());
        id
    }

    /// Remove a drawing by ID
    pub fn remove(&mut self, id: Uuid) -> bool {
        let len_before = self.drawings.len();
        self.drawings.retain(|d| d.id() != id);
        self.styles.remove(&id);
        self.visibility.remove(&id);
        self.locked.remove(&id);
        self.panel_ids.remove(&id);

        if self.selected == Some(id) {
            self.selected = None;
        }

        self.drawings.len() != len_before
    }

    /// Get the panel ID for a drawing
    pub fn panel_id(&self, id: Uuid) -> Option<&str> {
        self.panel_ids.get(&id).map(|s| s.as_str())
    }

    /// Set the panel ID for a drawing
    pub fn set_panel_id(&mut self, id: Uuid, panel_id: &str) {
        self.panel_ids.insert(id, panel_id.to_string());
    }

    /// Get a drawing by ID
    pub fn get(&self, id: Uuid) -> Option<&dyn Drawing<X>> {
        self.drawings.iter().find(|d| d.id() == id).map(|d| d.as_ref())
    }

    /// Get a mutable drawing by ID
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Box<dyn Drawing<X>>> {
        self.drawings.iter_mut().find(|d| d.id() == id)
    }

    /// Get all drawings
    pub fn all(&self) -> impl Iterator<Item = &dyn Drawing<X>> {
        self.drawings.iter().map(|d| d.as_ref())
    }

    /// Get all visible drawings with their styles
    pub fn visible(&self) -> impl Iterator<Item = (&dyn Drawing<X>, &DrawingStyle)> {
        self.drawings
            .iter()
            .filter(|d| *self.visibility.get(&d.id()).unwrap_or(&true))
            .map(|d| {
                let style = self.styles.get(&d.id()).unwrap_or(default_drawing_style());
                (d.as_ref(), style)
            })
    }

    /// Get all visible drawings for a specific panel with their styles
    pub fn visible_for_panel<'a>(
        &'a self,
        panel_id: &'a str,
    ) -> impl Iterator<Item = (&'a dyn Drawing<X>, &'a DrawingStyle)> {
        self.drawings
            .iter()
            .filter(move |d| {
                let is_visible = *self.visibility.get(&d.id()).unwrap_or(&true);
                let matches_panel = self
                    .panel_ids
                    .get(&d.id())
                    .map(|p| p == panel_id)
                    .unwrap_or(panel_id == "main");
                is_visible && matches_panel
            })
            .map(|d| {
                let style = self.styles.get(&d.id()).unwrap_or(default_drawing_style());
                (d.as_ref(), style)
            })
    }

    /// Get the number of drawings
    pub fn len(&self) -> usize {
        self.drawings.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.drawings.is_empty()
    }

    /// Clear all drawings
    pub fn clear(&mut self) {
        self.drawings.clear();
        self.styles.clear();
        self.visibility.clear();
        self.locked.clear();
        self.panel_ids.clear();
        self.selected = None;
    }

    // === Selection ===

    /// Get the currently selected drawing ID
    pub fn selected(&self) -> Option<Uuid> {
        self.selected
    }

    /// Get the selected drawing
    pub fn selected_drawing(&self) -> Option<&dyn Drawing<X>> {
        self.selected.and_then(|id| self.get(id))
    }

    /// Get the selected drawing mutably
    pub fn selected_drawing_mut(&mut self) -> Option<&mut Box<dyn Drawing<X>>> {
        let id = self.selected?;
        self.get_mut(id)
    }

    /// Select a drawing by ID
    pub fn select(&mut self, id: Option<Uuid>) {
        self.selected = id;
    }

    /// Select a drawing at the given point (hit test)
    pub fn select_at(&mut self, point: &ChartPoint<X>, tolerance: f64) -> Option<Uuid> {
        let id = self.hit_test(point, tolerance);
        self.selected = id;
        id
    }

    /// Deselect all drawings
    pub fn deselect(&mut self) {
        self.selected = None;
    }

    // === Hit Testing ===

    /// Find the topmost drawing at the given point
    pub fn hit_test(&self, point: &ChartPoint<X>, tolerance: f64) -> Option<Uuid> {
        // Iterate from top (last) to bottom (first)
        for drawing in self.drawings.iter().rev() {
            if *self.visibility.get(&drawing.id()).unwrap_or(&true) {
                if drawing.hit_test(point, tolerance) {
                    return Some(drawing.id());
                }
            }
        }
        None
    }

    /// Find all drawings at the given point
    pub fn hit_test_all(&self, point: &ChartPoint<X>, tolerance: f64) -> Vec<Uuid> {
        self.drawings
            .iter()
            .filter(|d| {
                *self.visibility.get(&d.id()).unwrap_or(&true) && d.hit_test(point, tolerance)
            })
            .map(|d| d.id())
            .collect()
    }

    // === Styles ===

    /// Get the style for a drawing
    pub fn style(&self, id: Uuid) -> Option<&DrawingStyle> {
        self.styles.get(&id)
    }

    /// Get mutable style for a drawing
    pub fn style_mut(&mut self, id: Uuid) -> Option<&mut DrawingStyle> {
        self.styles.get_mut(&id)
    }

    /// Set the style for a drawing
    pub fn set_style(&mut self, id: Uuid, style: DrawingStyle) {
        self.styles.insert(id, style);
    }

    // === Visibility ===

    /// Check if a drawing is visible
    pub fn is_visible(&self, id: Uuid) -> bool {
        *self.visibility.get(&id).unwrap_or(&true)
    }

    /// Set visibility for a drawing
    pub fn set_visible(&mut self, id: Uuid, visible: bool) {
        self.visibility.insert(id, visible);
    }

    /// Toggle visibility for a drawing
    pub fn toggle_visible(&mut self, id: Uuid) -> bool {
        let visible = !self.is_visible(id);
        self.visibility.insert(id, visible);
        visible
    }

    /// Hide all drawings
    pub fn hide_all(&mut self) {
        for id in self.drawings.iter().map(|d| d.id()) {
            self.visibility.insert(id, false);
        }
    }

    /// Show all drawings
    pub fn show_all(&mut self) {
        for id in self.drawings.iter().map(|d| d.id()) {
            self.visibility.insert(id, true);
        }
    }

    // === Locking ===

    /// Check if a drawing is locked
    pub fn is_locked(&self, id: Uuid) -> bool {
        *self.locked.get(&id).unwrap_or(&false)
    }

    /// Set locked state for a drawing
    pub fn set_locked(&mut self, id: Uuid, locked: bool) {
        self.locked.insert(id, locked);
    }

    /// Toggle locked state for a drawing
    pub fn toggle_locked(&mut self, id: Uuid) -> bool {
        let locked = !self.is_locked(id);
        self.locked.insert(id, locked);
        locked
    }

    // === Z-Order ===

    /// Bring a drawing to the front (top)
    pub fn bring_to_front(&mut self, id: Uuid) {
        if let Some(idx) = self.drawings.iter().position(|d| d.id() == id) {
            let drawing = self.drawings.remove(idx);
            self.drawings.push(drawing);
        }
    }

    /// Send a drawing to the back (bottom)
    pub fn send_to_back(&mut self, id: Uuid) {
        if let Some(idx) = self.drawings.iter().position(|d| d.id() == id) {
            let drawing = self.drawings.remove(idx);
            self.drawings.insert(0, drawing);
        }
    }

    /// Move a drawing up one layer
    pub fn move_up(&mut self, id: Uuid) {
        if let Some(idx) = self.drawings.iter().position(|d| d.id() == id) {
            if idx < self.drawings.len() - 1 {
                self.drawings.swap(idx, idx + 1);
            }
        }
    }

    /// Move a drawing down one layer
    pub fn move_down(&mut self, id: Uuid) {
        if let Some(idx) = self.drawings.iter().position(|d| d.id() == id) {
            if idx > 0 {
                self.drawings.swap(idx, idx - 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawings::HorizontalLine;
    use trdelnik_core::Index;

    #[test]
    fn test_drawing_store() {
        let mut store: DrawingStore<Index> = DrawingStore::new();
        assert!(store.is_empty());

        let line = HorizontalLine::new(100.0);
        let id = store.add(line);

        assert_eq!(store.len(), 1);
        assert!(store.get(id).is_some());
        assert!(store.is_visible(id));
    }

    #[test]
    fn test_selection() {
        let mut store: DrawingStore<Index> = DrawingStore::new();

        let line1 = HorizontalLine::new(100.0);
        let line2 = HorizontalLine::new(200.0);

        let id1 = store.add(line1);
        let id2 = store.add(line2);

        assert!(store.selected().is_none());

        store.select(Some(id1));
        assert_eq!(store.selected(), Some(id1));

        store.select(Some(id2));
        assert_eq!(store.selected(), Some(id2));

        store.deselect();
        assert!(store.selected().is_none());
    }

    #[test]
    fn test_z_order() {
        let mut store: DrawingStore<Index> = DrawingStore::new();

        let line1 = HorizontalLine::new(100.0);
        let line2 = HorizontalLine::new(200.0);
        let line3 = HorizontalLine::new(300.0);

        let id1 = store.add(line1);
        let _id2 = store.add(line2);
        let id3 = store.add(line3);

        // Initially: [1, 2, 3]
        assert_eq!(store.drawings[0].id(), id1);

        // Bring 1 to front: [2, 3, 1]
        store.bring_to_front(id1);
        assert_eq!(store.drawings.last().unwrap().id(), id1);

        // Send 3 to back: [3, 2, 1]
        store.send_to_back(id3);
        assert_eq!(store.drawings[0].id(), id3);
    }
}
