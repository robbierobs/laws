//! Keyboard input handling
//! 
//! Handles all keyboard events and translates them to messages.

use crossterm::event::{KeyCode, KeyEvent};
use super::{App, Message, Service, Focus, InputMode, VpcViewMode};
use crate::ui::components::Component;

/// Result from service input handlers
enum InputResult {
    None,
    Message(Message),
    Action(Message), // Action that needs confirmation
}

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
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => Some(Message::cancel_action()),
                _ => None,
            };
        }

        // Handle profile switcher - profile selection
        if self.input_mode == InputMode::ProfileSwitcherProfile {
            match key.code {
                KeyCode::Esc => {
                    return Some(Message::cancel_profile_switcher());
                }
                KeyCode::Char('R') => {
                    // Toggle read-only mode
                    self.pending_read_only = !self.pending_read_only;
                }
                KeyCode::Enter => {
                    // Store selected profile and move to region selection
                    let selected = self.available_profiles.get(self.profile_switcher_index)
                        .map(|s| if s == "default" { None } else { Some(s.clone()) })
                        .unwrap_or(None);
                    self.pending_profile = selected;
                    self.input_mode = InputMode::ProfileSwitcherRegion;
                    // Pre-select current region in region list
                    if let Some(idx) = self.available_regions.iter().position(|r| r == &self.region) {
                        self.region_switcher_index = idx;
                    }
                    return None;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.available_profiles.is_empty() {
                        self.profile_switcher_index = 
                            (self.profile_switcher_index + 1) % self.available_profiles.len();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.available_profiles.is_empty() {
                        self.profile_switcher_index = if self.profile_switcher_index == 0 {
                            self.available_profiles.len() - 1
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
            match key.code {
                KeyCode::Esc => {
                    // Cancel and go back to normal mode
                    self.pending_profile = None;
                    return Some(Message::cancel_profile_switcher());
                }
                KeyCode::Enter => {
                    // Confirm and switch profile/region
                    let profile = self.pending_profile.clone();
                    let region = self.available_regions.get(self.region_switcher_index)
                        .cloned()
                        .unwrap_or_else(|| "us-east-1".to_string());
                    let read_only = self.pending_read_only;
                    self.pending_profile = None;
                    return Some(Message::switch_profile_region(profile, region, read_only));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.available_regions.is_empty() {
                        self.region_switcher_index = 
                            (self.region_switcher_index + 1) % self.available_regions.len();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.available_regions.is_empty() {
                        self.region_switcher_index = if self.region_switcher_index == 0 {
                            self.available_regions.len() - 1
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
                        Service::Backup | Service::CloudTrail => return Some(Message::cycle_view_mode()),
                        Service::VPC if self.services.vpc.view_mode != VpcViewMode::SecurityGroupRules => return Some(Message::cycle_view_mode()),
                        Service::IAM if self.services.iam.view_mode.is_main_tab() => return Some(Message::cycle_view_mode()),
                        _ => {}
                    }
                }

                // Action log toggle
                if key.code == KeyCode::Char('A') {
                    return Some(Message::toggle_action_log());
                }

                // Arrow navigation for view modes
                match key.code {
                    KeyCode::Right | KeyCode::Char('l') => {
                        match self.current_service {
                            Service::Backup | Service::CloudTrail => return Some(Message::next_view()),
                            Service::VPC if self.services.vpc.view_mode != VpcViewMode::SecurityGroupRules => return Some(Message::next_view()),
                            Service::IAM if self.services.iam.view_mode.is_main_tab() => return Some(Message::next_view()),
                            _ => {}
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        match self.current_service {
                            Service::Backup | Service::CloudTrail => return Some(Message::previous_view()),
                            Service::VPC if self.services.vpc.view_mode != VpcViewMode::SecurityGroupRules => return Some(Message::previous_view()),
                            Service::IAM if self.services.iam.view_mode.is_main_tab() => return Some(Message::previous_view()),
                            _ => {}
                        }
                    }
                    _ => {}
                }

                // Service-specific input handling
                let result = match self.current_service {
                    Service::EC2 => self.handle_ec2_input(key),
                    Service::S3 => self.handle_s3_input(key),
                    Service::RDS => self.handle_rds_input(key),
                    Service::DynamoDB => self.handle_dynamodb_input(key),
                    Service::Lambda => self.handle_lambda_input(key),
                    Service::VPC => self.handle_vpc_input(key),
                    Service::IAM => self.handle_iam_input(key),
                    Service::Backup => self.handle_backup_input(key),
                    Service::CloudTrail => self.handle_cloudtrail_input(key),
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
            KeyCode::Char('d') => Some(Message::toggle_detail_panel()),
            _ => None,
        }
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
                if self.services.ec2.list_state.selected().is_none() && !self.services.ec2.instances.is_empty() {
                    self.services.ec2.list_state.select(Some(0));
                }
            }
            Service::S3 => {
                if self.services.s3.current_bucket.is_some() {
                    if self.services.s3.object_list_state.selected().is_none() && !self.services.s3.objects.is_empty() {
                        self.services.s3.object_list_state.select(Some(0));
                    }
                } else if self.services.s3.list_state.selected().is_none() && !self.services.s3.buckets.is_empty() {
                    self.services.s3.list_state.select(Some(0));
                }
            }
            Service::RDS => {
                if self.services.rds.list_state.selected().is_none() && !self.services.rds.instances.is_empty() {
                    self.services.rds.list_state.select(Some(0));
                }
            }
            Service::DynamoDB => {
                if self.services.dynamodb.list_state.selected().is_none() && !self.services.dynamodb.tables.is_empty() {
                    self.services.dynamodb.list_state.select(Some(0));
                }
            }
            Service::Lambda => {
                if self.services.lambda.list_state.selected().is_none() && !self.services.lambda.functions.is_empty() {
                    self.services.lambda.list_state.select(Some(0));
                }
            }
            Service::VPC => self.auto_select_vpc(),
            Service::IAM => self.auto_select_iam(),
            Service::Backup => self.auto_select_backup(),
            Service::CloudTrail => self.auto_select_cloudtrail(),
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
            if has_items { self.services.vpc.list_state.select(Some(0)); }
        }
    }

    fn auto_select_iam(&mut self) {
        if self.services.iam.list_state.selected().is_none() {
            use crate::app::IamViewMode;
            let has_items = match self.services.iam.view_mode {
                IamViewMode::Users => !self.services.iam.users.is_empty(),
                IamViewMode::Roles => !self.services.iam.roles.is_empty(),
                IamViewMode::Policies => !self.services.iam.policies.is_empty(),
                IamViewMode::UserAttachedPolicies | IamViewMode::RoleAttachedPolicies => !self.services.iam.current_policies.is_empty(),
                IamViewMode::PolicyDocument => false,
            };
            if has_items { self.services.iam.list_state.select(Some(0)); }
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
            if has_items { self.services.backup.list_state.select(Some(0)); }
        }
    }

    fn auto_select_cloudtrail(&mut self) {
        if self.services.cloudtrail.list_state.selected().is_none() {
            use crate::app::CloudTrailViewMode;
            let has_items = match self.services.cloudtrail.view_mode {
                CloudTrailViewMode::Trails => !self.services.cloudtrail.trails.is_empty(),
                CloudTrailViewMode::Events => !self.services.cloudtrail.events.is_empty(),
            };
            if has_items { self.services.cloudtrail.list_state.select(Some(0)); }
        }
    }

    // ====================================
    // Service-specific input handlers
    // ====================================

    fn handle_ec2_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.services.ec2.instances.is_empty() {
                    let i = self.services.ec2.list_state.selected().map_or(0, |i| if i >= self.services.ec2.instances.len() - 1 { 0 } else { i + 1 });
                    self.services.ec2.list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.services.ec2.instances.is_empty() {
                    let i = self.services.ec2.list_state.selected().map_or(0, |i| if i == 0 { self.services.ec2.instances.len() - 1 } else { i - 1 });
                    self.services.ec2.list_state.select(Some(i));
                }
            }
            KeyCode::Char('s') => {
                if let Some(i) = self.services.ec2.list_state.selected() {
                    if let Some(instance) = self.services.ec2.instances.get(i) {
                        return InputResult::Action(Message::ec2_start(instance.instance_id.clone()));
                    }
                }
            }
            KeyCode::Char('S') => {
                if let Some(i) = self.services.ec2.list_state.selected() {
                    if let Some(instance) = self.services.ec2.instances.get(i) {
                        return InputResult::Action(Message::ec2_stop(instance.instance_id.clone()));
                    }
                }
            }
            KeyCode::Char('R') => {
                if let Some(i) = self.services.ec2.list_state.selected() {
                    if let Some(instance) = self.services.ec2.instances.get(i) {
                        return InputResult::Action(Message::ec2_reboot(instance.instance_id.clone()));
                    }
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_s3_input(&mut self, key: KeyEvent) -> InputResult {
        if self.services.s3.current_bucket.is_some() {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.services.s3.objects.is_empty() {
                        let i = self.services.s3.object_list_state.selected().map_or(0, |i| if i >= self.services.s3.objects.len() - 1 { 0 } else { i + 1 });
                        self.services.s3.object_list_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.services.s3.objects.is_empty() {
                        let i = self.services.s3.object_list_state.selected().map_or(0, |i| if i == 0 { self.services.s3.objects.len() - 1 } else { i - 1 });
                        self.services.s3.object_list_state.select(Some(i));
                    }
                }
                KeyCode::Char('D') => {
                    if let Some(i) = self.services.s3.object_list_state.selected() {
                        if let Some(obj) = self.services.s3.objects.get(i) {
                            if let Some(bucket) = &self.services.s3.current_bucket {
                                return InputResult::Action(Message::s3_delete_object(bucket.clone(), obj.key.clone()));
                            }
                        }
                    }
                }
                KeyCode::Char('o') => {
                    // Open object - download to temp and view
                    if let Some(i) = self.services.s3.object_list_state.selected() {
                        if let Some(obj) = self.services.s3.objects.get(i) {
                            if let Some(bucket) = &self.services.s3.current_bucket {
                                return InputResult::Message(Message::s3_open_object(bucket.clone(), obj.key.clone()));
                            }
                        }
                    }
                }
                KeyCode::Char('w') => {
                    // Download/Write object to ~/Downloads
                    if let Some(i) = self.services.s3.object_list_state.selected() {
                        if let Some(obj) = self.services.s3.objects.get(i) {
                            if let Some(bucket) = &self.services.s3.current_bucket {
                                return InputResult::Message(Message::s3_download_object(bucket.clone(), obj.key.clone()));
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Backspace => return InputResult::Message(Message::s3_leave_bucket()),
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.services.s3.buckets.is_empty() {
                        let i = self.services.s3.list_state.selected().map_or(0, |i| if i >= self.services.s3.buckets.len() - 1 { 0 } else { i + 1 });
                        self.services.s3.list_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.services.s3.buckets.is_empty() {
                        let i = self.services.s3.list_state.selected().map_or(0, |i| if i == 0 { self.services.s3.buckets.len() - 1 } else { i - 1 });
                        self.services.s3.list_state.select(Some(i));
                    }
                }
                KeyCode::Enter => {
                    if let Some(i) = self.services.s3.list_state.selected() {
                        if let Some(bucket) = self.services.s3.buckets.get(i) {
                            return InputResult::Message(Message::s3_load_objects(bucket.name.clone()));
                        }
                    }
                }
                KeyCode::Char('i') => {
                    if let Some(i) = self.services.s3.list_state.selected() {
                        if let Some(bucket) = self.services.s3.buckets.get(i) {
                            return InputResult::Message(Message::s3_load_bucket_details(bucket.name.clone()));
                        }
                    }
                }
                _ => {}
            }
        }
        InputResult::None
    }

    fn handle_rds_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.services.rds.instances.is_empty() {
                    let i = self.services.rds.list_state.selected().map_or(0, |i| if i >= self.services.rds.instances.len() - 1 { 0 } else { i + 1 });
                    self.services.rds.list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.services.rds.instances.is_empty() {
                    let i = self.services.rds.list_state.selected().map_or(0, |i| if i == 0 { self.services.rds.instances.len() - 1 } else { i - 1 });
                    self.services.rds.list_state.select(Some(i));
                }
            }
            KeyCode::Char('s') => {
                if let Some(i) = self.services.rds.list_state.selected() {
                    if let Some(inst) = self.services.rds.instances.get(i) {
                        return InputResult::Action(Message::rds_start(inst.db_instance_identifier.clone()));
                    }
                }
            }
            KeyCode::Char('S') => {
                if let Some(i) = self.services.rds.list_state.selected() {
                    if let Some(inst) = self.services.rds.instances.get(i) {
                        return InputResult::Action(Message::rds_stop(inst.db_instance_identifier.clone()));
                    }
                }
            }
            KeyCode::Char('R') => {
                if let Some(i) = self.services.rds.list_state.selected() {
                    if let Some(inst) = self.services.rds.instances.get(i) {
                        return InputResult::Action(Message::rds_reboot(inst.db_instance_identifier.clone()));
                    }
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_dynamodb_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::DynamoDbViewMode;
        if self.services.dynamodb.view_mode == DynamoDbViewMode::Items {
            // In items view
            let len = self.services.dynamodb.items.len();
            match key.code {
                KeyCode::Esc | KeyCode::Backspace => return InputResult::Message(Message::dynamodb_exit_drill_down()),
                KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                    let i = self.services.dynamodb.item_list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                    self.services.dynamodb.item_list_state.select(Some(i));
                }
                KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                    let i = self.services.dynamodb.item_list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                    self.services.dynamodb.item_list_state.select(Some(i));
                }
                KeyCode::Char('D') => {
                    // Delete selected item
                    if let Some(idx) = self.services.dynamodb.item_list_state.selected() {
                        if let Some(item) = self.services.dynamodb.items.get(idx) {
                            if let Some(table_name) = &self.services.dynamodb.current_table {
                                // Build key attributes from item
                                let key_attrs: std::collections::HashMap<String, String> = item.attributes.clone();
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
                    if let Some(table_name) = &self.services.dynamodb.current_table {
                        return InputResult::Message(Message::dynamodb_load_items(table_name.clone()));
                    }
                }
                _ => {}
            }
        } else {
            // In tables list view
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.services.dynamodb.tables.is_empty() {
                        let i = self.services.dynamodb.list_state.selected().map_or(0, |i| if i >= self.services.dynamodb.tables.len() - 1 { 0 } else { i + 1 });
                        self.services.dynamodb.list_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.services.dynamodb.tables.is_empty() {
                        let i = self.services.dynamodb.list_state.selected().map_or(0, |i| if i == 0 { self.services.dynamodb.tables.len() - 1 } else { i - 1 });
                        self.services.dynamodb.list_state.select(Some(i));
                    }
                }
                KeyCode::Enter => return InputResult::Message(Message::dynamodb_drill_down()),
                _ => {}
            }
        }
        InputResult::None
    }

    fn handle_lambda_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.services.lambda.functions.is_empty() {
                    let i = self.services.lambda.list_state.selected().map_or(0, |i| if i >= self.services.lambda.functions.len() - 1 { 0 } else { i + 1 });
                    self.services.lambda.list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.services.lambda.functions.is_empty() {
                    let i = self.services.lambda.list_state.selected().map_or(0, |i| if i == 0 { self.services.lambda.functions.len() - 1 } else { i - 1 });
                    self.services.lambda.list_state.select(Some(i));
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_vpc_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::VpcViewMode;
        let len = match self.services.vpc.view_mode {
            VpcViewMode::Vpcs => self.services.vpc.vpcs.len(),
            VpcViewMode::Subnets => self.services.vpc.subnets.len(),
            VpcViewMode::SecurityGroups => self.services.vpc.security_groups.len(),
            VpcViewMode::SecurityGroupRules => self.services.vpc.current_sg_rules.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self.services.vpc.list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.services.vpc.list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self.services.vpc.list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.services.vpc.list_state.select(Some(i));
            }
            KeyCode::Enter if self.services.vpc.view_mode == VpcViewMode::SecurityGroups => return InputResult::Message(Message::vpc_drill_down_sg()),
            KeyCode::Esc if self.services.vpc.view_mode == VpcViewMode::SecurityGroupRules => return InputResult::Message(Message::vpc_exit_sg_rules()),
            // Toggle between inbound and outbound rules with 't' or 'v'
            KeyCode::Char('t') | KeyCode::Char('v') if self.services.vpc.view_mode == VpcViewMode::SecurityGroupRules => {
                return InputResult::Message(Message::vpc_toggle_sg_rules_direction());
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_iam_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::IamViewMode;
        let len = match self.services.iam.view_mode {
            IamViewMode::Users => self.services.iam.users.len(),
            IamViewMode::Roles => self.services.iam.roles.len(),
            IamViewMode::Policies => self.services.iam.policies.len(),
            IamViewMode::UserAttachedPolicies | IamViewMode::RoleAttachedPolicies => self.services.iam.current_policies.len(),
            IamViewMode::PolicyDocument => 0,
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self.services.iam.list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.services.iam.list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self.services.iam.list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.services.iam.list_state.select(Some(i));
            }
            KeyCode::Enter => {
                return match self.services.iam.view_mode {
                    IamViewMode::Users => InputResult::Message(Message::iam_drill_down_user()),
                    IamViewMode::Roles => InputResult::Message(Message::iam_drill_down_role()),
                    IamViewMode::Policies | IamViewMode::UserAttachedPolicies | IamViewMode::RoleAttachedPolicies => InputResult::Message(Message::iam_drill_down_policy()),
                    IamViewMode::PolicyDocument => InputResult::None,
                };
            }
            KeyCode::Esc if !self.services.iam.view_mode.is_main_tab() => return InputResult::Message(Message::iam_exit_drill_down()),
            _ => {}
        }
        InputResult::None
    }

    fn handle_backup_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::BackupViewMode;
        let len = match self.services.backup.view_mode {
            BackupViewMode::Vaults => self.services.backup.vaults.len(),
            BackupViewMode::Plans => self.services.backup.plans.len(),
            BackupViewMode::Jobs => self.services.backup.jobs.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self.services.backup.list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.services.backup.list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self.services.backup.list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.services.backup.list_state.select(Some(i));
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_cloudtrail_input(&mut self, key: KeyEvent) -> InputResult {
        use crate::app::CloudTrailViewMode;
        let len = match self.services.cloudtrail.view_mode {
            CloudTrailViewMode::Trails => self.services.cloudtrail.trails.len(),
            CloudTrailViewMode::Events => self.services.cloudtrail.events.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self.services.cloudtrail.list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.services.cloudtrail.list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self.services.cloudtrail.list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.services.cloudtrail.list_state.select(Some(i));
            }
            _ => {}
        }
        InputResult::None
    }
}
