use ratatui::widgets::TableState;
use std::collections::HashMap;
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt};
use crossterm::event::{KeyCode, KeyEvent};

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
    /// Pending edit operation (bucket, key, path) - for synchronous editor handling
    pub pending_edit: Option<(String, String, String)>,
}

impl S3State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're currently viewing objects inside a bucket
    pub fn is_viewing_objects(&self) -> bool {
        self.current_bucket.is_some()
    }

    /// Get the currently selected bucket, if any
    pub fn selected_bucket(&self) -> Option<&S3Bucket> {
        self.list_state.selected().and_then(|i| self.buckets.get(i))
    }

    /// Get the currently selected object, if any
    pub fn selected_object(&self) -> Option<&S3Object> {
        self.object_list_state
            .selected()
            .and_then(|i| self.objects.get(i))
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
                _ => {}
            }
        }
        InputResult::None
    }
}
