use ratatui::widgets::TableState;
use crate::models::ec2::Ec2Instance;
use crate::app::{handle_list_navigation, InputResult, Message, ServiceInputHandler, Service};
use crate::app::global_search::{AutoSelectable, Searchable, SearchResult};
use crossterm::event::{KeyCode, KeyEvent};
use crate::app::states::ServiceInternal;
use crate::app::EventSender;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::aws::traits::AwsService;
use crate::event::AwsEvent;

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

impl Searchable for Ec2State {
    fn get_search_results(&self) -> Vec<SearchResult> {
        let mut results = Vec::new();
        for instance in &self.instances {
            let name = instance.name.clone().unwrap_or_default();
            let mut result = SearchResult::new(Service::EC2, "EC2 Instance", &instance.instance_id);
            if !name.is_empty() {
                result = result.with_secondary(name);
            }
            if !instance.tags.is_empty() {
                result = result.with_tags(instance.tags.clone());
            }
            results.push(result);
        }
        results
    }
}

impl AutoSelectable for Ec2State {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        if let Some(idx) = self.instances.iter().position(|i| i.instance_id == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for Ec2State {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        if handle_list_navigation(&mut self.list_state, self.instances.len(), key) {
            return InputResult::None;
        }
        match key.code {
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

impl ServiceInternal for Ec2State {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.ec2.clone();
        let handle = spawn_list_task(
            tx,
            move || async move { crate::aws::ec2::Ec2Service::new(client).list().await },
            AwsEvent::Ec2InstancesLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::EC2_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.instances.clear();
        self.list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() && !self.instances.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ec2_state_new() {
        let state = Ec2State::new();
        assert!(state.instances.is_empty());
        assert_eq!(state.list_state.selected(), None);
    }

    #[test]
    fn test_ec2_state_selection_empty() {
        let state = Ec2State::new();
        assert_eq!(state.selected_instance(), None);
        assert_eq!(state.selected_instance_id(), None);
    }

    #[test]
    fn test_ec2_state_clear_removes_instances() {
        let mut state = Ec2State::new();
        state.instances.push(crate::models::ec2::Ec2Instance {
            instance_id: "i-test".to_string(),
            name: None,
            state: crate::models::ec2::InstanceState::Running,
            instance_type: "t2.micro".to_string(),
            public_ip: None,
            private_ip: None,
            launch_time: None,
            subnet_id: None,
            vpc_id: None,
            security_groups: vec![],
            availability_zone: None,
            platform: None,
            architecture: None,
            ami_id: None,
            key_name: None,
            monitoring_state: None,
            tags: vec![],
        });
        
        assert_eq!(state.instances.len(), 1);
        state.clear();
        assert_eq!(state.instances.len(), 0);
    }
}
