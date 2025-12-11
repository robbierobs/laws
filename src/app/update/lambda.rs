use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};

impl App {
    pub(super) async fn handle_invoke_lambda(&mut self, function_name: String, event_tx: EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.action_log.push(format!("Invoking Lambda function: {}", function_name));
        
        let client = clients.lambda.clone();
        
        tokio::spawn(async move {
            let service = crate::aws::lambda::LambdaService::new(client);
            match service.invoke_function(&function_name).await {
                Ok(payload) => {
                    // Truncate payload if too long for event message
                    let display_payload = if payload.len() > 100 {
                        format!("{}...", &payload[0..100])
                    } else {
                        payload
                    };
                    event_tx.send(Event::Aws(AwsEvent::ActionCompleted(
                        format!("Function invoked. Payload: {}", display_payload)
                    ))).await.ok();
                }
                Err(e) => {
                    event_tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });
    }
}
