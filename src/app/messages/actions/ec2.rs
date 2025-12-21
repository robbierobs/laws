//! EC2-specific actions

use super::super::confirmable::ConfirmableAction;

/// EC2-specific actions
#[derive(Debug, Clone)]
pub enum Ec2Action {
    Start(String),
    Stop(String),
    Reboot(String),
    Terminate(String),
}

impl ConfirmableAction for Ec2Action {
    fn confirmation_description(&self) -> String {
        match self {
            Self::Start(id) => format!("Start EC2 instance {}", id),
            Self::Stop(id) => format!("Stop EC2 instance {}", id),
            Self::Reboot(id) => format!("Reboot EC2 instance {}", id),
            Self::Terminate(id) => format!("Terminate EC2 instance {}", id),
        }
    }
}
