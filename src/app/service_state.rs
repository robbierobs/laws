//! Service-specific state structs
//!
//! This module consolidates service-specific state that was previously scattered
//! across 50+ fields in the App struct. Each service has its own state struct
//! containing all data, list states, and view modes specific to that service.

#![allow(dead_code)]

use ratatui::widgets::TableState;
use std::collections::HashMap;

use crate::models::backup::{BackupJob, BackupPlan, BackupVault, RecoveryPoint};
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::models::dynamodb::{DynamoDbItem, DynamoDbTable};
use crate::models::ec2::Ec2Instance;
use crate::models::ecs::{EcsCluster, EcsService, EcsTask, EcsTaskDefinition};
use crate::models::iam::{IamPolicy, IamRole, IamUser};
use crate::models::lambda::LambdaFunction;
use crate::models::rds::RdsInstance;
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crate::models::secretsmanager::Secret;
use crate::models::vpc::{SecurityGroup, SecurityGroupRule, Subnet, Vpc};

use super::{
    BackupViewMode, CloudTrailViewMode, DynamoDbViewMode, EcsViewMode, IamViewMode, VpcViewMode,
};
use super::{InputResult, Message, ServiceInputHandler, TableStateExt};
use crossterm::event::{KeyCode, KeyEvent};

// ============================================================================
// EC2 State
// ============================================================================

/// State for EC2 service
#[derive(Default)]
pub struct Ec2State {
    pub instances: Vec<Ec2Instance>,
    pub list_state: TableState,
}

impl Ec2State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected instance, if any
    pub fn selected_instance(&self) -> Option<&Ec2Instance> {
        self.list_state
            .selected()
            .and_then(|i| self.instances.get(i))
    }

    /// Get the instance ID of the currently selected instance
    pub fn selected_instance_id(&self) -> Option<String> {
        self.selected_instance().map(|i| i.instance_id.clone())
    }
}

impl ServiceInputHandler for Ec2State {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(self.instances.len()),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(self.instances.len()),
            KeyCode::Char('s') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(instance) = self.instances.get(i) {
                        return InputResult::Action(Message::ec2_start(
                            instance.instance_id.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('S') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(instance) = self.instances.get(i) {
                        return InputResult::Action(Message::ec2_stop(
                            instance.instance_id.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('R') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(instance) = self.instances.get(i) {
                        return InputResult::Action(Message::ec2_reboot(
                            instance.instance_id.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(instance) = self.instances.get(i) {
                        return InputResult::Action(Message::ec2_terminate(
                            instance.instance_id.clone(),
                        ));
                    }
                }
            }
            _ => {}
        }
        InputResult::None
    }
}

// ============================================================================
// S3 State
// ============================================================================

/// State for S3 service
#[derive(Default)]
pub struct S3State {
    pub buckets: Vec<S3Bucket>,
    pub list_state: TableState,
    pub current_bucket: Option<String>,
    pub objects: Vec<S3Object>,
    pub object_list_state: TableState,
    pub bucket_details: HashMap<String, S3BucketDetails>,
    /// Content of the last opened object (if it's text)
    pub opened_object_content: Option<String>,
    /// Path to the last opened object file
    pub opened_object_path: Option<String>,
    /// Key of the opened object (for display in popup title)
    pub opened_object_key: Option<String>,
    /// Whether to show the object viewer popup
    pub show_object_viewer: bool,
    /// Scroll offset for the object viewer
    pub viewer_scroll_offset: u16,
    /// Pending edit operation (bucket, key, path) - for synchronous editor handling
    pub pending_edit: Option<(String, String, String)>,
}

impl S3State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're currently viewing objects inside a bucket
    pub fn is_viewing_objects(&self) -> bool {
        self.current_bucket.is_some()
    }

    /// Get the currently selected bucket, if any
    pub fn selected_bucket(&self) -> Option<&S3Bucket> {
        self.list_state.selected().and_then(|i| self.buckets.get(i))
    }

    /// Get the currently selected object, if any
    pub fn selected_object(&self) -> Option<&S3Object> {
        self.object_list_state
            .selected()
            .and_then(|i| self.objects.get(i))
    }
}

impl ServiceInputHandler for S3State {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        if self.current_bucket.is_some() {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.object_list_state.nav_down(self.objects.len());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.object_list_state.nav_up(self.objects.len());
                }
                KeyCode::Char('X') | KeyCode::Delete => {
                    if let Some(i) = self.object_list_state.selected() {
                        if let Some(obj) = self.objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Action(Message::s3_delete_object(
                                    bucket.clone(),
                                    obj.key.clone(),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char('o') => {
                    // Open object - download to temp and view
                    if let Some(i) = self.object_list_state.selected() {
                        if let Some(obj) = self.objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Message(Message::s3_open_object(
                                    bucket.clone(),
                                    obj.key.clone(),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char('E') => {
                    // Edit object - download, open in $EDITOR, upload changes
                    if let Some(i) = self.object_list_state.selected() {
                        if let Some(obj) = self.objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Action(Message::s3_edit_object(
                                    bucket.clone(),
                                    obj.key.clone(),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char('w') => {
                    // Download/Write object to ~/Downloads
                    if let Some(i) = self.object_list_state.selected() {
                        if let Some(obj) = self.objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Message(Message::s3_download_object(
                                    bucket.clone(),
                                    obj.key.clone(),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Backspace => {
                    return InputResult::Message(Message::s3_leave_bucket())
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.list_state.nav_down(self.buckets.len());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.list_state.nav_up(self.buckets.len());
                }
                KeyCode::Enter => {
                    if let Some(i) = self.list_state.selected() {
                        if let Some(bucket) = self.buckets.get(i) {
                            return InputResult::Message(Message::s3_load_objects(
                                bucket.name.clone(),
                            ));
                        }
                    }
                }
                KeyCode::Char('i') => {
                    if let Some(i) = self.list_state.selected() {
                        if let Some(bucket) = self.buckets.get(i) {
                            return InputResult::Message(Message::s3_load_bucket_details(
                                bucket.name.clone(),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        InputResult::None
    }
}

// ============================================================================
// RDS State
// ============================================================================

/// State for RDS service
#[derive(Default)]
pub struct RdsState {
    pub instances: Vec<RdsInstance>,
    pub list_state: TableState,
}

impl RdsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected instance, if any
    pub fn selected_instance(&self) -> Option<&RdsInstance> {
        self.list_state
            .selected()
            .and_then(|i| self.instances.get(i))
    }

    /// Get the instance ID of the currently selected instance
    pub fn selected_instance_id(&self) -> Option<String> {
        self.selected_instance()
            .map(|i| i.db_instance_identifier.clone())
    }
}

impl ServiceInputHandler for RdsState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(self.instances.len()),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(self.instances.len()),
            KeyCode::Char('s') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(inst) = self.instances.get(i) {
                        return InputResult::Action(Message::rds_start(
                            inst.db_instance_identifier.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('S') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(inst) = self.instances.get(i) {
                        return InputResult::Action(Message::rds_stop(
                            inst.db_instance_identifier.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('R') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(inst) = self.instances.get(i) {
                        return InputResult::Action(Message::rds_reboot(
                            inst.db_instance_identifier.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(inst) = self.instances.get(i) {
                        return InputResult::Action(Message::rds_delete(
                            inst.db_instance_identifier.clone(),
                        ));
                    }
                }
            }
            _ => {}
        }
        InputResult::None
    }
}

// ============================================================================
// DynamoDB State
// ============================================================================

/// State for DynamoDB service
#[derive(Default)]
pub struct DynamoDbState {
    pub tables: Vec<DynamoDbTable>,
    pub list_state: TableState,
    pub view_mode: DynamoDbViewMode,
    pub current_table: Option<String>,
    pub items: Vec<DynamoDbItem>,
    pub item_list_state: TableState,
}

impl DynamoDbState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're currently viewing items inside a table
    pub fn is_viewing_items(&self) -> bool {
        self.current_table.is_some()
    }

    /// Get the currently selected table, if any
    pub fn selected_table(&self) -> Option<&DynamoDbTable> {
        self.list_state.selected().and_then(|i| self.tables.get(i))
    }

    /// Get the currently selected item, if any
    pub fn selected_item(&self) -> Option<&DynamoDbItem> {
        self.item_list_state
            .selected()
            .and_then(|i| self.items.get(i))
    }
}

impl ServiceInputHandler for DynamoDbState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::DynamoDbViewMode;
        if self.view_mode == DynamoDbViewMode::Items {
            // In items view
            match key.code {
                KeyCode::Esc | KeyCode::Backspace => {
                    return InputResult::Message(Message::dynamodb_exit_drill_down())
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.item_list_state.nav_down(self.items.len());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.item_list_state.nav_up(self.items.len());
                }
                KeyCode::Char('X') | KeyCode::Delete => {
                    // Delete selected item
                    if let Some(idx) = self.item_list_state.selected() {
                        if let Some(item) = self.items.get(idx) {
                            if let Some(table_name) = &self.current_table {
                                // Build key attributes from item
                                let key_attrs: std::collections::HashMap<String, String> =
                                    item.attributes.clone();
                                return InputResult::Action(Message::dynamodb_delete_item(
                                    table_name.clone(),
                                    key_attrs,
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char('r') => {
                    // Refresh items
                    if let Some(table_name) = &self.current_table {
                        return InputResult::Message(Message::dynamodb_load_items(
                            table_name.clone(),
                        ));
                    }
                }
                _ => {}
            }
        } else {
            // In tables list view
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.list_state.nav_down(self.tables.len());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.list_state.nav_up(self.tables.len());
                }
                KeyCode::Enter => return InputResult::Message(Message::dynamodb_drill_down()),
                _ => {}
            }
        }
        InputResult::None
    }
}

// ============================================================================
// Lambda State
// ============================================================================

/// State for Lambda service
#[derive(Default)]
pub struct LambdaState {
    pub functions: Vec<LambdaFunction>,
    pub list_state: TableState,
    pub function_details:
        std::collections::HashMap<String, crate::models::lambda::LambdaFunctionDetails>,
}

impl LambdaState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected function, if any
    pub fn selected_function(&self) -> Option<&LambdaFunction> {
        self.list_state
            .selected()
            .and_then(|i| self.functions.get(i))
    }
}

impl ServiceInputHandler for LambdaState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.functions.len());
                if let Some(f) = self.selected_function() {
                    return InputResult::Message(crate::app::Message::lambda_load_details(
                        f.function_name.clone(),
                    ));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.functions.len());
                if let Some(f) = self.selected_function() {
                    return InputResult::Message(crate::app::Message::lambda_load_details(
                        f.function_name.clone(),
                    ));
                }
            }
            KeyCode::Char('I') => {
                if let Some(f) = self.selected_function() {
                    return InputResult::Action(Message::lambda_invoke(f.function_name.clone()));
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(f) = self.selected_function() {
                    return InputResult::Action(Message::lambda_delete(f.function_name.clone()));
                }
            }
            _ => {}
        }
        InputResult::None
    }
}

// ============================================================================
// VPC State
// ============================================================================

/// State for VPC service
#[derive(Default)]
pub struct VpcState {
    pub vpcs: Vec<Vpc>,
    pub subnets: Vec<Subnet>,
    pub security_groups: Vec<SecurityGroup>,
    pub list_state: TableState,
    pub view_mode: VpcViewMode,
    pub current_sg_rules: Vec<SecurityGroupRule>,
    pub selected_sg_id: Option<String>,
    pub sg_rules_inbound: bool,
}

impl VpcState {
    pub fn new() -> Self {
        Self {
            sg_rules_inbound: true, // Default to showing inbound rules
            ..Self::default()
        }
    }

    /// Get the currently selected VPC, if any
    pub fn selected_vpc(&self) -> Option<&Vpc> {
        if self.view_mode == VpcViewMode::Vpcs {
            self.list_state.selected().and_then(|i| self.vpcs.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected subnet, if any
    pub fn selected_subnet(&self) -> Option<&Subnet> {
        if self.view_mode == VpcViewMode::Subnets {
            self.list_state.selected().and_then(|i| self.subnets.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected security group, if any
    pub fn selected_security_group(&self) -> Option<&SecurityGroup> {
        if self.view_mode == VpcViewMode::SecurityGroups {
            self.list_state
                .selected()
                .and_then(|i| self.security_groups.get(i))
        } else {
            None
        }
    }
}

impl ServiceInputHandler for VpcState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::VpcViewMode;
        let len = match self.view_mode {
            VpcViewMode::Vpcs => self.vpcs.len(),
            VpcViewMode::Subnets => self.subnets.len(),
            VpcViewMode::SecurityGroups => self.security_groups.len(),
            VpcViewMode::SecurityGroupRules => self.current_sg_rules.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter if self.view_mode == VpcViewMode::SecurityGroups => {
                return InputResult::Message(Message::vpc_drill_down_sg())
            }
            KeyCode::Esc if self.view_mode == VpcViewMode::SecurityGroupRules => {
                return InputResult::Message(Message::vpc_exit_sg_rules())
            }
            // Toggle between inbound and outbound rules with 't' or 'v'
            KeyCode::Char('t') | KeyCode::Char('v')
                if self.view_mode == VpcViewMode::SecurityGroupRules =>
            {
                return InputResult::Message(Message::vpc_toggle_sg_rules_direction());
            }
            KeyCode::Char('X') | KeyCode::Delete
                if self.view_mode == VpcViewMode::SecurityGroups =>
            {
                if let Some(sg) = self.selected_security_group() {
                    return InputResult::Action(Message::vpc_delete_security_group(
                        sg.group_id.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }
}

// ============================================================================
// IAM State
// ============================================================================

/// State for IAM service
#[derive(Default)]
pub struct IamState {
    pub roles: Vec<IamRole>,
    pub users: Vec<IamUser>,
    pub policies: Vec<IamPolicy>,
    pub list_state: TableState,
    pub view_mode: IamViewMode,
    pub previous_view_mode: IamViewMode,
    pub current_policies: Vec<IamPolicy>,
    pub current_policy_document: String,
    pub selected_entity_name: Option<String>,
}

impl IamState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected user, if any
    pub fn selected_user(&self) -> Option<&IamUser> {
        if self.view_mode == IamViewMode::Users {
            self.list_state.selected().and_then(|i| self.users.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected role, if any
    pub fn selected_role(&self) -> Option<&IamRole> {
        if self.view_mode == IamViewMode::Roles {
            self.list_state.selected().and_then(|i| self.roles.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected policy, if any
    pub fn selected_policy(&self) -> Option<&IamPolicy> {
        if self.view_mode == IamViewMode::Policies {
            self.list_state
                .selected()
                .and_then(|i| self.policies.get(i))
        } else {
            None
        }
    }
}

impl ServiceInputHandler for IamState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::IamViewMode;
        let len = match self.view_mode {
            IamViewMode::Users => self.users.len(),
            IamViewMode::Roles => self.roles.len(),
            IamViewMode::Policies => self.policies.len(),
            IamViewMode::UserAttachedPolicies | IamViewMode::RoleAttachedPolicies => {
                self.current_policies.len()
            }
            IamViewMode::PolicyDocument => 0,
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter => {
                return match self.view_mode {
                    IamViewMode::Users => InputResult::Message(Message::iam_drill_down_user()),
                    IamViewMode::Roles => InputResult::Message(Message::iam_drill_down_role()),
                    IamViewMode::Policies
                    | IamViewMode::UserAttachedPolicies
                    | IamViewMode::RoleAttachedPolicies => {
                        InputResult::Message(Message::iam_drill_down_policy())
                    }
                    IamViewMode::PolicyDocument => InputResult::None,
                };
            }
            KeyCode::Esc if !self.view_mode.is_main_tab() => {
                return InputResult::Message(Message::iam_exit_drill_down())
            }
            KeyCode::Char('X') | KeyCode::Delete if self.view_mode.is_main_tab() => {
                match self.view_mode {
                    IamViewMode::Users => {
                        if let Some(user) = self.selected_user() {
                            return InputResult::Action(Message::iam_delete_user(
                                user.user_name.clone(),
                            ));
                        }
                    }
                    IamViewMode::Roles => {
                        if let Some(role) = self.selected_role() {
                            return InputResult::Action(Message::iam_delete_role(
                                role.role_name.clone(),
                            ));
                        }
                    }
                    IamViewMode::Policies => {
                        if let Some(policy) = self.selected_policy() {
                            if let Some(arn) = &policy.arn {
                                return InputResult::Action(Message::iam_delete_policy(
                                    arn.clone(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        InputResult::None
    }
}

// ============================================================================
// Backup State
// ============================================================================

/// State for AWS Backup service
#[derive(Default)]
pub struct BackupState {
    pub vaults: Vec<BackupVault>,
    pub plans: Vec<BackupPlan>,
    pub jobs: Vec<BackupJob>,
    pub recovery_points: Vec<RecoveryPoint>,
    pub list_state: TableState,
    pub view_mode: BackupViewMode,
}

impl BackupState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected vault, if any
    pub fn selected_vault(&self) -> Option<&BackupVault> {
        if self.view_mode == BackupViewMode::Vaults {
            self.list_state.selected().and_then(|i| self.vaults.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected plan, if any
    pub fn selected_plan(&self) -> Option<&BackupPlan> {
        if self.view_mode == BackupViewMode::Plans {
            self.list_state.selected().and_then(|i| self.plans.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected job, if any
    pub fn selected_job(&self) -> Option<&BackupJob> {
        if self.view_mode == BackupViewMode::Jobs {
            self.list_state.selected().and_then(|i| self.jobs.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected recovery point, if any
    pub fn selected_recovery_point(&self) -> Option<&RecoveryPoint> {
        if self.view_mode == BackupViewMode::RecoveryPoints {
            self.list_state.selected().and_then(|i| self.recovery_points.get(i))
        } else {
            None
        }
    }
}

impl ServiceInputHandler for BackupState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::BackupViewMode;
        let len = match self.view_mode {
            BackupViewMode::Vaults => self.vaults.len(),
            BackupViewMode::Plans => self.plans.len(),
            BackupViewMode::Jobs => self.jobs.len(),
            BackupViewMode::RecoveryPoints => self.recovery_points.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter => {
                if self.view_mode == BackupViewMode::Vaults {
                    if let Some(vault) = self.selected_vault() {
                        return InputResult::Message(Message::backup_load_recovery_points(
                            vault.backup_vault_name.clone(),
                        ));
                    }
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                if self.view_mode == BackupViewMode::RecoveryPoints {
                    // Go back to vaults
                    return InputResult::Message(Message::backup_leave_vault());
                }
            }
            _ => {}
        }
        InputResult::None
    }
}

// ============================================================================
// CloudTrail State
// ============================================================================

/// State for CloudTrail service
#[derive(Default)]
pub struct CloudTrailState {
    pub trails: Vec<Trail>,
    pub events: Vec<CloudTrailEvent>,
    pub list_state: TableState,
    pub view_mode: CloudTrailViewMode,
    pub selected_event_detail: Option<String>,
    pub show_detail_modal: bool,
}

impl CloudTrailState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected trail, if any
    pub fn selected_trail(&self) -> Option<&Trail> {
        if self.view_mode == CloudTrailViewMode::Trails {
            self.list_state.selected().and_then(|i| self.trails.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected event, if any
    pub fn selected_event(&self) -> Option<&CloudTrailEvent> {
        if self.view_mode == CloudTrailViewMode::Events {
            self.list_state.selected().and_then(|i| self.events.get(i))
        } else {
            None
        }
    }
}

impl ServiceInputHandler for CloudTrailState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        if self.show_detail_modal {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                return InputResult::Message(Message::cloudtrail_close_event_details());
            }
            return InputResult::None;
        }

        use crate::app::CloudTrailViewMode;
        let len = match self.view_mode {
            CloudTrailViewMode::Trails => self.trails.len(),
            CloudTrailViewMode::Events => self.events.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter | KeyCode::Char('o') if self.view_mode == CloudTrailViewMode::Events => {
                if let Some(event) = self.selected_event() {
                    let json = serde_json::to_string_pretty(event).unwrap_or_default();
                    return InputResult::Message(Message::cloudtrail_show_event_details(json));
                }
            }
            KeyCode::Char('X') | KeyCode::Delete
                if self.view_mode == CloudTrailViewMode::Trails =>
            {
                if let Some(trail) = self.selected_trail() {
                    return InputResult::Action(Message::cloudtrail_delete_trail(
                        trail.name.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }
}

// ============================================================================
// Secrets Manager State
// ============================================================================

/// State for Secrets Manager service
#[derive(Default)]
pub struct SecretsManagerState {
    pub secrets: Vec<Secret>,
    pub list_state: TableState,
    pub secret_value: Option<String>,
    pub show_secret_modal: bool,
}

impl SecretsManagerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected secret, if any
    pub fn selected_secret(&self) -> Option<&Secret> {
        self.list_state.selected().and_then(|i| self.secrets.get(i))
    }
}

impl ServiceInputHandler for SecretsManagerState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        if self.show_secret_modal {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                return InputResult::Message(Message::secretsmanager_close_value());
            }
            return InputResult::None;
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(self.secrets.len()),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(self.secrets.len()),
            KeyCode::Char('s') | KeyCode::Enter => {
                if let Some(secret) = self.selected_secret() {
                    if let Some(arn) = &secret.arn {
                        return InputResult::Message(Message::secretsmanager_get_value(
                            arn.clone(),
                        ));
                    }
                }
            }
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(secret) = self.selected_secret() {
                    if let Some(arn) = &secret.arn {
                        return InputResult::Action(Message::secretsmanager_delete_secret(
                            arn.clone(),
                        ));
                    }
                }
            }
            _ => {}
        }

        InputResult::None
    }
}

// ============================================================================
// Service States Container
// ============================================================================

/// Container for all service-specific states
///
/// This consolidates what was previously 50+ individual fields in the App struct
/// into a single organized structure with clear ownership.
#[derive(Default)]
pub struct ServiceStates {
    pub ec2: Ec2State,
    pub s3: S3State,
    pub rds: RdsState,
    pub dynamodb: DynamoDbState,
    pub lambda: LambdaState,
    pub vpc: VpcState,
    pub iam: IamState,
    pub backup: BackupState,
    pub cloudtrail: CloudTrailState,
    pub secretsmanager: SecretsManagerState,
    pub ecs: EcsState,
}

impl ServiceStates {
    pub fn new() -> Self {
        Self {
            ec2: Ec2State::new(),
            s3: S3State::new(),
            rds: RdsState::new(),
            dynamodb: DynamoDbState::new(),
            lambda: LambdaState::new(),
            vpc: VpcState::new(),
            iam: IamState::new(),
            backup: BackupState::new(),
            cloudtrail: CloudTrailState::new(),
            secretsmanager: SecretsManagerState::new(),
            ecs: EcsState::new(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ec2_state_new() {
        let state = Ec2State::new();
        assert!(state.instances.is_empty());
        assert!(state.list_state.selected().is_none());
    }

    #[test]
    fn test_ec2_state_selected_instance() {
        use crate::models::ec2::InstanceState;

        let mut state = Ec2State::new();
        assert!(state.selected_instance().is_none());

        // Add an instance and select it
        state.instances.push(Ec2Instance {
            instance_id: "i-1234567890abcdef0".to_string(),
            name: Some("test-instance".to_string()),
            instance_type: "t2.micro".to_string(),
            state: InstanceState::Running,
            public_ip: None,
            private_ip: Some("10.0.0.1".to_string()),
            launch_time: None,
            vpc_id: None,
            subnet_id: None,
            availability_zone: None,
            architecture: None,
            platform: None,
            ami_id: None,
            key_name: None,
            monitoring_state: None,
            security_groups: vec![],
            tags: vec![],
        });
        state.list_state.select(Some(0));

        assert!(state.selected_instance().is_some());
        assert_eq!(
            state.selected_instance_id(),
            Some("i-1234567890abcdef0".to_string())
        );
    }

    #[test]
    fn test_s3_state_viewing_objects() {
        let mut state = S3State::new();
        assert!(!state.is_viewing_objects());

        state.current_bucket = Some("my-bucket".to_string());
        assert!(state.is_viewing_objects());
    }

    #[test]
    fn test_vpc_state_new_defaults() {
        let state = VpcState::new();
        assert!(state.sg_rules_inbound); // Should default to true
        assert_eq!(state.view_mode, VpcViewMode::Vpcs);
    }

    #[test]
    fn test_service_states_new() {
        let states = ServiceStates::new();
        assert!(states.ec2.instances.is_empty());
        assert!(states.s3.buckets.is_empty());
        assert!(states.rds.instances.is_empty());
        assert!(states.dynamodb.tables.is_empty());
        assert!(states.lambda.functions.is_empty());
        assert!(states.vpc.vpcs.is_empty());
        assert!(states.iam.users.is_empty());
        assert!(states.backup.vaults.is_empty());
        assert!(states.cloudtrail.trails.is_empty());
    }

    #[test]
    fn test_dynamodb_state_viewing_items() {
        let mut state = DynamoDbState::new();
        assert!(!state.is_viewing_items());

        state.current_table = Some("my-table".to_string());
        assert!(state.is_viewing_items());
    }
}

// ============================================================================
// ECS State
// ============================================================================

use super::ecs_modals::{ServiceEditorState, TaskDefSelectorState};

/// State for ECS service
#[derive(Default)]
pub struct EcsState {
    // Data
    pub clusters: Vec<EcsCluster>,
    pub services: Vec<EcsService>,
    pub tasks: Vec<EcsTask>,
    pub current_task_definition: Option<EcsTaskDefinition>,

    // Navigation state
    pub list_state: TableState,
    pub view_mode: EcsViewMode,

    // Drill-down tracking
    pub selected_cluster_arn: Option<String>,
    pub selected_service_arn: Option<String>,
    pub selected_service_name: Option<String>,

    // Detail panel scroll
    pub detail_scroll_offset: usize,

    // Task definition selector modal (legacy - to be removed)
    pub show_task_definition_selector: bool,
    pub task_definitions_list: Vec<String>,
    pub task_definitions_list_state: TableState,

    // Pending edit operation (family, path) - for synchronous editor handling
    pub pending_edit: Option<(String, String)>,

    // Modal states (refactored)
    pub service_editor: ServiceEditorState,
    pub task_def_selector: TaskDefSelectorState,
}

impl EcsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected cluster (only valid in Clusters view)
    pub fn selected_cluster(&self) -> Option<&EcsCluster> {
        if self.view_mode != EcsViewMode::Clusters {
            return None;
        }
        self.list_state
            .selected()
            .and_then(|i| self.clusters.get(i))
    }

    /// Get the currently selected service (only valid in Services view)
    pub fn selected_service(&self) -> Option<&EcsService> {
        if self.view_mode != EcsViewMode::Services {
            return None;
        }
        self.list_state
            .selected()
            .and_then(|i| self.services.get(i))
    }

    /// Get the currently selected task (only valid in Tasks view)
    pub fn selected_task(&self) -> Option<&EcsTask> {
        if self.view_mode != EcsViewMode::Tasks {
            return None;
        }
        self.list_state.selected().and_then(|i| self.tasks.get(i))
    }

    /// Get count for current view
    fn current_list_len(&self) -> usize {
        match self.view_mode {
            EcsViewMode::Clusters => self.clusters.len(),
            EcsViewMode::Services => self.services.len(),
            EcsViewMode::Tasks => self.tasks.len(),
            EcsViewMode::TaskDefinition => 0,
        }
    }

    /// Clear tasks and services when navigating back
    pub fn clear_services(&mut self) {
        self.services.clear();
        self.selected_service_arn = None;
        self.selected_service_name = None;
    }

    pub fn clear_tasks(&mut self) {
        self.tasks.clear();
        self.current_task_definition = None;
    }

    /// Prepare the service editor modal with current service values
    pub fn prepare_service_editor(&mut self) {
        // Extract values from selected service first to avoid borrow conflict
        let (service_name, task_def) = {
            if let Some(service) = self.selected_service() {
                (
                    service.service_name.clone(),
                    service.task_definition.clone().unwrap_or_default(),
                )
            } else {
                return;
            }
        };
        
        self.service_editor.init(service_name, task_def);
    }

    /// Reset the service editor state
    pub fn reset_service_editor(&mut self) {
        self.service_editor.reset();
    }

    /// Prepare the task definition selector modal
    pub fn prepare_task_def_selector(&mut self) {
        // Extract service name to avoid borrow conflict
        let service_name = {
            if let Some(service) = self.selected_service() {
                service.service_name.clone()
            } else {
                return;
            }
        };
        
        self.task_def_selector.init(service_name);
    }

    /// Reset the task definition selector state
    pub fn reset_task_def_selector(&mut self) {
        self.task_def_selector.reset();
    }

    /// Get the currently selected task definition in the selector
    pub fn selected_task_def_in_selector(&self) -> Option<&EcsTaskDefinition> {
        self.task_def_selector.selected()
    }

    /// Navigate up in task definition selector
    pub fn task_def_selector_up(&mut self) {
        self.task_def_selector.nav_up();
    }

    /// Navigate down in task definition selector
    pub fn task_def_selector_down(&mut self) {
        self.task_def_selector.nav_down();
    }
}

impl ServiceInputHandler for EcsState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match self.view_mode {
            EcsViewMode::Clusters => self.handle_clusters_input(key),
            EcsViewMode::Services => self.handle_services_input(key),
            EcsViewMode::Tasks => self.handle_tasks_input(key),
            EcsViewMode::TaskDefinition => self.handle_task_definition_input(key),
        }
    }
}

impl EcsState {
    fn handle_clusters_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.current_list_len());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.current_list_len());
            }
            KeyCode::Enter => {
                if let Some(cluster) = self.selected_cluster() {
                    return InputResult::Message(Message::ecs_view_services(
                        cluster.cluster_arn.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_services_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.current_list_len());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.current_list_len());
            }
            KeyCode::Enter => {
                // Drill into tasks for this service
                if let Some(service) = self.selected_service() {
                    return InputResult::Message(Message::ecs_view_tasks(
                        service.service_arn.clone(),
                    ));
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                return InputResult::Message(Message::ecs_back_to_clusters());
            }
            // 't' - View task definition for this service
            KeyCode::Char('t') => {
                if let Some(service) = self.selected_service() {
                    if let Some(td) = &service.task_definition {
                        return InputResult::Message(Message::ecs_view_task_definition(td.clone()));
                    }
                }
            }
            // 'd' - Force new deployment
            KeyCode::Char('d') => {
                if let (Some(cluster_arn), Some(service)) =
                    (&self.selected_cluster_arn, self.selected_service())
                {
                    return InputResult::Action(Message::ecs_force_new_deployment(
                        cluster_arn.clone(),
                        service.service_name.clone(),
                    ));
                }
            }
            // '+' - Scale up
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if let (Some(cluster_arn), Some(service)) =
                    (&self.selected_cluster_arn, self.selected_service())
                {
                    let new_count = service.desired_count + 1;
                    return InputResult::Action(Message::ecs_update_desired_count(
                        cluster_arn.clone(),
                        service.service_name.clone(),
                        new_count,
                    ));
                }
            }
            // '-' - Scale down
            KeyCode::Char('-') => {
                if let (Some(cluster_arn), Some(service)) =
                    (&self.selected_cluster_arn, self.selected_service())
                {
                    let new_count = (service.desired_count - 1).max(0);
                    return InputResult::Action(Message::ecs_update_desired_count(
                        cluster_arn.clone(),
                        service.service_name.clone(),
                        new_count,
                    ));
                }
            }
            // 'e' - Edit service (modify task definition, CPU, memory)
            KeyCode::Char('e') => {
                if self.selected_service().is_some() {
                    self.prepare_service_editor();
                    return InputResult::OpenInputMode(crate::app::InputMode::EcsServiceEditor);
                }
            }
            // 'T' - Select task definition (browse task definitions with details)
            KeyCode::Char('T') => {
                if self.selected_service().is_some() {
                    self.prepare_task_def_selector();
                    return InputResult::OpenInputMode(crate::app::InputMode::EcsTaskDefSelector);
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_tasks_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.current_list_len());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.current_list_len());
            }
            KeyCode::Esc | KeyCode::Backspace => {
                return InputResult::Message(Message::ecs_back_to_services());
            }
            // 't' - View task definition for this task
            KeyCode::Char('t') | KeyCode::Enter => {
                if let Some(task) = self.selected_task() {
                    return InputResult::Message(Message::ecs_view_task_definition(
                        task.task_definition_arn.clone(),
                    ));
                }
            }
            // 'S' - Stop task
            KeyCode::Char('S') => {
                if let (Some(cluster_arn), Some(task)) =
                    (&self.selected_cluster_arn, self.selected_task())
                {
                    return InputResult::Action(Message::ecs_stop_task(
                        cluster_arn.clone(),
                        task.task_arn.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_task_definition_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => {
                return InputResult::Message(Message::ecs_back_to_tasks());
            }
            // Scroll in detail view
            KeyCode::Down | KeyCode::Char('j') => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_add(10);
            }
            KeyCode::PageUp => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.detail_scroll_offset = 0;
            }
            // 'E' - Edit task definition
            KeyCode::Char('E') => {
                if let Some(td) = &self.current_task_definition {
                    return InputResult::Action(Message::ecs_edit_task_definition(
                        td.task_definition_arn.clone(),
                    ));
                }
            }
            // 'X' - Deregister task definition
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(td) = &self.current_task_definition {
                    return InputResult::Action(Message::ecs_deregister_task_definition(
                        td.task_definition_arn.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }
}
