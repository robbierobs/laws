//! CloudTrail update handlers
//!
//! Handles CloudTrail-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use crate::event::{AwsEvent, Event};

impl App {
    pub(super) fn handle_delete_trail(&mut self, name: String, event_tx: crate::app::EventSender) {
        self.action_log.push(format!("Deleting CloudTrail: {}", name));
        
        self.spawn_aws_task(event_tx, task_keys::CLOUDTRAIL_ACTION, move |clients, tx| async move {
            let service = crate::aws::cloudtrail::CloudTrailService::new(clients.cloudtrail.clone());
            match service.delete_trail(&name).await {
                Ok(_) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(
                        format!("Trail {} deleted", name)
                    )))).await.ok();
                    // Trigger refresh
                    tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
    }
}
