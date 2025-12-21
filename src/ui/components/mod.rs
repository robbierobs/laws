pub mod header;
pub mod sidebar;
pub mod resource_list;
pub mod detail_panel;
pub mod action_bar;
pub mod modals;
pub mod input;
pub mod loading;
pub mod tabs;
pub mod action_log;
pub mod table;
pub mod detail_builder;

// Re-export modal functions for backwards compatibility
// This allows existing code to use `modal::render_*` without changes
pub mod modal {
    pub use super::modals::*;
}

use ratatui::{Frame, layout::Rect};
use crossterm::event::KeyEvent;
use crate::app::Message;

pub trait Component {
    /// Render the component to the frame
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Handle keyboard input, return optional message
    fn handle_key(&mut self, key: KeyEvent) -> Option<Message>;
}
