//! CloudTrail update handlers
//!
//! Handles CloudTrail-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use crate::app::messages::CloudTrailAction;
use crate::event::{AwsEvent, Event};

impl App {
    /// Main entry point for CloudTrail actions
    pub(super) fn handle_cloudtrail_action(
        &mut self,
        action: CloudTrailAction,
        event_tx: crate::app::EventSender,
    ) {
        match action {
            CloudTrailAction::ShowEventDetails(json) => {
                self.services.cloudtrail.selected_event_detail = Some(json);
                self.services.cloudtrail.show_detail_modal = true;
            }
            CloudTrailAction::CloseEventDetails => {
                self.services.cloudtrail.show_detail_modal = false;
                self.services.cloudtrail.selected_event_detail = None;
            }
            CloudTrailAction::DeleteTrail(name) => {
                self.handle_delete_trail(name, event_tx);
            }
        }
    }

    fn handle_delete_trail(&mut self, name: String, event_tx: crate::app::EventSender) {
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

