//! RDS update handlers
//!
//! Handles RDS-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use crate::event::{AwsEvent, Event};

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
            let result = match action.as_str() {
                "start" => service.start_instance(&id).await,
                "stop" => service.stop_instance(&id).await,
                "reboot" => service.reboot_instance(&id).await,
                _ => return,
            };

            match result {
                Ok(_) => {
                    let msg = format!("{}ed RDS instance {}", action.trim_end_matches('e'), id);
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(msg))).await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::RDS_ACTION, handle);
    }
}
