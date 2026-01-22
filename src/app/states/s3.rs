use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::event::{AwsEvent, Event};
use crate::app::messages::ServiceAction;
use crate::app::messages::S3Action;
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

impl ServiceInternal for S3State {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.s3.clone();
        let tx_clone = tx.clone();
        let delay_ms = config.s3_detail_delay_ms;

        let handle = tokio::spawn(async move {
            let service = crate::aws::s3::S3Service::new(client.clone());
            match service.list_buckets().await {
                Ok(buckets) => {
                    // 1. Notify that buckets are loaded
                    tx_clone
                        .send(Event::Aws(Box::new(AwsEvent::S3BucketsLoaded(
                            buckets.clone(),
                        ))))
                        .await
                        .ok();

                    // 2. Trigger detail loading for top buckets (rate limited)
                    for bucket in buckets.iter().take(20) {
                        if delay_ms > 0 {
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        }

                        tx_clone
                            .send(Event::Message(Message::Service(ServiceAction::S3(
                                S3Action::LoadBucketDetails(bucket.name.clone()),
                            ))))
                            .await
                            .ok();
                    }
                }
                Err(e) => {
                    if report_errors {
                        tx_clone
                            .send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                            .await
                            .ok();
                    }
                }
            }
        });
        tasks.spawn(task_keys::S3_REFRESH, handle);

        // Also refresh objects if inside a bucket
        if let Some(bucket_name) = &self.current_bucket {
            let bucket_name = bucket_name.clone();
            let client = clients.s3.clone();
            let tx = tx.clone();
            let handle = tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                match service.list_objects(&bucket_name).await {
                    Ok(objects) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::S3ObjectsLoaded(objects))))
                            .await
                            .ok();
                    }
                    Err(e) => {
                        if report_errors {
                            tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                                .await
                                .ok();
                        }
                    }
                }
            });
            tasks.spawn(task_keys::S3_OBJECTS, handle);
        }
    }

    fn clear(&mut self) {
        self.buckets.clear();
        self.objects.clear();
        self.current_bucket = None;
        self.bucket_details.clear();
        self.opened_object_content = None;
        self.opened_object_path = None;
        self.opened_object_key = None;
        self.show_object_viewer = false;
        self.opened_object_bytes = None;
        self.pending_edit = None;
        self.list_state.select(Some(0));
        self.object_list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.current_bucket.is_some() {
            // In objects view
            if self.object_list_state.selected().is_none() && !self.objects.is_empty() {
                self.object_list_state.select(Some(0));
            }
        } else {
            // In buckets view
            if self.list_state.selected().is_none() && !self.buckets.is_empty() {
                self.list_state.select(Some(0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bucket(name: &str) -> S3Bucket {
        S3Bucket {
            name: name.to_string(),
            creation_date: None,
            region: None,
        }
    }

    fn make_object(key: &str) -> S3Object {
        S3Object {
            key: key.to_string(),
            size: 0,
            last_modified: None,
            storage_class: None,
            etag: None,
        }
    }

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
        assert_eq!(state.list_state.selected(), None);
        assert_eq!(state.object_list_state.selected(), None);
        assert!(!state.show_object_viewer);
        assert!(!state.show_create_bucket_modal);
        assert!(state.create_bucket_input.is_empty());
        assert_eq!(state.viewer_mode, ViewerMode::Text);
    }

    #[test]
    fn test_s3_state_selection_empty() {
        let state = S3State::new();
        assert_eq!(state.selected_bucket(), None);
        assert_eq!(state.selected_object(), None);
    }

    #[test]
    fn test_s3_state_selection_populated() {
        let mut state = S3State::new();
        state.buckets = vec![make_bucket("alpha"), make_bucket("beta")];
        state.list_state.select(Some(1));

        let selected_bucket = state.selected_bucket().expect("bucket selection");
        assert_eq!(selected_bucket.name, "beta");

        state.current_bucket = Some("alpha".to_string());
        state.objects = vec![make_object("first.txt"), make_object("second.txt")];
        state.object_list_state.select(Some(0));

        let selected_object = state.selected_object().expect("object selection");
        assert_eq!(selected_object.key, "first.txt");
    }

    #[test]
    fn test_s3_state_auto_select_first_bucket_view() {
        let mut state = S3State::new();
        state.auto_select_first();
        assert_eq!(state.list_state.selected(), None);

        state.buckets = vec![make_bucket("alpha")];
        state.auto_select_first();
        assert_eq!(state.list_state.selected(), Some(0));
    }

    #[test]
    fn test_s3_state_auto_select_first_object_view() {
        let mut state = S3State::new();
        state.current_bucket = Some("alpha".to_string());
        state.auto_select_first();
        assert_eq!(state.object_list_state.selected(), None);

        state.objects = vec![make_object("first.txt")];
        state.auto_select_first();
        assert_eq!(state.object_list_state.selected(), Some(0));
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

    #[test]
    fn test_s3_state_clear_resets_state() {
        let mut state = S3State::new();
        state.buckets = vec![make_bucket("alpha")];
        state.objects = vec![make_object("file.txt")];
        state.current_bucket = Some("alpha".to_string());
        state.bucket_details.insert("alpha".to_string(), S3BucketDetails::default());
        state.opened_object_content = Some("content".to_string());
        state.opened_object_path = Some("/tmp/file.txt".to_string());
        state.opened_object_key = Some("file.txt".to_string());
        state.show_object_viewer = true;
        state.opened_object_bytes = Some(vec![1, 2, 3]);
        state.pending_edit = Some(("alpha".to_string(), "file.txt".to_string(), "/tmp/file.txt".to_string()));
        state.list_state.select(Some(1));
        state.object_list_state.select(Some(1));

        state.clear();

        assert!(state.buckets.is_empty());
        assert!(state.objects.is_empty());
        assert!(state.current_bucket.is_none());
        assert!(state.bucket_details.is_empty());
        assert!(state.opened_object_content.is_none());
        assert!(state.opened_object_path.is_none());
        assert!(state.opened_object_key.is_none());
        assert!(!state.show_object_viewer);
        assert!(state.opened_object_bytes.is_none());
        assert!(state.pending_edit.is_none());
        assert_eq!(state.list_state.selected(), Some(0));
        assert_eq!(state.object_list_state.selected(), Some(0));
    }
}
