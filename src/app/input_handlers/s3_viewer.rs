//! S3 Object Viewer modal input handling

use crate::app::states::s3::S3State;
use crate::app::Message;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard input for the S3 object viewer modal.
///
/// Returns `None` - the event is always consumed when the viewer is open.
pub fn handle_s3_viewer_input(s3_state: &mut S3State, key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            // Close the viewer and clear state
            s3_state.show_object_viewer = false;
            s3_state.opened_object_content = None;
            s3_state.opened_object_path = None;
            s3_state.opened_object_key = None;
            s3_state.opened_object_bytes = None;
            s3_state.viewer_scroll_offset = 0;
            s3_state.viewer_mode = crate::app::states::s3::ViewerMode::Text;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            s3_state.viewer_scroll_offset = s3_state.viewer_scroll_offset.saturating_add(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            s3_state.viewer_scroll_offset = s3_state.viewer_scroll_offset.saturating_sub(1);
        }
        KeyCode::Char('g') | KeyCode::Home => {
            s3_state.viewer_scroll_offset = 0;
        }
        KeyCode::Char('G') | KeyCode::End => {
            // Scroll to end - approximate based on content length
            if let Some(content) = &s3_state.opened_object_content {
                let line_count = content.lines().count() as u16;
                s3_state.viewer_scroll_offset = line_count.saturating_sub(10);
            }
        }
        KeyCode::PageDown => {
            s3_state.viewer_scroll_offset = s3_state.viewer_scroll_offset.saturating_add(20);
        }
        KeyCode::PageUp => {
            s3_state.viewer_scroll_offset = s3_state.viewer_scroll_offset.saturating_sub(20);
        }
        KeyCode::Tab => {
            // Toggle between Text and Hex view
            s3_state.toggle_viewer_mode();
            s3_state.viewer_scroll_offset = 0;
        }
        _ => {}
    }
    None // Event consumed, no message to send
}
