//! IAM-specific actions

use super::super::confirmable::ConfirmableAction;

/// IAM-specific actions
#[derive(Debug, Clone)]
pub enum IamAction {
    DrillDownUser,
    DrillDownRole,
    DrillDownPolicy,
    ExitDrillDown,
    DeleteUser(String),
    DeleteRole(String),
    DeletePolicy(String),
}

impl ConfirmableAction for IamAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::DeleteUser(name) => format!("Delete IAM User {}", name),
            Self::DeleteRole(name) => format!("Delete IAM Role {}", name),
            Self::DeletePolicy(arn) => format!("Delete IAM Policy {}", arn),
            _ => "IAM operation".to_string(),
        }
    }
}
