pub mod action_bar;
pub mod action_log;
pub mod detail_builder;
pub mod detail_panel;
pub mod error_panel;
pub mod filter_modal;
pub mod header;
pub mod input;
pub mod loading;
pub mod modals;
pub mod resource_list;
pub mod sidebar;
pub mod table;
pub mod tabs;

// Re-export modal functions for backwards compatibility
// This allows existing code to use `modal::render_*` without changes
pub mod modal {
    pub use super::modals::*;
}

use crate::app::Message;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub trait Component {
    /// Render the component to the frame
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Handle keyboard input, return optional message
    fn handle_key(&mut self, key: KeyEvent) -> Option<Message>;
}
