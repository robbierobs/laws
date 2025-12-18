use ratatui::widgets::TableState;
use crate::models::lambda::LambdaFunction;
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
use crossterm::event::{KeyCode, KeyEvent};

/// State for Lambda service
#[derive(Default)]
pub struct LambdaState {
    pub functions: Vec<LambdaFunction>,
    pub list_state: TableState,
    pub function_details:
        std::collections::HashMap<String, crate::models::lambda::LambdaFunctionDetails>,
}

impl LambdaState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected function, if any
    pub fn selected_function(&self) -> Option<&LambdaFunction> {
        self.list_state
            .selected()
            .and_then(|i| self.functions.get(i))
    }
}

impl crate::app::global_search::Searchable for LambdaState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        for func in &self.functions {
            let mut result =
                SearchResult::new(Service::Lambda, "Lambda Function", &func.function_name);
            if let Some(ref desc) = func.description {
                if !desc.is_empty() {
                    result = result.with_secondary(desc.clone());
                }
            }
            results.push(result);
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for LambdaState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        if let Some(idx) = self.functions.iter().position(|f| f.function_name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for LambdaState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.functions.len());
                if let Some(f) = self.selected_function() {
                    return InputResult::Message(crate::app::Message::lambda_load_details(
                        f.function_name.clone(),
                    ));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.functions.len());
                if let Some(f) = self.selected_function() {
                    return InputResult::Message(crate::app::Message::lambda_load_details(
                        f.function_name.clone(),
                    ));
                }
            }
            KeyCode::Char('I') => {
                if let Some(f) = self.selected_function() {
                    return InputResult::Action(Message::lambda_invoke(f.function_name.clone()));
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(f) = self.selected_function() {
                    return InputResult::Action(Message::lambda_delete(f.function_name.clone()));
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
        self.selected_function().map(|f| f.function_name.clone())
    }
}

impl ServiceInternal for LambdaState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.lambda.clone();
        let handle = spawn_list_task(
            tx,
            move || async move {
                crate::aws::lambda::LambdaService::new(client)
                    .list_functions()
                    .await
            },
            AwsEvent::LambdaFunctionsLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::LAMBDA_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.functions.clear();
        self.function_details.clear();
        self.list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() && !self.functions.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}
