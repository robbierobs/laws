//! SQS update handlers
//!
//! Handles SQS-specific state mutations and async operations.

use crate::app::messages::SqsAction;
use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};

impl App {
    /// Main entry point for SQS actions
    pub(super) fn handle_sqs_action(&mut self, action: SqsAction, event_tx: EventSender) {
        match action {
            SqsAction::PurgeQueue(url) => {
                self.handle_purge_queue(url, event_tx);
            }
            SqsAction::DeleteQueue(url) => {
                self.handle_delete_queue(url, event_tx);
            }
        }
    }

    /// Purge all messages from an SQS queue
    fn handle_purge_queue(&mut self, queue_url: String, event_tx: EventSender) {
        self.action_log
            .push(format!("Purging SQS queue: {}", queue_url));

        let url = queue_url.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::SQS_ACTION,
            move |clients, tx| async move {
                let service = crate::aws::sqs::SqsService::new(clients.sqs.clone());
                match service.purge_queue(&url).await {
                    Ok(_) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(format!(
                            "Queue purged: {}",
                            url
                        )))))
                        .await
                        .ok();
                        tx.send(crate::event::Event::Message(crate::app::Message::refresh()))
                            .await
                            .ok();
                    }
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                            .await
                            .ok();
                    }
                }
            },
        );
    }

    /// Delete an SQS queue
    fn handle_delete_queue(&mut self, queue_url: String, event_tx: EventSender) {
        self.action_log
            .push(format!("Deleting SQS queue: {}", queue_url));

        let url = queue_url.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::SQS_ACTION,
            move |clients, tx| async move {
                let service = crate::aws::sqs::SqsService::new(clients.sqs.clone());
                match service.delete_queue(&url).await {
                    Ok(_) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(format!(
                            "Queue deleted: {}",
                            url
                        )))))
                        .await
                        .ok();
                        tx.send(crate::event::Event::Message(crate::app::Message::refresh()))
                            .await
                            .ok();
                    }
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                            .await
                            .ok();
                    }
                }
            },
        );
    }
}
