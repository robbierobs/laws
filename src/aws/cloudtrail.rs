use crate::models::cloudtrail::{Trail, CloudTrailEvent};
use aws_sdk_cloudtrail::Client;

pub struct CloudTrailService {
    client: Client,
}

impl CloudTrailService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_trails(&self) -> anyhow::Result<Vec<Trail>> {
        let response = self.client
            .describe_trails()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list trails: {}", e))?;

        let trails = response.trail_list()
            .iter()
            .map(|t| Trail::from_aws(t))
            .collect();

        Ok(trails)
    }

    pub async fn lookup_events(&self, max_results: i32) -> anyhow::Result<Vec<CloudTrailEvent>> {
        let response = self.client
            .lookup_events()
            .max_results(max_results)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to lookup events: {}", e))?;

        let events = response.events()
            .iter()
            .map(|e| CloudTrailEvent::from_aws(e))
            .collect();

        Ok(events)
    }
}

impl crate::aws::traits::AwsService<Trail> for CloudTrailService {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<Trail>>> + Send + 'a>> {
        Box::pin(self.list_trails())
    }
}
