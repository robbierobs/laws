//! View mode enums for services with multiple views
//!
//! These enums define the different views available within a service (e.g., VPCs, Subnets, Security Groups).

use crate::app::ViewMode;

// ============================================================================
// VPC View Mode
// ============================================================================

/// View mode for VPC service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum VpcViewMode {
    #[default]
    Vpcs = 0,
    Subnets = 1,
    SecurityGroups = 2,
    SecurityGroupRules = 3,
}

impl ViewMode for VpcViewMode {
    fn all() -> &'static [Self] {
        &[
            Self::Vpcs,
            Self::Subnets,
            Self::SecurityGroups,
            Self::SecurityGroupRules,
        ]
    }

    fn main_tabs() -> &'static [Self] {
        &[Self::Vpcs, Self::Subnets, Self::SecurityGroups]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Vpcs,
            1 => Self::Subnets,
            2 => Self::SecurityGroups,
            3 => Self::SecurityGroupRules,
            _ => Self::Vpcs,
        }
    }

    fn is_main_tab(&self) -> bool {
        !matches!(self, Self::SecurityGroupRules)
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Vpcs => "VPCs",
            Self::Subnets => "Subnets",
            Self::SecurityGroups => "Security Groups",
            Self::SecurityGroupRules => "Rules",
        }
    }
}

// ============================================================================
// IAM View Mode
// ============================================================================

/// View mode for IAM service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum IamViewMode {
    #[default]
    Users = 0,
    Roles = 1,
    Policies = 2,
    UserAttachedPolicies = 3,
    RoleAttachedPolicies = 4,
    PolicyDocument = 5,
}

impl ViewMode for IamViewMode {
    fn all() -> &'static [Self] {
        &[
            Self::Users,
            Self::Roles,
            Self::Policies,
            Self::UserAttachedPolicies,
            Self::RoleAttachedPolicies,
            Self::PolicyDocument,
        ]
    }

    fn main_tabs() -> &'static [Self] {
        &[Self::Users, Self::Roles, Self::Policies]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Users,
            1 => Self::Roles,
            2 => Self::Policies,
            3 => Self::UserAttachedPolicies,
            4 => Self::RoleAttachedPolicies,
            5 => Self::PolicyDocument,
            _ => Self::Users,
        }
    }

    fn is_main_tab(&self) -> bool {
        matches!(self, Self::Users | Self::Roles | Self::Policies)
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Users => "Users",
            Self::Roles => "Roles",
            Self::Policies => "Policies",
            Self::UserAttachedPolicies => "User Policies",
            Self::RoleAttachedPolicies => "Role Policies",
            Self::PolicyDocument => "Policy Document",
        }
    }
}

// ============================================================================
// Backup View Mode
// ============================================================================

/// View mode for Backup service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BackupViewMode {
    #[default]
    Vaults = 0,
    Plans = 1,
    Jobs = 2,
    RecoveryPoints = 3,
}

impl ViewMode for BackupViewMode {
    fn all() -> &'static [Self] {
        &[Self::Vaults, Self::Plans, Self::Jobs, Self::RecoveryPoints]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Vaults,
            1 => Self::Plans,
            2 => Self::Jobs,
            3 => Self::RecoveryPoints,
            _ => Self::Vaults,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Vaults => "Vaults",
            Self::Plans => "Plans",
            Self::Jobs => "Jobs",
            Self::RecoveryPoints => "Recovery Points",
        }
    }
}

// ============================================================================
// CloudTrail View Mode
// ============================================================================

/// View mode for CloudTrail service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CloudTrailViewMode {
    #[default]
    Trails = 0,
    Events = 1,
}

impl ViewMode for CloudTrailViewMode {
    fn all() -> &'static [Self] {
        &[Self::Trails, Self::Events]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Trails,
            1 => Self::Events,
            _ => Self::Trails,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Trails => "Trails",
            Self::Events => "Events",
        }
    }
}

// ============================================================================
// ECS View Mode
// ============================================================================

/// View mode for ECS service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum EcsViewMode {
    #[default]
    Clusters = 0,
    Services = 1,
    Tasks = 2,
    TaskDefinition = 3,
}

impl ViewMode for EcsViewMode {
    fn all() -> &'static [Self] {
        &[
            Self::Clusters,
            Self::Services,
            Self::Tasks,
            Self::TaskDefinition,
        ]
    }

    fn main_tabs() -> &'static [Self] {
        &[Self::Clusters] // Only Clusters is a top-level tab
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Clusters,
            1 => Self::Services,
            2 => Self::Tasks,
            3 => Self::TaskDefinition,
            _ => Self::Clusters,
        }
    }

    fn is_main_tab(&self) -> bool {
        matches!(self, Self::Clusters)
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Clusters => "Clusters",
            Self::Services => "Services",
            Self::Tasks => "Tasks",
            Self::TaskDefinition => "Task Definition",
        }
    }
}

// ============================================================================
// DynamoDB View Mode
// ============================================================================

/// View mode for DynamoDB service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DynamoDbViewMode {
    #[default]
    Tables = 0,
    Items = 1,
}

impl ViewMode for DynamoDbViewMode {
    fn all() -> &'static [Self] {
        &[Self::Tables, Self::Items]
    }

    fn main_tabs() -> &'static [Self] {
        &[Self::Tables] // Only Tables is a main tab
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Tables,
            1 => Self::Items,
            _ => Self::Tables,
        }
    }

    fn is_main_tab(&self) -> bool {
        matches!(self, Self::Tables)
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Tables => "Tables",
            Self::Items => "Items",
        }
    }
}

// ============================================================================
// ECR View Mode
// ============================================================================

/// View mode for ECR service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum EcrViewMode {
    #[default]
    Repositories = 0,
    Images = 1,
}

impl ViewMode for EcrViewMode {
    fn all() -> &'static [Self] {
        &[Self::Repositories, Self::Images]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Repositories,
            1 => Self::Images,
            _ => Self::Repositories,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Repositories => "Repositories",
            Self::Images => "Images",
        }
    }
}

// ============================================================================
// Budgets View Mode
// ============================================================================

/// View mode for Budgets service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BudgetsViewMode {
    #[default]
    Budgets = 0,
    Notifications = 1,
    BillingViews = 2,
}

impl ViewMode for BudgetsViewMode {
    fn all() -> &'static [Self] {
        &[Self::Budgets, Self::Notifications, Self::BillingViews]
    }

    fn main_tabs() -> &'static [Self] {
        &[Self::Budgets, Self::BillingViews]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Budgets,
            1 => Self::Notifications,
            2 => Self::BillingViews,
            _ => Self::Budgets,
        }
    }

    fn is_main_tab(&self) -> bool {
        matches!(self, Self::Budgets | Self::BillingViews)
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Budgets => "Budgets",
            Self::Notifications => "Alerts",
            Self::BillingViews => "Billing Views",
        }
    }
}
