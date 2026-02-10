use crate::error::AppResult;
use crate::models::sqs::SqsQueue;
use crate::utils::error::format_sdk_error;
use aws_sdk_sqs::Client;

crate::aws_service_struct!(SqsService, Client);

impl SqsService {
    pub async fn list_queues(&self) -> AppResult<Vec<SqsQueue>> {
        let mut queues = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.list_queues();
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("SQS", "list_queues", "all", e))?;

            for url in response.queue_urls() {
                if let Ok(queue) = self.get_queue_attributes(url).await {
                    queues.push(queue);
                }
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(queues)
    }

    async fn get_queue_attributes(&self, queue_url: &str) -> AppResult<SqsQueue> {
        use aws_sdk_sqs::types::QueueAttributeName;

        let response = self
            .client
            .get_queue_attributes()
            .queue_url(queue_url)
            .attribute_names(QueueAttributeName::All)
            .send()
            .await
            .map_err(|e| format_sdk_error("SQS", "get_queue_attributes", queue_url, e))?;

        Ok(SqsQueue::from_aws(queue_url, response.attributes()))
    }

    #[allow(dead_code)] // TODO: Wire up SQS send message action
    pub async fn send_message(&self, queue_url: &str, body: &str) -> AppResult<String> {
        let response = self
            .client
            .send_message()
            .queue_url(queue_url)
            .message_body(body)
            .send()
            .await
            .map_err(|e| format_sdk_error("SQS", "send_message", queue_url, e))?;

        Ok(response
            .message_id()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string()))
    }

    pub async fn purge_queue(&self, queue_url: &str) -> AppResult<()> {
        self.client
            .purge_queue()
            .queue_url(queue_url)
            .send()
            .await
            .map_err(|e| format_sdk_error("SQS", "purge_queue", queue_url, e))?;
        Ok(())
    }

    pub async fn delete_queue(&self, queue_url: &str) -> AppResult<()> {
        self.client
            .delete_queue()
            .queue_url(queue_url)
            .send()
            .await
            .map_err(|e| format_sdk_error("SQS", "delete_queue", queue_url, e))?;
        Ok(())
    }
}

impl crate::aws::traits::AwsService<SqsQueue> for SqsService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<SqsQueue>>> + Send + 'a>>
    {
        Box::pin(self.list_queues())
    }
}
