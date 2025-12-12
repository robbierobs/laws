use crate::error::AppResult;
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

    pub async fn list_instances(&self) -> AppResult<Vec<RdsInstance>> {
        let response = self.client
            .describe_db_instances()
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "describe", "all", e))?;

        let instances = response
            .db_instances()
            .iter()
            .map(RdsInstance::from_aws)
            .collect();

        Ok(instances)
    }

    pub async fn start_instance(&self, db_instance_identifier: &str) -> AppResult<()> {
        self.client
            .start_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "start", db_instance_identifier, e))?;
        Ok(())
    }

    pub async fn stop_instance(&self, db_instance_identifier: &str) -> AppResult<()> {
        self.client
            .stop_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "stop", db_instance_identifier, e))?;
        Ok(())
    }

    pub async fn reboot_instance(&self, db_instance_identifier: &str) -> AppResult<()> {
        self.client
            .reboot_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "reboot", db_instance_identifier, e))?;
        Ok(())
    }


    pub async fn delete_instance(&self, db_instance_identifier: &str) -> AppResult<()> {
        self.client
            .delete_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .skip_final_snapshot(true)
            .send()
            .await
            .map_err(|e| format_sdk_error("RDS", "delete", db_instance_identifier, e))?;
        Ok(())
    }

}

impl crate::aws::traits::AwsService<RdsInstance> for RdsService {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<RdsInstance>>> + Send + 'a>> {
        Box::pin(self.list_instances())
    }
}
