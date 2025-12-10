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
            state: job.state().map(|s| s.as_str().to_string()).unwrap_or_default(),
            creation_date: job.creation_date().map(|d| d.to_string()),
            completion_date: job.completion_date().map(|d| d.to_string()),
            percent_done: job.percent_done().map(|s| s.to_string()),
        }
    }

    pub fn state_color(&self) -> ratatui::style::Color {
        use crate::ui::theme::THEME;
        match self.state.to_uppercase().as_str() {
            "COMPLETED" => THEME.success,
            "RUNNING" | "PENDING" | "CREATED" => THEME.warning,
            "FAILED" | "ABORTED" | "EXPIRED" => THEME.error,
            _ => THEME.muted,
        }
    }
}
