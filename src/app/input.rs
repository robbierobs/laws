//! Keyboard input handling
//!
//! Handles all keyboard events and translates them to messages.

use super::{
    App, Focus, GlobalMessage, InputMode, InputResult, Message, Service, ServiceInputHandler,
    VpcViewMode,
};
use crate::ui::components::Component;
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    /// Reset list selection to first item for current service
    pub fn reset_selection(&mut self) {
        match self.current_service {
            Service::EC2 => self.services.ec2.list_state.select(Some(0)),
            Service::S3 => {
                if self.services.s3.current_bucket.is_some() {
                    self.services.s3.object_list_state.select(Some(0));
                } else {
                    self.services.s3.list_state.select(Some(0));
                }
            }
            Service::RDS => self.services.rds.list_state.select(Some(0)),
            Service::DynamoDB => self.services.dynamodb.list_state.select(Some(0)),
            Service::Lambda => self.services.lambda.list_state.select(Some(0)),
            Service::VPC => self.services.vpc.list_state.select(Some(0)),
            Service::IAM => self.services.iam.list_state.select(Some(0)),
            Service::Backup => self.services.backup.list_state.select(Some(0)),
            Service::CloudTrail => self.services.cloudtrail.list_state.select(Some(0)),
            Service::SecretsManager => self.services.secretsmanager.list_state.select(Some(0)),
            Service::ECS => self.services.ecs.list_state.select(Some(0)),
        }
    }

    /// Request an action (with confirmation check)
    fn request_action(&mut self, action: Message) -> Option<Message> {
        if self.read_only {
            self.error_message = Some("Read-only mode: Action not allowed".to_string());
            None
        } else {
            self.pending_action = Some(action);
            self.show_confirmation = true;
            None
        }
    }

    /// Main keyboard event handler
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Message> {
        self.error_message = None;

        // Handle S3 object viewer popup
        if self.services.s3.show_object_viewer {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.services.s3.show_object_viewer = false;
                    self.services.s3.opened_object_content = None;
                    self.services.s3.opened_object_path = None;
                    self.services.s3.opened_object_key = None;
                    self.services.s3.viewer_scroll_offset = 0;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.services.s3.viewer_scroll_offset =
                        self.services.s3.viewer_scroll_offset.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.services.s3.viewer_scroll_offset =
                        self.services.s3.viewer_scroll_offset.saturating_sub(1);
                }
                KeyCode::Char('g') | KeyCode::Home => {
                    self.services.s3.viewer_scroll_offset = 0;
                }
                KeyCode::Char('G') | KeyCode::End => {
                    // Scroll to end - approximate based on content length
                    if let Some(content) = &self.services.s3.opened_object_content {
                        let line_count = content.lines().count() as u16;
                        self.services.s3.viewer_scroll_offset = line_count.saturating_sub(10);
                    }
                }
                KeyCode::PageDown => {
                    self.services.s3.viewer_scroll_offset =
                        self.services.s3.viewer_scroll_offset.saturating_add(20);
                }
                KeyCode::PageUp => {
                    self.services.s3.viewer_scroll_offset =
                        self.services.s3.viewer_scroll_offset.saturating_sub(20);
                }
                _ => {}
            }
            return None;
        }

        // Handle confirmation modal
        if self.show_confirmation {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => Some(Message::confirm_action()),
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    Some(Message::cancel_action())
                }
                _ => None,
            };
        }

        // Handle profile switcher - profile selection
        if self.input_mode == InputMode::ProfileSwitcherProfile {
            // If filter is active, handle text input
            if self.profile_filter_active {
                match key.code {
                    KeyCode::Esc => {
                        self.profile_filter_active = false;
                        self.profile_filter.clear();
                        self.profile_switcher_index = 0;
                    }
                    KeyCode::Enter => {
                        self.profile_filter_active = false;
                    }
                    KeyCode::Backspace => {
                        self.profile_filter.pop();
                        self.profile_switcher_index = 0;
                    }
                    KeyCode::Char(c) => {
                        self.profile_filter.push(c);
                        self.profile_switcher_index = 0;
                    }
                    _ => {}
                }
                return None;
            }

            // Normal navigation mode
            match key.code {
                KeyCode::Esc => {
                    self.profile_filter.clear();
                    return Some(Message::cancel_profile_switcher());
                }
                KeyCode::Char('/') => {
                    self.profile_filter_active = true;
                }
                KeyCode::Char('R') => {
                    // Toggle read-only mode
                    self.pending_read_only = !self.pending_read_only;
                }
                KeyCode::Enter => {
                    // Store selected profile and move to region selection
                    let filtered = self.filtered_profiles();
                    let selected = filtered
                        .get(self.profile_switcher_index)
                        .map(|s| {
                            if *s == "default" {
                                None
                            } else {
                                Some((*s).clone())
                            }
                        })
                        .unwrap_or(None);
                    self.pending_profile = selected;
                    self.profile_filter.clear();
                    self.input_mode = InputMode::ProfileSwitcherRegion;
                    // Pre-select current region in region list
                    if let Some(idx) = self
                        .filtered_regions()
                        .iter()
                        .position(|r| *r == &self.region)
                    {
                        self.region_switcher_index = idx;
                    } else {
                        self.region_switcher_index = 0;
                    }
                    return None;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let filtered_len = self.filtered_profiles().len();
                    if filtered_len > 0 {
                        self.profile_switcher_index =
                            (self.profile_switcher_index + 1) % filtered_len;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let filtered_len = self.filtered_profiles().len();
                    if filtered_len > 0 {
                        self.profile_switcher_index = if self.profile_switcher_index == 0 {
                            filtered_len - 1
                        } else {
                            self.profile_switcher_index - 1
                        };
                    }
                }
                _ => {}
            }
            return None;
        }

        // Handle profile switcher - region selection
        if self.input_mode == InputMode::ProfileSwitcherRegion {
            // If filter is active, handle text input
            if self.region_filter_active {
                match key.code {
                    KeyCode::Esc => {
                        self.region_filter_active = false;
                        self.region_filter.clear();
                        self.region_switcher_index = 0;
                    }
                    KeyCode::Enter => {
                        self.region_filter_active = false;
                    }
                    KeyCode::Backspace => {
                        self.region_filter.pop();
                        self.region_switcher_index = 0;
                    }
                    KeyCode::Char(c) => {
                        self.region_filter.push(c);
                        self.region_switcher_index = 0;
                    }
                    _ => {}
                }
                return None;
            }

            // Normal navigation mode
            match key.code {
                KeyCode::Esc => {
                    // Cancel and go back to normal mode
                    self.pending_profile = None;
                    self.region_filter.clear();
                    return Some(Message::cancel_profile_switcher());
                }
                KeyCode::Char('/') => {
                    self.region_filter_active = true;
                }
                KeyCode::Enter => {
                    // Confirm and switch profile/region
                    let profile = self.pending_profile.clone();
                    let filtered = self.filtered_regions();
                    let region = filtered
                        .get(self.region_switcher_index)
                        .cloned()
                        .cloned()
                        .unwrap_or_else(|| "us-east-1".to_string());
                    let read_only = self.pending_read_only;
                    self.pending_profile = None;
                    self.region_filter.clear();
                    return Some(Message::switch_profile_region(profile, region, read_only));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let filtered_len = self.filtered_regions().len();
                    if filtered_len > 0 {
                        self.region_switcher_index =
                            (self.region_switcher_index + 1) % filtered_len;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let filtered_len = self.filtered_regions().len();
                    if filtered_len > 0 {
                        self.region_switcher_index = if self.region_switcher_index == 0 {
                            filtered_len - 1
                        } else {
                            self.region_switcher_index - 1
                        };
                    }
                }
                _ => {}
            }
            return None;
        }

        // Handle filter input mode
        if self.input_mode == InputMode::Filtering {
            match key.code {
                KeyCode::Enter => self.input_mode = InputMode::Normal,
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.filter_input.clear();
                    self.reset_selection();
                }
                KeyCode::Backspace => {
                    self.filter_input.pop();
                    self.reset_selection();
                }
                KeyCode::Char(c) => {
                    self.filter_input.push(c);
                    self.reset_selection();
                }
                _ => {}
            }
            return None;
        }

        if key.code == KeyCode::Tab {
            self.toggle_focus();
            return None;
        }

        // Route to focused component
        match self.focus {
            Focus::Sidebar => {
                if let Some(msg) = self.sidebar.handle_key(key) {
                    return Some(msg);
                }
            }
            Focus::Main => {
                // Filter mode
                if key.code == KeyCode::Char('/') {
                    self.input_mode = InputMode::Filtering;
                    self.filter_input.clear();
                    self.reset_selection();
                    return None;
                }

                // View mode cycling
                if key.code == KeyCode::Char('v') {
                    match self.current_service {
                        Service::Backup | Service::CloudTrail => {
                            return Some(Message::cycle_view_mode())
                        }
                        Service::VPC
                            if self.services.vpc.view_mode != VpcViewMode::SecurityGroupRules =>
                        {
                            return Some(Message::cycle_view_mode())
                        }
                        Service::IAM if self.services.iam.view_mode.is_main_tab() => {
                            return Some(Message::cycle_view_mode())
                        }
                        _ => {}
                    }
                }

                // Action log toggle
                if key.code == KeyCode::Char('A') {
                    return Some(Message::toggle_action_log());
                }

                // Arrow navigation for view modes
                match key.code {
                    KeyCode::Right | KeyCode::Char('l') => match self.current_service {
                        Service::Backup | Service::CloudTrail => return Some(Message::next_view()),
                        Service::VPC
                            if self.services.vpc.view_mode != VpcViewMode::SecurityGroupRules =>
                        {
                            return Some(Message::next_view())
                        }
                        Service::IAM if self.services.iam.view_mode.is_main_tab() => {
                            return Some(Message::next_view())
                        }
                        _ => {}
                    },
                    KeyCode::Left | KeyCode::Char('h') => match self.current_service {
                        Service::Backup | Service::CloudTrail => {
                            return Some(Message::previous_view())
                        }
                        Service::VPC
                            if self.services.vpc.view_mode != VpcViewMode::SecurityGroupRules =>
                        {
                            return Some(Message::previous_view())
                        }
                        Service::IAM if self.services.iam.view_mode.is_main_tab() => {
                            return Some(Message::previous_view())
                        }
                        _ => {}
                    },
                    _ => {}
                }

                // Service-specific input handling
                let result = match self.current_service {
                    Service::EC2 => self.services.ec2.handle_input(key),
                    Service::S3 => self.services.s3.handle_input(key),
                    Service::RDS => self.services.rds.handle_input(key),
                    Service::DynamoDB => self.services.dynamodb.handle_input(key),
                    Service::Lambda => self.services.lambda.handle_input(key),
                    Service::VPC => self.services.vpc.handle_input(key),
                    Service::IAM => self.services.iam.handle_input(key),
                    Service::Backup => self.services.backup.handle_input(key),
                    Service::CloudTrail => self.services.cloudtrail.handle_input(key),
                    Service::SecretsManager => self.services.secretsmanager.handle_input(key),
                    Service::ECS => self.services.ecs.handle_input(key),
                };

                match result {
                    InputResult::Message(msg) => return Some(msg),
                    InputResult::Action(action) => return self.request_action(action),
                    InputResult::None => {}
                }
            }
        }

        // Global keys
        match key.code {
            KeyCode::Char('r') => Some(Message::refresh()),
            KeyCode::Char('y') => self.handle_copy(),
            KeyCode::Char('q') => Some(Message::quit()),
            KeyCode::Char('P') => Some(Message::open_profile_switcher()),
            KeyCode::Char('1') => Some(Message::navigate(Service::EC2)),
            KeyCode::Char('2') => Some(Message::navigate(Service::S3)),
            KeyCode::Char('3') => Some(Message::navigate(Service::RDS)),
            KeyCode::Char('4') => Some(Message::navigate(Service::DynamoDB)),
            KeyCode::Char('5') => Some(Message::navigate(Service::Lambda)),
            KeyCode::Char('6') => Some(Message::navigate(Service::VPC)),
            KeyCode::Char('7') => Some(Message::navigate(Service::IAM)),
            KeyCode::Char('8') => Some(Message::navigate(Service::Backup)),
            KeyCode::Char('9') => Some(Message::navigate(Service::CloudTrail)),
            KeyCode::Char('0') => Some(Message::navigate(Service::SecretsManager)),
            KeyCode::Char('d') => Some(Message::toggle_detail_panel()),
            KeyCode::Char('D') => Some(Message::Global(GlobalMessage::ToggleDetailFullscreen)),
            KeyCode::PageUp => Some(Message::Global(GlobalMessage::DetailScrollUp)),
            KeyCode::PageDown => Some(Message::Global(GlobalMessage::DetailScrollDown)),
            _ => None,
        }
    }

    fn handle_copy(&self) -> Option<Message> {
        let text = match self.current_service {
            Service::EC2 => self.services.ec2.selected_instance_id(),
            Service::S3 => {
                if self.services.s3.current_bucket.is_some() {
                    self.services.s3.selected_object().map(|o| o.key.clone())
                } else {
                    self.services.s3.selected_bucket().map(|b| b.name.clone())
                }
            }
            Service::RDS => self.services.rds.selected_instance_id(),
            Service::DynamoDB => {
                // For tables, return table name
                // For items, we could format as JSON, but for now let's stick to IDs/names if possible
                // Detailed item copy is better handled in a specific view
                self.services
                    .dynamodb
                    .selected_table()
                    .map(|t| t.table_name.clone())
            }
            Service::Lambda => self
                .services
                .lambda
                .selected_function()
                .map(|f| f.function_name.clone()),
            Service::VPC => {
                use crate::app::VpcViewMode;
                match self.services.vpc.view_mode {
                    VpcViewMode::Vpcs => self.services.vpc.selected_vpc().map(|v| v.vpc_id.clone()),
                    VpcViewMode::Subnets => self
                        .services
                        .vpc
                        .selected_subnet()
                        .map(|s| s.subnet_id.clone()),
                    VpcViewMode::SecurityGroups => self
                        .services
                        .vpc
                        .selected_security_group()
                        .map(|sg| sg.group_id.clone()),
                    VpcViewMode::SecurityGroupRules => None, // Hard to pick a single ID
                }
            }
            Service::IAM => {
                use crate::app::IamViewMode;
                match self.services.iam.view_mode {
                    IamViewMode::Users => self
                        .services
                        .iam
                        .selected_user()
                        .map(|u| u.user_name.clone()),
                    IamViewMode::Roles => self
                        .services
                        .iam
                        .selected_role()
                        .map(|r| r.role_name.clone()),
                    IamViewMode::Policies => self
                        .services
                        .iam
                        .selected_policy()
                        .map(|p| p.policy_name.clone()),
                    _ => None,
                }
            }
            Service::Backup => {
                use crate::app::BackupViewMode;
                match self.services.backup.view_mode {
                    BackupViewMode::Vaults => self
                        .services
                        .backup
                        .selected_vault()
                        .map(|v| v.backup_vault_name.clone()),
                    BackupViewMode::Plans => self
                        .services
                        .backup
                        .selected_plan()
                        .map(|p| p.backup_plan_id.clone()),
                    BackupViewMode::Jobs => self
                        .services
                        .backup
                        .selected_job()
                        .map(|j| j.backup_job_id.clone()),
                }
            }
            Service::CloudTrail => {
                use crate::app::CloudTrailViewMode;
                match self.services.cloudtrail.view_mode {
                    CloudTrailViewMode::Trails => self
                        .services
                        .cloudtrail
                        .selected_trail()
                        .map(|t| t.name.clone()),
                    CloudTrailViewMode::Events => self
                        .services
                        .cloudtrail
                        .selected_event()
                        .and_then(|e| e.event_id.clone()),
                }
            }
            Service::SecretsManager => self
                .services
                .secretsmanager
                .selected_secret()
                .map(|s| s.name.clone()),
            Service::ECS => {
                use crate::app::EcsViewMode;
                match self.services.ecs.view_mode {
                    EcsViewMode::Clusters => self
                        .services
                        .ecs
                        .selected_cluster()
                        .map(|c| c.cluster_arn.clone()),
                    EcsViewMode::Services => self
                        .services
                        .ecs
                        .selected_service()
                        .map(|s| s.service_arn.clone()),
                    EcsViewMode::Tasks => self
                        .services
                        .ecs
                        .selected_task()
                        .map(|t| t.task_arn.clone()),
                    EcsViewMode::TaskDefinition => self
                        .services
                        .ecs
                        .current_task_definition
                        .as_ref()
                        .map(|td| td.task_definition_arn.clone()),
                }
            }
        };

        text.map(Message::copy_to_clipboard)
    }

    /// Toggle focus between sidebar and main pane
    pub fn toggle_focus(&mut self) {
        match self.focus {
            Focus::Sidebar => {
                self.focus = Focus::Main;
                self.sidebar.is_focused = false;
                self.auto_select_first_item();
            }
            Focus::Main => {
                self.focus = Focus::Sidebar;
                self.sidebar.is_focused = true;
            }
        }
    }

    fn auto_select_first_item(&mut self) {
        match self.current_service {
            Service::EC2 => {
                if self.services.ec2.list_state.selected().is_none()
                    && !self.services.ec2.instances.is_empty()
                {
                    self.services.ec2.list_state.select(Some(0));
                }
            }
            Service::S3 => {
                if self.services.s3.current_bucket.is_some() {
                    if self.services.s3.object_list_state.selected().is_none()
                        && !self.services.s3.objects.is_empty()
                    {
                        self.services.s3.object_list_state.select(Some(0));
                    }
                } else if self.services.s3.list_state.selected().is_none()
                    && !self.services.s3.buckets.is_empty()
                {
                    self.services.s3.list_state.select(Some(0));
                }
            }
            Service::RDS => {
                if self.services.rds.list_state.selected().is_none()
                    && !self.services.rds.instances.is_empty()
                {
                    self.services.rds.list_state.select(Some(0));
                }
            }
            Service::DynamoDB => {
                if self.services.dynamodb.list_state.selected().is_none()
                    && !self.services.dynamodb.tables.is_empty()
                {
                    self.services.dynamodb.list_state.select(Some(0));
                }
            }
            Service::Lambda => {
                if self.services.lambda.list_state.selected().is_none()
                    && !self.services.lambda.functions.is_empty()
                {
                    self.services.lambda.list_state.select(Some(0));
                }
            }
            Service::VPC => self.auto_select_vpc(),
            Service::IAM => self.auto_select_iam(),
            Service::Backup => self.auto_select_backup(),
            Service::CloudTrail => self.auto_select_cloudtrail(),
            Service::SecretsManager => {
                if self.services.secretsmanager.list_state.selected().is_none()
                    && !self.services.secretsmanager.secrets.is_empty()
                {
                    self.services.secretsmanager.list_state.select(Some(0));
                }
            }
            Service::ECS => {
                if self.services.ecs.list_state.selected().is_none()
                    && !self.services.ecs.clusters.is_empty()
                {
                    self.services.ecs.list_state.select(Some(0));
                }
            }
        }
    }

    fn auto_select_vpc(&mut self) {
        if self.services.vpc.list_state.selected().is_none() {
            use crate::app::VpcViewMode;
            let has_items = match self.services.vpc.view_mode {
                VpcViewMode::Vpcs => !self.services.vpc.vpcs.is_empty(),
                VpcViewMode::Subnets => !self.services.vpc.subnets.is_empty(),
                VpcViewMode::SecurityGroups => !self.services.vpc.security_groups.is_empty(),
                VpcViewMode::SecurityGroupRules => !self.services.vpc.current_sg_rules.is_empty(),
            };
            if has_items {
                self.services.vpc.list_state.select(Some(0));
            }
        }
    }

    fn auto_select_iam(&mut self) {
        if self.services.iam.list_state.selected().is_none() {
            use crate::app::IamViewMode;
            let has_items = match self.services.iam.view_mode {
                IamViewMode::Users => !self.services.iam.users.is_empty(),
                IamViewMode::Roles => !self.services.iam.roles.is_empty(),
                IamViewMode::Policies => !self.services.iam.policies.is_empty(),
                IamViewMode::UserAttachedPolicies | IamViewMode::RoleAttachedPolicies => {
                    !self.services.iam.current_policies.is_empty()
                }
                IamViewMode::PolicyDocument => false,
            };
            if has_items {
                self.services.iam.list_state.select(Some(0));
            }
        }
    }

    fn auto_select_backup(&mut self) {
        if self.services.backup.list_state.selected().is_none() {
            use crate::app::BackupViewMode;
            let has_items = match self.services.backup.view_mode {
                BackupViewMode::Vaults => !self.services.backup.vaults.is_empty(),
                BackupViewMode::Plans => !self.services.backup.plans.is_empty(),
                BackupViewMode::Jobs => !self.services.backup.jobs.is_empty(),
            };
            if has_items {
                self.services.backup.list_state.select(Some(0));
            }
        }
    }

    fn auto_select_cloudtrail(&mut self) {
        if self.services.cloudtrail.list_state.selected().is_none() {
            use crate::app::CloudTrailViewMode;
            let has_items = match self.services.cloudtrail.view_mode {
                CloudTrailViewMode::Trails => !self.services.cloudtrail.trails.is_empty(),
                CloudTrailViewMode::Events => !self.services.cloudtrail.events.is_empty(),
            };
            if has_items {
                self.services.cloudtrail.list_state.select(Some(0));
            }
        }
    }

    // ====================================
    // Service-specific input handlers
    // ====================================
}
