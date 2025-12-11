use crate::error::AppResult;
use crate::models::ec2::Ec2Instance;
use crate::utils::error::format_sdk_error;
use aws_sdk_ec2::Client;

pub struct Ec2Service {
    client: Client,
}

impl Ec2Service {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_instances(&self) -> AppResult<Vec<Ec2Instance>> {
        let response = self.client
            .describe_instances()
            .send()
            .await
            .map_err(|e| format_sdk_error("EC2", "describe", "all", e))?;

        let instances = response
            .reservations()
            .iter()
            .flat_map(|r| r.instances())
            .map(|i| Ec2Instance::from_aws(i.clone()))
            .collect();

        Ok(instances)
    }

    pub async fn start_instance(&self, instance_id: &str) -> AppResult<()> {
        self.client
            .start_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| format_sdk_error("EC2", "start", instance_id, e))?;
        Ok(())
    }

    pub async fn stop_instance(&self, instance_id: &str) -> AppResult<()> {
        self.client
            .stop_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| format_sdk_error("EC2", "stop", instance_id, e))?;
        Ok(())
    }

    pub async fn reboot_instance(&self, instance_id: &str) -> AppResult<()> {
        self.client
            .reboot_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| format_sdk_error("EC2", "reboot", instance_id, e))?;
        Ok(())
    }


}

impl crate::aws::traits::AwsService<Ec2Instance> for Ec2Service {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<Ec2Instance>>> + Send + 'a>> {
        Box::pin(self.list_instances())
    }
}

