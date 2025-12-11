//! EC2 update handlers
//!
//! Handles EC2-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use crate::event::{AwsEvent, Event};
use tokio::sync::mpsc;

impl App {
    pub(super) fn handle_ec2_action(
        &mut self,
        action: &str,
        id: String,
        event_tx: mpsc::UnboundedSender<Event>,
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
            let result = match action.as_str() {
                "start" => service.start_instance(&id).await,
                "stop" => service.stop_instance(&id).await,
                "reboot" => service.reboot_instance(&id).await,
                _ => return,
            };

            match result {
                Ok(_) => {
                    let msg = format!("{}ed instance {}", action.trim_end_matches('e'), id);
                    tx.send(Event::Aws(AwsEvent::ActionCompleted(msg))).ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                }
            }
        });

        self.tasks.spawn(task_keys::EC2_ACTION, handle);
    }
}
