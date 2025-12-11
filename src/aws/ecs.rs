use crate::error::AppResult;
use crate::models::ecs::{EcsCluster, EcsService};
use crate::utils::error::format_sdk_error;
use aws_sdk_ecs::Client;

pub struct EcsClient {
    client: Client,
}

impl EcsClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_clusters(&self) -> AppResult<Vec<EcsCluster>> {
        let list_output = self
            .client
            .list_clusters()
            .send()
            .await
            .map_err(|e| format_sdk_error("ECS", "list_clusters", "all", e))?;

        let cluster_arns = list_output.cluster_arns();
        if cluster_arns.is_empty() {
            return Ok(Vec::new());
        }

        // Describe clusters to get details
        let describe_output = self
            .client
            .describe_clusters()
            .set_clusters(Some(cluster_arns.into()))
            .send()
            .await
            .map_err(|e| format_sdk_error("ECS", "describe_clusters", "all", e))?;

        let clusters = describe_output
            .clusters()
            .iter()
            .map(|c| EcsCluster::from_aws(c.clone()))
            .collect();

        Ok(clusters)
    }

    pub async fn list_services(&self, cluster_arn: &str) -> AppResult<Vec<EcsService>> {
        let list_output = self
            .client
            .list_services()
            .cluster(cluster_arn)
            .send()
            .await
            .map_err(|e| format_sdk_error("ECS", "list_services", cluster_arn, e))?;

        let service_arns = list_output.service_arns();
        if service_arns.is_empty() {
            return Ok(Vec::new());
        }

        // DescribeServices: up to 10 services per call, so chunk if needed
        let mut services = Vec::new();

        for chunk in service_arns.chunks(10) {
            let describe_output = self
                .client
                .describe_services()
                .cluster(cluster_arn)
                .set_services(Some(chunk.iter().map(|s| s.to_string()).collect()))
                .send()
                .await
                .map_err(|e| format_sdk_error("ECS", "describe_services", cluster_arn, e))?;

            services.extend(
                describe_output
                    .services()
                    .iter()
                    .map(|s| EcsService::from_aws(s.clone())),
            );
        }

        Ok(services)
    }
}

impl crate::aws::traits::AwsService<EcsCluster> for EcsClient {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<EcsCluster>>> + Send + 'a>>
    {
        Box::pin(self.list_clusters())
    }
}
