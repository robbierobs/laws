//! VPC-specific actions

use super::super::confirmable::ConfirmableAction;

/// VPC-specific actions
#[derive(Debug, Clone)]
pub enum VpcAction {
    DrillDownSecurityGroup,
    ExitSecurityGroupRules,
    ToggleSgRulesDirection,
    DeleteSecurityGroup(String),
}

impl ConfirmableAction for VpcAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::DeleteSecurityGroup(id) => format!("Delete Security Group {}", id),
            _ => "VPC operation".to_string(),
        }
    }
}
