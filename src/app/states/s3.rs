use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt};
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;
use std::collections::HashMap;

/// Viewer mode for S3 object content
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ViewerMode {
    #[default]
    Text,
    Hex,
}

impl ViewerMode {
    pub fn next(self) -> Self {
        match self {
            ViewerMode::Text => ViewerMode::Hex,
            ViewerMode::Hex => ViewerMode::Text,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ViewerMode::Text => "Text",
            ViewerMode::Hex => "Hex",
        }
    }
}

/// State for S3 service
#[derive(Default)]
pub struct S3State {
    pub buckets: Vec<S3Bucket>,
    pub list_state: TableState,
    pub current_bucket: Option<String>,
    pub objects: Vec<S3Object>,
    pub object_list_state: TableState,
    pub bucket_details: HashMap<String, S3BucketDetails>,
    /// Content of the last opened object (if it's text)
    pub opened_object_content: Option<String>,
    /// Path to the last opened object file
    pub opened_object_path: Option<String>,
    /// Key of the opened object (for display in popup title)
    pub opened_object_key: Option<String>,
    /// Whether to show the object viewer popup
    pub show_object_viewer: bool,
    /// Scroll offset for the object viewer
    pub viewer_scroll_offset: u16,
    /// Current viewer mode (Text or Hex)
    pub viewer_mode: ViewerMode,
    /// Raw bytes of the opened object (for hex view)
    pub opened_object_bytes: Option<Vec<u8>>,
    /// Pending edit operation (bucket, key, path) - for synchronous editor handling
    pub pending_edit: Option<(String, String, String)>,
    /// Whether bucket creation modal is shown
    pub show_create_bucket_modal: bool,
    /// Input buffer for new bucket name
    pub create_bucket_input: String,
}

impl S3State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're currently viewing objects inside a bucket
    #[allow(dead_code)] // Helper for conditional rendering
    pub fn is_viewing_objects(&self) -> bool {
        self.current_bucket.is_some()
    }

    /// Get the currently selected bucket, if any
    pub fn selected_bucket(&self) -> Option<&S3Bucket> {
        self.list_state.selected().and_then(|i| self.buckets.get(i))
    }

    /// Reset bucket creation modal state
    pub fn reset_create_bucket_modal(&mut self) {
        self.show_create_bucket_modal = false;
        self.create_bucket_input.clear();
    }

    /// Toggle viewer mode
    pub fn toggle_viewer_mode(&mut self) {
        self.viewer_mode = self.viewer_mode.next();
    }

    /// Get the currently selected object, if any
    pub fn selected_object(&self) -> Option<&S3Object> {
        self.object_list_state
            .selected()
            .and_then(|i| self.objects.get(i))
    }
}

impl crate::app::global_search::Searchable for S3State {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        // S3 Buckets
        for bucket in &self.buckets {
            results.push(SearchResult::new(Service::S3, "S3 Bucket", &bucket.name));
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for S3State {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        // Navigate to bucket list and select the bucket
        self.current_bucket = None; // Ensure we're at bucket level
        if let Some(idx) = self.buckets.iter().position(|b| b.name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for S3State {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        if self.current_bucket.is_some() {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.object_list_state.nav_down(self.objects.len());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.object_list_state.nav_up(self.objects.len());
                }
                KeyCode::Char('X') | KeyCode::Delete => {
                    if let Some(i) = self.object_list_state.selected() {
                        if let Some(obj) = self.objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Action(Message::s3_delete_object(
                                    bucket.clone(),
                                    obj.key.clone(),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char('o') => {
                    // Open object - download to temp and view
                    if let Some(i) = self.object_list_state.selected() {
                        if let Some(obj) = self.objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Message(Message::s3_open_object(
                                    bucket.clone(),
                                    obj.key.clone(),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char('E') => {
                    // Edit object - download, open in $EDITOR, upload changes
                    if let Some(i) = self.object_list_state.selected() {
                        if let Some(obj) = self.objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Action(Message::s3_edit_object(
                                    bucket.clone(),
                                    obj.key.clone(),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char('w') => {
                    // Download/Write object to ~/Downloads
                    if let Some(i) = self.object_list_state.selected() {
                        if let Some(obj) = self.objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Message(Message::s3_download_object(
                                    bucket.clone(),
                                    obj.key.clone(),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Backspace => {
                    return InputResult::Message(Message::s3_leave_bucket())
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.list_state.nav_down(self.buckets.len());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.list_state.nav_up(self.buckets.len());
                }
                KeyCode::Enter => {
                    if let Some(i) = self.list_state.selected() {
                        if let Some(bucket) = self.buckets.get(i) {
                            return InputResult::Message(Message::s3_load_objects(
                                bucket.name.clone(),
                            ));
                        }
                    }
                }
                KeyCode::Char('i') => {
                    if let Some(i) = self.list_state.selected() {
                        if let Some(bucket) = self.buckets.get(i) {
                            return InputResult::Message(Message::s3_load_bucket_details(
                                bucket.name.clone(),
                            ));
                        }
                    }
                }
                KeyCode::Char('C') => {
                    // Show create bucket modal - transition to input mode
                    self.show_create_bucket_modal = true;
                    self.create_bucket_input.clear();
                    return InputResult::OpenInputMode(crate::app::InputMode::S3BucketCreation);
                }
                KeyCode::Char('X') => {
                    // Delete selected bucket (requires confirmation)
                    if let Some(bucket) = self.selected_bucket() {
                        return InputResult::Action(Message::s3_delete_bucket(bucket.name.clone()));
                    }
                }
                _ => {}
            }
        }
        InputResult::None
    }

    fn reset_selection(&mut self) {
        if self.current_bucket.is_some() {
            self.object_list_state.select(Some(0));
        } else {
            self.list_state.select(Some(0));
        }
    }

    fn get_copiable_text(&self) -> Option<String> {
        if self.current_bucket.is_some() {
            self.selected_object().map(|o| o.key.clone())
        } else {
            self.selected_bucket().map(|b| b.name.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_mode_toggle() {
        assert_eq!(ViewerMode::Text.next(), ViewerMode::Hex);
        assert_eq!(ViewerMode::Hex.next(), ViewerMode::Text);
    }

    #[test]
    fn test_viewer_mode_labels() {
        assert_eq!(ViewerMode::Text.label(), "Text");
        assert_eq!(ViewerMode::Hex.label(), "Hex");
    }

    #[test]
    fn test_s3_state_default() {
        let state = S3State::new();
        assert!(state.buckets.is_empty());
        assert!(state.objects.is_empty());
        assert!(state.current_bucket.is_none());
        assert!(!state.show_object_viewer);
        assert!(!state.show_create_bucket_modal);
        assert!(state.create_bucket_input.is_empty());
        assert_eq!(state.viewer_mode, ViewerMode::Text);
    }

    #[test]
    fn test_reset_create_bucket_modal() {
        let mut state = S3State::new();
        state.show_create_bucket_modal = true;
        state.create_bucket_input = "test-bucket".to_string();

        state.reset_create_bucket_modal();

        assert!(!state.show_create_bucket_modal);
        assert!(state.create_bucket_input.is_empty());
    }

    #[test]
    fn test_toggle_viewer_mode() {
        let mut state = S3State::new();
        assert_eq!(state.viewer_mode, ViewerMode::Text);

        state.toggle_viewer_mode();
        assert_eq!(state.viewer_mode, ViewerMode::Hex);

        state.toggle_viewer_mode();
        assert_eq!(state.viewer_mode, ViewerMode::Text);
    }

    #[test]
    fn test_is_viewing_objects() {
        let mut state = S3State::new();
        assert!(!state.is_viewing_objects());

        state.current_bucket = Some("test-bucket".to_string());
        assert!(state.is_viewing_objects());
    }
}
