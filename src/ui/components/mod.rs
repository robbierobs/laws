pub mod header;
pub mod sidebar;
pub mod resource_list;
pub mod detail_panel;
pub mod action_bar;
pub mod modal;
pub mod input;
pub mod loading;

use ratatui::{Frame, layout::Rect};
use crossterm::event::KeyEvent;
use crate::app::Message;

pub trait Component {
    /// Render the component to the frame
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Handle keyboard input, return optional message
    fn handle_key(&mut self, key: KeyEvent) -> Option<Message>;

    /// Check if component can handle input (is focused)
    fn is_focused(&self) -> bool { false }
}

pub trait StatefulComponent {
    type State;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State);
    fn handle_key(&mut self, key: KeyEvent, state: &mut Self::State) -> Option<Message>;
}
