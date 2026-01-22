use crate::models::{collect_tags, find_name_tag};
use crate::ui::theme::THEME;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ec2Instance {
    pub instance_id: String,
    pub name: Option<String>,
    pub state: InstanceState,
    pub instance_type: String,
    pub public_ip: Option<String>,
    pub private_ip: Option<String>,
    pub launch_time: Option<String>,
    // Additional detail fields
    pub subnet_id: Option<String>,
    pub vpc_id: Option<String>,
    pub security_groups: Vec<SecurityGroupInfo>,
    pub availability_zone: Option<String>,
    pub platform: Option<String>,
    pub architecture: Option<String>,
    pub ami_id: Option<String>,
    pub key_name: Option<String>,
    pub monitoring_state: Option<String>,
    pub tags: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityGroupInfo {
    pub group_id: String,
    pub group_name: String,
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

impl std::fmt::Display for InstanceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceState::Pending => write!(f, "Pending"),
            InstanceState::Running => write!(f, "Running"),
            InstanceState::ShuttingDown => write!(f, "ShuttingDown"),
            InstanceState::Terminated => write!(f, "Terminated"),
            InstanceState::Stopping => write!(f, "Stopping"),
            InstanceState::Stopped => write!(f, "Stopped"),
            InstanceState::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl Ec2Instance {
    pub fn state_color(&self) -> Color {
        match self.state {
            InstanceState::Running => THEME.success,
            InstanceState::Stopped => THEME.error,
            InstanceState::Pending | InstanceState::Stopping => THEME.warning,
            _ => THEME.muted,
        }
    }

    pub fn from_aws(instance: aws_sdk_ec2::types::Instance) -> Self {
        let name = find_name_tag(instance.tags().iter());
        let tags = collect_tags(instance.tags().iter());

        let state = instance
            .state()
            .and_then(|s| s.name())
            .map(|n| InstanceState::from(n.clone()))
            .unwrap_or(InstanceState::Unknown("unknown".to_string()));

        let security_groups = instance
            .security_groups()
            .iter()
            .map(|sg| SecurityGroupInfo {
                group_id: sg.group_id().unwrap_or_default().to_string(),
                group_name: sg.group_name().unwrap_or_default().to_string(),
            })
            .collect();

        Self {
            instance_id: instance.instance_id().unwrap_or_default().to_string(),
            name,
            state,
            instance_type: instance
                .instance_type()
                .map(|t| t.as_str().to_string())
                .unwrap_or_default(),
            public_ip: instance.public_ip_address().map(|s| s.to_string()),
            private_ip: instance.private_ip_address().map(|s| s.to_string()),
            launch_time: instance.launch_time().map(|t| t.to_string()),
            subnet_id: instance.subnet_id().map(|s| s.to_string()),
            vpc_id: instance.vpc_id().map(|s| s.to_string()),
            security_groups,
            availability_zone: instance
                .placement()
                .and_then(|p| p.availability_zone().map(|s| s.to_string())),
            platform: instance.platform().map(|p| p.as_str().to_string()),
            architecture: instance.architecture().map(|a| a.as_str().to_string()),
            ami_id: instance.image_id().map(|s| s.to_string()),
            key_name: instance.key_name().map(|s| s.to_string()),
            monitoring_state: instance
                .monitoring()
                .and_then(|m| m.state())
                .map(|s| s.as_str().to_string()),
            tags,
        }
    }
}

impl crate::models::Filterable for Ec2Instance {
    fn matches_filter(&self, filter: &str) -> bool {
        self.instance_id.to_lowercase().contains(filter)
            || self
                .name
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
            || self
                .public_ip
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Filterable;

    #[test]
    fn test_instance_state_display_running() {
        assert_eq!(InstanceState::Running.to_string(), "Running");
    }

    #[test]
    fn test_instance_state_display_stopped() {
        assert_eq!(InstanceState::Stopped.to_string(), "Stopped");
    }

    #[test]
    fn test_instance_state_display_pending() {
        assert_eq!(InstanceState::Pending.to_string(), "Pending");
    }

    #[test]
    fn test_instance_state_display_terminated() {
        assert_eq!(InstanceState::Terminated.to_string(), "Terminated");
    }

    #[test]
    fn test_instance_state_display_unknown() {
        let state = InstanceState::Unknown("custom-state".to_string());
        assert_eq!(state.to_string(), "custom-state");
    }

    #[test]
    fn test_instance_state_all_variants() {
        // Ensure all variants have a displayable representation
        let states = vec![
            InstanceState::Pending,
            InstanceState::Running,
            InstanceState::ShuttingDown,
            InstanceState::Terminated,
            InstanceState::Stopping,
            InstanceState::Stopped,
            InstanceState::Unknown("test".to_string()),
        ];

        for state in states {
            let display = state.to_string();
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_ec2_instance_matches_filter_by_id() {
        let instance = Ec2Instance {
            instance_id: "i-12345abc".to_string(),
            name: Some("web-server".to_string()),
            state: InstanceState::Running,
            instance_type: "t2.micro".to_string(),
            public_ip: Some("1.2.3.4".to_string()),
            private_ip: None,
            launch_time: None,
            subnet_id: None,
            vpc_id: None,
            security_groups: vec![],
            availability_zone: None,
            platform: None,
            architecture: None,
            ami_id: None,
            key_name: None,
            monitoring_state: None,
            tags: vec![],
        };

        assert!(instance.matches_filter("12345"));
        assert!(instance.matches_filter("i-12345abc"));
    }

    #[test]
    fn test_ec2_instance_matches_filter_by_name() {
        let instance = Ec2Instance {
            instance_id: "i-abc".to_string(),
            name: Some("production-web".to_string()),
            state: InstanceState::Running,
            instance_type: "t2.micro".to_string(),
            public_ip: None,
            private_ip: None,
            launch_time: None,
            subnet_id: None,
            vpc_id: None,
            security_groups: vec![],
            availability_zone: None,
            platform: None,
            architecture: None,
            ami_id: None,
            key_name: None,
            monitoring_state: None,
            tags: vec![],
        };

        assert!(instance.matches_filter("production"));
        assert!(instance.matches_filter("web"));
    }
}
