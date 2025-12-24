//! Action Log modal input handling

use crossterm::event::{KeyCode, KeyEvent};

/// State needed for action log input handling
pub struct ActionLogState<'a> {
    pub expanded: &'a mut bool,
    pub selected_index: &'a mut usize,
    pub detail_scroll: &'a mut u16,
    pub log_len: usize,
}

/// Handle keyboard input for the action log popup.
/// Returns true if the popup should close, false to stay open.
pub fn handle_action_log_input(state: &mut ActionLogState<'_>, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('A') | KeyCode::Esc => {
            *state.expanded = false;
            *state.selected_index = 0;
            *state.detail_scroll = 0;
            true // Popup closed
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let max_idx = state.log_len.saturating_sub(1);
            if *state.selected_index < max_idx {
                *state.selected_index += 1;
                *state.detail_scroll = 0; // Reset scroll on selection change
            }
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if *state.selected_index > 0 {
                *state.selected_index -= 1;
                *state.detail_scroll = 0; // Reset scroll on selection change
            }
            false
        }
        KeyCode::PageDown => {
            *state.detail_scroll = state.detail_scroll.saturating_add(10);
            false
        }
        KeyCode::PageUp => {
            *state.detail_scroll = state.detail_scroll.saturating_sub(10);
            false
        }
        KeyCode::Home | KeyCode::Char('g') => {
            *state.detail_scroll = 0;
            false
        }
        KeyCode::Char('G') | KeyCode::End => {
            // Large value to scroll to end
            *state.detail_scroll = u16::MAX;
            false
        }
        _ => false,
    }
}
