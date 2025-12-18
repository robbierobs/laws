//! Profile Switcher modal input handling

use crate::app::profile_switcher::ProfileSwitcherState;

use crossterm::event::{KeyCode, KeyEvent};

/// Result of handling Profile Switcher input
pub enum ProfileSwitcherResult {
    /// Stay in modal, no message
    Continue,
    /// Exit modal (cancelled)
    Cancel,
    /// Profile selected, should transition to Region selection
    ProfileSelected,
    /// Region selected, should perform the switch
    Switch {
        profile: Option<String>,
        region: String,
        read_only: bool,
    },
}

/// Handle input for the Profile selection phase
pub fn handle_profile_selection_input(
    state: &mut ProfileSwitcherState,
    key: KeyEvent,
) -> ProfileSwitcherResult {
    // If filter is active, handle text input
    if state.profile_filter_active {
        match key.code {
            KeyCode::Esc => {
                state.profile_filter_active = false;
                state.profile_filter.clear();
                state.profile_switcher_index = 0;
            }
            KeyCode::Enter => {
                state.profile_filter_active = false;
            }
            KeyCode::Backspace => {
                state.profile_filter.pop();
                state.profile_switcher_index = 0;
            }
            KeyCode::Char(c) => {
                state.profile_filter.push(c);
                state.profile_switcher_index = 0;
            }
            _ => {}
        }
        return ProfileSwitcherResult::Continue;
    }

    // Normal navigation mode
    match key.code {
        KeyCode::Esc => {
            state.profile_filter.clear();
            ProfileSwitcherResult::Cancel
        }
        KeyCode::Char('/') => {
            state.profile_filter_active = true;
            ProfileSwitcherResult::Continue
        }
        KeyCode::Char('R') => {
            // Toggle read-only mode
            state.pending_read_only = !state.pending_read_only;
            ProfileSwitcherResult::Continue
        }
        KeyCode::Enter => {
            // Store selected profile and move to region selection
            let filtered = state.filtered_profiles();
            let selected = filtered
                .get(state.profile_switcher_index)
                .map(|s| {
                    if *s == "default" {
                        None
                    } else {
                        Some((*s).clone())
                    }
                })
                .unwrap_or(None);
            state.pending_profile = selected;
            state.profile_filter.clear();
            
            // Pre-select current region in region list
            // Note: Caller (App) needs to pass in current region to do this strictly,
            // but we can just reset to 0 or handle it elsewhere.
            // In the original code, it checked against `self.region`.
            // We'll leave the region index logic to the state initialization or caller for now,
            // or we could add a method to state to "prepare_region_selection(current_region)".
            // For now, let's return ProfileSelected and let input.rs handle the transition logic if needed,
            // or just assume index 0 which is safe.
            // Actually, we can't see `app.region` here easily.
            // But we already initialized region_switcher_index in `new`.
            // If the user hasn't changed it, it remains correct.
            
            ProfileSwitcherResult::ProfileSelected
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.nav_down_profile();
            ProfileSwitcherResult::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.nav_up_profile();
            ProfileSwitcherResult::Continue
        }
        _ => ProfileSwitcherResult::Continue,
    }
}

/// Handle input for the Region selection phase
pub fn handle_region_selection_input(
    state: &mut ProfileSwitcherState,
    key: KeyEvent,
) -> ProfileSwitcherResult {
    // If filter is active, handle text input
    if state.region_filter_active {
        match key.code {
            KeyCode::Esc => {
                state.region_filter_active = false;
                state.region_filter.clear();
                state.region_switcher_index = 0;
            }
            KeyCode::Enter => {
                state.region_filter_active = false;
            }
            KeyCode::Backspace => {
                state.region_filter.pop();
                state.region_switcher_index = 0;
            }
            KeyCode::Char(c) => {
                state.region_filter.push(c);
                state.region_switcher_index = 0;
            }
            _ => {}
        }
        return ProfileSwitcherResult::Continue;
    }

    // Normal navigation mode
    match key.code {
        KeyCode::Esc => {
            // Cancel and go back to normal mode
            state.pending_profile = None;
            state.region_filter.clear();
            ProfileSwitcherResult::Cancel
        }
        KeyCode::Char('/') => {
            state.region_filter_active = true;
            ProfileSwitcherResult::Continue
        }
        KeyCode::Enter => {
            // Confirm and switch profile/region
            let profile = state.pending_profile.clone();
            let filtered = state.filtered_regions();
            let region = filtered
                .get(state.region_switcher_index)
                .cloned()
                .cloned()
                .unwrap_or_else(|| "us-east-1".to_string());
            let read_only = state.pending_read_only;
            state.pending_profile = None;
            state.region_filter.clear();
            
            ProfileSwitcherResult::Switch {
                profile,
                region,
                read_only,
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.nav_down_region();
            ProfileSwitcherResult::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.nav_up_region();
            ProfileSwitcherResult::Continue
        }
        _ => ProfileSwitcherResult::Continue,
    }
}
