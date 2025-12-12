use crate::error::{AppError, AppResult};
use crate::models::ecs::{EcsCluster, EcsService, EcsTask, EcsTaskDefinition};
use crate::utils::error::format_sdk_error;
use aws_sdk_ecs::Client;

pub struct EcsClient {
    pub(crate) client: Client,
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

    /// Register a new task definition from JSON
    /// 
    /// The JSON should be an EcsTaskDefinition struct. This function strips read-only
    /// fields (taskDefinitionArn, revision, status, registeredAt, registeredBy) and
    /// uses the remaining fields to register a new revision.
    pub async fn register_task_definition(&self, task_def_json: &str) -> AppResult<String> {
        // Parse the JSON into our model
        let task_def: crate::models::ecs::EcsTaskDefinition = serde_json::from_str(task_def_json)
            .map_err(|e| AppError::internal(format!("Failed to parse task definition JSON: {}", e)))?;

        // Build the registration request, stripping read-only fields
        let mut request = self.client.register_task_definition()
            .family(&task_def.family);

        // Optional top-level fields
        if let Some(ref role) = task_def.task_role_arn {
            request = request.task_role_arn(role);
        }
        if let Some(ref role) = task_def.execution_role_arn {
            request = request.execution_role_arn(role);
        }
        if let Some(ref mode) = task_def.network_mode {
            if let Ok(nm) = mode.parse::<aws_sdk_ecs::types::NetworkMode>() {
                request = request.network_mode(nm);
            }
        }
        if let Some(ref cpu) = task_def.cpu {
            request = request.cpu(cpu);
        }
        if let Some(ref mem) = task_def.memory {
            request = request.memory(mem);
        }
        if let Some(ref pid) = task_def.pid_mode {
            if let Ok(pm) = pid.parse::<aws_sdk_ecs::types::PidMode>() {
                request = request.pid_mode(pm);
            }
        }
        if let Some(ref ipc) = task_def.ipc_mode {
            if let Ok(im) = ipc.parse::<aws_sdk_ecs::types::IpcMode>() {
                request = request.ipc_mode(im);
            }
        }

        // Requires compatibilities
        for compat in &task_def.requires_compatibilities {
            if let Ok(c) = compat.parse::<aws_sdk_ecs::types::Compatibility>() {
                request = request.requires_compatibilities(c);
            }
        }

        // Runtime platform
        if let Some(ref rp) = task_def.runtime_platform {
            let mut runtime_platform = aws_sdk_ecs::types::RuntimePlatform::builder();
            if let Some(ref arch) = rp.cpu_architecture {
                if let Ok(a) = arch.parse::<aws_sdk_ecs::types::CpuArchitecture>() {
                    runtime_platform = runtime_platform.cpu_architecture(a);
                }
            }
            if let Some(ref os) = rp.operating_system_family {
                if let Ok(o) = os.parse::<aws_sdk_ecs::types::OsFamily>() {
                    runtime_platform = runtime_platform.operating_system_family(o);
                }
            }
            request = request.runtime_platform(runtime_platform.build());
        }

        // Ephemeral storage
        if let Some(size) = task_def.ephemeral_storage_size_gib {
            request = request.ephemeral_storage(
                aws_sdk_ecs::types::EphemeralStorage::builder()
                    .size_in_gib(size)
                    .build()
            );
        }

        // Container definitions
        for cd in &task_def.container_definitions {
            let mut container = aws_sdk_ecs::types::ContainerDefinition::builder()
                .name(&cd.name)
                .cpu(cd.cpu)
                .essential(cd.essential)
                .privileged(cd.privileged)
                .readonly_root_filesystem(cd.readonly_root_filesystem);

            if let Some(ref image) = cd.image {
                container = container.image(image);
            }
            if let Some(mem) = cd.memory {
                container = container.memory(mem);
            }
            if let Some(mem_res) = cd.memory_reservation {
                container = container.memory_reservation(mem_res);
            }
            if let Some(ref user) = cd.user {
                container = container.user(user);
            }
            if let Some(ref wd) = cd.working_directory {
                container = container.working_directory(wd);
            }
            if let Some(start) = cd.start_timeout {
                container = container.start_timeout(start);
            }
            if let Some(stop) = cd.stop_timeout {
                container = container.stop_timeout(stop);
            }

            // Entry point and command
            for ep in &cd.entry_point {
                container = container.entry_point(ep);
            }
            for cmd in &cd.command {
                container = container.command(cmd);
            }

            // Environment variables
            for (name, value) in &cd.environment {
                container = container.environment(
                    aws_sdk_ecs::types::KeyValuePair::builder()
                        .name(name)
                        .value(value)
                        .build()
                );
            }

            // Secrets
            for (name, value_from) in &cd.secrets {
                container = container.secrets(
                    aws_sdk_ecs::types::Secret::builder()
                        .name(name)
                        .value_from(value_from)
                        .build()
                        .map_err(|e| AppError::internal(format!("Failed to build secret: {}", e)))?
                );
            }

            // Port mappings
            for pm in &cd.port_mappings {
                let mut port_mapping = aws_sdk_ecs::types::PortMapping::builder();
                if let Some(cp) = pm.container_port {
                    port_mapping = port_mapping.container_port(cp);
                }
                if let Some(hp) = pm.host_port {
                    port_mapping = port_mapping.host_port(hp);
                }
                if let Some(ref proto) = pm.protocol {
                    if let Ok(p) = proto.parse::<aws_sdk_ecs::types::TransportProtocol>() {
                        port_mapping = port_mapping.protocol(p);
                    }
                }
                if let Some(ref name) = pm.name {
                    port_mapping = port_mapping.name(name);
                }
                if let Some(ref ap) = pm.app_protocol {
                    if let Ok(a) = ap.parse::<aws_sdk_ecs::types::ApplicationProtocol>() {
                        port_mapping = port_mapping.app_protocol(a);
                    }
                }
                container = container.port_mappings(port_mapping.build());
            }

            // Log configuration
            if let Some(ref lc) = cd.log_configuration {
                let mut log_config = aws_sdk_ecs::types::LogConfiguration::builder();
                if let Ok(driver) = lc.log_driver.parse::<aws_sdk_ecs::types::LogDriver>() {
                    log_config = log_config.log_driver(driver);
                }
                for (key, value) in &lc.options {
                    log_config = log_config.options(key.clone(), value.clone());
                }
                for (name, value_from) in &lc.secret_options {
                    log_config = log_config.secret_options(
                        aws_sdk_ecs::types::Secret::builder()
                            .name(name)
                            .value_from(value_from)
                            .build()
                            .map_err(|e| AppError::internal(format!("Failed to build log secret: {}", e)))?
                    );
                }
                if let Ok(built) = log_config.build() {
                    container = container.log_configuration(built);
                }
            }

            // Health check
            if let Some(ref hc) = cd.health_check {
                let mut health_check = aws_sdk_ecs::types::HealthCheck::builder();
                for cmd in &hc.command {
                    health_check = health_check.command(cmd);
                }
                if let Some(interval) = hc.interval {
                    health_check = health_check.interval(interval);
                }
                if let Some(timeout) = hc.timeout {
                    health_check = health_check.timeout(timeout);
                }
                if let Some(retries) = hc.retries {
                    health_check = health_check.retries(retries);
                }
                if let Some(start_period) = hc.start_period {
                    health_check = health_check.start_period(start_period);
                }
                if let Ok(built) = health_check.build() {
                    container = container.health_check(built);
                }
            }

            // Mount points
            for mp in &cd.mount_points {
                let mut mount_point = aws_sdk_ecs::types::MountPoint::builder()
                    .read_only(mp.read_only);
                if let Some(ref sv) = mp.source_volume {
                    mount_point = mount_point.source_volume(sv);
                }
                if let Some(ref cp) = mp.container_path {
                    mount_point = mount_point.container_path(cp);
                }
                container = container.mount_points(mount_point.build());
            }

            // Depends on
            for dep in &cd.depends_on {
                if let Ok(cond) = dep.condition.parse::<aws_sdk_ecs::types::ContainerCondition>() {
                    container = container.depends_on(
                        aws_sdk_ecs::types::ContainerDependency::builder()
                            .container_name(&dep.container_name)
                            .condition(cond)
                            .build()
                            .map_err(|e| AppError::internal(format!("Failed to build dependency: {}", e)))?
                    );
                }
            }

            request = request.container_definitions(container.build());
        }

        // Volumes
        for vol in &task_def.volumes {
            let mut volume = aws_sdk_ecs::types::Volume::builder()
                .name(&vol.name);
            
            if let Some(ref hp) = vol.host_path {
                volume = volume.host(
                    aws_sdk_ecs::types::HostVolumeProperties::builder()
                        .source_path(hp)
                        .build()
                );
            }
            
            request = request.volumes(volume.build());
        }

        // Placement constraints
        for pc in &task_def.placement_constraints {
            let mut constraint = aws_sdk_ecs::types::TaskDefinitionPlacementConstraint::builder();
            if let Some(ref t) = pc.r#type {
                if let Ok(pt) = t.parse::<aws_sdk_ecs::types::TaskDefinitionPlacementConstraintType>() {
                    constraint = constraint.r#type(pt);
                }
            }
            if let Some(ref expr) = pc.expression {
                constraint = constraint.expression(expr);
            }
            request = request.placement_constraints(constraint.build());
        }

        // Send the request
        let output = request.send().await
            .map_err(|e| format_sdk_error("ECS", "register_task_definition", &task_def.family, e))?;

        // Return the new task definition ARN
        let new_arn = output
            .task_definition()
            .and_then(|td| td.task_definition_arn())
            .ok_or_else(|| AppError::internal("No task definition ARN in registration response"))?;

        Ok(new_arn.to_string())
    }

    /// List all revisions of a task definition family
    pub async fn list_task_definitions(
        &self,
        family_prefix: Option<&str>,
    ) -> AppResult<Vec<String>> {
        let mut task_definitions = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.list_task_definitions();
            if let Some(family) = family_prefix {
                request = request.family_prefix(family);
            }
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let output = request
                .send()
                .await
                .map_err(|e| format_sdk_error("ECS", "list_task_definitions", "all", e))?;

            task_definitions.extend(
                output
                    .task_definition_arns()
                    .iter()
                    .map(|arn| arn.to_string()),
            );

            next_token = output.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(task_definitions)
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
