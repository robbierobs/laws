//! Message and enum definitions for the application state machine
//!
//! This module uses a hierarchical message structure:
//! - `Message::Global` for app-wide operations (navigation, quit, UI toggles)
//! - `Message::Service` for service-specific actions

use std::collections::HashMap;

/// AWS Service types supported by the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    EC2,
    S3,
    RDS,
    DynamoDB,
    Lambda,
    VPC,
    IAM,
    Backup,
    CloudTrail,
}

impl Service {
    pub fn as_str(&self) -> &str {
        match self {
            Service::EC2 => "EC2",
            Service::S3 => "S3",
            Service::RDS => "RDS",
            Service::DynamoDB => "DynamoDB",
            Service::Lambda => "Lambda",
            Service::VPC => "VPC",
            Service::IAM => "IAM",
            Service::Backup => "Backup",
            Service::CloudTrail => "CloudTrail",
        }
    }

    pub fn iterator() -> impl Iterator<Item = Self> {
        [
            Self::EC2,
            Self::S3,
            Self::RDS,
            Self::DynamoDB,
            Self::Lambda,
            Self::VPC,
            Self::IAM,
            Self::Backup,
            Self::CloudTrail,
        ]
        .iter()
        .copied()
    }
}

// ============================================================================
// View Mode Enums
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

impl VpcViewMode {
    pub fn next(self) -> Self {
        match self {
            Self::Vpcs => Self::Subnets,
            Self::Subnets => Self::SecurityGroups,
            Self::SecurityGroups => Self::Vpcs,
            Self::SecurityGroupRules => Self::SecurityGroupRules,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Vpcs => Self::SecurityGroups,
            Self::Subnets => Self::Vpcs,
            Self::SecurityGroups => Self::Subnets,
            Self::SecurityGroupRules => Self::SecurityGroupRules,
        }
    }

    pub fn to_index(self) -> usize {
        self as usize
    }
}

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

impl IamViewMode {
    pub fn next(self) -> Self {
        match self {
            Self::Users => Self::Roles,
            Self::Roles => Self::Policies,
            Self::Policies => Self::Users,
            _ => self,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Users => Self::Policies,
            Self::Roles => Self::Users,
            Self::Policies => Self::Roles,
            _ => self,
        }
    }

    pub fn to_index(self) -> usize {
        self as usize
    }

    pub fn is_main_tab(self) -> bool {
        matches!(self, Self::Users | Self::Roles | Self::Policies)
    }
}

/// View mode for Backup service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BackupViewMode {
    #[default]
    Vaults = 0,
    Plans = 1,
    Jobs = 2,
}

impl BackupViewMode {
    pub fn next(self) -> Self {
        match self {
            Self::Vaults => Self::Plans,
            Self::Plans => Self::Jobs,
            Self::Jobs => Self::Vaults,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Vaults => Self::Jobs,
            Self::Plans => Self::Vaults,
            Self::Jobs => Self::Plans,
        }
    }

    pub fn to_index(self) -> usize {
        self as usize
    }
}

/// View mode for CloudTrail service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CloudTrailViewMode {
    #[default]
    Trails = 0,
    Events = 1,
}

impl CloudTrailViewMode {
    pub fn next(self) -> Self {
        match self {
            Self::Trails => Self::Events,
            Self::Events => Self::Trails,
        }
    }

    pub fn previous(self) -> Self {
        self.next()
    }

    pub fn to_index(self) -> usize {
        self as usize
    }
}

/// View mode for DynamoDB service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DynamoDbViewMode {
    #[default]
    Tables = 0,
    Items = 1,
}

impl DynamoDbViewMode {
    pub fn to_index(self) -> usize {
        self as usize
    }
}

// ============================================================================
// Per-Service Action Enums
// ============================================================================

/// EC2-specific actions
#[derive(Debug, Clone)]
pub enum Ec2Action {
    Start(String),
    Stop(String),
    Reboot(String),
}

/// S3-specific actions
#[derive(Debug, Clone)]
pub enum S3Action {
    LoadObjects(String),
    LoadBucketDetails(String),
    DeleteObject { bucket: String, key: String },
    LeaveBucket,
}

/// RDS-specific actions
#[derive(Debug, Clone)]
pub enum RdsAction {
    Start(String),
    Stop(String),
    Reboot(String),
}

/// DynamoDB-specific actions
#[derive(Debug, Clone)]
pub enum DynamoDbAction {
    DrillDownTable,
    ExitDrillDown,
    LoadItems(String),
    DeleteItem { table_name: String, key_attrs: HashMap<String, String> },
}

/// Lambda-specific actions (placeholder for future)
#[derive(Debug, Clone)]
pub enum LambdaAction {
    // No actions currently supported
}

/// VPC-specific actions
#[derive(Debug, Clone)]
pub enum VpcAction {
    DrillDownSecurityGroup,
    ExitSecurityGroupRules,
    ToggleSgRulesDirection,
}

/// IAM-specific actions
#[derive(Debug, Clone)]
pub enum IamAction {
    DrillDownUser,
    DrillDownRole,
    DrillDownPolicy,
    ExitDrillDown,
}

/// Backup-specific actions (placeholder for future)
#[derive(Debug, Clone)]
pub enum BackupAction {
    // No actions currently supported
}

/// CloudTrail-specific actions (placeholder for future)
#[derive(Debug, Clone)]
pub enum CloudTrailAction {
    // No actions currently supported
}

// ============================================================================
// Message Hierarchy
// ============================================================================

/// Global application messages (not service-specific)
#[derive(Debug, Clone)]
pub enum GlobalMessage {
    /// Navigate to a specific service
    Navigate(Service),
    /// Quit the application
    Quit,
    /// Refresh data for current service
    RefreshData,
    /// Toggle detail panel visibility
    ToggleDetailPanel,
    /// Toggle action log visibility
    ToggleActionLog,
    /// Cycle through view modes (for services with tabs)
    CycleViewMode,
    /// Move to next view
    NextView,
    /// Move to previous view
    PreviousView,
    /// Confirm pending action
    ConfirmAction,
    /// Cancel pending action
    CancelAction,
}

/// Service-specific messages
#[derive(Debug, Clone)]
pub enum ServiceAction {
    Ec2(Ec2Action),
    S3(S3Action),
    Rds(RdsAction),
    DynamoDb(DynamoDbAction),
    Lambda(LambdaAction),
    Vpc(VpcAction),
    Iam(IamAction),
    Backup(BackupAction),
    CloudTrail(CloudTrailAction),
}

/// Main application message type (Elm Architecture style)
#[derive(Debug, Clone)]
pub enum Message {
    /// Global app-wide messages
    Global(GlobalMessage),
    /// Service-specific actions
    Service(ServiceAction),
}

// ============================================================================
// Helper implementations for convenience construction
// ============================================================================

impl Message {
    // Global message constructors
    pub fn navigate(service: Service) -> Self {
        Message::Global(GlobalMessage::Navigate(service))
    }
    
    pub fn quit() -> Self {
        Message::Global(GlobalMessage::Quit)
    }
    
    pub fn refresh() -> Self {
        Message::Global(GlobalMessage::RefreshData)
    }
    
    pub fn toggle_detail_panel() -> Self {
        Message::Global(GlobalMessage::ToggleDetailPanel)
    }
    
    pub fn toggle_action_log() -> Self {
        Message::Global(GlobalMessage::ToggleActionLog)
    }
    
    pub fn cycle_view_mode() -> Self {
        Message::Global(GlobalMessage::CycleViewMode)
    }
    
    pub fn next_view() -> Self {
        Message::Global(GlobalMessage::NextView)
    }
    
    pub fn previous_view() -> Self {
        Message::Global(GlobalMessage::PreviousView)
    }
    
    pub fn confirm_action() -> Self {
        Message::Global(GlobalMessage::ConfirmAction)
    }
    
    pub fn cancel_action() -> Self {
        Message::Global(GlobalMessage::CancelAction)
    }
    
    // EC2 message constructors
    pub fn ec2_start(instance_id: String) -> Self {
        Message::Service(ServiceAction::Ec2(Ec2Action::Start(instance_id)))
    }
    
    pub fn ec2_stop(instance_id: String) -> Self {
        Message::Service(ServiceAction::Ec2(Ec2Action::Stop(instance_id)))
    }
    
    pub fn ec2_reboot(instance_id: String) -> Self {
        Message::Service(ServiceAction::Ec2(Ec2Action::Reboot(instance_id)))
    }
    
    // S3 message constructors
    pub fn s3_load_objects(bucket: String) -> Self {
        Message::Service(ServiceAction::S3(S3Action::LoadObjects(bucket)))
    }
    
    pub fn s3_load_bucket_details(bucket: String) -> Self {
        Message::Service(ServiceAction::S3(S3Action::LoadBucketDetails(bucket)))
    }
    
    pub fn s3_delete_object(bucket: String, key: String) -> Self {
        Message::Service(ServiceAction::S3(S3Action::DeleteObject { bucket, key }))
    }
    
    pub fn s3_leave_bucket() -> Self {
        Message::Service(ServiceAction::S3(S3Action::LeaveBucket))
    }
    
    // RDS message constructors
    pub fn rds_start(instance_id: String) -> Self {
        Message::Service(ServiceAction::Rds(RdsAction::Start(instance_id)))
    }
    
    pub fn rds_stop(instance_id: String) -> Self {
        Message::Service(ServiceAction::Rds(RdsAction::Stop(instance_id)))
    }
    
    pub fn rds_reboot(instance_id: String) -> Self {
        Message::Service(ServiceAction::Rds(RdsAction::Reboot(instance_id)))
    }
    
    // DynamoDB message constructors
    pub fn dynamodb_drill_down() -> Self {
        Message::Service(ServiceAction::DynamoDb(DynamoDbAction::DrillDownTable))
    }
    
    pub fn dynamodb_exit_drill_down() -> Self {
        Message::Service(ServiceAction::DynamoDb(DynamoDbAction::ExitDrillDown))
    }
    
    pub fn dynamodb_load_items(table_name: String) -> Self {
        Message::Service(ServiceAction::DynamoDb(DynamoDbAction::LoadItems(table_name)))
    }
    
    pub fn dynamodb_delete_item(table_name: String, key_attrs: HashMap<String, String>) -> Self {
        Message::Service(ServiceAction::DynamoDb(DynamoDbAction::DeleteItem { table_name, key_attrs }))
    }
    
    // VPC message constructors
    pub fn vpc_drill_down_sg() -> Self {
        Message::Service(ServiceAction::Vpc(VpcAction::DrillDownSecurityGroup))
    }
    
    pub fn vpc_exit_sg_rules() -> Self {
        Message::Service(ServiceAction::Vpc(VpcAction::ExitSecurityGroupRules))
    }
    
    pub fn vpc_toggle_sg_rules_direction() -> Self {
        Message::Service(ServiceAction::Vpc(VpcAction::ToggleSgRulesDirection))
    }
    
    // IAM message constructors
    pub fn iam_drill_down_user() -> Self {
        Message::Service(ServiceAction::Iam(IamAction::DrillDownUser))
    }
    
    pub fn iam_drill_down_role() -> Self {
        Message::Service(ServiceAction::Iam(IamAction::DrillDownRole))
    }
    
    pub fn iam_drill_down_policy() -> Self {
        Message::Service(ServiceAction::Iam(IamAction::DrillDownPolicy))
    }
    
    pub fn iam_exit_drill_down() -> Self {
        Message::Service(ServiceAction::Iam(IamAction::ExitDrillDown))
    }
}

// ============================================================================
// UI State Enums
// ============================================================================

/// Which pane has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Main,
}

/// Current input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filtering,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_as_str() {
        assert_eq!(Service::EC2.as_str(), "EC2");
        assert_eq!(Service::S3.as_str(), "S3");
        assert_eq!(Service::RDS.as_str(), "RDS");
        assert_eq!(Service::DynamoDB.as_str(), "DynamoDB");
        assert_eq!(Service::Lambda.as_str(), "Lambda");
        assert_eq!(Service::VPC.as_str(), "VPC");
        assert_eq!(Service::IAM.as_str(), "IAM");
        assert_eq!(Service::Backup.as_str(), "Backup");
        assert_eq!(Service::CloudTrail.as_str(), "CloudTrail");
    }

    #[test]
    fn test_service_iterator() {
        let services: Vec<Service> = Service::iterator().collect();
        assert_eq!(services.len(), 9);
        assert!(services.contains(&Service::EC2));
        assert!(services.contains(&Service::S3));
    }

    #[test]
    fn test_vpc_view_mode_navigation() {
        let mode = VpcViewMode::Vpcs;
        assert_eq!(mode.next(), VpcViewMode::Subnets);
        
        let mode = VpcViewMode::Subnets;
        assert_eq!(mode.next(), VpcViewMode::SecurityGroups);
        
        let mode = VpcViewMode::SecurityGroups;
        assert_eq!(mode.next(), VpcViewMode::Vpcs); // Wraps around
    }

    #[test]
    fn test_vpc_view_mode_previous() {
        let mode = VpcViewMode::Vpcs;
        assert_eq!(mode.previous(), VpcViewMode::SecurityGroups); // Wraps around
        
        let mode = VpcViewMode::SecurityGroups;
        assert_eq!(mode.previous(), VpcViewMode::Subnets);
    }

    #[test]
    fn test_iam_view_mode_navigation() {
        let mode = IamViewMode::Users;
        assert_eq!(mode.next(), IamViewMode::Roles);
        
        let mode = IamViewMode::Roles;
        assert_eq!(mode.next(), IamViewMode::Policies);
        
        let mode = IamViewMode::Policies;
        assert_eq!(mode.next(), IamViewMode::Users); // Wraps around
    }

    #[test]
    fn test_iam_view_mode_is_main_tab() {
        assert!(IamViewMode::Users.is_main_tab());
        assert!(IamViewMode::Roles.is_main_tab());
        assert!(IamViewMode::Policies.is_main_tab());
        assert!(!IamViewMode::UserAttachedPolicies.is_main_tab());
    }

    #[test]
    fn test_message_navigate() {
        let msg = Message::navigate(Service::EC2);
        match msg {
            Message::Global(GlobalMessage::Navigate(service)) => {
                assert_eq!(service, Service::EC2);
            }
            _ => panic!("Expected Navigate message"),
        }
    }

    #[test]
    fn test_message_quit() {
        let msg = Message::quit();
        assert!(matches!(msg, Message::Global(GlobalMessage::Quit)));
    }

    #[test]
    fn test_message_ec2_start() {
        let msg = Message::ec2_start("i-12345".to_string());
        match msg {
            Message::Service(ServiceAction::Ec2(Ec2Action::Start(id))) => {
                assert_eq!(id, "i-12345");
            }
            _ => panic!("Expected EC2 Start action"),
        }
    }

    #[test]
    fn test_message_s3_load_objects() {
        let msg = Message::s3_load_objects("my-bucket".to_string());
        match msg {
            Message::Service(ServiceAction::S3(S3Action::LoadObjects(bucket))) => {
                assert_eq!(bucket, "my-bucket");
            }
            _ => panic!("Expected S3 LoadObjects action"),
        }
    }

    #[test]
    fn test_message_s3_delete_object() {
        let msg = Message::s3_delete_object("my-bucket".to_string(), "key.txt".to_string());
        match msg {
            Message::Service(ServiceAction::S3(S3Action::DeleteObject { bucket, key })) => {
                assert_eq!(bucket, "my-bucket");
                assert_eq!(key, "key.txt");
            }
            _ => panic!("Expected S3 DeleteObject action"),
        }
    }

    #[test]
    fn test_message_rds_actions() {
        let start = Message::rds_start("db-1".to_string());
        assert!(matches!(start, Message::Service(ServiceAction::Rds(RdsAction::Start(_)))));
        
        let stop = Message::rds_stop("db-1".to_string());
        assert!(matches!(stop, Message::Service(ServiceAction::Rds(RdsAction::Stop(_)))));
        
        let reboot = Message::rds_reboot("db-1".to_string());
        assert!(matches!(reboot, Message::Service(ServiceAction::Rds(RdsAction::Reboot(_)))));
    }

    #[test]
    fn test_message_dynamodb_actions() {
        let drill = Message::dynamodb_drill_down();
        assert!(matches!(drill, Message::Service(ServiceAction::DynamoDb(DynamoDbAction::DrillDownTable))));
        
        let exit = Message::dynamodb_exit_drill_down();
        assert!(matches!(exit, Message::Service(ServiceAction::DynamoDb(DynamoDbAction::ExitDrillDown))));
    }

    #[test]
    fn test_message_vpc_actions() {
        let drill = Message::vpc_drill_down_sg();
        assert!(matches!(drill, Message::Service(ServiceAction::Vpc(VpcAction::DrillDownSecurityGroup))));
        
        let toggle = Message::vpc_toggle_sg_rules_direction();
        assert!(matches!(toggle, Message::Service(ServiceAction::Vpc(VpcAction::ToggleSgRulesDirection))));
    }

    #[test]
    fn test_message_iam_actions() {
        let user = Message::iam_drill_down_user();
        assert!(matches!(user, Message::Service(ServiceAction::Iam(IamAction::DrillDownUser))));
        
        let role = Message::iam_drill_down_role();
        assert!(matches!(role, Message::Service(ServiceAction::Iam(IamAction::DrillDownRole))));
        
        let policy = Message::iam_drill_down_policy();
        assert!(matches!(policy, Message::Service(ServiceAction::Iam(IamAction::DrillDownPolicy))));
    }

    #[test]
    fn test_message_view_mode_navigation() {
        let cycle = Message::cycle_view_mode();
        assert!(matches!(cycle, Message::Global(GlobalMessage::CycleViewMode)));
        
        let next = Message::next_view();
        assert!(matches!(next, Message::Global(GlobalMessage::NextView)));
        
        let prev = Message::previous_view();
        assert!(matches!(prev, Message::Global(GlobalMessage::PreviousView)));
    }

    #[test]
    fn test_message_action_confirmation() {
        let confirm = Message::confirm_action();
        assert!(matches!(confirm, Message::Global(GlobalMessage::ConfirmAction)));
        
        let cancel = Message::cancel_action();
        assert!(matches!(cancel, Message::Global(GlobalMessage::CancelAction)));
    }

    #[test]
    fn test_focus_enum() {
        let focus = Focus::Sidebar;
        assert_eq!(focus, Focus::Sidebar);
        assert_ne!(focus, Focus::Main);
    }

    #[test]
    fn test_input_mode_enum() {
        let mode = InputMode::Normal;
        assert_eq!(mode, InputMode::Normal);
        assert_ne!(mode, InputMode::Filtering);
    }
}
