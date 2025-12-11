use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::ui::theme::THEME;

// ============================================================================
// ECS Cluster
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsCluster {
    pub cluster_arn: String,
    pub cluster_name: String,
    pub status: String,
    pub running_tasks_count: i32,
    pub pending_tasks_count: i32,
    pub active_services_count: i32,
    pub registered_container_instances_count: i32,
    pub capacity_providers: Vec<String>,
    pub default_capacity_provider_strategy: Vec<CapacityProviderStrategyItem>,
    pub settings: Vec<ClusterSetting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityProviderStrategyItem {
    pub capacity_provider: String,
    pub weight: i32,
    pub base: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSetting {
    pub name: String,
    pub value: String,
}

impl EcsCluster {
    pub fn from_aws(cluster: aws_sdk_ecs::types::Cluster) -> Self {
        Self {
            cluster_arn: cluster.cluster_arn().unwrap_or_default().to_string(),
            cluster_name: cluster.cluster_name().unwrap_or_default().to_string(),
            status: cluster.status().unwrap_or_default().to_string(),
            running_tasks_count: cluster.running_tasks_count(),
            pending_tasks_count: cluster.pending_tasks_count(),
            active_services_count: cluster.active_services_count(),
            registered_container_instances_count: cluster.registered_container_instances_count(),
            capacity_providers: cluster
                .capacity_providers()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            default_capacity_provider_strategy: cluster
                .default_capacity_provider_strategy()
                .iter()
                .map(|s| CapacityProviderStrategyItem {
                    capacity_provider: s.capacity_provider().to_string(),
                    weight: s.weight(),
                    base: s.base(),
                })
                .collect(),
            settings: cluster
                .settings()
                .iter()
                .filter_map(|s| {
                    Some(ClusterSetting {
                        name: s.name()?.as_str().to_string(),
                        value: s.value()?.to_string(),
                    })
                })
                .collect(),
        }
    }

    pub fn state_color(&self) -> Color {
        match self.status.to_uppercase().as_str() {
            "ACTIVE" => THEME.success,
            "INACTIVE" => THEME.muted,
            "FAILED" | "PROVISIONING" => THEME.warning,
            _ => THEME.fg,
        }
    }
}

impl crate::models::Filterable for EcsCluster {
    fn matches_filter(&self, filter: &str) -> bool {
        self.cluster_name.to_lowercase().contains(filter)
            || self.cluster_arn.to_lowercase().contains(filter)
    }
}

// ============================================================================
// ECS Service
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsService {
    pub service_arn: String,
    pub service_name: String,
    pub cluster_arn: String,
    pub status: String,
    pub desired_count: i32,
    pub running_count: i32,
    pub pending_count: i32,
    pub task_definition: Option<String>,
    pub launch_type: Option<String>,
    pub platform_version: Option<String>,
    pub platform_family: Option<String>,
    pub scheduling_strategy: Option<String>,
    pub deployment_controller: Option<String>,
    pub created_at: Option<String>,
    pub created_by: Option<String>,
    pub enable_ecs_managed_tags: bool,
    pub propagate_tags: Option<String>,
    pub role_arn: Option<String>,
    pub health_check_grace_period_seconds: Option<i32>,
    pub deployments: Vec<EcsDeployment>,
    pub events: Vec<EcsServiceEvent>,
    pub load_balancers: Vec<EcsLoadBalancer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsDeployment {
    pub id: String,
    pub status: String,
    pub task_definition: String,
    pub desired_count: i32,
    pub running_count: i32,
    pub pending_count: i32,
    pub failed_tasks: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub rollout_state: Option<String>,
    pub rollout_state_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsServiceEvent {
    pub id: Option<String>,
    pub created_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsLoadBalancer {
    pub target_group_arn: Option<String>,
    pub load_balancer_name: Option<String>,
    pub container_name: Option<String>,
    pub container_port: Option<i32>,
}

impl EcsService {
    pub fn from_aws(service: aws_sdk_ecs::types::Service) -> Self {
        Self {
            service_arn: service.service_arn().unwrap_or_default().to_string(),
            service_name: service.service_name().unwrap_or_default().to_string(),
            cluster_arn: service.cluster_arn().unwrap_or_default().to_string(),
            status: service.status().unwrap_or_default().to_string(),
            desired_count: service.desired_count(),
            running_count: service.running_count(),
            pending_count: service.pending_count(),
            task_definition: service.task_definition().map(|s| s.to_string()),
            launch_type: service.launch_type().map(|s| s.as_str().to_string()),
            platform_version: service.platform_version().map(|s| s.to_string()),
            platform_family: service.platform_family().map(|s| s.to_string()),
            scheduling_strategy: service
                .scheduling_strategy()
                .map(|s| s.as_str().to_string()),
            deployment_controller: service
                .deployment_controller()
                .map(|d| d.r#type().as_str().to_string()),
            created_at: service.created_at().map(|t| t.to_string()),
            created_by: service.created_by().map(|s| s.to_string()),
            enable_ecs_managed_tags: service.enable_ecs_managed_tags(),
            propagate_tags: service.propagate_tags().map(|p| p.as_str().to_string()),
            role_arn: service.role_arn().map(|s| s.to_string()),
            health_check_grace_period_seconds: service.health_check_grace_period_seconds(),
            deployments: service
                .deployments()
                .iter()
                .map(|d| EcsDeployment {
                    id: d.id().unwrap_or_default().to_string(),
                    status: d.status().unwrap_or_default().to_string(),
                    task_definition: d.task_definition().unwrap_or_default().to_string(),
                    desired_count: d.desired_count(),
                    running_count: d.running_count(),
                    pending_count: d.pending_count(),
                    failed_tasks: d.failed_tasks(),
                    created_at: d.created_at().map(|t| t.to_string()),
                    updated_at: d.updated_at().map(|t| t.to_string()),
                    rollout_state: d.rollout_state().map(|s| s.as_str().to_string()),
                    rollout_state_reason: d.rollout_state_reason().map(|s| s.to_string()),
                })
                .collect(),
            events: service
                .events()
                .iter()
                .take(10) // Only keep last 10 events
                .map(|e| EcsServiceEvent {
                    id: e.id().map(|s| s.to_string()),
                    created_at: e.created_at().map(|t| t.to_string()),
                    message: e.message().map(|s| s.to_string()),
                })
                .collect(),
            load_balancers: service
                .load_balancers()
                .iter()
                .map(|lb| EcsLoadBalancer {
                    target_group_arn: lb.target_group_arn().map(|s| s.to_string()),
                    load_balancer_name: lb.load_balancer_name().map(|s| s.to_string()),
                    container_name: lb.container_name().map(|s| s.to_string()),
                    container_port: lb.container_port(),
                })
                .collect(),
        }
    }

    pub fn state_color(&self) -> Color {
        match self.status.to_uppercase().as_str() {
            "ACTIVE" => THEME.success,
            "DRAINING" => THEME.warning,
            "INACTIVE" => THEME.muted,
            _ => THEME.fg,
        }
    }

    /// Get a short display name for the task definition
    pub fn task_definition_short(&self) -> String {
        self.task_definition
            .as_ref()
            .and_then(|td| td.split('/').last())
            .unwrap_or("N/A")
            .to_string()
    }
}

impl crate::models::Filterable for EcsService {
    fn matches_filter(&self, filter: &str) -> bool {
        self.service_name.to_lowercase().contains(filter)
            || self.service_arn.to_lowercase().contains(filter)
    }
}

// ============================================================================
// ECS Task
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsTask {
    pub task_arn: String,
    pub task_definition_arn: String,
    pub cluster_arn: String,
    pub container_instance_arn: Option<String>,
    pub last_status: String,
    pub desired_status: String,
    pub health_status: Option<String>,
    pub connectivity: Option<String>,
    pub connectivity_at: Option<String>,
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub launch_type: Option<String>,
    pub platform_version: Option<String>,
    pub platform_family: Option<String>,
    pub started_at: Option<String>,
    pub started_by: Option<String>,
    pub stopped_at: Option<String>,
    pub stopped_reason: Option<String>,
    pub stop_code: Option<String>,
    pub created_at: Option<String>,
    pub group: Option<String>,
    pub availability_zone: Option<String>,
    pub enable_execute_command: bool,
    pub containers: Vec<EcsContainer>,
    pub attachments: Vec<EcsAttachment>,
    pub tags: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsContainer {
    pub container_arn: Option<String>,
    pub name: String,
    pub image: Option<String>,
    pub image_digest: Option<String>,
    pub runtime_id: Option<String>,
    pub last_status: Option<String>,
    pub exit_code: Option<i32>,
    pub reason: Option<String>,
    pub health_status: Option<String>,
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub memory_reservation: Option<String>,
    pub gpu_ids: Vec<String>,
    pub network_bindings: Vec<EcsNetworkBinding>,
    pub network_interfaces: Vec<EcsNetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsNetworkBinding {
    pub bind_ip: Option<String>,
    pub container_port: Option<i32>,
    pub host_port: Option<i32>,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsNetworkInterface {
    pub attachment_id: Option<String>,
    pub private_ipv4_address: Option<String>,
    pub ipv6_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsAttachment {
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub details: Vec<(String, String)>,
}

impl EcsTask {
    pub fn from_aws(task: aws_sdk_ecs::types::Task) -> Self {
        Self {
            task_arn: task.task_arn().unwrap_or_default().to_string(),
            task_definition_arn: task.task_definition_arn().unwrap_or_default().to_string(),
            cluster_arn: task.cluster_arn().unwrap_or_default().to_string(),
            container_instance_arn: task.container_instance_arn().map(|s| s.to_string()),
            last_status: task.last_status().unwrap_or_default().to_string(),
            desired_status: task.desired_status().unwrap_or_default().to_string(),
            health_status: task.health_status().map(|h| h.as_str().to_string()),
            connectivity: task.connectivity().map(|c| c.as_str().to_string()),
            connectivity_at: task.connectivity_at().map(|t| t.to_string()),
            cpu: task.cpu().map(|s| s.to_string()),
            memory: task.memory().map(|s| s.to_string()),
            launch_type: task.launch_type().map(|l| l.as_str().to_string()),
            platform_version: task.platform_version().map(|s| s.to_string()),
            platform_family: task.platform_family().map(|s| s.to_string()),
            started_at: task.started_at().map(|t| t.to_string()),
            started_by: task.started_by().map(|s| s.to_string()),
            stopped_at: task.stopped_at().map(|t| t.to_string()),
            stopped_reason: task.stopped_reason().map(|s| s.to_string()),
            stop_code: task.stop_code().map(|c| c.as_str().to_string()),
            created_at: task.created_at().map(|t| t.to_string()),
            group: task.group().map(|s| s.to_string()),
            availability_zone: task.availability_zone().map(|s| s.to_string()),
            enable_execute_command: task.enable_execute_command(),
            containers: task
                .containers()
                .iter()
                .map(|c| EcsContainer {
                    container_arn: c.container_arn().map(|s| s.to_string()),
                    name: c.name().unwrap_or_default().to_string(),
                    image: c.image().map(|s| s.to_string()),
                    image_digest: c.image_digest().map(|s| s.to_string()),
                    runtime_id: c.runtime_id().map(|s| s.to_string()),
                    last_status: c.last_status().map(|s| s.to_string()),
                    exit_code: c.exit_code(),
                    reason: c.reason().map(|s| s.to_string()),
                    health_status: c.health_status().map(|h| h.as_str().to_string()),
                    cpu: c.cpu().map(|s| s.to_string()),
                    memory: c.memory().map(|s| s.to_string()),
                    memory_reservation: c.memory_reservation().map(|s| s.to_string()),
                    gpu_ids: c.gpu_ids().iter().map(|s| s.to_string()).collect(),
                    network_bindings: c
                        .network_bindings()
                        .iter()
                        .map(|nb| EcsNetworkBinding {
                            bind_ip: nb.bind_ip().map(|s| s.to_string()),
                            container_port: nb.container_port(),
                            host_port: nb.host_port(),
                            protocol: nb.protocol().map(|p| p.as_str().to_string()),
                        })
                        .collect(),
                    network_interfaces: c
                        .network_interfaces()
                        .iter()
                        .map(|ni| EcsNetworkInterface {
                            attachment_id: ni.attachment_id().map(|s| s.to_string()),
                            private_ipv4_address: ni.private_ipv4_address().map(|s| s.to_string()),
                            ipv6_address: ni.ipv6_address().map(|s| s.to_string()),
                        })
                        .collect(),
                })
                .collect(),
            attachments: task
                .attachments()
                .iter()
                .map(|a| EcsAttachment {
                    id: a.id().map(|s| s.to_string()),
                    r#type: a.r#type().map(|s| s.to_string()),
                    status: a.status().map(|s| s.to_string()),
                    details: a
                        .details()
                        .iter()
                        .filter_map(|d| Some((d.name()?.to_string(), d.value()?.to_string())))
                        .collect(),
                })
                .collect(),
            tags: task
                .tags()
                .iter()
                .filter_map(|t| Some((t.key()?.to_string(), t.value()?.to_string())))
                .collect(),
        }
    }

    pub fn state_color(&self) -> Color {
        match self.last_status.to_uppercase().as_str() {
            "RUNNING" => THEME.success,
            "PENDING" | "ACTIVATING" | "PROVISIONING" => THEME.warning,
            "STOPPED" | "DEACTIVATING" | "STOPPING" => THEME.error,
            "DEPROVISIONING" => THEME.muted,
            _ => THEME.fg,
        }
    }

    pub fn health_color(&self) -> Color {
        match self.health_status.as_deref() {
            Some("HEALTHY") => THEME.success,
            Some("UNHEALTHY") => THEME.error,
            Some("UNKNOWN") => THEME.warning,
            _ => THEME.muted,
        }
    }

    /// Get short task ID from ARN
    pub fn task_id(&self) -> String {
        self.task_arn
            .split('/')
            .last()
            .unwrap_or(&self.task_arn)
            .to_string()
    }

    /// Get short task definition name
    pub fn task_definition_short(&self) -> String {
        self.task_definition_arn
            .split('/')
            .last()
            .unwrap_or(&self.task_definition_arn)
            .to_string()
    }
}

impl crate::models::Filterable for EcsTask {
    fn matches_filter(&self, filter: &str) -> bool {
        self.task_arn.to_lowercase().contains(filter)
            || self.last_status.to_lowercase().contains(filter)
            || self
                .containers
                .iter()
                .any(|c| c.name.to_lowercase().contains(filter))
    }
}

// ============================================================================
// ECS Task Definition
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsTaskDefinition {
    pub task_definition_arn: String,
    pub family: String,
    pub revision: i32,
    pub status: String,
    pub task_role_arn: Option<String>,
    pub execution_role_arn: Option<String>,
    pub network_mode: Option<String>,
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub requires_compatibilities: Vec<String>,
    pub runtime_platform: Option<EcsRuntimePlatform>,
    pub container_definitions: Vec<EcsContainerDefinition>,
    pub volumes: Vec<EcsVolume>,
    pub placement_constraints: Vec<EcsPlacementConstraint>,
    pub registered_at: Option<String>,
    pub registered_by: Option<String>,
    pub pid_mode: Option<String>,
    pub ipc_mode: Option<String>,
    pub proxy_configuration: Option<EcsProxyConfiguration>,
    pub ephemeral_storage_size_gib: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsRuntimePlatform {
    pub cpu_architecture: Option<String>,
    pub operating_system_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsContainerDefinition {
    pub name: String,
    pub image: Option<String>,
    pub cpu: i32,
    pub memory: Option<i32>,
    pub memory_reservation: Option<i32>,
    pub essential: bool,
    pub entry_point: Vec<String>,
    pub command: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub secrets: Vec<(String, String)>,
    pub port_mappings: Vec<EcsPortMapping>,
    pub mount_points: Vec<EcsMountPoint>,
    pub volumes_from: Vec<EcsVolumeFrom>,
    pub linux_parameters: Option<EcsLinuxParameters>,
    pub health_check: Option<EcsHealthCheck>,
    pub log_configuration: Option<EcsLogConfiguration>,
    pub depends_on: Vec<EcsContainerDependency>,
    pub start_timeout: Option<i32>,
    pub stop_timeout: Option<i32>,
    pub user: Option<String>,
    pub working_directory: Option<String>,
    pub privileged: bool,
    pub readonly_root_filesystem: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsPortMapping {
    pub container_port: Option<i32>,
    pub host_port: Option<i32>,
    pub protocol: Option<String>,
    pub name: Option<String>,
    pub app_protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsMountPoint {
    pub source_volume: Option<String>,
    pub container_path: Option<String>,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsVolumeFrom {
    pub source_container: Option<String>,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsLinuxParameters {
    pub init_process_enabled: bool,
    pub shared_memory_size: Option<i32>,
    pub max_swap: Option<i32>,
    pub swappiness: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsHealthCheck {
    pub command: Vec<String>,
    pub interval: Option<i32>,
    pub timeout: Option<i32>,
    pub retries: Option<i32>,
    pub start_period: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsLogConfiguration {
    pub log_driver: String,
    pub options: Vec<(String, String)>,
    pub secret_options: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsContainerDependency {
    pub container_name: String,
    pub condition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsVolume {
    pub name: String,
    pub host_path: Option<String>,
    pub efs_volume_configuration: Option<EcsEfsConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsEfsConfiguration {
    pub file_system_id: String,
    pub root_directory: Option<String>,
    pub transit_encryption: Option<String>,
    pub transit_encryption_port: Option<i32>,
    pub authorization_config: Option<EcsEfsAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsEfsAuthConfig {
    pub access_point_id: Option<String>,
    pub iam: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsPlacementConstraint {
    pub r#type: Option<String>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsProxyConfiguration {
    pub r#type: Option<String>,
    pub container_name: String,
    pub properties: Vec<(String, String)>,
}

#[allow(dead_code)]
impl EcsTaskDefinition {
    pub fn from_aws(td: aws_sdk_ecs::types::TaskDefinition) -> Self {
        Self {
            task_definition_arn: td.task_definition_arn().unwrap_or_default().to_string(),
            family: td.family().unwrap_or_default().to_string(),
            revision: td.revision(),
            status: td
                .status()
                .map(|s| s.as_str().to_string())
                .unwrap_or_default(),
            task_role_arn: td.task_role_arn().map(|s| s.to_string()),
            execution_role_arn: td.execution_role_arn().map(|s| s.to_string()),
            network_mode: td.network_mode().map(|n| n.as_str().to_string()),
            cpu: td.cpu().map(|s| s.to_string()),
            memory: td.memory().map(|s| s.to_string()),
            requires_compatibilities: td
                .requires_compatibilities()
                .iter()
                .map(|c| c.as_str().to_string())
                .collect(),
            runtime_platform: td.runtime_platform().map(|rp| EcsRuntimePlatform {
                cpu_architecture: rp.cpu_architecture().map(|a| a.as_str().to_string()),
                operating_system_family: rp
                    .operating_system_family()
                    .map(|f| f.as_str().to_string()),
            }),
            container_definitions: td
                .container_definitions()
                .iter()
                .map(|cd| EcsContainerDefinition {
                    name: cd.name().unwrap_or_default().to_string(),
                    image: cd.image().map(|s| s.to_string()),
                    cpu: cd.cpu(),
                    memory: cd.memory(),
                    memory_reservation: cd.memory_reservation(),
                    essential: cd.essential().unwrap_or(true),
                    entry_point: cd.entry_point().iter().map(|s| s.to_string()).collect(),
                    command: cd.command().iter().map(|s| s.to_string()).collect(),
                    environment: cd
                        .environment()
                        .iter()
                        .filter_map(|e| Some((e.name()?.to_string(), e.value()?.to_string())))
                        .collect(),
                    secrets: cd
                        .secrets()
                        .iter()
                        .map(|s| (s.name().to_string(), s.value_from().to_string()))
                        .collect(),
                    port_mappings: cd
                        .port_mappings()
                        .iter()
                        .map(|pm| EcsPortMapping {
                            container_port: pm.container_port(),
                            host_port: pm.host_port(),
                            protocol: pm.protocol().map(|p| p.as_str().to_string()),
                            name: pm.name().map(|s| s.to_string()),
                            app_protocol: pm.app_protocol().map(|p| p.as_str().to_string()),
                        })
                        .collect(),
                    mount_points: cd
                        .mount_points()
                        .iter()
                        .map(|mp| EcsMountPoint {
                            source_volume: mp.source_volume().map(|s| s.to_string()),
                            container_path: mp.container_path().map(|s| s.to_string()),
                            read_only: mp.read_only().unwrap_or(false),
                        })
                        .collect(),
                    volumes_from: cd
                        .volumes_from()
                        .iter()
                        .map(|vf| EcsVolumeFrom {
                            source_container: vf.source_container().map(|s| s.to_string()),
                            read_only: vf.read_only().unwrap_or(false),
                        })
                        .collect(),
                    linux_parameters: cd.linux_parameters().map(|lp| EcsLinuxParameters {
                        init_process_enabled: lp.init_process_enabled().unwrap_or(false),
                        shared_memory_size: lp.shared_memory_size(),
                        max_swap: lp.max_swap(),
                        swappiness: lp.swappiness(),
                    }),
                    health_check: cd.health_check().map(|hc| EcsHealthCheck {
                        command: hc.command().iter().map(|s| s.to_string()).collect(),
                        interval: hc.interval(),
                        timeout: hc.timeout(),
                        retries: hc.retries(),
                        start_period: hc.start_period(),
                    }),
                    log_configuration: cd.log_configuration().map(|lc| EcsLogConfiguration {
                        log_driver: lc.log_driver().as_str().to_string(),
                        options: lc
                            .options()
                            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                            .unwrap_or_default(),
                        secret_options: lc
                            .secret_options()
                            .iter()
                            .map(|s| (s.name().to_string(), s.value_from().to_string()))
                            .collect(),
                    }),
                    depends_on: cd
                        .depends_on()
                        .iter()
                        .map(|d| EcsContainerDependency {
                            container_name: d.container_name().to_string(),
                            condition: d.condition().as_str().to_string(),
                        })
                        .collect(),
                    start_timeout: cd.start_timeout(),
                    stop_timeout: cd.stop_timeout(),
                    user: cd.user().map(|s| s.to_string()),
                    working_directory: cd.working_directory().map(|s| s.to_string()),
                    privileged: cd.privileged().unwrap_or(false),
                    readonly_root_filesystem: cd.readonly_root_filesystem().unwrap_or(false),
                })
                .collect(),
            volumes: td
                .volumes()
                .iter()
                .map(|v| EcsVolume {
                    name: v.name().unwrap_or_default().to_string(),
                    host_path: v
                        .host()
                        .and_then(|h| h.source_path().map(|s| s.to_string())),
                    efs_volume_configuration: v.efs_volume_configuration().map(|efs| {
                        EcsEfsConfiguration {
                            file_system_id: efs.file_system_id().to_string(),
                            root_directory: efs.root_directory().map(|s| s.to_string()),
                            transit_encryption: efs
                                .transit_encryption()
                                .map(|t| t.as_str().to_string()),
                            transit_encryption_port: efs.transit_encryption_port(),
                            authorization_config: efs.authorization_config().map(|ac| {
                                EcsEfsAuthConfig {
                                    access_point_id: ac.access_point_id().map(|s| s.to_string()),
                                    iam: ac.iam().map(|i| i.as_str().to_string()),
                                }
                            }),
                        }
                    }),
                })
                .collect(),
            placement_constraints: td
                .placement_constraints()
                .iter()
                .map(|pc| EcsPlacementConstraint {
                    r#type: pc.r#type().map(|t| t.as_str().to_string()),
                    expression: pc.expression().map(|s| s.to_string()),
                })
                .collect(),
            registered_at: td.registered_at().map(|t| t.to_string()),
            registered_by: td.registered_by().map(|s| s.to_string()),
            pid_mode: td.pid_mode().map(|p| p.as_str().to_string()),
            ipc_mode: td.ipc_mode().map(|i| i.as_str().to_string()),
            proxy_configuration: td.proxy_configuration().map(|pc| EcsProxyConfiguration {
                r#type: pc.r#type().map(|t| t.as_str().to_string()),
                container_name: pc.container_name().to_string(),
                properties: pc
                    .properties()
                    .iter()
                    .filter_map(|p| Some((p.name()?.to_string(), p.value()?.to_string())))
                    .collect(),
            }),
            ephemeral_storage_size_gib: td.ephemeral_storage().map(|es| es.size_in_gib()),
        }
    }

    pub fn status_color(&self) -> Color {
        match self.status.to_uppercase().as_str() {
            "ACTIVE" => THEME.success,
            "INACTIVE" => THEME.muted,
            "DELETE_IN_PROGRESS" => THEME.warning,
            _ => THEME.fg,
        }
    }

    /// Get short display name (family:revision)
    pub fn short_name(&self) -> String {
        format!("{}:{}", self.family, self.revision)
    }

    /// Total memory across all containers (if defined at container level)
    pub fn total_container_memory(&self) -> i32 {
        self.container_definitions
            .iter()
            .filter_map(|c| c.memory)
            .sum()
    }

    /// Total CPU across all containers
    pub fn total_container_cpu(&self) -> i32 {
        self.container_definitions.iter().map(|c| c.cpu).sum()
    }
}
