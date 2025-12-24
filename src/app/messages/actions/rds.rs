//! RDS-specific actions

use super::super::confirmable::ConfirmableAction;

/// RDS-specific actions
#[derive(Debug, Clone)]
pub enum RdsAction {
    Start(String),
    Stop(String),
    Reboot(String),
    Delete(String),
}

impl ConfirmableAction for RdsAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::Start(id) => format!("Start RDS instance {}", id),
            Self::Stop(id) => format!("Stop RDS instance {}", id),
            Self::Reboot(id) => format!("Reboot RDS instance {}", id),
            Self::Delete(id) => format!("Delete RDS instance {}", id),
        }
    }
}
