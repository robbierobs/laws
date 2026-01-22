use ratatui::widgets::TableState;
use crate::models::rds::RdsInstance;
use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
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

impl ServiceInternal for RdsState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.rds.clone();
        let handle = spawn_list_task(
            tx,
            move || async move {
                crate::aws::rds::RdsService::new(client)
                    .list_instances()
                    .await
            },
            AwsEvent::RdsInstancesLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::RDS_REFRESH, handle);
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
    use crate::app::global_search::AutoSelectable;

    fn sample_instance(id: &str) -> RdsInstance {
        RdsInstance {
            db_instance_identifier: id.to_string(),
            db_instance_class: "db.t3.micro".to_string(),
            engine: "postgres".to_string(),
            engine_version: Some("16.1".to_string()),
            status: "available".to_string(),
            endpoint: None,
            port: None,
            master_username: Some("admin".to_string()),
            allocated_storage: Some(20),
            availability_zone: Some("us-east-1a".to_string()),
            multi_az: false,
            publicly_accessible: false,
            storage_type: Some("gp3".to_string()),
            storage_encrypted: true,
            vpc_id: Some("vpc-123".to_string()),
            db_subnet_group: Some("subnet-group".to_string()),
            security_groups: vec!["sg-123".to_string()],
            backup_retention_period: Some(7),
            preferred_backup_window: Some("02:00-03:00".to_string()),
            preferred_maintenance_window: Some("sun:05:00-sun:06:00".to_string()),
            created_time: Some("2024-01-01T00:00:00Z".to_string()),
            auto_minor_version_upgrade: true,
            license_model: Some("postgresql-license".to_string()),
            iops: None,
            deletion_protection: false,
            tags: vec![],
        }
    }

    #[test]
    fn test_rds_state_new() {
        let state = RdsState::new();
        assert!(state.instances.is_empty());
        assert_eq!(state.list_state.selected(), None);
    }

    #[test]
    fn test_rds_state_selection_empty() {
        let state = RdsState::new();
        assert_eq!(state.selected_instance(), None);
        assert_eq!(state.selected_instance_id(), None);
    }

    #[test]
    fn test_rds_state_select_by_id() {
        let mut state = RdsState::new();
        state.instances.push(sample_instance("db-1"));
        state.instances.push(sample_instance("db-2"));

        assert!(state.select_by_id("db-2"));
        assert_eq!(state.list_state.selected(), Some(1));
        assert_eq!(state.selected_instance_id(), Some("db-2".to_string()));
        assert!(!state.select_by_id("db-missing"));
    }

    #[test]
    fn test_rds_state_auto_select_first() {
        let mut state = RdsState::new();
        state.auto_select_first();
        assert_eq!(state.list_state.selected(), None);

        state.instances.push(sample_instance("db-1"));
        state.auto_select_first();
        assert_eq!(state.list_state.selected(), Some(0));
    }

    #[test]
    fn test_rds_state_clear_removes_instances() {
        let mut state = RdsState::new();
        state.instances.push(sample_instance("db-1"));

        assert_eq!(state.instances.len(), 1);
        state.clear();
        assert!(state.instances.is_empty());
        assert_eq!(state.list_state.selected(), Some(0));
    }
}
