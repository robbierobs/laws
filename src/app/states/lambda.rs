use ratatui::widgets::TableState;
use crate::models::lambda::LambdaFunction;
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt};
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
}
