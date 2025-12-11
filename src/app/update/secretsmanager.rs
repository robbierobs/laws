//! SecretsManager update handlers
//!
//! Handles SecretsManager-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use crate::event::{AwsEvent, Event};

impl App {
    pub(super) fn handle_get_secret_value(
        &mut self,
        arn: String,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.secretsmanager.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            let service = crate::aws::secretsmanager::SecretsManagerService::new(client);
            match service.get_secret_value(&arn).await {
                Ok(value) => {
                    tx.send(Event::Aws(AwsEvent::SecretsManagerSecretValueLoaded(value)))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::SECRETSMANAGER_ACTION, handle);
    }
}
