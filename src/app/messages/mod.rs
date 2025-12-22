//! Message and enum definitions for the application state machine
//!
//! This module uses a hierarchical message structure:
//! - `Message::Global` for app-wide operations (navigation, quit, UI toggles)
//! - `Message::Service` for service-specific actions

// AWS service names use capitalized acronyms (EC2, S3, RDS, VPC, IAM, ECS, ECR)
#![allow(clippy::upper_case_acronyms)]

use std::collections::HashMap;

// Sub-modules
mod service;
mod view_modes;
pub mod actions;
pub mod confirmable;

// Re-export Service enum
pub use service::Service;

// Re-export ViewMode enums
pub use view_modes::{
    BackupViewMode, CloudTrailViewMode, DynamoDbViewMode, EcrViewMode, EcsViewMode, IamViewMode,
    VpcViewMode,
};

// Re-export action enums
pub use actions::{
    BackupAction, CloudTrailAction, CloudTrailLookupParams, DynamoDbAction, Ec2Action, EcrAction, EcsAction, IamAction,
    LambdaAction, RdsAction, S3Action, SecretsManagerAction, VpcAction,
};

// Re-export confirmable trait
pub use confirmable::ConfirmableAction;

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
    /// Open global search modal
    OpenGlobalSearch,
    /// Close global search modal
    CloseGlobalSearch,
    /// Navigate to a search result (service + resource ID)
    GotoSearchResult {
        service: Service,
        resource_id: String,
    },
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

impl ConfirmableAction for ServiceAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::Ec2(a) => a.confirmation_description(),
            Self::S3(a) => a.confirmation_description(),
            Self::Rds(a) => a.confirmation_description(),
            Self::DynamoDb(a) => a.confirmation_description(),
            Self::Lambda(a) => a.confirmation_description(),
            Self::Vpc(a) => a.confirmation_description(),
            Self::Iam(a) => a.confirmation_description(),
            Self::Backup(a) => a.confirmation_description(),
            Self::CloudTrail(a) => a.confirmation_description(),
            Self::SecretsManager(a) => a.confirmation_description(),
            Self::Ecs(a) => a.confirmation_description(),
            Self::Ecr(a) => a.confirmation_description(),
        }
    }
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

    pub fn open_global_search() -> Self {
        Message::Global(GlobalMessage::OpenGlobalSearch)
    }

    #[allow(dead_code)]
    pub fn close_global_search() -> Self {
        Message::Global(GlobalMessage::CloseGlobalSearch)
    }

    pub fn goto_search_result(service: Service, resource_id: String) -> Self {
        Message::Global(GlobalMessage::GotoSearchResult {
            service,
            resource_id,
        })
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

    pub fn s3_create_bucket(name: String) -> Self {
        Message::Service(ServiceAction::S3(S3Action::CreateBucket(name)))
    }

    pub fn s3_delete_bucket(name: String) -> Self {
        Message::Service(ServiceAction::S3(S3Action::DeleteBucket(name)))
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

    pub fn backup_open_filter_modal() -> Self {
        Message::Service(ServiceAction::Backup(BackupAction::OpenFilterModal))
    }

    pub fn backup_close_filter_modal() -> Self {
        Message::Service(ServiceAction::Backup(BackupAction::CloseFilterModal))
    }

    pub fn backup_apply_filters() -> Self {
        Message::Service(ServiceAction::Backup(BackupAction::ApplyFilters))
    }

    pub fn backup_clear_filters() -> Self {
        Message::Service(ServiceAction::Backup(BackupAction::ClearFilters))
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

    pub fn cloudtrail_open_filter_modal() -> Self {
        Message::Service(ServiceAction::CloudTrail(CloudTrailAction::OpenFilterModal))
    }

    pub fn cloudtrail_load_more_events() -> Self {
        Message::Service(ServiceAction::CloudTrail(CloudTrailAction::LoadMoreEvents))
    }

    #[allow(dead_code)] // Used internally via direct Message construction
    pub fn cloudtrail_apply_filters(params: CloudTrailLookupParams) -> Self {
        Message::Service(ServiceAction::CloudTrail(CloudTrailAction::ApplyFilters(params)))
    }

    pub fn cloudtrail_clear_filters() -> Self {
        Message::Service(ServiceAction::CloudTrail(CloudTrailAction::ClearFilters))
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
        Message::Service(ServiceAction::Ecs(EcsAction::UpdateServiceTaskDefinition {
            cluster_arn,
            service_name,
            task_definition_arn,
        }))
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

    pub fn ecr_pull_image(repository_uri: String, image_tag: String) -> Self {
        Message::Service(ServiceAction::Ecr(EcrAction::PullImage {
            repository_uri,
            image_tag,
        }))
    }

    pub fn ecr_load_more_images() -> Self {
        Message::Service(ServiceAction::Ecr(EcrAction::LoadMoreImages))
    }

    pub fn ecr_open_filter_modal() -> Self {
        Message::Service(ServiceAction::Ecr(EcrAction::OpenFilterModal))
    }

    pub fn ecr_close_filter_modal() -> Self {
        Message::Service(ServiceAction::Ecr(EcrAction::CloseFilterModal))
    }

    pub fn ecr_apply_filters() -> Self {
        Message::Service(ServiceAction::Ecr(EcrAction::ApplyFilters))
    }

    pub fn ecr_clear_filters() -> Self {
        Message::Service(ServiceAction::Ecr(EcrAction::ClearFilters))
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
    /// Global search modal
    GlobalSearch,
    /// S3 bucket creation modal
    S3BucketCreation,
    /// CloudTrail event filter modal
    CloudTrailEventFilter,
    /// ECR image filter modal
    EcrImageFilter,
    /// Backup job filter modal
    BackupJobFilter,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ViewMode;

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
