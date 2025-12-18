//! ECS-specific actions

/// ECS-specific actions
#[derive(Debug, Clone)]
pub enum EcsAction {
    // Navigation
    ViewServices(String),       // cluster_arn
    ViewTasks(String),          // service_arn
    ViewTaskDefinition(String), // task_definition_arn
    BackToClusters,
    BackToServices,
    BackToTasks,

    // Service actions
    UpdateDesiredCount {
        cluster_arn: String,
        service_name: String,
        desired_count: i32,
    },
    ForceNewDeployment {
        cluster_arn: String,
        service_name: String,
    },
    #[allow(dead_code)] // Planned: update service to use different task definition
    UpdateServiceTaskDefinition {
        cluster_arn: String,
        service_name: String,
        task_definition_arn: String,
    },
    /// Modify ECS Service - supports task definition, CPU, and Memory changes
    /// Note: CPU and Memory require creating a new task definition revision
    UpdateService {
        cluster_arn: String,
        service_name: String,
        /// New task definition ARN (full ARN or family:revision)
        task_definition: Option<String>,
        /// CPU value (in Fargate units: "256", "512", "1024", etc.)
        cpu: Option<String>,
        /// Memory value (in MiB: "512", "1024", "2048", etc.)
        memory: Option<String>,
        /// Force a new deployment even if no other changes
        force_new_deployment: bool,
    },

    // Task actions
    StopTask {
        cluster_arn: String,
        task_arn: String,
    },

    // Task definition actions
    DeregisterTaskDefinition(String), // task_definition_arn
    EditTaskDefinition(String),       // task_definition_arn
    #[allow(dead_code)] // Planned: list all revisions of a task definition family
    ListTaskDefinitions(String), // family name
    /// Load full task definitions for the selector modal
    LoadTaskDefinitionsForSelector(String), // family name
}
