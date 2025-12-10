use crate::models::rds::RdsInstance;
use aws_sdk_rds::Client;

pub struct RdsService {
    client: Client,
}

fn format_rds_error<E: std::fmt::Debug>(e: aws_sdk_rds::error::SdkError<E>, action: &str, identifier: &str) -> anyhow::Error {
    let msg = if let Some(svc_err) = e.as_service_error() {
        let code = format!("{:?}", svc_err).split('{').next().unwrap_or("Unknown").trim().to_string();
        format!("RDS {} failed for '{}': {}", action, identifier, code)
    } else {
        let err_str = format!("{}", e);
        if err_str.contains("Unhandled") || err_str.contains("unhandled") {
            format!("RDS {} for '{}': Not supported (LocalStack limitation?)", action, identifier)
        } else {
            format!("RDS {} failed for '{}': {}", action, identifier, err_str)
        }
    };
    anyhow::anyhow!(msg)
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
            .map_err(|e| format_rds_error(e, "describe", "all"))?;

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
            .map_err(|e| format_rds_error(e, "start", db_instance_identifier))?;
        Ok(())
    }

    pub async fn stop_instance(&self, db_instance_identifier: &str) -> anyhow::Result<()> {
        self.client
            .stop_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .map_err(|e| format_rds_error(e, "stop", db_instance_identifier))?;
        Ok(())
    }

    pub async fn reboot_instance(&self, db_instance_identifier: &str) -> anyhow::Result<()> {
        self.client
            .reboot_db_instance()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .map_err(|e| format_rds_error(e, "reboot", db_instance_identifier))?;
        Ok(())
    }

    pub async fn create_snapshot(&self, db_instance_identifier: &str, snapshot_identifier: &str) -> anyhow::Result<()> {
        self.client
            .create_db_snapshot()
            .db_instance_identifier(db_instance_identifier)
            .db_snapshot_identifier(snapshot_identifier)
            .send()
            .await
            .map_err(|e| format_rds_error(e, "create snapshot", db_instance_identifier))?;
        Ok(())
    }
}
