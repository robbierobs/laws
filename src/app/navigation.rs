//! Navigation helpers for list-based UI components
//!
//! This module provides reusable navigation functions to reduce code duplication
//! across service state handlers. Instead of each service implementing its own
//! up/down/wrap-around logic, they can use these helpers.

use ratatui::widgets::TableState;

/// Navigate up in a list with wrap-around
/// 
/// Returns the new index, or None if the list is empty.
#[inline]
pub fn nav_up(current: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(current.map_or(0, |i| {
        if i == 0 { len - 1 } else { i - 1 }
    }))
}

/// Navigate down in a list with wrap-around
/// 
/// Returns the new index, or None if the list is empty.
#[inline]
pub fn nav_down(current: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(current.map_or(0, |i| {
        if i >= len - 1 { 0 } else { i + 1 }
    }))
}

/// Navigate to first item in a list
#[inline]
#[allow(dead_code)] // Part of navigation API for completeness
pub fn nav_first(len: usize) -> Option<usize> {
    if len == 0 { None } else { Some(0) }
}

/// Navigate to last item in a list
#[inline]
#[allow(dead_code)] // Part of navigation API for completeness
pub fn nav_last(len: usize) -> Option<usize> {
    if len == 0 { None } else { Some(len - 1) }
}

/// Extension trait for TableState to add navigation helpers
#[allow(dead_code)] // Some methods are for API completeness
pub trait TableStateExt {
    /// Navigate up with wrap-around
    fn nav_up(&mut self, len: usize);
    /// Navigate down with wrap-around
    fn nav_down(&mut self, len: usize);
    /// Navigate to first item
    fn nav_first(&mut self, len: usize);
    /// Navigate to last item
    fn nav_last(&mut self, len: usize);
    /// Ensure selection is valid for the current list length
    fn clamp_selection(&mut self, len: usize);
}

impl TableStateExt for TableState {
    fn nav_up(&mut self, len: usize) {
        if let Some(new_idx) = nav_up(self.selected(), len) {
            self.select(Some(new_idx));
        }
    }

    fn nav_down(&mut self, len: usize) {
        if let Some(new_idx) = nav_down(self.selected(), len) {
            self.select(Some(new_idx));
        }
    }

    fn nav_first(&mut self, len: usize) {
        if let Some(new_idx) = nav_first(len) {
            self.select(Some(new_idx));
        }
    }

    fn nav_last(&mut self, len: usize) {
        if let Some(new_idx) = nav_last(len) {
            self.select(Some(new_idx));
        }
    }

    fn clamp_selection(&mut self, len: usize) {
        if len == 0 {
            self.select(None);
        } else if let Some(i) = self.selected() {
            if i >= len {
                self.select(Some(len - 1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_up_empty() {
        assert_eq!(nav_up(None, 0), None);
        assert_eq!(nav_up(Some(0), 0), None);
    }

    #[test]
    fn test_nav_up_single() {
        assert_eq!(nav_up(None, 1), Some(0));
        assert_eq!(nav_up(Some(0), 1), Some(0));
    }

    #[test]
    fn test_nav_up_wrap() {
        assert_eq!(nav_up(Some(0), 5), Some(4));
        assert_eq!(nav_up(Some(2), 5), Some(1));
    }

    #[test]
    fn test_nav_down_empty() {
        assert_eq!(nav_down(None, 0), None);
        assert_eq!(nav_down(Some(0), 0), None);
    }

    #[test]
    fn test_nav_down_single() {
        assert_eq!(nav_down(None, 1), Some(0));
        assert_eq!(nav_down(Some(0), 1), Some(0));
    }

    #[test]
    fn test_nav_down_wrap() {
        assert_eq!(nav_down(Some(4), 5), Some(0));
        assert_eq!(nav_down(Some(2), 5), Some(3));
    }

    #[test]
    fn test_table_state_ext() {
        let mut state = TableState::default();
        
        // Empty list
        state.nav_down(0);
        assert_eq!(state.selected(), None);
        
        // Navigate in list of 3
        state.nav_down(3);
        assert_eq!(state.selected(), Some(0));
        
        state.nav_down(3);
        assert_eq!(state.selected(), Some(1));
        
        state.nav_up(3);
        assert_eq!(state.selected(), Some(0));
        
        // Wrap
        state.nav_up(3);
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn test_clamp_selection() {
        let mut state = TableState::default();
        state.select(Some(10));
        
        state.clamp_selection(5);
        assert_eq!(state.selected(), Some(4));
        
        state.clamp_selection(0);
        assert_eq!(state.selected(), None);
    }
}
