use ratatui::widgets::TableState;
use crate::models::ecr::{EcrImage, EcrRepository};
use crate::app::{EcrViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
use crossterm::event::{KeyCode, KeyEvent};

/// State for ECR service
#[derive(Default)]
pub struct EcrState {
    pub repositories: Vec<EcrRepository>,
    pub images: Vec<EcrImage>,
    pub list_state: TableState,
    pub view_mode: EcrViewMode,
    pub selected_repo_name: Option<String>,
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
            self.list_state.selected().and_then(|i| self.images.get(i))
        } else {
            None
        }
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
    }
}
