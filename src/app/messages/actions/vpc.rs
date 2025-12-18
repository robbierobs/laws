//! VPC-specific actions

/// VPC-specific actions
#[derive(Debug, Clone)]
pub enum VpcAction {
    DrillDownSecurityGroup,
    ExitSecurityGroupRules,
    ToggleSgRulesDirection,
    DeleteSecurityGroup(String),
}
