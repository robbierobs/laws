use ratatui::widgets::TableState;
use crate::models::ec2::Ec2Instance;
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt};
use crossterm::event::{KeyCode, KeyEvent};

/// State for EC2 service
#[derive(Default)]
pub struct Ec2State {
    pub instances: Vec<Ec2Instance>,
    pub list_state: TableState,
}

impl Ec2State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected instance, if any
    pub fn selected_instance(&self) -> Option<&Ec2Instance> {
        self.list_state
            .selected()
            .and_then(|i| self.instances.get(i))
    }

    /// Get the instance ID of the currently selected instance
    pub fn selected_instance_id(&self) -> Option<String> {
        self.selected_instance().map(|i| i.instance_id.clone())
    }
}

impl ServiceInputHandler for Ec2State {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(self.instances.len()),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(self.instances.len()),
            KeyCode::Char('s') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(instance) = self.instances.get(i) {
                        return InputResult::Action(Message::ec2_start(
                            instance.instance_id.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('S') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(instance) = self.instances.get(i) {
                        return InputResult::Action(Message::ec2_stop(
                            instance.instance_id.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('R') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(instance) = self.instances.get(i) {
                        return InputResult::Action(Message::ec2_reboot(
                            instance.instance_id.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(instance) = self.instances.get(i) {
                        return InputResult::Action(Message::ec2_terminate(
                            instance.instance_id.clone(),
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
