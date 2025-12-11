//! EC2 update handlers
//!
//! Handles EC2-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use super::instance_actions::execute_instance_action;

impl App {
    pub(super) fn handle_ec2_action(
        &mut self,
        action: &str,
        id: String,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.ec2.clone();
        let tx = event_tx;
        let action = action.to_string();

        let handle = tokio::spawn(async move {
            let service = crate::aws::ec2::Ec2Service::new(client);
            execute_instance_action(service, &action, id, tx).await;
        });

        self.tasks.spawn(task_keys::EC2_ACTION, handle);
    }
}
