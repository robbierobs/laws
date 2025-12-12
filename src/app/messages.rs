//! Message and enum definitions for the application state machine
//!
//! This module uses a hierarchical message structure:
//! - `Message::Global` for app-wide operations (navigation, quit, UI toggles)
//! - `Message::Service` for service-specific actions

use crate::app::ViewMode;
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
    SecretsManager,
    ECS,
    ECR,
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
            Service::ECS => "ECS",
            Service::ECR => "ECR",
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
            Self::ECS,
            Self::ECR,
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
        &[Self::Vpcs, Self::Subnets, Self::SecurityGroups, Self::SecurityGroupRules]
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
    // next() and prev() now use trait defaults which check is_main_tab()
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
        &[Self::Users, Self::Roles, Self::Policies, Self::UserAttachedPolicies, Self::RoleAttachedPolicies, Self::PolicyDocument]
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
    // next() and prev() now use trait defaults which check is_main_tab()
}

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
    // next() and prev() now use trait defaults which check is_main_tab()
}

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
    // next() and prev() now use trait defaults which check is_main_tab()
}

// to_index() removed, use ViewMode::index() trait method instead

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
// Per-Service Action Enums
// ============================================================================

/// EC2-specific actions
#[derive(Debug, Clone)]
pub enum Ec2Action {
    Start(String),
    Stop(String),
    Reboot(String),
    Terminate(String),
}

/// S3-specific actions
#[derive(Debug, Clone)]
pub enum S3Action {
    LoadObjects(String),
    LoadBucketDetails(String),
    DeleteObject {
        bucket: String,
        key: String,
    },
    /// Download object to ~/Downloads directory
    DownloadObject {
        bucket: String,
        key: String,
    },
    /// Open object (download to temp dir and display in terminal/viewer)
    OpenObject {
        bucket: String,
        key: String,
    },
    /// Edit object in $EDITOR and upload changes
    EditObject {
        bucket: String,
        key: String,
    },
    LeaveBucket,
}

/// RDS-specific actions
#[derive(Debug, Clone)]

pub enum RdsAction {
    Start(String),
    Stop(String),
    Reboot(String),
    Delete(String),
}

/// DynamoDB-specific actions
#[derive(Debug, Clone)]
pub enum DynamoDbAction {
    DrillDownTable,
    ExitDrillDown,
    LoadItems(String),
    DeleteItem {
        table_name: String,
        key_attrs: HashMap<String, String>,
    },
}

/// Lambda-specific actions
#[derive(Debug, Clone)]
pub enum LambdaAction {
    InvokeFunction(String),
    DeleteFunction(String),
    LoadFunctionDetails(String),
}

/// VPC-specific actions
#[derive(Debug, Clone)]
pub enum VpcAction {
    DrillDownSecurityGroup,
    ExitSecurityGroupRules,
    ToggleSgRulesDirection,
    DeleteSecurityGroup(String),
}

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

/// Backup-specific actions (placeholder for future)
#[derive(Debug, Clone)]
pub enum BackupAction {
    LoadRecoveryPoints(String),
    LeaveVault,
}

/// CloudTrail-specific actions
#[derive(Debug, Clone)]
pub enum CloudTrailAction {
    ShowEventDetails(String),
    CloseEventDetails,
    DeleteTrail(String),
}

/// SecretsManager-specific actions (placeholder for future)
#[derive(Debug, Clone)]
pub enum SecretsManagerAction {
    GetSecretValue(String),
    CloseSecretValue,
    DeleteSecret(String),
}

/// ECS-specific actions
#[derive(Debug, Clone)]
pub enum EcsAction {
    // Navigation
    ViewServices(String),       // cluster_arn
    ViewTasks(String),          // service_arn
    ViewTaskDefinition(String), // task_definition_arn
    BackToClusters,
    BackToServices,
    BackToTasks,

    // Service actions
    UpdateDesiredCount {
        cluster_arn: String,
        service_name: String,
        desired_count: i32,
    },
    ForceNewDeployment {
        cluster_arn: String,
        service_name: String,
    },
    #[allow(dead_code)] // Planned: update service to use different task definition
    UpdateServiceTaskDefinition {
        cluster_arn: String,
        service_name: String,
        task_definition_arn: String,
    },
    /// Modify ECS Service - supports task definition, CPU, and Memory changes
    /// Note: CPU and Memory require creating a new task definition revision
    UpdateService {
        cluster_arn: String,
        service_name: String,
        /// New task definition ARN (full ARN or family:revision)
        task_definition: Option<String>,
        /// CPU value (in Fargate units: "256", "512", "1024", etc.)
        cpu: Option<String>,
        /// Memory value (in MiB: "512", "1024", "2048", etc.)
        memory: Option<String>,
        /// Force a new deployment even if no other changes
        force_new_deployment: bool,
    },

    // Task actions
    StopTask {
        cluster_arn: String,
        task_arn: String,
    },

    // Task definition actions
    DeregisterTaskDefinition(String), // task_definition_arn
    EditTaskDefinition(String),       // task_definition_arn
    #[allow(dead_code)] // Planned: list all revisions of a task definition family
    ListTaskDefinitions(String),      // family name
    /// Load full task definitions for the selector modal
    LoadTaskDefinitionsForSelector(String), // family name
}

/// ECR-specific actions
#[derive(Debug, Clone)]
pub enum EcrAction {
    LoadImages(String),
    BackToRepositories,
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
    SwitchProfileRegion {
        profile: Option<String>,
        region: String,
        read_only: bool,
    },
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
    Ecs(EcsAction),
    Ecr(EcrAction),
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
        Message::Service(ServiceAction::CloudTrail(
            CloudTrailAction::ShowEventDetails(details),
        ))
    }

    pub fn cloudtrail_close_event_details() -> Self {
        Message::Service(ServiceAction::CloudTrail(
            CloudTrailAction::CloseEventDetails,
        ))
    }

    pub fn switch_profile_region(profile: Option<String>, region: String, read_only: bool) -> Self {
        Message::Global(GlobalMessage::SwitchProfileRegion {
            profile,
            region,
            read_only,
        })
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

    pub fn ec2_terminate(instance_id: String) -> Self {
        Message::Service(ServiceAction::Ec2(Ec2Action::Terminate(instance_id)))
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

    pub fn s3_edit_object(bucket: String, key: String) -> Self {
        Message::Service(ServiceAction::S3(S3Action::EditObject { bucket, key }))
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
        Message::Service(ServiceAction::DynamoDb(DynamoDbAction::LoadItems(
            table_name,
        )))
    }

    pub fn dynamodb_delete_item(table_name: String, key_attrs: HashMap<String, String>) -> Self {
        Message::Service(ServiceAction::DynamoDb(DynamoDbAction::DeleteItem {
            table_name,
            key_attrs,
        }))
    }

    // Lambda message constructors
    pub fn lambda_invoke(function_name: String) -> Self {
        Message::Service(ServiceAction::Lambda(LambdaAction::InvokeFunction(
            function_name,
        )))
    }

    pub fn lambda_delete(function_name: String) -> Self {
        Message::Service(ServiceAction::Lambda(LambdaAction::DeleteFunction(
            function_name,
        )))
    }

    pub fn lambda_load_details(function_name: String) -> Self {
        Message::Service(ServiceAction::Lambda(LambdaAction::LoadFunctionDetails(
            function_name,
        )))
    }

    // RDS message constructors
    pub fn rds_delete(instance_id: String) -> Self {
        Message::Service(ServiceAction::Rds(RdsAction::Delete(instance_id)))
    }

    pub fn backup_load_recovery_points(vault_name: String) -> Self {
        Message::Service(ServiceAction::Backup(BackupAction::LoadRecoveryPoints(
            vault_name,
        )))
    }

    pub fn backup_leave_vault() -> Self {
        Message::Service(ServiceAction::Backup(BackupAction::LeaveVault))
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

    pub fn vpc_delete_security_group(group_id: String) -> Self {
        Message::Service(ServiceAction::Vpc(VpcAction::DeleteSecurityGroup(group_id)))
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

    pub fn iam_delete_user(name: String) -> Self {
        Message::Service(ServiceAction::Iam(IamAction::DeleteUser(name)))
    }

    pub fn iam_delete_role(name: String) -> Self {
        Message::Service(ServiceAction::Iam(IamAction::DeleteRole(name)))
    }

    pub fn iam_delete_policy(arn: String) -> Self {
        Message::Service(ServiceAction::Iam(IamAction::DeletePolicy(arn)))
    }

    // CloudTrail message constructors
    pub fn cloudtrail_delete_trail(name: String) -> Self {
        Message::Service(ServiceAction::CloudTrail(CloudTrailAction::DeleteTrail(
            name,
        )))
    }

    // SecretsManager message constructors
    pub fn secretsmanager_get_value(arn: String) -> Self {
        Message::Service(ServiceAction::SecretsManager(
            SecretsManagerAction::GetSecretValue(arn),
        ))
    }

    pub fn secretsmanager_close_value() -> Self {
        Message::Service(ServiceAction::SecretsManager(
            SecretsManagerAction::CloseSecretValue,
        ))
    }

    pub fn secretsmanager_delete_secret(arn: String) -> Self {
        Message::Service(ServiceAction::SecretsManager(
            SecretsManagerAction::DeleteSecret(arn),
        ))
    }

    // ECS message constructors
    pub fn ecs_view_services(cluster_arn: String) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::ViewServices(cluster_arn)))
    }

    pub fn ecs_view_tasks(service_arn: String) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::ViewTasks(service_arn)))
    }

    pub fn ecs_view_task_definition(task_definition_arn: String) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::ViewTaskDefinition(
            task_definition_arn,
        )))
    }

    pub fn ecs_back_to_clusters() -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::BackToClusters))
    }

    pub fn ecs_back_to_services() -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::BackToServices))
    }

    pub fn ecs_back_to_tasks() -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::BackToTasks))
    }

    pub fn ecs_update_desired_count(
        cluster_arn: String,
        service_name: String,
        desired_count: i32,
    ) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::UpdateDesiredCount {
            cluster_arn,
            service_name,
            desired_count,
        }))
    }

    pub fn ecs_force_new_deployment(cluster_arn: String, service_name: String) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::ForceNewDeployment {
            cluster_arn,
            service_name,
        }))
    }

    pub fn ecs_stop_task(cluster_arn: String, task_arn: String) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::StopTask {
            cluster_arn,
            task_arn,
        }))
    }

    pub fn ecs_deregister_task_definition(task_definition_arn: String) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::DeregisterTaskDefinition(
            task_definition_arn,
        )))
    }

    pub fn ecs_edit_task_definition(task_definition_arn: String) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::EditTaskDefinition(
            task_definition_arn,
        )))
    }

    #[allow(dead_code)] // Planned: list all revisions of a task definition family
    pub fn ecs_list_task_definitions(family: String) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::ListTaskDefinitions(family)))
    }

    #[allow(dead_code)] // Planned: update service to use different task definition
    pub fn ecs_update_service_task_definition(
        cluster_arn: String,
        service_name: String,
        task_definition_arn: String,
    ) -> Self {
        Message::Service(ServiceAction::Ecs(
            EcsAction::UpdateServiceTaskDefinition {
                cluster_arn,
                service_name,
                task_definition_arn,
            },
        ))
    }

    /// Create a message to update an ECS service with optional task definition, CPU, and memory changes
    pub fn ecs_update_service(
        cluster_arn: String,
        service_name: String,
        task_definition: Option<String>,
        cpu: Option<String>,
        memory: Option<String>,
        force_new_deployment: bool,
    ) -> Self {
        Message::Service(ServiceAction::Ecs(EcsAction::UpdateService {
            cluster_arn,
            service_name,
            task_definition,
            cpu,
            memory,
            force_new_deployment,
        }))
    }

    /// Load task definitions with full details for the selector modal
    pub fn ecs_load_task_definitions_for_selector(family: String) -> Self {
        Message::Service(ServiceAction::Ecs(
            EcsAction::LoadTaskDefinitionsForSelector(family),
        ))
    }


    pub fn ecr_load_images(repository_name: String) -> Self {
        Message::Service(ServiceAction::Ecr(EcrAction::LoadImages(repository_name)))
    }

    pub fn ecr_back_to_repos() -> Self {
        Message::Service(ServiceAction::Ecr(EcrAction::BackToRepositories))
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
    /// ECS Service editor modal
    EcsServiceEditor,
    /// ECS Task Definition selector modal
    EcsTaskDefSelector,
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
        assert_eq!(services.len(), 12); // All 12 services
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

        matches!(
            start,
            Message::Service(ServiceAction::Ec2(Ec2Action::Start(_)))
        );
        matches!(
            stop,
            Message::Service(ServiceAction::Ec2(Ec2Action::Stop(_)))
        );
        matches!(
            reboot,
            Message::Service(ServiceAction::Ec2(Ec2Action::Reboot(_)))
        );
    }

    #[test]
    fn test_message_rds_actions() {
        let start = Message::rds_start("db-123".to_string());
        let stop = Message::rds_stop("db-456".to_string());

        matches!(
            start,
            Message::Service(ServiceAction::Rds(RdsAction::Start(_)))
        );
        matches!(
            stop,
            Message::Service(ServiceAction::Rds(RdsAction::Stop(_)))
        );
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
