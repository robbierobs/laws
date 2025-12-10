use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ec2Instance {
    pub instance_id: String,
    pub name: Option<String>,
    pub state: InstanceState,
    pub instance_type: String,
    pub public_ip: Option<String>,
    pub private_ip: Option<String>,
    pub launch_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstanceState {
    Pending,
    Running,
    ShuttingDown,
    Terminated,
    Stopping,
    Stopped,
    Unknown(String),
}

impl From<aws_sdk_ec2::types::InstanceStateName> for InstanceState {
    fn from(value: aws_sdk_ec2::types::InstanceStateName) -> Self {
        match value {
            aws_sdk_ec2::types::InstanceStateName::Pending => InstanceState::Pending,
            aws_sdk_ec2::types::InstanceStateName::Running => InstanceState::Running,
            aws_sdk_ec2::types::InstanceStateName::ShuttingDown => InstanceState::ShuttingDown,
            aws_sdk_ec2::types::InstanceStateName::Terminated => InstanceState::Terminated,
            aws_sdk_ec2::types::InstanceStateName::Stopping => InstanceState::Stopping,
            aws_sdk_ec2::types::InstanceStateName::Stopped => InstanceState::Stopped,
            other => InstanceState::Unknown(other.as_str().to_string()),
        }
    }
}

impl Ec2Instance {
    pub fn from_aws(instance: aws_sdk_ec2::types::Instance) -> Self {
        let name = instance.tags().iter().find(|t| t.key() == Some("Name")).and_then(|t| t.value().map(|v| v.to_string()));
        
        let state = instance.state()
            .and_then(|s| s.name())
            .map(|n| InstanceState::from(n.clone()))
            .unwrap_or(InstanceState::Unknown("unknown".to_string()));

        Self {
            instance_id: instance.instance_id().unwrap_or_default().to_string(),
            name,
            state,
            instance_type: instance.instance_type().map(|t| t.as_str().to_string()).unwrap_or_default(),
            public_ip: instance.public_ip_address().map(|s| s.to_string()),
            private_ip: instance.private_ip_address().map(|s| s.to_string()),
            launch_time: instance.launch_time().map(|t| t.to_string()),
        }
    }
}
