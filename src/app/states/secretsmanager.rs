use ratatui::widgets::TableState;
use crate::models::secretsmanager::Secret;
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt};
use crossterm::event::{KeyCode, KeyEvent};

/// State for Secrets Manager service
#[derive(Default)]
pub struct SecretsManagerState {
    pub secrets: Vec<Secret>,
    pub list_state: TableState,
    pub secret_value: Option<String>,
    pub show_secret_modal: bool,
}

impl SecretsManagerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected secret, if any
    pub fn selected_secret(&self) -> Option<&Secret> {
        self.list_state.selected().and_then(|i| self.secrets.get(i))
    }
}

impl ServiceInputHandler for SecretsManagerState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        if self.show_secret_modal {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                return InputResult::Message(Message::secretsmanager_close_value());
            }
            return InputResult::None;
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(self.secrets.len()),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(self.secrets.len()),
            KeyCode::Char('s') | KeyCode::Enter => {
                if let Some(secret) = self.selected_secret() {
                    if let Some(arn) = &secret.arn {
                        return InputResult::Message(Message::secretsmanager_get_value(
                            arn.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(secret) = self.selected_secret() {
                    if let Some(arn) = &secret.arn {
                        return InputResult::Action(Message::secretsmanager_delete_secret(
                            arn.clone(),
                        ));
                    }
                }
            }
            _ => {}
        }

        InputResult::None
    }
}
