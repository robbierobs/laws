use crate::models::ec2::Ec2Instance;
use aws_sdk_ec2::Client;

pub struct Ec2Service {
    client: Client,
}

fn format_ec2_error<E: std::fmt::Debug>(e: aws_sdk_ec2::error::SdkError<E>, action: &str, instance_id: &str) -> anyhow::Error {
    let msg = if let Some(svc_err) = e.as_service_error() {
        let code = format!("{:?}", svc_err).split('{').next().unwrap_or("Unknown").trim().to_string();
        format!("EC2 {} failed for '{}': {}", action, instance_id, code)
    } else {
        // Check for common error patterns
        let err_str = format!("{}", e);
        if err_str.contains("Unhandled") || err_str.contains("unhandled") {
            format!("EC2 {} for '{}': Not supported (LocalStack limitation?)", action, instance_id)
        } else {
            format!("EC2 {} failed for '{}': {}", action, instance_id, err_str)
        }
    };
    anyhow::anyhow!(msg)
}

impl Ec2Service {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_instances(&self) -> anyhow::Result<Vec<Ec2Instance>> {
        let response = self.client
            .describe_instances()
            .send()
            .await
            .map_err(|e| format_ec2_error(e, "describe", "all"))?;

        let instances = response
            .reservations()
            .iter()
            .flat_map(|r| r.instances())
            .map(|i| Ec2Instance::from_aws(i.clone()))
            .collect();

        Ok(instances)
    }

    pub async fn start_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .start_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| format_ec2_error(e, "start", instance_id))?;
        Ok(())
    }

    pub async fn stop_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .stop_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| format_ec2_error(e, "stop", instance_id))?;
        Ok(())
    }

    pub async fn reboot_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .reboot_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| format_ec2_error(e, "reboot", instance_id))?;
        Ok(())
    }

    pub async fn terminate_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| format_ec2_error(e, "terminate", instance_id))?;
        Ok(())
    }
}

