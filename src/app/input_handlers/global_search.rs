//! Global Search modal input handling

use crate::app::global_search::GlobalSearchState;
use crate::app::{Message, Service};
use crossterm::event::{KeyCode, KeyEvent};

/// Result of handling Global Search input
pub enum GlobalSearchInputResult {
    /// Stay in modal, no message
    Continue,
    /// Exit modal, no message (cancelled)
    Cancel,
    /// Exit modal with goto search result message
    Select(Service, String),
    /// Query changed, caller should refresh search results
    QueryChanged,
}

/// Handle keyboard input for the Global Search modal.
pub fn handle_global_search_input(
    search_state: &mut GlobalSearchState,
    key: KeyEvent,
) -> GlobalSearchInputResult {
    match key.code {
        KeyCode::Esc => {
            search_state.clear();
            GlobalSearchInputResult::Cancel
        }
        KeyCode::Down | KeyCode::Char('j') 
            if key.modifiers.is_empty() || search_state.query.is_empty() => 
        {
            search_state.nav_down();
            GlobalSearchInputResult::Continue
        }
        KeyCode::Up | KeyCode::Char('k') 
            if key.modifiers.is_empty() || search_state.query.is_empty() => 
        {
            search_state.nav_up();
            GlobalSearchInputResult::Continue
        }
        KeyCode::Enter => {
            if let Some(result) = search_state.selected() {
                let service = result.service;
                let resource_id = result.primary_id.clone();
                search_state.clear();
                GlobalSearchInputResult::Select(service, resource_id)
            } else {
                GlobalSearchInputResult::Continue
            }
        }
        KeyCode::Backspace => {
            search_state.query.pop();
            GlobalSearchInputResult::QueryChanged
        }
        KeyCode::Char(c) => {
            search_state.query.push(c);
            GlobalSearchInputResult::QueryChanged
        }
        _ => GlobalSearchInputResult::Continue,
    }
}
