use crate::models::rds::RdsInstance;
use crate::utils::error::format_sdk_error;
use aws_sdk_rds::Client;

pub struct RdsService {
    client: Client,
}

impl RdsService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_instances(&self) -> anyhow::Result<Vec<RdsInstance>> {
        let response = self.client
            .describe_db_instances()
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "describe", "all", e))?;

        let instances = response
            .db_instances()
            .iter()
            .map(|i| RdsInstance::from_aws(i))
            .collect();

        Ok(instances)
    }

    pub async fn start_instance(&self, db_instance_identifier: &str) -> anyhow::Result<()> {
        self.client
            .start_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "start", db_instance_identifier, e))?;
        Ok(())
    }

    pub async fn stop_instance(&self, db_instance_identifier: &str) -> anyhow::Result<()> {
        self.client
            .stop_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "stop", db_instance_identifier, e))?;
        Ok(())
    }

    pub async fn reboot_instance(&self, db_instance_identifier: &str) -> anyhow::Result<()> {
        self.client
            .reboot_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "reboot", db_instance_identifier, e))?;
        Ok(())
    }


}
