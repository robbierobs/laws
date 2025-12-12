//! Global search functionality
//!
//! Allows searching across all AWS resources from any screen.

use crate::app::Service;

/// A unified search result that can represent any AWS resource
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The service this result belongs to
    pub service: Service,
    /// Resource type label (e.g., "EC2 Instance", "Lambda Function")
    pub resource_type: String,
    /// Primary identifier (e.g., instance ID, function name)
    pub primary_id: String,
    /// Optional secondary info (e.g., name tag, description)
    pub secondary_info: Option<String>,
    /// Optional tags for tag-based searching
    pub tags: Vec<(String, String)>,
}

impl SearchResult {
    pub fn new(
        service: Service,
        resource_type: impl Into<String>,
        primary_id: impl Into<String>,
    ) -> Self {
        Self {
            service,
            resource_type: resource_type.into(),
            primary_id: primary_id.into(),
            secondary_info: None,
            tags: Vec::new(),
        }
    }

    pub fn with_secondary(mut self, info: impl Into<String>) -> Self {
        self.secondary_info = Some(info.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<(String, String)>) -> Self {
        self.tags = tags;
        self
    }

    /// Check if this result matches the search query
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Check primary ID
        if self.primary_id.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check secondary info
        if let Some(ref info) = self.secondary_info {
            if info.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        // Check resource type
        if self.resource_type.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check tags (key and value)
        for (key, value) in &self.tags {
            if key.to_lowercase().contains(&query_lower)
                || value.to_lowercase().contains(&query_lower)
            {
                return true;
            }
        }

        false
    }

    /// Get a display string for the result
    #[allow(dead_code)]
    pub fn display(&self) -> String {
        if let Some(ref info) = self.secondary_info {
            format!("{} ({}) - {}", self.primary_id, self.resource_type, info)
        } else {
            format!("{} ({})", self.primary_id, self.resource_type)
        }
    }
}

/// State for the global search feature
#[derive(Default)]
pub struct GlobalSearchState {
    /// The current search query
    pub query: String,
    /// All search results
    pub results: Vec<SearchResult>,
    /// Currently selected result index
    pub selected_index: usize,
    /// Scroll offset for displaying results
    pub scroll_offset: usize,
    /// Whether to search tags specifically (tag: prefix)
    pub tag_search_mode: bool,
}

impl GlobalSearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.tag_search_mode = false;
    }

    /// Get the currently selected result, if any
    pub fn selected(&self) -> Option<&SearchResult> {
        self.results.get(self.selected_index)
    }

    /// Navigate down in results
    pub fn nav_down(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.results.len();
        }
    }

    /// Navigate up in results
    pub fn nav_up(&mut self) {
        if !self.results.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.results.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Filter results based on current query
    pub fn filter(&mut self, all_results: &[SearchResult]) {
        if self.query.is_empty() {
            self.results = all_results.to_vec();
        } else {
            // Check if it's a tag search (tag:value format)
            let query = if self.query.starts_with("tag:") {
                self.tag_search_mode = true;
                &self.query[4..]
            } else {
                self.tag_search_mode = false;
                &self.query
            };

            self.results = all_results
                .iter()
                .filter(|r| {
                    if self.tag_search_mode {
                        // Only search in tags
                        let query_lower = query.to_lowercase();
                        r.tags.iter().any(|(k, v)| {
                            k.to_lowercase().contains(&query_lower)
                                || v.to_lowercase().contains(&query_lower)
                        })
                    } else {
                        r.matches(query)
                    }
                })
                .cloned()
                .collect();
        }

        // Reset selection if it's now out of bounds
        if self.selected_index >= self.results.len() {
            self.selected_index = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_matches_primary_id() {
        let result = SearchResult::new(Service::EC2, "EC2 Instance", "i-1234567890abcdef0");
        assert!(result.matches("i-123"));
        assert!(result.matches("abcdef"));
        assert!(!result.matches("xyz"));
    }

    #[test]
    fn test_search_result_matches_secondary() {
        let result = SearchResult::new(Service::EC2, "EC2 Instance", "i-123")
            .with_secondary("web-server-prod");
        assert!(result.matches("web-server"));
        assert!(result.matches("prod"));
    }

    #[test]
    fn test_search_result_matches_tags() {
        let result = SearchResult::new(Service::EC2, "EC2 Instance", "i-123")
            .with_tags(vec![
                ("Name".to_string(), "my-instance".to_string()),
                ("Environment".to_string(), "production".to_string()),
            ]);
        assert!(result.matches("my-instance"));
        assert!(result.matches("production"));
        assert!(result.matches("Name"));
    }

    #[test]
    fn test_global_search_state_navigation() {
        let mut state = GlobalSearchState::new();
        state.results = vec![
            SearchResult::new(Service::EC2, "EC2", "i-1"),
            SearchResult::new(Service::EC2, "EC2", "i-2"),
            SearchResult::new(Service::EC2, "EC2", "i-3"),
        ];

        assert_eq!(state.selected_index, 0);
        state.nav_down();
        assert_eq!(state.selected_index, 1);
        state.nav_down();
        assert_eq!(state.selected_index, 2);
        state.nav_down();
        assert_eq!(state.selected_index, 0); // Wraps

        state.nav_up();
        assert_eq!(state.selected_index, 2); // Wraps back
    }

    #[test]
    fn test_tag_search_mode() {
        let mut state = GlobalSearchState::new();
        state.query = "tag:prod".to_string();

        let all_results = vec![
            SearchResult::new(Service::EC2, "EC2", "i-1")
                .with_tags(vec![("Environment".to_string(), "production".to_string())]),
            SearchResult::new(Service::EC2, "EC2", "i-2")
                .with_secondary("prod-server"), // This shouldn't match in tag mode
        ];

        state.filter(&all_results);

        assert_eq!(state.results.len(), 1);
        assert_eq!(state.results[0].primary_id, "i-1");
    }
}
