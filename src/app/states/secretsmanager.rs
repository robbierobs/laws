use ratatui::widgets::TableState;
use crate::models::secretsmanager::Secret;
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
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

impl crate::app::global_search::Searchable for SecretsManagerState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        for secret in &self.secrets {
            let mut result = SearchResult::new(Service::SecretsManager, "Secret", &secret.name);
            if let Some(ref desc) = secret.description {
                if !desc.is_empty() {
                    result = result.with_secondary(desc.clone());
                }
            }
            results.push(result);
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for SecretsManagerState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        if let Some(idx) = self.secrets.iter().position(|s| s.name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
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

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn get_copiable_text(&self) -> Option<String> {
        self.selected_secret().map(|s| s.name.clone())
    }
}

impl ServiceInternal for SecretsManagerState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.secretsmanager.clone();
        let handle = spawn_list_task(
            tx,
            move || async move {
                crate::aws::secretsmanager::SecretsManagerService::new(client)
                    .list_secrets()
                    .await
            },
            AwsEvent::SecretsManagerSecretsLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::SECRETSMANAGER_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.secrets.clear();
        self.secret_value = None;
        self.show_secret_modal = false;
        self.list_state.select(Some(0));
    }
}
