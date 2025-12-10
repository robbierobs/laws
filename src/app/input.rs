//! Keyboard input handling
//! 
//! Handles all keyboard events and translates them to messages.

use crossterm::event::{KeyCode, KeyEvent};
use super::{App, Message, Service, Focus, InputMode};
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
            Service::EC2 => self.ec2_list_state.select(Some(0)),
            Service::S3 => {
                if self.current_bucket.is_some() {
                    self.s3_object_list_state.select(Some(0));
                } else {
                    self.s3_list_state.select(Some(0));
                }
            }
            Service::RDS => self.rds_list_state.select(Some(0)),
            Service::DynamoDB => self.dynamodb_list_state.select(Some(0)),
            Service::Lambda => self.lambda_list_state.select(Some(0)),
            Service::VPC => self.vpc_list_state.select(Some(0)),
            Service::IAM => self.iam_list_state.select(Some(0)),
            Service::Backup => self.backup_list_state.select(Some(0)),
            Service::CloudTrail => self.cloudtrail_list_state.select(Some(0)),
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

        // Handle confirmation modal
        if self.show_confirmation {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => Some(Message::ConfirmAction),
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => Some(Message::CancelAction),
                _ => None,
            };
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
                        Service::Backup | Service::CloudTrail => return Some(Message::CycleViewMode),
                        Service::VPC if self.vpc_view_mode != 3 => return Some(Message::CycleViewMode),
                        Service::IAM if self.iam_view_mode < 3 => return Some(Message::CycleViewMode),
                        _ => {}
                    }
                }

                // Action log toggle
                if key.code == KeyCode::Char('A') {
                    return Some(Message::ToggleActionLog);
                }

                // Arrow navigation for view modes
                match key.code {
                    KeyCode::Right | KeyCode::Char('l') => {
                        match self.current_service {
                            Service::Backup | Service::CloudTrail => return Some(Message::NextView),
                            Service::VPC if self.vpc_view_mode != 3 => return Some(Message::NextView),
                            Service::IAM if self.iam_view_mode < 3 => return Some(Message::NextView),
                            _ => {}
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        match self.current_service {
                            Service::Backup | Service::CloudTrail => return Some(Message::PreviousView),
                            Service::VPC if self.vpc_view_mode != 3 => return Some(Message::PreviousView),
                            Service::IAM if self.iam_view_mode < 3 => return Some(Message::PreviousView),
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
                    Service::DynamoDB => { self.handle_dynamodb_input(key); InputResult::None }
                    Service::Lambda => { self.handle_lambda_input(key); InputResult::None }
                    Service::VPC => self.handle_vpc_input(key),
                    Service::IAM => self.handle_iam_input(key),
                    Service::Backup => { self.handle_backup_input(key); InputResult::None }
                    Service::CloudTrail => { self.handle_cloudtrail_input(key); InputResult::None }
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
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('1') => Some(Message::NavigateToService(Service::EC2)),
            KeyCode::Char('2') => Some(Message::NavigateToService(Service::S3)),
            KeyCode::Char('3') => Some(Message::NavigateToService(Service::RDS)),
            KeyCode::Char('4') => Some(Message::NavigateToService(Service::DynamoDB)),
            KeyCode::Char('5') => Some(Message::NavigateToService(Service::Lambda)),
            KeyCode::Char('6') => Some(Message::NavigateToService(Service::VPC)),
            KeyCode::Char('7') => Some(Message::NavigateToService(Service::IAM)),
            KeyCode::Char('8') => Some(Message::NavigateToService(Service::Backup)),
            KeyCode::Char('9') => Some(Message::NavigateToService(Service::CloudTrail)),
            KeyCode::Char('d') => Some(Message::ToggleDetailPanel),
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
                if self.ec2_list_state.selected().is_none() && !self.ec2_instances.is_empty() {
                    self.ec2_list_state.select(Some(0));
                }
            }
            Service::S3 => {
                if self.current_bucket.is_some() {
                    if self.s3_object_list_state.selected().is_none() && !self.s3_objects.is_empty() {
                        self.s3_object_list_state.select(Some(0));
                    }
                } else if self.s3_list_state.selected().is_none() && !self.s3_buckets.is_empty() {
                    self.s3_list_state.select(Some(0));
                }
            }
            Service::RDS => {
                if self.rds_list_state.selected().is_none() && !self.rds_instances.is_empty() {
                    self.rds_list_state.select(Some(0));
                }
            }
            Service::DynamoDB => {
                if self.dynamodb_list_state.selected().is_none() && !self.dynamodb_tables.is_empty() {
                    self.dynamodb_list_state.select(Some(0));
                }
            }
            Service::Lambda => {
                if self.lambda_list_state.selected().is_none() && !self.lambda_functions.is_empty() {
                    self.lambda_list_state.select(Some(0));
                }
            }
            Service::VPC => self.auto_select_vpc(),
            Service::IAM => self.auto_select_iam(),
            Service::Backup => self.auto_select_backup(),
            Service::CloudTrail => self.auto_select_cloudtrail(),
        }
    }

    fn auto_select_vpc(&mut self) {
        if self.vpc_list_state.selected().is_none() {
            let has_items = match self.vpc_view_mode {
                0 => !self.vpcs.is_empty(),
                1 => !self.subnets.is_empty(),
                2 => !self.security_groups.is_empty(),
                3 => !self.current_sg_rules.is_empty(),
                _ => false,
            };
            if has_items { self.vpc_list_state.select(Some(0)); }
        }
    }

    fn auto_select_iam(&mut self) {
        if self.iam_list_state.selected().is_none() {
            let has_items = match self.iam_view_mode {
                0 => !self.iam_users.is_empty(),
                1 => !self.iam_roles.is_empty(),
                2 => !self.iam_policies.is_empty(),
                3 | 4 => !self.current_iam_policies.is_empty(),
                _ => false,
            };
            if has_items { self.iam_list_state.select(Some(0)); }
        }
    }

    fn auto_select_backup(&mut self) {
        if self.backup_list_state.selected().is_none() {
            let has_items = match self.backup_view_mode {
                0 => !self.backup_vaults.is_empty(),
                1 => !self.backup_plans.is_empty(),
                2 => !self.backup_jobs.is_empty(),
                _ => false,
            };
            if has_items { self.backup_list_state.select(Some(0)); }
        }
    }

    fn auto_select_cloudtrail(&mut self) {
        if self.cloudtrail_list_state.selected().is_none() {
            let has_items = match self.cloudtrail_view_mode {
                0 => !self.cloudtrail_trails.is_empty(),
                1 => !self.cloudtrail_events.is_empty(),
                _ => false,
            };
            if has_items { self.cloudtrail_list_state.select(Some(0)); }
        }
    }

    // ====================================
    // Service-specific input handlers
    // ====================================

    fn handle_ec2_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.ec2_instances.is_empty() {
                    let i = self.ec2_list_state.selected().map_or(0, |i| if i >= self.ec2_instances.len() - 1 { 0 } else { i + 1 });
                    self.ec2_list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.ec2_instances.is_empty() {
                    let i = self.ec2_list_state.selected().map_or(0, |i| if i == 0 { self.ec2_instances.len() - 1 } else { i - 1 });
                    self.ec2_list_state.select(Some(i));
                }
            }
            KeyCode::Char('s') => {
                if let Some(i) = self.ec2_list_state.selected() {
                    if let Some(instance) = self.ec2_instances.get(i) {
                        return InputResult::Action(Message::StartInstance(instance.instance_id.clone()));
                    }
                }
            }
            KeyCode::Char('S') => {
                if let Some(i) = self.ec2_list_state.selected() {
                    if let Some(instance) = self.ec2_instances.get(i) {
                        return InputResult::Action(Message::StopInstance(instance.instance_id.clone()));
                    }
                }
            }
            KeyCode::Char('R') => {
                if let Some(i) = self.ec2_list_state.selected() {
                    if let Some(instance) = self.ec2_instances.get(i) {
                        return InputResult::Action(Message::RebootInstance(instance.instance_id.clone()));
                    }
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_s3_input(&mut self, key: KeyEvent) -> InputResult {
        if self.current_bucket.is_some() {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.s3_objects.is_empty() {
                        let i = self.s3_object_list_state.selected().map_or(0, |i| if i >= self.s3_objects.len() - 1 { 0 } else { i + 1 });
                        self.s3_object_list_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.s3_objects.is_empty() {
                        let i = self.s3_object_list_state.selected().map_or(0, |i| if i == 0 { self.s3_objects.len() - 1 } else { i - 1 });
                        self.s3_object_list_state.select(Some(i));
                    }
                }
                KeyCode::Char('D') => {
                    if let Some(i) = self.s3_object_list_state.selected() {
                        if let Some(obj) = self.s3_objects.get(i) {
                            if let Some(bucket) = &self.current_bucket {
                                return InputResult::Action(Message::DeleteS3Object(bucket.clone(), obj.key.clone()));
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Backspace => return InputResult::Message(Message::LeaveS3Bucket),
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.s3_buckets.is_empty() {
                        let i = self.s3_list_state.selected().map_or(0, |i| if i >= self.s3_buckets.len() - 1 { 0 } else { i + 1 });
                        self.s3_list_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.s3_buckets.is_empty() {
                        let i = self.s3_list_state.selected().map_or(0, |i| if i == 0 { self.s3_buckets.len() - 1 } else { i - 1 });
                        self.s3_list_state.select(Some(i));
                    }
                }
                KeyCode::Enter => {
                    if let Some(i) = self.s3_list_state.selected() {
                        if let Some(bucket) = self.s3_buckets.get(i) {
                            return InputResult::Message(Message::LoadS3Objects(bucket.name.clone()));
                        }
                    }
                }
                KeyCode::Char('i') => {
                    if let Some(i) = self.s3_list_state.selected() {
                        if let Some(bucket) = self.s3_buckets.get(i) {
                            return InputResult::Message(Message::LoadBucketDetails(bucket.name.clone()));
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
                if !self.rds_instances.is_empty() {
                    let i = self.rds_list_state.selected().map_or(0, |i| if i >= self.rds_instances.len() - 1 { 0 } else { i + 1 });
                    self.rds_list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.rds_instances.is_empty() {
                    let i = self.rds_list_state.selected().map_or(0, |i| if i == 0 { self.rds_instances.len() - 1 } else { i - 1 });
                    self.rds_list_state.select(Some(i));
                }
            }
            KeyCode::Char('s') => {
                if let Some(i) = self.rds_list_state.selected() {
                    if let Some(inst) = self.rds_instances.get(i) {
                        return InputResult::Action(Message::StartRdsInstance(inst.db_instance_identifier.clone()));
                    }
                }
            }
            KeyCode::Char('S') => {
                if let Some(i) = self.rds_list_state.selected() {
                    if let Some(inst) = self.rds_instances.get(i) {
                        return InputResult::Action(Message::StopRdsInstance(inst.db_instance_identifier.clone()));
                    }
                }
            }
            KeyCode::Char('R') => {
                if let Some(i) = self.rds_list_state.selected() {
                    if let Some(inst) = self.rds_instances.get(i) {
                        return InputResult::Action(Message::RebootRdsInstance(inst.db_instance_identifier.clone()));
                    }
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_dynamodb_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.dynamodb_tables.is_empty() {
                    let i = self.dynamodb_list_state.selected().map_or(0, |i| if i >= self.dynamodb_tables.len() - 1 { 0 } else { i + 1 });
                    self.dynamodb_list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.dynamodb_tables.is_empty() {
                    let i = self.dynamodb_list_state.selected().map_or(0, |i| if i == 0 { self.dynamodb_tables.len() - 1 } else { i - 1 });
                    self.dynamodb_list_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn handle_lambda_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.lambda_functions.is_empty() {
                    let i = self.lambda_list_state.selected().map_or(0, |i| if i >= self.lambda_functions.len() - 1 { 0 } else { i + 1 });
                    self.lambda_list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.lambda_functions.is_empty() {
                    let i = self.lambda_list_state.selected().map_or(0, |i| if i == 0 { self.lambda_functions.len() - 1 } else { i - 1 });
                    self.lambda_list_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn handle_vpc_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.vpc_view_mode {
            0 => self.vpcs.len(),
            1 => self.subnets.len(),
            2 => self.security_groups.len(),
            3 => self.current_sg_rules.len(),
            _ => 0,
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self.vpc_list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.vpc_list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self.vpc_list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.vpc_list_state.select(Some(i));
            }
            KeyCode::Enter if self.vpc_view_mode == 2 => return InputResult::Message(Message::DrillDownSecurityGroup),
            KeyCode::Esc if self.vpc_view_mode == 3 => return InputResult::Message(Message::ExitSecurityGroupRules),
            _ => {}
        }
        InputResult::None
    }

    fn handle_iam_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.iam_view_mode {
            0 => self.iam_users.len(),
            1 => self.iam_roles.len(),
            2 => self.iam_policies.len(),
            3 | 4 => self.current_iam_policies.len(),
            _ => 0,
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self.iam_list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.iam_list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self.iam_list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.iam_list_state.select(Some(i));
            }
            KeyCode::Enter => {
                return match self.iam_view_mode {
                    0 => InputResult::Message(Message::DrillDownIamUser),
                    1 => InputResult::Message(Message::DrillDownIamRole),
                    2 | 3 | 4 => InputResult::Message(Message::DrillDownIamPolicy),
                    _ => InputResult::None,
                };
            }
            KeyCode::Esc if self.iam_view_mode >= 3 => return InputResult::Message(Message::ExitIamDrillDown),
            _ => {}
        }
        InputResult::None
    }

    fn handle_backup_input(&mut self, key: KeyEvent) {
        let len = match self.backup_view_mode {
            0 => self.backup_vaults.len(),
            1 => self.backup_plans.len(),
            2 => self.backup_jobs.len(),
            _ => 0,
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self.backup_list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.backup_list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self.backup_list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.backup_list_state.select(Some(i));
            }
            _ => {}
        }
    }

    fn handle_cloudtrail_input(&mut self, key: KeyEvent) {
        let len = match self.cloudtrail_view_mode {
            0 => self.cloudtrail_trails.len(),
            1 => self.cloudtrail_events.len(),
            _ => 0,
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                let i = self.cloudtrail_list_state.selected().map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
                self.cloudtrail_list_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                let i = self.cloudtrail_list_state.selected().map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.cloudtrail_list_state.select(Some(i));
            }
            _ => {}
        }
    }
}
