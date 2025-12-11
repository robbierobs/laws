//! Message and enum definitions for the application state machine
//!
//! This module uses a hierarchical message structure:
//! - `Message::Global` for app-wide operations (navigation, quit, UI toggles)
//! - `Message::Service` for service-specific actions

use std::collections::HashMap;
use crate::app::ViewMode;

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
    SecretsManager,
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
            Service::SecretsManager => "SecretsManager",
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
            Self::SecretsManager,
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

impl ViewMode for VpcViewMode {
    fn all() -> &'static [Self] {
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
            _ => Self::Vpcs,
        }
    }
    
    fn label(&self) -> &'static str {
        match self {
            Self::Vpcs => "VPCs",
            Self::Subnets => "Subnets",
            Self::SecurityGroups => "Security Groups",
            Self::SecurityGroupRules => "Rules",
        }
    }

    // Override next/prev to prevent styling out of tabs if we are in drill-down
    fn next(&self) -> Self {
        if matches!(self, Self::SecurityGroupRules) {
            *self
        } else {
            let i = self.index();
            let all = Self::all();
            let len = all.len();
            let next_idx = (i + 1) % len;
            Self::from_index(next_idx)
        }
    }

    fn prev(&self) -> Self {
        if matches!(self, Self::SecurityGroupRules) {
            *self
        } else {
            let i = self.index();
            let all = Self::all();
            let len = all.len();
            let prev_idx = if i == 0 { len - 1 } else { i - 1 };
            Self::from_index(prev_idx)
        }
    }
}

// to_index() removed, use ViewMode::index() trait method instead

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
            _ => Self::Users,
        }
    }
    
    fn label(&self) -> &'static str {
        match self {
            Self::Users => "Users",
            Self::Roles => "Roles",
            Self::Policies => "Policies",
            _ => "Details",
        }
    }

    fn next(&self) -> Self {
        if !self.is_main_tab() {
            *self
        } else {
            let i = self.index();
            let all = Self::all();
            let len = all.len();
            let next_idx = (i + 1) % len;
            Self::from_index(next_idx)
        }
    }

    fn prev(&self) -> Self {
        if !self.is_main_tab() {
            *self
        } else {
            let i = self.index();
            let all = Self::all();
            let len = all.len();
            let prev_idx = if i == 0 { len - 1 } else { i - 1 };
            Self::from_index(prev_idx)
        }
    }
}

impl IamViewMode {
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

impl ViewMode for BackupViewMode {
    fn all() -> &'static [Self] {
        &[Self::Vaults, Self::Plans, Self::Jobs]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Vaults,
            1 => Self::Plans,
            2 => Self::Jobs,
            _ => Self::Vaults,
        }
    }
    
    fn label(&self) -> &'static str {
        match self {
            Self::Vaults => "Vaults",
            Self::Plans => "Plans",
            Self::Jobs => "Jobs",
        }
    }
}

// to_index(), next(), previous() removed, use ViewMode trait methods instead

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

// to_index(), next(), previous() removed, use ViewMode trait methods instead

/// View mode for DynamoDB service
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DynamoDbViewMode {
    #[default]
    Tables = 0,
    Items = 1,
}

impl ViewMode for DynamoDbViewMode {
    fn all() -> &'static [Self] {
        &[Self::Tables]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Tables,
            _ => Self::Tables,
        }
    }
    
    fn label(&self) -> &'static str {
        match self {
            Self::Tables => "Tables",
            Self::Items => "Items",
        }
    }

    fn next(&self) -> Self {
        // Only one tab, so effectively no op unless we want to cycle same mode
        *self
    }
    fn prev(&self) -> Self {
        *self
    }
}

// to_index() removed, use ViewMode::index() trait method instead

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
    /// Download object to ~/Downloads directory
    DownloadObject { bucket: String, key: String },
    /// Open object (download to temp dir and display in terminal/viewer)
    OpenObject { bucket: String, key: String },
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

/// Lambda-specific actions
#[derive(Debug, Clone)]
pub enum LambdaAction {
    InvokeFunction(String),
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

/// CloudTrail-specific actions
#[derive(Debug, Clone)]
pub enum CloudTrailAction {
    ShowEventDetails(String),
    CloseEventDetails,
}

/// SecretsManager-specific actions (placeholder for future)
#[derive(Debug, Clone)]
pub enum SecretsManagerAction {
    GetSecretValue(String),
    CloseSecretValue,
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
    /// Toggle fullscreen detail panel mode
    ToggleDetailFullscreen,
    /// Scroll detail panel up
    DetailScrollUp,
    /// Scroll detail panel down
    DetailScrollDown,
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
    /// Open profile switcher modal
    OpenProfileSwitcher,
    /// Cancel profile switcher
    CancelProfileSwitcher,
    /// Switch to selected profile and region
    SwitchProfileRegion { profile: Option<String>, region: String, read_only: bool },
    /// Copy text to system clipboard
    CopyToClipboard(String),
}

/// Service-specific messages
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    SecretsManager(SecretsManagerAction),
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
    
    pub fn open_profile_switcher() -> Self {
        Message::Global(GlobalMessage::OpenProfileSwitcher)
    }
    
    pub fn cancel_profile_switcher() -> Self {
        Message::Global(GlobalMessage::CancelProfileSwitcher)
    }

    pub fn cloudtrail_show_event_details(details: String) -> Self {
        Message::Service(ServiceAction::CloudTrail(CloudTrailAction::ShowEventDetails(details)))
    }

    pub fn cloudtrail_close_event_details() -> Self {
        Message::Service(ServiceAction::CloudTrail(CloudTrailAction::CloseEventDetails))
    }
    
    pub fn switch_profile_region(profile: Option<String>, region: String, read_only: bool) -> Self {
        Message::Global(GlobalMessage::SwitchProfileRegion { profile, region, read_only })
    }

    pub fn copy_to_clipboard(text: String) -> Self {
        Message::Global(GlobalMessage::CopyToClipboard(text))
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
    
    pub fn s3_download_object(bucket: String, key: String) -> Self {
        Message::Service(ServiceAction::S3(S3Action::DownloadObject { bucket, key }))
    }
    
    pub fn s3_open_object(bucket: String, key: String) -> Self {
        Message::Service(ServiceAction::S3(S3Action::OpenObject { bucket, key }))
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
    
    // Lambda message constructors
    pub fn lambda_invoke(function_name: String) -> Self {
        Message::Service(ServiceAction::Lambda(LambdaAction::InvokeFunction(function_name)))
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
    /// Profile switcher modal - selecting profile
    ProfileSwitcherProfile,
    /// Profile switcher modal - selecting region  
    ProfileSwitcherRegion,
}

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
    }

    #[test]
    fn test_service_iterator() {
        let services: Vec<Service> = Service::iterator().collect();
        assert_eq!(services.len(), 10); // All 10 services
        assert!(services.contains(&Service::EC2));
        assert!(services.contains(&Service::SecretsManager));
    }

    #[test]
    fn test_message_quit() {
        let msg = Message::quit();
        matches!(msg, Message::Global(GlobalMessage::Quit));
    }

    #[test]
    fn test_message_navigate() {
        let msg = Message::navigate(Service::S3);
        if let Message::Global(GlobalMessage::Navigate(svc)) = msg {
            assert_eq!(svc, Service::S3);
        } else {
            panic!("Expected Navigate message");
        }
    }

    #[test]
    fn test_message_ec2_actions() {
        let start = Message::ec2_start("i-123".to_string());
        let stop = Message::ec2_stop("i-456".to_string());
        let reboot = Message::ec2_reboot("i-789".to_string());
        
        matches!(start, Message::Service(ServiceAction::Ec2(Ec2Action::Start(_))));
        matches!(stop, Message::Service(ServiceAction::Ec2(Ec2Action::Stop(_))));
        matches!(reboot, Message::Service(ServiceAction::Ec2(Ec2Action::Reboot(_))));
    }

    #[test]
    fn test_message_rds_actions() {
        let start = Message::rds_start("db-123".to_string());
        let stop = Message::rds_stop("db-456".to_string());
        
        matches!(start, Message::Service(ServiceAction::Rds(RdsAction::Start(_))));
        matches!(stop, Message::Service(ServiceAction::Rds(RdsAction::Stop(_))));
    }

    #[test]
    fn test_vpc_view_mode_navigation() {
        let mode = VpcViewMode::Vpcs;
        assert_eq!(mode.next(), VpcViewMode::Subnets);
        assert_eq!(mode.next().next(), VpcViewMode::SecurityGroups);
        
        // Should wrap around
        let sg_mode = VpcViewMode::SecurityGroups;
        assert_eq!(sg_mode.next(), VpcViewMode::Vpcs);
    }

    #[test]
    fn test_vpc_view_mode_labels() {
        assert_eq!(VpcViewMode::Vpcs.label(), "VPCs");
        assert_eq!(VpcViewMode::Subnets.label(), "Subnets");
        assert_eq!(VpcViewMode::SecurityGroups.label(), "Security Groups");
    }

    #[test]
    fn test_iam_view_mode_is_main_tab() {
        assert!(IamViewMode::Users.is_main_tab());
        assert!(IamViewMode::Roles.is_main_tab());
        assert!(IamViewMode::Policies.is_main_tab());
        assert!(!IamViewMode::UserAttachedPolicies.is_main_tab());
        assert!(!IamViewMode::PolicyDocument.is_main_tab());
    }
}
