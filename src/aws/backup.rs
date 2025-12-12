use crate::error::AppResult;
use crate::models::backup::{BackupJob, BackupPlan, BackupVault};
use crate::utils::error::format_sdk_error;
use aws_sdk_backup::Client;

pub struct BackupService {
    client: Client,
}

impl BackupService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_backup_plans(&self) -> AppResult<Vec<BackupPlan>> {
        let response = self
            .client
            .list_backup_plans()
            .send()
            .await
            .map_err(|e| format_sdk_error("Backup", "list_backup_plans", "all", e))?;

        let plans = response
            .backup_plans_list()
            .iter()
            .map(BackupPlan::from_aws)
            .collect();

        Ok(plans)
    }

    pub async fn list_backup_vaults(&self) -> AppResult<Vec<BackupVault>> {
        let response = self
            .client
            .list_backup_vaults()
            .send()
            .await
            .map_err(|e| format_sdk_error("Backup", "list_backup_vaults", "all", e))?;

        let vaults = response
            .backup_vault_list()
            .iter()
            .map(BackupVault::from_aws)
            .collect();

        Ok(vaults)
    }

    pub async fn list_backup_jobs(&self) -> AppResult<Vec<BackupJob>> {
        let response = self
            .client
            .list_backup_jobs()
            .max_results(50)
            .send()
            .await
            .map_err(|e| format_sdk_error("Backup", "list_backup_jobs", "all", e))?;

        let jobs = response
            .backup_jobs()
            .iter()
            .map(BackupJob::from_aws)
            .collect();

        Ok(jobs)
    }

    pub async fn list_recovery_points(&self, vault_name: &str) -> AppResult<Vec<crate::models::backup::RecoveryPoint>> {
        let response = self
            .client
            .list_recovery_points_by_backup_vault()
            .backup_vault_name(vault_name)
            .max_results(50)
            .send()
            .await
            .map_err(|e| format_sdk_error("Backup", "list_recovery_points", vault_name, e))?;

        let points = response
            .recovery_points()
            .iter()
            .map(crate::models::backup::RecoveryPoint::from_aws)
            .collect();

        Ok(points)
    }
}

impl crate::aws::traits::AwsService<BackupVault> for BackupService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AppResult<Vec<BackupVault>>> + Send + 'a>,
    > {
        Box::pin(self.list_backup_vaults())
    }
}
