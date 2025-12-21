use ratatui::widgets::TableState;
use crate::models::ecr::{EcrImage, EcrRepository};
use crate::app::{EcrViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::app::pagination::PaginatedList;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
use crossterm::event::{KeyCode, KeyEvent};

/// Field to sort ECR images by
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageSortField {
    #[default]
    PushedAt,
    Tag,
    Size,
}

impl ImageSortField {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PushedAt => "Pushed",
            Self::Tag => "Tag",
            Self::Size => "Size",
        }
    }
    
    pub fn next(&self) -> Self {
        match self {
            Self::PushedAt => Self::Tag,
            Self::Tag => Self::Size,
            Self::Size => Self::PushedAt,
        }
    }
}

/// Sort direction for ECR images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageSortDirection {
    #[default]
    Descending, // Most recent first (default for time-based data)
    Ascending,
}

impl ImageSortDirection {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ascending => "↑",
            Self::Descending => "↓",
        }
    }
    
    pub fn toggle(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// Tag status filter for ECR images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagStatusFilter {
    #[default]
    Any,
    Tagged,
    Untagged,
}

impl TagStatusFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Any => "All",
            Self::Tagged => "Tagged",
            Self::Untagged => "Untagged",
        }
    }
    
    pub fn next(&self) -> Self {
        match self {
            Self::Any => Self::Tagged,
            Self::Tagged => Self::Untagged,
            Self::Untagged => Self::Any,
        }
    }
    
    pub fn as_api_value(&self) -> Option<&'static str> {
        match self {
            Self::Any => None, // Don't send filter for "Any"
            Self::Tagged => Some("TAGGED"),
            Self::Untagged => Some("UNTAGGED"),
        }
    }
}

/// State for ECR service
#[derive(Default)]
pub struct EcrState {
    pub repositories: Vec<EcrRepository>,
    /// Paginated list of images in the selected repository
    pub images: PaginatedList<EcrImage>,
    pub list_state: TableState,
    pub view_mode: EcrViewMode,
    pub selected_repo_name: Option<String>,
    
    // Sort state for images
    /// Current sort field for images
    pub sort_field: ImageSortField,
    /// Sort direction for images
    pub sort_direction: ImageSortDirection,
    
    // Filter state
    /// Tag status filter
    pub tag_status_filter: TagStatusFilter,
}

impl EcrState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected repository, if any
    pub fn selected_repository(&self) -> Option<&EcrRepository> {
        if self.view_mode == EcrViewMode::Repositories {
            self.list_state.selected().and_then(|i| self.repositories.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected image, if any
    pub fn selected_image(&self) -> Option<&EcrImage> {
        if self.view_mode == EcrViewMode::Images {
            self.list_state.selected().and_then(|i| self.images.items.get(i))
        } else {
            None
        }
    }

    /// Get repository by name (used when viewing images to access repo metadata)
    pub fn get_repository_by_name(&self, name: &str) -> Option<&EcrRepository> {
        self.repositories.iter().find(|r| r.repository_name == name)
    }

    /// Get the current repository URI when viewing images
    pub fn current_repository_uri(&self) -> Option<String> {
        self.selected_repo_name
            .as_ref()
            .and_then(|name| self.get_repository_by_name(name))
            .and_then(|repo| repo.repository_uri.clone())
    }

    /// Sort images based on current sort field and direction
    pub fn sort_images(&mut self) {
        let ascending = self.sort_direction == ImageSortDirection::Ascending;
        
        self.images.items.sort_by(|a, b| {
            let cmp = match self.sort_field {
                ImageSortField::PushedAt => {
                    // Compare by pushed_at (string comparison works for ISO dates)
                    a.image_pushed_at.cmp(&b.image_pushed_at)
                }
                ImageSortField::Tag => {
                    // Compare by first tag (or digest if untagged)
                    let tag_a = a.image_tags.first().cloned().unwrap_or_else(|| a.image_digest.clone());
                    let tag_b = b.image_tags.first().cloned().unwrap_or_else(|| b.image_digest.clone());
                    tag_a.cmp(&tag_b)
                }
                ImageSortField::Size => {
                    // Compare by size
                    a.image_size_in_bytes.cmp(&b.image_size_in_bytes)
                }
            };
            
            if ascending { cmp } else { cmp.reverse() }
        });
    }
}

impl crate::app::global_search::Searchable for EcrState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        // Ecr Repositories
        for repo in &self.repositories {
            results.push(SearchResult::new(
                Service::ECR,
                "ECR Repository",
                &repo.repository_name,
            ));
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for EcrState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        self.view_mode = EcrViewMode::Repositories;
        if let Some(idx) = self.repositories.iter().position(|r| r.repository_name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for EcrState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.view_mode {
            EcrViewMode::Repositories => self.repositories.len(),
            EcrViewMode::Images => self.images.len(),
        };

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter => {
                if self.view_mode == EcrViewMode::Repositories {
                    if let Some(repo) = self.selected_repository() {
                        return InputResult::Message(Message::ecr_load_images(
                            repo.repository_name.clone(),
                        ));
                    }
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                if self.view_mode == EcrViewMode::Images {
                    return InputResult::Message(Message::ecr_back_to_repos());
                }
            }
            // Pull image with docker
            KeyCode::Char('p') => {
                if self.view_mode == EcrViewMode::Images {
                    if let (Some(repo_uri), Some(image)) = (self.current_repository_uri(), self.selected_image()) {
                        // Use first tag if available, otherwise use digest
                        let tag = image.image_tags.first()
                            .cloned()
                            .unwrap_or_else(|| image.image_digest.clone());
                        return InputResult::Message(Message::ecr_pull_image(repo_uri, tag));
                    }
                }
            }
            // Cycle sort field (Images view only)
            KeyCode::Char('s') if self.view_mode == EcrViewMode::Images => {
                self.sort_field = self.sort_field.next();
                self.sort_images();
            }
            // Toggle sort direction (Images view only)
            KeyCode::Char('S') if self.view_mode == EcrViewMode::Images => {
                self.sort_direction = self.sort_direction.toggle();
                self.sort_images();
            }
            // Load more images (Images view only, when more are available)
            KeyCode::Char('L') if self.view_mode == EcrViewMode::Images && self.images.has_more => {
                return InputResult::Message(Message::ecr_load_more_images());
            }
            // Toggle tag status filter (Images view only)
            KeyCode::Char('F') if self.view_mode == EcrViewMode::Images => {
                return InputResult::Message(Message::ecr_toggle_tag_filter());
            }
            _ => {}
        }
        InputResult::None
    }

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn get_copiable_text(&self) -> Option<String> {
        match self.view_mode {
            EcrViewMode::Repositories => self.selected_repository().map(|r| r.repository_name.clone()),
            EcrViewMode::Images => self.selected_image().map(|i| i.image_digest.clone()),
        }
    }
}

impl ServiceInternal for EcrState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.ecr.clone();
        let handle = spawn_list_task(
            tx,
            move || async move {
                crate::aws::ecr::EcrService::new(client)
                    .list_repositories()
                    .await
            },
            AwsEvent::EcrRepositoriesLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::ECR_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.repositories.clear();
        self.images.clear();
        self.selected_repo_name = None;
        self.view_mode = EcrViewMode::Repositories;
        self.list_state.select(Some(0));
        // Reset sort state
        self.sort_field = ImageSortField::default();
        self.sort_direction = ImageSortDirection::default();
        // Reset filter state
        self.tag_status_filter = TagStatusFilter::default();
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() {
            let has_items = match self.view_mode {
                EcrViewMode::Repositories => !self.repositories.is_empty(),
                EcrViewMode::Images => !self.images.is_empty(),
            };
            if has_items {
                self.list_state.select(Some(0));
            }
        }
    }
}
