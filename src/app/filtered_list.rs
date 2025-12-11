//! FilteredList - Generic filtered list with caching for optimized rendering
//!
//! This module provides a `FilteredList<T>` that caches filtered indices to avoid
//! recomputing filters on every render frame. The cache is invalidated when:
//! - The filter text changes
//! - The underlying items change (via `set_items`)
//!
//! # Example
//! ```ignore
//! let mut list = FilteredList::new();
//! list.set_items(vec!["apple", "banana", "cherry"]);
//! list.set_filter("an", |item| item.to_lowercase());
//! assert_eq!(list.filtered_count(), 2); // "banana" and "cherry" (via 'an')
//! ```

#![allow(dead_code)]

use ratatui::widgets::ListState;

/// A list with built-in filtering and selection state
#[derive(Debug, Clone)]
pub struct FilteredList<T> {
    /// All items in the list
    items: Vec<T>,
    
    /// Current filter text (lowercased for case-insensitive matching)
    filter: String,
    
    /// Cached indices of items that match the current filter
    /// When filter is empty, this contains all indices [0, 1, 2, ...]
    filtered_indices: Vec<usize>,
    
    /// Selection state for ratatui widgets
    pub list_state: ListState,
    
    /// Flag to track if cache needs rebuilding
    cache_dirty: bool,
}

impl<T> Default for FilteredList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FilteredList<T> {
    /// Create a new empty FilteredList
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            filter: String::new(),
            filtered_indices: Vec::new(),
            list_state: ListState::default(),
            cache_dirty: true,
        }
    }
    
    /// Create a new FilteredList with initial items
    pub fn with_items(items: Vec<T>) -> Self {
        let mut list = Self::new();
        list.set_items(items);
        list
    }
    
    /// Set items and invalidate the filter cache
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.cache_dirty = true;
        // Reset selection if out of bounds
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.items.len() {
                self.list_state.select(if self.items.is_empty() { None } else { Some(0) });
            }
        }
    }
    
    /// Get a reference to all items
    pub fn items(&self) -> &[T] {
        &self.items
    }
    
    /// Get a mutable reference to all items
    /// Note: You must call `invalidate()` after modifying items
    pub fn items_mut(&mut self) -> &mut Vec<T> {
        self.cache_dirty = true;
        &mut self.items
    }
    
    /// Get the number of items (unfiltered)
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    /// Get the current filter text
    pub fn filter(&self) -> &str {
        &self.filter
    }
    
    /// Set the filter text and invalidate cache if changed
    /// The filter is automatically lowercased for case-insensitive matching
    pub fn set_filter(&mut self, filter: &str) {
        let filter_lower = filter.to_lowercase();
        if self.filter != filter_lower {
            self.filter = filter_lower;
            self.cache_dirty = true;
        }
    }
    
    /// Clear the filter
    pub fn clear_filter(&mut self) {
        if !self.filter.is_empty() {
            self.filter.clear();
            self.cache_dirty = true;
        }
    }
    
    /// Manually invalidate the cache (call after modifying items)
    pub fn invalidate(&mut self) {
        self.cache_dirty = true;
    }
    
    /// Rebuild the filter cache using the provided filter function
    /// The filter function should return a searchable string for each item
    pub fn rebuild_cache<F>(&mut self, filter_fn: F)
    where
        F: Fn(&T) -> String,
    {
        if !self.cache_dirty {
            return;
        }
        
        self.filtered_indices.clear();
        
        if self.filter.is_empty() {
            // No filter - include all indices
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            // Filter items using the provided function
            for (idx, item) in self.items.iter().enumerate() {
                let searchable = filter_fn(item).to_lowercase();
                if searchable.contains(&self.filter) {
                    self.filtered_indices.push(idx);
                }
            }
        }
        
        // Adjust selection if it's now out of bounds
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.filtered_indices.len() {
                self.list_state.select(if self.filtered_indices.is_empty() { 
                    None 
                } else { 
                    Some(0) 
                });
            }
        }
        
        self.cache_dirty = false;
    }
    
    /// Get the number of filtered items
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }
    
    /// Get the filtered indices
    pub fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }
    
    /// Iterate over filtered items
    pub fn filtered_items(&self) -> impl Iterator<Item = &T> {
        self.filtered_indices.iter().map(move |&idx| &self.items[idx])
    }
    
    /// Get the currently selected item (if any) from the filtered list
    pub fn selected_item(&self) -> Option<&T> {
        self.list_state
            .selected()
            .and_then(|sel_idx| self.filtered_indices.get(sel_idx))
            .and_then(|&item_idx| self.items.get(item_idx))
    }
    
    /// Get the real index (into `items`) of the currently selected filtered item
    pub fn selected_item_index(&self) -> Option<usize> {
        self.list_state
            .selected()
            .and_then(|sel_idx| self.filtered_indices.get(sel_idx))
            .copied()
    }
    
    /// Select the next item in the filtered list
    pub fn select_next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current >= self.filtered_indices.len() - 1 {
            0 // Wrap to beginning
        } else {
            current + 1
        };
        self.list_state.select(Some(next));
    }
    
    /// Select the previous item in the filtered list
    pub fn select_previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        
        let current = self.list_state.selected().unwrap_or(0);
        let prev = if current == 0 {
            self.filtered_indices.len() - 1 // Wrap to end
        } else {
            current - 1
        };
        self.list_state.select(Some(prev));
    }
    
    /// Select the first item in the filtered list
    pub fn select_first(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
        }
    }
    
    /// Select the last item in the filtered list
    pub fn select_last(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(self.filtered_indices.len() - 1));
        }
    }
    
    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.list_state.select(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_empty() {
        let list: FilteredList<String> = FilteredList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.filtered_count(), 0);
    }
    
    #[test]
    fn test_set_items() {
        let mut list = FilteredList::new();
        list.set_items(vec!["apple", "banana", "cherry"]);
        
        // Need to rebuild cache before filtered_count is accurate
        list.rebuild_cache(|item| item.to_string());
        
        assert_eq!(list.len(), 3);
        assert_eq!(list.filtered_count(), 3);
    }
    
    #[test]
    fn test_filter_items() {
        let mut list = FilteredList::with_items(vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Apricot".to_string(),
            "Cherry".to_string(),
        ]);
        
        list.set_filter("ap");
        list.rebuild_cache(|item| item.clone());
        
        assert_eq!(list.filtered_count(), 2); // Apple, Apricot
        
        let filtered: Vec<_> = list.filtered_items().collect();
        assert_eq!(filtered[0], "Apple");
        assert_eq!(filtered[1], "Apricot");
    }
    
    #[test]
    fn test_case_insensitive_filter() {
        let mut list = FilteredList::with_items(vec![
            "Apple".to_string(),
            "BANANA".to_string(),
            "cherry".to_string(),
        ]);
        
        list.set_filter("AN");
        list.rebuild_cache(|item| item.clone());
        
        assert_eq!(list.filtered_count(), 1); // BANANA
        assert_eq!(list.filtered_items().next().unwrap(), "BANANA");
    }
    
    #[test]
    fn test_clear_filter() {
        let mut list = FilteredList::with_items(vec!["a", "b", "c"]);
        
        list.set_filter("a");
        list.rebuild_cache(|item| item.to_string());
        assert_eq!(list.filtered_count(), 1);
        
        list.clear_filter();
        list.rebuild_cache(|item| item.to_string());
        assert_eq!(list.filtered_count(), 3);
    }
    
    #[test]
    fn test_selection_navigation() {
        let mut list = FilteredList::with_items(vec!["a", "b", "c", "d"]);
        list.rebuild_cache(|item| item.to_string());
        
        list.select_first();
        assert_eq!(list.selected_item(), Some(&"a"));
        
        list.select_next();
        assert_eq!(list.selected_item(), Some(&"b"));
        
        list.select_last();
        assert_eq!(list.selected_item(), Some(&"d"));
        
        list.select_previous();
        assert_eq!(list.selected_item(), Some(&"c"));
    }
    
    #[test]
    fn test_selection_wrapping() {
        let mut list = FilteredList::with_items(vec!["a", "b", "c"]);
        list.rebuild_cache(|item| item.to_string());
        
        list.select_last();
        list.select_next();
        assert_eq!(list.selected_item(), Some(&"a")); // Wrapped to beginning
        
        list.select_first();
        list.select_previous();
        assert_eq!(list.selected_item(), Some(&"c")); // Wrapped to end
    }
    
    #[test]
    fn test_selected_item_index() {
        let mut list = FilteredList::with_items(vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
        ]);
        
        list.set_filter("an");
        list.rebuild_cache(|item| item.clone());
        
        list.select_first();
        // "Banana" is at real index 1
        assert_eq!(list.selected_item_index(), Some(1));
    }
}
