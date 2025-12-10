use crate::models::backup::{BackupPlan, BackupVault, BackupJob};
use aws_sdk_backup::Client;

pub struct BackupService {
    client: Client,
}

impl BackupService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_backup_plans(&self) -> anyhow::Result<Vec<BackupPlan>> {
        let response = self.client
            .list_backup_plans()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list backup plans: {}", e))?;

        let plans = response.backup_plans_list()
            .iter()
            .map(|p| BackupPlan::from_aws(p))
            .collect();

        Ok(plans)
    }

    pub async fn list_backup_vaults(&self) -> anyhow::Result<Vec<BackupVault>> {
        let response = self.client
            .list_backup_vaults()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list backup vaults: {}", e))?;

        let vaults = response.backup_vault_list()
            .iter()
            .map(|v| BackupVault::from_aws(v))
            .collect();

        Ok(vaults)
    }

    pub async fn list_backup_jobs(&self) -> anyhow::Result<Vec<BackupJob>> {
        let response = self.client
            .list_backup_jobs()
            .max_results(50)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list backup jobs: {}", e))?;

        let jobs = response.backup_jobs()
            .iter()
            .map(|j| BackupJob::from_aws(j))
            .collect();

        Ok(jobs)
    }
}
