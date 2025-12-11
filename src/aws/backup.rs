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
            .map(|p| BackupPlan::from_aws(p))
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
            .map(|v| BackupVault::from_aws(v))
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
            .map(|j| BackupJob::from_aws(j))
            .collect();

        Ok(jobs)
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
