use crate::error::AppResult;
use crate::models::ecs::{EcsCluster, EcsService, EcsTask, EcsTaskDefinition};
use crate::utils::error::format_sdk_error;
use aws_sdk_ecs::Client;

pub struct EcsClient {
    client: Client,
}

impl EcsClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    // ========================================================================
    // Cluster Operations
    // ========================================================================

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

        let describe_output = self
            .client
            .describe_clusters()
            .set_clusters(Some(cluster_arns.into()))
            .include(aws_sdk_ecs::types::ClusterField::Settings)
            .include(aws_sdk_ecs::types::ClusterField::Statistics)
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

    // ========================================================================
    // Service Operations
    // ========================================================================

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
                .include(aws_sdk_ecs::types::ServiceField::Tags)
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

    pub async fn update_service_desired_count(
        &self,
        cluster_arn: &str,
        service_name: &str,
        desired_count: i32,
    ) -> AppResult<()> {
        self.client
            .update_service()
            .cluster(cluster_arn)
            .service(service_name)
            .desired_count(desired_count)
            .send()
            .await
            .map_err(|e| format_sdk_error("ECS", "update_service", service_name, e))?;
        Ok(())
    }

    pub async fn force_new_deployment(
        &self,
        cluster_arn: &str,
        service_name: &str,
    ) -> AppResult<()> {
        self.client
            .update_service()
            .cluster(cluster_arn)
            .service(service_name)
            .force_new_deployment(true)
            .send()
            .await
            .map_err(|e| format_sdk_error("ECS", "force_deployment", service_name, e))?;
        Ok(())
    }

    // ========================================================================
    // Task Operations
    // ========================================================================

    pub async fn list_tasks(
        &self,
        cluster_arn: &str,
        service_name: Option<&str>,
    ) -> AppResult<Vec<EcsTask>> {
        let mut request = self.client.list_tasks().cluster(cluster_arn);

        if let Some(service) = service_name {
            request = request.service_name(service);
        }

        let list_output = request
            .send()
            .await
            .map_err(|e| format_sdk_error("ECS", "list_tasks", cluster_arn, e))?;

        let task_arns = list_output.task_arns();
        if task_arns.is_empty() {
            return Ok(Vec::new());
        }

        self.describe_tasks(cluster_arn, task_arns).await
    }

    pub async fn describe_tasks(
        &self,
        cluster_arn: &str,
        task_arns: &[String],
    ) -> AppResult<Vec<EcsTask>> {
        if task_arns.is_empty() {
            return Ok(Vec::new());
        }

        // DescribeTasks: up to 100 tasks per call
        let mut tasks = Vec::new();

        for chunk in task_arns.chunks(100) {
            let describe_output = self
                .client
                .describe_tasks()
                .cluster(cluster_arn)
                .set_tasks(Some(chunk.iter().map(|s| s.to_string()).collect()))
                .include(aws_sdk_ecs::types::TaskField::Tags)
                .send()
                .await
                .map_err(|e| format_sdk_error("ECS", "describe_tasks", cluster_arn, e))?;

            tasks.extend(
                describe_output
                    .tasks()
                    .iter()
                    .map(|t| EcsTask::from_aws(t.clone())),
            );
        }

        Ok(tasks)
    }

    pub async fn stop_task(
        &self,
        cluster_arn: &str,
        task_arn: &str,
        reason: &str,
    ) -> AppResult<()> {
        self.client
            .stop_task()
            .cluster(cluster_arn)
            .task(task_arn)
            .reason(reason)
            .send()
            .await
            .map_err(|e| format_sdk_error("ECS", "stop_task", task_arn, e))?;
        Ok(())
    }

    // ========================================================================
    // Task Definition Operations
    // ========================================================================

    pub async fn describe_task_definition(
        &self,
        task_definition: &str,
    ) -> AppResult<EcsTaskDefinition> {
        let output = self
            .client
            .describe_task_definition()
            .task_definition(task_definition)
            .include(aws_sdk_ecs::types::TaskDefinitionField::Tags)
            .send()
            .await
            .map_err(|e| format_sdk_error("ECS", "describe_task_definition", task_definition, e))?;

        let td = output
            .task_definition()
            .ok_or_else(|| crate::error::AppError::NotFound {
                resource_type: "TaskDefinition".to_string(),
                resource_id: task_definition.to_string(),
            })?;

        Ok(EcsTaskDefinition::from_aws(td.clone()))
    }

    #[allow(dead_code)]
    pub async fn list_task_definition_families(&self) -> AppResult<Vec<String>> {
        let mut families = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.list_task_definition_families();
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let output = request
                .send()
                .await
                .map_err(|e| format_sdk_error("ECS", "list_task_definition_families", "all", e))?;

            families.extend(output.families().iter().map(|f| f.to_string()));

            next_token = output.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(families)
    }

    pub async fn deregister_task_definition(&self, task_definition: &str) -> AppResult<()> {
        self.client
            .deregister_task_definition()
            .task_definition(task_definition)
            .send()
            .await
            .map_err(|e| {
                format_sdk_error("ECS", "deregister_task_definition", task_definition, e)
            })?;
        Ok(())
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
