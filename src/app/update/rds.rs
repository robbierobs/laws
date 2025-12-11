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
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.rds.clone();
        let tx = event_tx;
        let action = action.to_string();

        let handle = tokio::spawn(async move {
            let service = crate::aws::rds::RdsService::new(client);
            execute_instance_action(service, &action, id, tx).await;
        });

        self.tasks.spawn(task_keys::RDS_ACTION, handle);
    }
}
