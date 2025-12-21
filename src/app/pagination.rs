//! Generic pagination wrapper for service lists
//!
//! Provides consistent pagination handling across services that support
//! incremental loading (ECR images, CloudTrail events, etc.)

/// Generic pagination state for any service
///
/// Encapsulates the common pattern of:
/// - A list of items
/// - A token for fetching the next page
/// - Loading state
///
/// # Example
/// ```ignore
/// let mut images: PaginatedList<EcrImage> = PaginatedList::new();
/// images.replace(vec![img1, img2], Some("token123".to_string()));
/// assert!(images.has_more);
/// images.append(vec![img3], None);
/// assert!(!images.has_more);
/// ```
#[derive(Debug)]
pub struct PaginatedList<T> {
    /// The items loaded so far
    pub items: Vec<T>,
    /// Token for fetching the next page (if any)
    pub next_token: Option<String>,
    /// Whether there are more items to load
    pub has_more: bool,
    /// Whether we're currently loading more items
    pub loading_more: bool,
}

// Manual Default impl to avoid requiring T: Default
impl<T> Default for PaginatedList<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            next_token: None,
            has_more: false,
            loading_more: false,
        }
    }
}

impl<T> PaginatedList<T> {
    /// Create a new empty paginated list
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_token: None,
            has_more: false,
            loading_more: false,
        }
    }

    /// Replace all items (fresh load or filter changed)
    ///
    /// Use this when:
    /// - Loading data for the first time
    /// - A filter or sort changed, requiring fresh results
    pub fn replace(&mut self, items: Vec<T>, next_token: Option<String>) {
        self.items = items;
        self.has_more = next_token.is_some();
        self.next_token = next_token;
        self.loading_more = false;
    }

    /// Append items from a subsequent page
    ///
    /// Use this when loading more items via "Load More" functionality.
    pub fn append(&mut self, items: Vec<T>, next_token: Option<String>) {
        self.items.extend(items);
        self.has_more = next_token.is_some();
        self.next_token = next_token;
        self.loading_more = false;
    }

    /// Clear all data and reset state
    pub fn clear(&mut self) {
        self.items.clear();
        self.next_token = None;
        self.has_more = false;
        self.loading_more = false;
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Start loading more items
    pub fn start_loading_more(&mut self) {
        self.loading_more = true;
    }

    /// Get iterator over items
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    /// Get mutable iterator over items
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }
}

// Allow &PaginatedList to iterate over items
impl<'a, T> IntoIterator for &'a PaginatedList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_empty_list() {
        let list: PaginatedList<String> = PaginatedList::new();
        assert!(list.is_empty());
        assert!(!list.has_more);
        assert!(!list.loading_more);
        assert!(list.next_token.is_none());
    }

    #[test]
    fn test_replace_sets_items_and_token() {
        let mut list: PaginatedList<String> = PaginatedList::new();
        list.replace(vec!["a".to_string(), "b".to_string()], Some("token".to_string()));
        
        assert_eq!(list.len(), 2);
        assert!(list.has_more);
        assert_eq!(list.next_token, Some("token".to_string()));
    }

    #[test]
    fn test_replace_without_token_sets_has_more_false() {
        let mut list: PaginatedList<String> = PaginatedList::new();
        list.replace(vec!["a".to_string()], None);
        
        assert!(!list.has_more);
        assert!(list.next_token.is_none());
    }

    #[test]
    fn test_append_extends_items() {
        let mut list: PaginatedList<String> = PaginatedList::new();
        list.replace(vec!["a".to_string()], Some("token1".to_string()));
        list.append(vec!["b".to_string(), "c".to_string()], Some("token2".to_string()));
        
        assert_eq!(list.len(), 3);
        assert_eq!(list.items, vec!["a", "b", "c"]);
        assert_eq!(list.next_token, Some("token2".to_string()));
    }

    #[test]
    fn test_clear_resets_all_state() {
        let mut list: PaginatedList<String> = PaginatedList::new();
        list.replace(vec!["a".to_string()], Some("token".to_string()));
        list.loading_more = true;
        
        list.clear();
        
        assert!(list.is_empty());
        assert!(!list.has_more);
        assert!(!list.loading_more);
        assert!(list.next_token.is_none());
    }

    #[test]
    fn test_start_loading_more() {
        let mut list: PaginatedList<String> = PaginatedList::new();
        assert!(!list.loading_more);
        
        list.start_loading_more();
        
        assert!(list.loading_more);
    }

    #[test]
    fn test_iter() {
        let mut list: PaginatedList<i32> = PaginatedList::new();
        list.replace(vec![1, 2, 3], None);
        
        let sum: i32 = list.iter().sum();
        assert_eq!(sum, 6);
    }
}
