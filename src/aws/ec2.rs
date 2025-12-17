use crate::error::AppResult;
use crate::models::ec2::Ec2Instance;
use crate::utils::error::format_sdk_error;
use aws_sdk_ec2::Client;

pub struct Ec2Service {
    client: Client,
}

/// Instance actions that can be performed on EC2 instances
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceAction {
    Start,
    Stop,
    Reboot,
    Terminate,
}

impl InstanceAction {
    /// Get the verb form for error messages
    fn verb(&self) -> &'static str {
        match self {
            InstanceAction::Start => "start",
            InstanceAction::Stop => "stop",
            InstanceAction::Reboot => "reboot",
            InstanceAction::Terminate => "terminate",
        }
    }
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

    /// Execute an instance action (start, stop, reboot, terminate)
    pub async fn execute_action(&self, action: InstanceAction, instance_id: &str) -> AppResult<()> {
        match action {
            InstanceAction::Start => {
                self.client
                    .start_instances()
                    .instance_ids(instance_id)
                    .send()
                    .await
                    .map_err(|e| format_sdk_error("EC2", action.verb(), instance_id, e))?;
            }
            InstanceAction::Stop => {
                self.client
                    .stop_instances()
                    .instance_ids(instance_id)
                    .send()
                    .await
                    .map_err(|e| format_sdk_error("EC2", action.verb(), instance_id, e))?;
            }
            InstanceAction::Reboot => {
                self.client
                    .reboot_instances()
                    .instance_ids(instance_id)
                    .send()
                    .await
                    .map_err(|e| format_sdk_error("EC2", action.verb(), instance_id, e))?;
            }
            InstanceAction::Terminate => {
                self.client
                    .terminate_instances()
                    .instance_ids(instance_id)
                    .send()
                    .await
                    .map_err(|e| format_sdk_error("EC2", action.verb(), instance_id, e))?;
            }
        }
        Ok(())
    }

    // Convenience methods that delegate to execute_action
    pub async fn start_instance(&self, instance_id: &str) -> AppResult<()> {
        self.execute_action(InstanceAction::Start, instance_id).await
    }

    pub async fn stop_instance(&self, instance_id: &str) -> AppResult<()> {
        self.execute_action(InstanceAction::Stop, instance_id).await
    }

    pub async fn reboot_instance(&self, instance_id: &str) -> AppResult<()> {
        self.execute_action(InstanceAction::Reboot, instance_id).await
    }

    pub async fn terminate_instance(&self, instance_id: &str) -> AppResult<()> {
        self.execute_action(InstanceAction::Terminate, instance_id).await
    }
}

impl crate::aws::traits::AwsService<Ec2Instance> for Ec2Service {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<Ec2Instance>>> + Send + 'a>> {
        Box::pin(self.list_instances())
    }
}

