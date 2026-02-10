use super::StateColor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPlan {
    pub backup_plan_id: String,
    pub backup_plan_name: String,
    pub backup_plan_arn: Option<String>,
    pub creation_date: Option<String>,
    pub last_execution_date: Option<String>,
    pub version_id: Option<String>,
}

impl BackupPlan {
    pub fn from_aws(plan: &aws_sdk_backup::types::BackupPlansListMember) -> Self {
        Self {
            backup_plan_id: plan.backup_plan_id().unwrap_or_default().to_string(),
            backup_plan_name: plan.backup_plan_name().unwrap_or_default().to_string(),
            backup_plan_arn: plan.backup_plan_arn().map(|s| s.to_string()),
            creation_date: plan.creation_date().map(|d| d.to_string()),
            last_execution_date: plan.last_execution_date().map(|d| d.to_string()),
            version_id: plan.version_id().map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupVault {
    pub backup_vault_name: String,
    pub backup_vault_arn: Option<String>,
    pub creation_date: Option<String>,
    pub number_of_recovery_points: i64,
    pub encryption_key_arn: Option<String>,
    pub locked: bool,
}

impl BackupVault {
    pub fn from_aws(vault: &aws_sdk_backup::types::BackupVaultListMember) -> Self {
        Self {
            backup_vault_name: vault.backup_vault_name().unwrap_or_default().to_string(),
            backup_vault_arn: vault.backup_vault_arn().map(|s| s.to_string()),
            creation_date: vault.creation_date().map(|d| d.to_string()),
            number_of_recovery_points: vault.number_of_recovery_points(),
            encryption_key_arn: vault.encryption_key_arn().map(|s| s.to_string()),
            locked: vault.locked().unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub backup_job_id: String,
    pub backup_vault_name: Option<String>,
    pub resource_arn: Option<String>,
    pub resource_type: Option<String>,
    pub state: String,
    pub creation_date: Option<String>,
    pub completion_date: Option<String>,
    pub percent_done: Option<String>,
}

impl BackupJob {
    pub fn from_aws(job: &aws_sdk_backup::types::BackupJob) -> Self {
        Self {
            backup_job_id: job.backup_job_id().unwrap_or_default().to_string(),
            backup_vault_name: job.backup_vault_name().map(|s| s.to_string()),
            resource_arn: job.resource_arn().map(|s| s.to_string()),
            resource_type: job.resource_type().map(|s| s.to_string()),
            state: job
                .state()
                .map(|s| s.as_str().to_string())
                .unwrap_or_default(),
            creation_date: job.creation_date().map(|d| d.to_string()),
            completion_date: job.completion_date().map(|d| d.to_string()),
            percent_done: job.percent_done().map(|s| s.to_string()),
        }
    }
}

impl StateColor for BackupJob {
    fn state_color(&self) -> ratatui::style::Color {
        use crate::ui::theme::THEME;
        match self.state.to_uppercase().as_str() {
            "COMPLETED" => THEME.success,
            "RUNNING" | "PENDING" | "CREATED" => THEME.warning,
            "FAILED" | "ABORTED" | "EXPIRED" => THEME.error,
            _ => THEME.muted,
        }
    }
}

impl crate::models::Filterable for BackupPlan {
    fn matches_filter(&self, filter: &str) -> bool {
        self.backup_plan_name.to_lowercase().contains(filter)
            || self.backup_plan_id.to_lowercase().contains(filter)
    }
}

impl crate::models::Filterable for BackupVault {
    fn matches_filter(&self, filter: &str) -> bool {
        self.backup_vault_name.to_lowercase().contains(filter)
    }
}

impl crate::models::Filterable for BackupJob {
    fn matches_filter(&self, filter: &str) -> bool {
        self.backup_job_id.to_lowercase().contains(filter)
            || self
                .resource_type
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
            || self.state.to_lowercase().contains(filter)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPoint {
    pub recovery_point_arn: String,
    pub backup_vault_name: String,
    pub recovery_point_type: Option<String>,
    pub status: Option<String>,
    pub creation_date: Option<String>,
    pub completion_date: Option<String>,
    pub backup_size_in_bytes: Option<i64>,
    pub resource_arn: Option<String>,
    pub resource_type: Option<String>,
    pub is_encrypted: bool,
    pub life_cycle: Option<String>,
}

impl RecoveryPoint {
    pub fn from_aws(rp: &aws_sdk_backup::types::RecoveryPointByBackupVault) -> Self {
        Self {
            recovery_point_arn: rp.recovery_point_arn().unwrap_or_default().to_string(),
            backup_vault_name: rp.backup_vault_name().unwrap_or_default().to_string(),
            recovery_point_type: None, // Not available in RecoveryPointByBackupVault
            status: rp.status().map(|s| s.as_str().to_string()),
            creation_date: rp.creation_date().map(|d| d.to_string()),
            completion_date: rp.completion_date().map(|d| d.to_string()),
            backup_size_in_bytes: rp.backup_size_in_bytes(),
            resource_arn: rp.resource_arn().map(|s| s.to_string()),
            resource_type: rp.resource_type().map(|s| s.to_string()),
            is_encrypted: rp.is_encrypted(),
            // Simplify lifecycle to just a string if present, or we can expand later
            life_cycle: rp.lifecycle().map(|l| format!("{:?}", l)),
        }
    }

    pub fn status_color(&self) -> ratatui::style::Color {
        use crate::ui::theme::THEME;
        match self.status.as_deref().unwrap_or("").to_uppercase().as_str() {
            "COMPLETED" => THEME.success,
            "PARTIAL" => THEME.warning,
            "DELETING" | "EXPIRED" => THEME.error,
            _ => THEME.muted,
        }
    }
}

impl crate::models::Filterable for RecoveryPoint {
    fn matches_filter(&self, filter: &str) -> bool {
        self.recovery_point_arn.to_lowercase().contains(filter)
            || self
                .resource_type
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
            || self
                .status
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}
