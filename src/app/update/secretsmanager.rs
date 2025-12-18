//! SecretsManager update handlers
//!
//! Handles SecretsManager-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use crate::app::messages::SecretsManagerAction;
use crate::event::{AwsEvent, Event};

impl App {
    /// Main entry point for SecretsManager actions
    pub(super) fn handle_secretsmanager_action(
        &mut self,
        action: SecretsManagerAction,
        event_tx: crate::app::EventSender,
    ) {
        match action {
            SecretsManagerAction::GetSecretValue(arn) => {
                self.handle_get_secret_value(arn, event_tx);
            }
            SecretsManagerAction::CloseSecretValue => {
                self.services.secretsmanager.show_secret_modal = false;
                self.services.secretsmanager.secret_value = None;
            }
            SecretsManagerAction::DeleteSecret(arn) => {
                self.handle_delete_secret(arn, event_tx);
            }
        }
    }

    fn handle_get_secret_value(
        &mut self,
        arn: String,
        event_tx: crate::app::EventSender,
    ) {
        self.spawn_aws_task(event_tx, task_keys::SECRETSMANAGER_ACTION, move |clients, tx| async move {
            let service = crate::aws::secretsmanager::SecretsManagerService::new(clients.secretsmanager.clone());
            match service.get_secret_value(&arn).await {
                Ok(value) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::SecretsManagerSecretValueLoaded(value))))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
    }

    fn handle_delete_secret(&mut self, arn: String, event_tx: crate::app::EventSender) {
        self.action_log.push(format!("Deleting Secret: {}", arn));
        
        self.spawn_aws_task(event_tx, task_keys::SECRETSMANAGER_ACTION, move |clients, tx| async move {
            let service = crate::aws::secretsmanager::SecretsManagerService::new(clients.secretsmanager.clone());
            match service.delete_secret(&arn).await {
                Ok(_) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(
                        format!("Secret {} deleted", arn)
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

