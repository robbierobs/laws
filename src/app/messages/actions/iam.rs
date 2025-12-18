//! IAM-specific actions

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
