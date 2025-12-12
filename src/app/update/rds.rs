//! RDS update handlers
//!
//! Handles RDS-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use super::instance_actions::execute_instance_action;

impl App {
    pub(super) fn handle_rds_action(
        &mut self,
        action: &str,
        id: String,
        event_tx: crate::app::EventSender,
    ) {
        let action = action.to_string();
        
        self.spawn_aws_task(event_tx, task_keys::RDS_ACTION, move |clients, tx| async move {
            let service = crate::aws::rds::RdsService::new(clients.rds.clone());
            execute_instance_action(service, &action, id, tx).await;
        });
    }
}
