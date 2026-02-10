//! ECS-specific actions

use super::super::confirmable::ConfirmableAction;

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

impl ConfirmableAction for EcsAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::StopTask { task_arn, .. } => {
                let short_arn = task_arn.split('/').next_back().unwrap_or(task_arn);
                format!("Stop ECS task {}", short_arn)
            }
            Self::DeregisterTaskDefinition(arn) => {
                let short_arn = arn.split('/').next_back().unwrap_or(arn);
                format!("Deregister task definition {}", short_arn)
            }
            Self::EditTaskDefinition(arn) => {
                let short_arn = arn.split('/').next_back().unwrap_or(arn);
                format!("Edit task definition {}", short_arn)
            }
            Self::UpdateDesiredCount {
                service_name,
                desired_count,
                ..
            } => {
                format!("Update {} desired count to {}", service_name, desired_count)
            }
            Self::ForceNewDeployment { service_name, .. } => {
                format!("Force new deployment for {}", service_name)
            }
            Self::UpdateService {
                service_name,
                task_definition,
                ..
            } => {
                if let Some(td) = task_definition {
                    let short_td = td.split('/').next_back().unwrap_or(td);
                    format!("Update {} to use {}", service_name, short_td)
                } else {
                    format!("Update service {}", service_name)
                }
            }
            _ => "ECS operation".to_string(),
        }
    }
}
