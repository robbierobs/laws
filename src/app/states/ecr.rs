use ratatui::widgets::TableState;
use crate::models::ecr::{EcrImage, EcrRepository};
use crate::app::{EcrViewMode, InputResult, Message, ServiceInputHandler, TableStateExt};
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
}
