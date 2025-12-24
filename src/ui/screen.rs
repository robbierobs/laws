//! Screen trait for service-specific rendering
//!
//! This trait enables polymorphic dispatch for service screens.

use ratatui::{layout::Rect, Frame};
use crate::app::App;

/// Trait for service screens that can render a list and optional detail panel
pub trait Screen {
    /// Render the screen content
    ///
    /// # Arguments
    /// * `frame` - The frame to render to
    /// * `list_area` - Optional area for the main list/table view (None in fullscreen detail mode)
    /// * `detail_area` - Optional area for detail panel
    /// * `app` - Mutable reference to app state (for TableState updates)
    fn render(&self, frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App);
}
