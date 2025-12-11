//! Service-specific state structs
//!
//! This module consolidates service-specific state that was previously scattered
//! across 50+ fields in the App struct. Each service has its own state struct
//! containing all data, list states, and view modes specific to that service.

#![allow(dead_code)]

use ratatui::widgets::TableState;
use std::collections::HashMap;

use crate::models::backup::{BackupJob, BackupPlan, BackupVault};
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::models::dynamodb::{DynamoDbItem, DynamoDbTable};
use crate::models::ec2::Ec2Instance;
use crate::models::ecs::{EcsCluster, EcsService};
use crate::models::iam::{IamPolicy, IamRole, IamUser};
use crate::models::lambda::LambdaFunction;
use crate::models::rds::RdsInstance;
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crate::models::secretsmanager::Secret;
use crate::models::vpc::{SecurityGroup, SecurityGroupRule, Subnet, Vpc};

use super::{
    BackupViewMode, CloudTrailViewMode, DynamoDbViewMode, EcsViewMode, IamViewMode, VpcViewMode,
};
use super::{InputResult, Message, ServiceInputHandler};
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
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.instances.is_empty() {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i >= self.instances.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.instances.is_empty() {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i == 0 {
                            self.instances.len() - 1
                        } else {
                            i - 1
                        }
                    });
                    self.list_state.select(Some(i));
                }
            }
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
                    if !self.objects.is_empty() {
                        let i = self.object_list_state.selected().map_or(0, |i| {
                            if i >= self.objects.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        });
                        self.object_list_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.objects.is_empty() {
                        let i = self.object_list_state.selected().map_or(0, |i| {
                            if i == 0 {
                                self.objects.len() - 1
                            } else {
                                i - 1
                            }
                        });
                        self.object_list_state.select(Some(i));
                    }
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
                    if !self.buckets.is_empty() {
                        let i = self.list_state.selected().map_or(0, |i| {
                            if i >= self.buckets.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        });
                        self.list_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.buckets.is_empty() {
                        let i = self.list_state.selected().map_or(0, |i| {
                            if i == 0 {
                                self.buckets.len() - 1
                            } else {
                                i - 1
                            }
                        });
                        self.list_state.select(Some(i));
                    }
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
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.instances.is_empty() {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i >= self.instances.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.instances.is_empty() {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i == 0 {
                            self.instances.len() - 1
                        } else {
                            i - 1
                        }
                    });
                    self.list_state.select(Some(i));
                }
            }
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
            let len = self.items.len();
            match key.code {
                KeyCode::Esc | KeyCode::Backspace => {
                    return InputResult::Message(Message::dynamodb_exit_drill_down())
                }
                KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                    let i = self.item_list_state.selected().map_or(0, |i| {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.item_list_state.select(Some(i));
                }
                KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                    let i = self.item_list_state.selected().map_or(0, |i| {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.item_list_state.select(Some(i));
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
                    if !self.tables.is_empty() {
                        let i = self.list_state.selected().map_or(0, |i| {
                            if i >= self.tables.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        });
                        self.list_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.tables.is_empty() {
                        let i = self.list_state.selected().map_or(0, |i| {
                            if i == 0 {
                                self.tables.len() - 1
                            } else {
                                i - 1
                            }
                        });
                        self.list_state.select(Some(i));
                    }
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
                if !self.functions.is_empty() {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i >= self.functions.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.list_state.select(Some(i));

                    if let Some(f) = self.selected_function() {
                        return InputResult::Message(crate::app::Message::lambda_load_details(
                            f.function_name.clone(),
                        ));
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.functions.is_empty() {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i == 0 {
                            self.functions.len() - 1
                        } else {
                            i - 1
                        }
                    });
                    self.list_state.select(Some(i));

                    if let Some(f) = self.selected_function() {
                        return InputResult::Message(crate::app::Message::lambda_load_details(
                            f.function_name.clone(),
                        ));
                    }
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
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.list_state.select(Some(i));
            }
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
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.list_state.select(Some(i));
            }
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
}

impl ServiceInputHandler for BackupState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::BackupViewMode;
        let len = match self.view_mode {
            BackupViewMode::Vaults => self.vaults.len(),
            BackupViewMode::Plans => self.plans.len(),
            BackupViewMode::Jobs => self.jobs.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.list_state.select(Some(i));
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
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.list_state.select(Some(i));
            }
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
                return InputResult::Message(crate::app::Message::Service(
                    crate::app::ServiceAction::SecretsManager(
                        crate::app::messages::SecretsManagerAction::CloseSecretValue,
                    ),
                ));
            }
            return InputResult::None;
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.secrets.is_empty() {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i >= self.secrets.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.secrets.is_empty() {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i == 0 {
                            self.secrets.len() - 1
                        } else {
                            i - 1
                        }
                    });
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Char('s') | KeyCode::Enter => {
                if let Some(secret) = self.selected_secret() {
                    if let Some(arn) = &secret.arn {
                        return InputResult::Message(crate::app::Message::Service(
                            crate::app::ServiceAction::SecretsManager(
                                crate::app::messages::SecretsManagerAction::GetSecretValue(
                                    arn.clone(),
                                ),
                            ),
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

/// State for ECS service
#[derive(Default)]
pub struct EcsState {
    pub clusters: Vec<EcsCluster>,
    pub services: Vec<EcsService>,
    pub list_state: TableState,
    pub view_mode: EcsViewMode,
    pub selected_cluster_arn: Option<String>,
}

impl EcsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn selected_cluster(&self) -> Option<&EcsCluster> {
        if self.view_mode != EcsViewMode::Clusters {
            return None;
        }
        self.list_state
            .selected()
            .and_then(|i| self.clusters.get(i))
    }

    pub fn selected_service(&self) -> Option<&EcsService> {
        if self.view_mode != EcsViewMode::Services {
            return None;
        }
        self.list_state
            .selected()
            .and_then(|i| self.services.get(i))
    }
}

impl ServiceInputHandler for EcsState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                let items_len = match self.view_mode {
                    EcsViewMode::Clusters => self.clusters.len(),
                    EcsViewMode::Services => self.services.len(),
                };
                if items_len > 0 {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i >= items_len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let items_len = match self.view_mode {
                    EcsViewMode::Clusters => self.clusters.len(),
                    EcsViewMode::Services => self.services.len(),
                };
                if items_len > 0 {
                    let i = self.list_state.selected().map_or(0, |i| {
                        if i == 0 {
                            items_len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                if let EcsViewMode::Clusters = self.view_mode {
                    if let Some(cluster) = self.selected_cluster() {
                        return InputResult::Message(Message::ecs_view_services(
                            cluster.cluster_arn.clone(),
                        ));
                    }
                }
            }
            KeyCode::Esc => {
                if let EcsViewMode::Services = self.view_mode {
                    return InputResult::Message(Message::ecs_back_to_clusters());
                }
            }
            _ => {}
        }
        InputResult::None
    }
}
