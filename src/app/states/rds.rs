use ratatui::widgets::TableState;
use crate::models::rds::RdsInstance;
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt};
use crossterm::event::{KeyCode, KeyEvent};

/// State for RDS service
#[derive(Default)]
pub struct RdsState {
    pub instances: Vec<RdsInstance>,
    pub list_state: TableState,
}

impl RdsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected instance, if any
    pub fn selected_instance(&self) -> Option<&RdsInstance> {
        self.list_state
            .selected()
            .and_then(|i| self.instances.get(i))
    }

    /// Get the instance ID of the currently selected instance
    pub fn selected_instance_id(&self) -> Option<String> {
        self.selected_instance()
            .map(|i| i.db_instance_identifier.clone())
    }
}

impl crate::app::global_search::Searchable for RdsState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        for instance in &self.instances {
            let mut result = SearchResult::new(
                Service::RDS,
                "RDS Instance",
                &instance.db_instance_identifier,
            );
            result = result.with_secondary(instance.engine.clone());
            results.push(result);
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for RdsState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        if let Some(idx) = self.instances.iter().position(|i| i.db_instance_identifier == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for RdsState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(self.instances.len()),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(self.instances.len()),
            KeyCode::Char('s') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(inst) = self.instances.get(i) {
                        return InputResult::Action(Message::rds_start(
                            inst.db_instance_identifier.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('S') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(inst) = self.instances.get(i) {
                        return InputResult::Action(Message::rds_stop(
                            inst.db_instance_identifier.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('R') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(inst) = self.instances.get(i) {
                        return InputResult::Action(Message::rds_reboot(
                            inst.db_instance_identifier.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(inst) = self.instances.get(i) {
                        return InputResult::Action(Message::rds_delete(
                            inst.db_instance_identifier.clone(),
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
        self.selected_instance_id()
    }
}
