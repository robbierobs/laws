use crate::app::filter_modal::{FilterFieldConfig, FilterModalConfig, FilterModalState};
use crate::app::pagination::PaginatedList;
use crate::app::states::ServiceInternal;
use crate::app::task_manager::{task_keys, TaskManager};
use crate::app::update::refresh::spawn_list_task;
use crate::app::{
    EcrViewMode, EventSender, InputResult, Message, ServiceInputHandler, TableStateExt,
};
use crate::aws::client::AwsClients;
use crate::event::AwsEvent;
use crate::models::ecr::{EcrImage, EcrRepository, ImageScanFindings};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

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

/// ECR filter field IDs (for type-safe access)
pub mod filter_fields {
    pub const TAG_STATUS: &str = "tag_status";
    pub const TAG_SEARCH: &str = "tag_search";
    pub const DIGEST_SEARCH: &str = "digest_search";
}

/// Get the ECR filter modal configuration
pub fn ecr_filter_config() -> FilterModalConfig {
    FilterModalConfig::new("ECR Image Filters")
        .field(FilterFieldConfig::cycle(
            filter_fields::TAG_STATUS,
            "Tag Status",
            vec!["All".into(), "Tagged".into(), "Untagged".into()],
        ))
        .field(
            FilterFieldConfig::text(filter_fields::TAG_SEARCH, "Tag Contains")
                .placeholder("e.g., latest, v1.0"),
        )
        .field(
            FilterFieldConfig::text(filter_fields::DIGEST_SEARCH, "Digest Contains")
                .placeholder("e.g., sha256:abc"),
        )
        .dimensions(50, 40)
}

/// Convert filter modal state to filter parameters
#[derive(Default, Clone)]
pub struct EcrImageFilters {
    /// Tag status: 0 = All, 1 = Tagged, 2 = Untagged
    pub tag_status_index: usize,
    /// Optional tag search string
    pub tag_search: String,
    /// Optional digest search string
    pub digest_search: String,
}

impl EcrImageFilters {
    pub fn from_modal_state(state: &FilterModalState) -> Self {
        Self {
            tag_status_index: state.get_cycle_index(filter_fields::TAG_STATUS),
            tag_search: state.get_text(filter_fields::TAG_SEARCH),
            digest_search: state.get_text(filter_fields::DIGEST_SEARCH),
        }
    }

    pub fn tag_status_api_value(&self) -> Option<&'static str> {
        match self.tag_status_index {
            1 => Some("TAGGED"),
            2 => Some("UNTAGGED"),
            _ => None,
        }
    }

    /// Check if an image matches the local filters (tag/digest search)
    pub fn matches(&self, image: &EcrImage) -> bool {
        // Tag search filter
        if !self.tag_search.is_empty() {
            let search = self.tag_search.to_lowercase();
            if !image
                .image_tags
                .iter()
                .any(|t| t.to_lowercase().contains(&search))
            {
                return false;
            }
        }

        // Digest search filter
        if !self.digest_search.is_empty() {
            let search = self.digest_search.to_lowercase();
            if !image.image_digest.to_lowercase().contains(&search) {
                return false;
            }
        }

        true
    }

    pub fn has_local_filters(&self) -> bool {
        !self.tag_search.is_empty() || !self.digest_search.is_empty()
    }

    /// Check if any filters are active (including API-level tag status)
    pub fn is_empty(&self) -> bool {
        self.tag_status_index == 0 && self.tag_search.is_empty() && self.digest_search.is_empty()
    }
}

/// State for ECR service
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

    // Filter state (using generic filter modal)
    /// Filter modal configuration
    pub filter_config: FilterModalConfig,
    /// Filter modal runtime state
    pub filter_modal: FilterModalState,
    /// Current active filters (cached from modal)
    pub current_filters: EcrImageFilters,

    // Scan findings (loaded on-demand for selected image)
    /// Digest of image for which we have loaded scan findings
    pub scan_findings_digest: Option<String>,
    /// Loaded scan findings for the selected image
    pub scan_findings: Option<ImageScanFindings>,
    /// Whether scan findings are currently loading
    pub scan_findings_loading: bool,
}

impl Default for EcrState {
    fn default() -> Self {
        let config = ecr_filter_config();
        let modal_state = FilterModalState::new(&config);
        Self {
            repositories: Vec::new(),
            images: PaginatedList::default(),
            list_state: TableState::default(),
            view_mode: EcrViewMode::default(),
            selected_repo_name: None,
            sort_field: ImageSortField::default(),
            sort_direction: ImageSortDirection::default(),
            filter_config: config,
            filter_modal: modal_state,
            current_filters: EcrImageFilters::default(),
            scan_findings_digest: None,
            scan_findings: None,
            scan_findings_loading: false,
        }
    }
}

impl EcrState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected repository, if any
    pub fn selected_repository(&self) -> Option<&EcrRepository> {
        if self.view_mode == EcrViewMode::Repositories {
            self.list_state
                .selected()
                .and_then(|i| self.repositories.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected image, if any
    pub fn selected_image(&self) -> Option<&EcrImage> {
        if self.view_mode == EcrViewMode::Images {
            self.list_state
                .selected()
                .and_then(|i| self.images.items.get(i))
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
                    let tag_a = a
                        .image_tags
                        .first()
                        .cloned()
                        .unwrap_or_else(|| a.image_digest.clone());
                    let tag_b = b
                        .image_tags
                        .first()
                        .cloned()
                        .unwrap_or_else(|| b.image_digest.clone());
                    tag_a.cmp(&tag_b)
                }
                ImageSortField::Size => {
                    // Compare by size
                    a.image_size_in_bytes.cmp(&b.image_size_in_bytes)
                }
            };

            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }
}

impl crate::app::global_search::Searchable for EcrState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::global_search::SearchResult;
        use crate::app::Service;

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
        if let Some(idx) = self
            .repositories
            .iter()
            .position(|r| r.repository_name == resource_id)
        {
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
                    if let (Some(repo_uri), Some(image)) =
                        (self.current_repository_uri(), self.selected_image())
                    {
                        // Use first tag if available, otherwise use digest
                        let tag = image
                            .image_tags
                            .first()
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
            // Open filter modal (Images view only)
            KeyCode::Char('F') if self.view_mode == EcrViewMode::Images => {
                return InputResult::Message(Message::ecr_open_filter_modal());
            }
            // Clear filters (Images view only, when filters active)
            KeyCode::Char('c')
                if self.view_mode == EcrViewMode::Images && !self.current_filters.is_empty() =>
            {
                return InputResult::Message(Message::ecr_clear_filters());
            }
            // Load scan findings for selected image
            KeyCode::Char('v') if self.view_mode == EcrViewMode::Images => {
                if let (Some(repo_name), Some(image)) =
                    (self.selected_repo_name.clone(), self.selected_image())
                {
                    return InputResult::Message(Message::ecr_load_scan_findings(
                        repo_name,
                        image.image_digest.clone(),
                    ));
                }
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
            EcrViewMode::Repositories => self
                .selected_repository()
                .map(|r| r.repository_name.clone()),
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
        self.filter_modal.close();
        self.filter_modal.clear_all();
        self.current_filters = EcrImageFilters::default();
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
