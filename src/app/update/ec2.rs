//! EC2 update handlers
//!
//! Handles EC2-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use super::instance_actions::execute_instance_action;
use crate::app::messages::Ec2Action;

impl App {
    /// Main entry point for EC2 actions
    pub(super) fn handle_ec2_action(
        &mut self,
        action: Ec2Action,
        event_tx: crate::app::EventSender,
    ) {
        let (action_str, id) = match action {
            Ec2Action::Start(id) => ("start", id),
            Ec2Action::Stop(id) => ("stop", id),
            Ec2Action::Reboot(id) => ("reboot", id),
            Ec2Action::Terminate(id) => ("terminate", id),
        };
        
        let action_string = action_str.to_string();
        self.spawn_aws_task(event_tx, task_keys::EC2_ACTION, move |clients, tx| async move {
            let service = crate::aws::ec2::Ec2Service::new(clients.ec2.clone());
            execute_instance_action(service, &action_string, id, tx).await;
        });
    }
}

