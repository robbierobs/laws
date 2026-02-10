use crate::error::AppResult;
use crate::aws::traits::paginate;
use crate::models::rds::RdsInstance;
use crate::utils::error::format_sdk_error;
use aws_sdk_rds::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(RdsService, Client);

impl RdsService {

    pub async fn list_instances(&self) -> AppResult<Vec<RdsInstance>> {
        let client = self.client.clone();
        paginate(
            move |marker| {
                let client = client.clone();
                async move {
                    let mut request = client.describe_db_instances();
                    if let Some(marker) = marker {
                        request = request.marker(marker);
                    }

                    request
                        .send()
                        .await
                        .map_err(|e| format_sdk_error("RDS", "describe", "all", e))
                }
            },
            |response| {
                let instances = response
                    .db_instances()
                    .iter()
                    .map(RdsInstance::from_aws)
                    .collect();
                let next_token = response.marker().map(|s| s.to_string());
                (instances, next_token)
            },
        )
        .await
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
