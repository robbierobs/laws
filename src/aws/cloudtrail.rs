use crate::error::AppResult;
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::utils::error::format_sdk_error;
use aws_sdk_cloudtrail::Client;

pub struct CloudTrailService {
    client: Client,
}

impl CloudTrailService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_trails(&self) -> AppResult<Vec<Trail>> {
        let response = self
            .client
            .describe_trails()
            .send()
            .await
            .map_err(|e| format_sdk_error("CloudTrail", "describe_trails", "all", e))?;

        let trails = response
            .trail_list()
            .iter()
            .map(|t| Trail::from_aws(t))
            .collect();

        Ok(trails)
    }

    pub async fn lookup_events(&self, max_results: i32) -> AppResult<Vec<CloudTrailEvent>> {
        let response = self
            .client
            .lookup_events()
            .max_results(max_results)
            .send()
            .await
            .map_err(|e| format_sdk_error("CloudTrail", "lookup_events", "all", e))?;

        let events = response
            .events()
            .iter()
            .map(|e| CloudTrailEvent::from_aws(e))
            .collect();

        Ok(events)
    }
    pub async fn delete_trail(&self, name: &str) -> AppResult<()> {
        self.client
            .delete_trail()
            .name(name)
            .send()
            .await
            .map_err(|e| format_sdk_error("CloudTrail", "delete_trail", name, e))?;
        Ok(())
    }
}

impl crate::aws::traits::AwsService<Trail> for CloudTrailService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<Trail>>> + Send + 'a>>
    {
        Box::pin(self.list_trails())
    }
}
