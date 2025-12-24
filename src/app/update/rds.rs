//! RDS update handlers
//!
//! Handles RDS-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use super::instance_actions::execute_instance_action;
use crate::app::messages::RdsAction;

impl App {
    /// Main entry point for RDS actions
    pub(super) fn handle_rds_action(
        &mut self,
        action: RdsAction,
        event_tx: crate::app::EventSender,
    ) {
        let (action_str, id) = match action {
            RdsAction::Start(id) => ("start", id),
            RdsAction::Stop(id) => ("stop", id),
            RdsAction::Reboot(id) => ("reboot", id),
            RdsAction::Delete(id) => ("terminate", id),
        };
        
        let action_string = action_str.to_string();
        self.spawn_aws_task(event_tx, task_keys::RDS_ACTION, move |clients, tx| async move {
            let service = crate::aws::rds::RdsService::new(clients.rds.clone());
            execute_instance_action(service, &action_string, id, tx).await;
        });
    }
}

