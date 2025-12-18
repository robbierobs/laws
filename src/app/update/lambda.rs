//! Lambda update handlers
//!
//! Handles Lambda-specific state mutations and async operations.
//! All handlers follow the standardized pattern: sync function that spawns
//! a tracked task via the TaskManager.

use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};

impl App {
    /// Invoke a Lambda function
    /// 
    /// Spawns a tracked async task that invokes the function and sends
    /// the result back via the event channel.
    pub(super) fn handle_invoke_lambda(&mut self, function_name: String, event_tx: EventSender) {
        self.action_log.push(format!("Invoking Lambda function: {}", function_name));
        
        // Clone for async block
        let func_name = function_name.clone();

        self.spawn_aws_task(
            event_tx, 
            task_keys::LAMBDA_ACTION, 
            move |clients, tx| async move {
                let service = crate::aws::lambda::LambdaService::new(clients.lambda.clone());
                match service.invoke_function(&func_name).await {
                    Ok(payload) => {
                        // Truncate payload if too long for event message
                        let display_payload = if payload.len() > 100 {
                            format!("{}...", &payload[0..100])
                        } else {
                            payload
                        };
                        tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(
                            format!("Function invoked. Payload: {}", display_payload)
                        )))).await.ok();
                    }
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                    }
                }
            }
        );
    }

    /// Delete a Lambda function
    /// 
    /// Spawns a tracked async task that deletes the function and triggers
    /// a refresh on success.
    pub(super) fn handle_delete_lambda(&mut self, function_name: String, event_tx: EventSender) {
        self.action_log.push(format!("Deleting Lambda function: {}", function_name));
        
        let func_name = function_name.clone();

        self.spawn_aws_task(
            event_tx, 
            task_keys::LAMBDA_ACTION, 
            move |clients, tx| async move {
                let service = crate::aws::lambda::LambdaService::new(clients.lambda.clone());
                match service.delete_function(&func_name).await {
                    Ok(_) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(
                            format!("Function {} deleted", func_name)
                        )))).await.ok();
                        // Trigger refresh
                        tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                    }
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                    }
                }
            }
        );
    }

    /// Load detailed information for a Lambda function
    /// 
    /// Spawns a tracked async task that fetches function details and sends
    /// them back via the event channel.
    pub(super) fn handle_load_function_details(&mut self, function_name: String, event_tx: EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        // Skip if already loading
        if let Some(details) = self.services.lambda.function_details.get_mut(&function_name) {
            if details.loading {
                return;
            }
        }
        
        // Mark as loading
        self.services.lambda.function_details.insert(
            function_name.clone(), 
            crate::models::lambda::LambdaFunctionDetails {
                loading: true,
                ..Default::default()
            }
        );
        
        let client = clients.lambda.clone();
        
        let handle = tokio::spawn(async move {
            let service = crate::aws::lambda::LambdaService::new(client);
            match service.get_function_details(&function_name).await {
                Ok(details) => {
                    event_tx.send(Event::Aws(Box::new(AwsEvent::LambdaFunctionDetailsLoaded {
                        function_name,
                        details,
                    }))).await.ok();
                }
                Err(e) => {
                    event_tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
        
        self.tasks.spawn(task_keys::LAMBDA_DETAILS, handle);
    }
}
