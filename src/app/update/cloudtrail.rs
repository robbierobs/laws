//! CloudTrail update handlers
//!
//! Handles CloudTrail-specific state mutations and async operations.

use super::super::App;

impl App {
    pub(super) fn handle_delete_trail(&mut self, name: String, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.action_log.push(format!("Deleting CloudTrail: {}", name));
        
        let client = clients.cloudtrail.clone();
        
        tokio::spawn(async move {
             let service = crate::aws::cloudtrail::CloudTrailService::new(client);
             match service.delete_trail(&name).await {
                 Ok(_) => {
                     event_tx.send(crate::event::Event::Aws(crate::event::AwsEvent::ActionCompleted(
                         format!("Trail {} deleted", name)
                     ))).await.ok();
                     // Trigger refresh
                     event_tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                 }
                 Err(e) => {
                     event_tx.send(crate::event::Event::Aws(crate::event::AwsEvent::Error(e.to_string()))).await.ok();
                 }
             }
        });
    }
}
