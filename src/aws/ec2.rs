use crate::models::ec2::Ec2Instance;
use aws_sdk_ec2::Client;

pub struct Ec2Service {
    client: Client,
}

impl Ec2Service {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_instances(&self) -> anyhow::Result<Vec<Ec2Instance>> {
        let response = self.client
            .describe_instances()
            .send()
            .await?;

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
            .await?;
        Ok(())
    }

    pub async fn stop_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .stop_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }

    pub async fn reboot_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .reboot_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }

    pub async fn terminate_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }
}
