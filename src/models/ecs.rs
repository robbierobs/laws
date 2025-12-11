use serde::{Deserialize, Serialize};
use ratatui::style::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsCluster {
    pub cluster_arn: String,
    pub cluster_name: String,
    pub status: String,
    pub running_tasks_count: i32,
    pub pending_tasks_count: i32,
    pub active_services_count: i32,
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
        }
    }

    pub fn state_color(&self) -> Color {
        use crate::ui::theme::THEME;
        match self.status.to_uppercase().as_str() {
            "ACTIVE" => THEME.success,
            "INACTIVE" => THEME.muted,
            "FAILED" => THEME.error,
            _ => THEME.fg,
        }
    }
}

impl crate::models::Filterable for EcsCluster {
    fn matches_filter(&self, filter: &str) -> bool {
        self.cluster_name.to_lowercase().contains(filter)
    }
}

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
        }
    }
    
    pub fn state_color(&self) -> Color {
        use crate::ui::theme::THEME;
        match self.status.to_uppercase().as_str() {
            "ACTIVE" => THEME.success,
            "DRAINING" => THEME.warning,
            "INACTIVE" => THEME.muted,
            _ => THEME.fg,
        }
    }
}

impl crate::models::Filterable for EcsService {
    fn matches_filter(&self, filter: &str) -> bool {
        self.service_name.to_lowercase().contains(filter)
    }
}
