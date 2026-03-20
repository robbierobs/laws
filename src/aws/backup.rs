use crate::error::AppResult;
use crate::models::backup::{BackupJob, BackupPlan, BackupVault};
use crate::utils::error::format_sdk_error;
use aws_sdk_backup::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(BackupService, Client);

impl BackupService {

    pub async fn list_backup_plans(&self) -> AppResult<Vec<BackupPlan>> {
        let mut plans = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.list_backup_plans();
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("Backup", "list_backup_plans", "all", e))?;

            plans.extend(
                response
                    .backup_plans_list()
                    .iter()
                    .map(BackupPlan::from_aws),
            );

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(plans)
    }

    pub async fn list_backup_vaults(&self) -> AppResult<Vec<BackupVault>> {
        let mut vaults = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.list_backup_vaults();
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("Backup", "list_backup_vaults", "all", e))?;

            vaults.extend(
                response
                    .backup_vault_list()
                    .iter()
                    .map(BackupVault::from_aws),
            );

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(vaults)
    }

    pub async fn list_backup_jobs(&self) -> AppResult<Vec<BackupJob>> {
        let mut jobs = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.list_backup_jobs().max_results(50);
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("Backup", "list_backup_jobs", "all", e))?;

            jobs.extend(
                response
                    .backup_jobs()
                    .iter()
                    .map(BackupJob::from_aws),
            );

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(jobs)
    }

    pub async fn list_recovery_points(&self, vault_name: &str) -> AppResult<Vec<crate::models::backup::RecoveryPoint>> {
        let mut points = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_recovery_points_by_backup_vault()
                .backup_vault_name(vault_name)
                .max_results(50);
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("Backup", "list_recovery_points", vault_name, e))?;

            points.extend(
                response
                    .recovery_points()
                    .iter()
                    .map(crate::models::backup::RecoveryPoint::from_aws),
            );

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

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
