use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdsInstance {
    pub db_instance_identifier: String,
    pub db_instance_class: String,
    pub engine: String,
    pub engine_version: Option<String>,
    pub status: String,
    pub endpoint: Option<String>,
    pub port: Option<i32>,
    pub master_username: Option<String>,
    pub allocated_storage: Option<i32>,
    pub availability_zone: Option<String>,
    pub multi_az: bool,
    pub publicly_accessible: bool,
    pub storage_type: Option<String>,
    pub storage_encrypted: bool,
    pub vpc_id: Option<String>,
    pub db_subnet_group: Option<String>,
    pub security_groups: Vec<String>,
    pub backup_retention_period: Option<i32>,
    pub preferred_backup_window: Option<String>,
    pub preferred_maintenance_window: Option<String>,
    pub created_time: Option<String>,
    pub auto_minor_version_upgrade: bool,
    pub license_model: Option<String>,
    pub iops: Option<i32>,
    pub deletion_protection: bool,
}

impl RdsInstance {
    pub fn from_aws(instance: &aws_sdk_rds::types::DbInstance) -> Self {
        let endpoint = instance.endpoint().map(|e| {
            format!("{}:{}", 
                e.address().unwrap_or("-"),
                e.port().unwrap_or(0)
            )
        });

        let security_groups: Vec<String> = instance.vpc_security_groups()
            .iter()
            .filter_map(|sg| sg.vpc_security_group_id().map(|s| s.to_string()))
            .collect();

        Self {
            db_instance_identifier: instance.db_instance_identifier().unwrap_or_default().to_string(),
            db_instance_class: instance.db_instance_class().unwrap_or_default().to_string(),
            engine: instance.engine().unwrap_or_default().to_string(),
            engine_version: instance.engine_version().map(|s| s.to_string()),
            status: instance.db_instance_status().unwrap_or_default().to_string(),
            endpoint,
            port: instance.endpoint().and_then(|e| e.port()),
            master_username: instance.master_username().map(|s| s.to_string()),
            allocated_storage: instance.allocated_storage(),
            availability_zone: instance.availability_zone().map(|s| s.to_string()),
            multi_az: instance.multi_az().unwrap_or(false),
            publicly_accessible: instance.publicly_accessible().unwrap_or(false),
            storage_type: instance.storage_type().map(|s| s.to_string()),
            storage_encrypted: instance.storage_encrypted().unwrap_or(false),
            vpc_id: instance.db_subnet_group()
                .and_then(|g| g.vpc_id())
                .map(|s| s.to_string()),
            db_subnet_group: instance.db_subnet_group()
                .and_then(|g| g.db_subnet_group_name())
                .map(|s| s.to_string()),
            security_groups,
            backup_retention_period: instance.backup_retention_period(),
            preferred_backup_window: instance.preferred_backup_window().map(|s| s.to_string()),
            preferred_maintenance_window: instance.preferred_maintenance_window().map(|s| s.to_string()),
            created_time: instance.instance_create_time().map(|t| t.to_string()),
            auto_minor_version_upgrade: instance.auto_minor_version_upgrade().unwrap_or(false),
            license_model: instance.license_model().map(|s| s.to_string()),
            iops: instance.iops(),
            deletion_protection: instance.deletion_protection().unwrap_or(false),
        }
    }
    
    /// Get a status color based on the instance status
    pub fn status_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self.status.to_lowercase().as_str() {
            "available" => Color::Green,
            "stopped" | "stopping" => Color::Red,
            "starting" | "creating" | "modifying" | "backing-up" | "rebooting" => Color::Yellow,
            "deleting" | "failed" => Color::LightRed,
            _ => Color::Gray,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdsCluster {
    pub cluster_identifier: String,
    pub engine: String,
    pub status: String,
    pub endpoint: Option<String>,
    pub reader_endpoint: Option<String>,
    pub members: Vec<String>,
}
