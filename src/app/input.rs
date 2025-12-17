//! Keyboard input handling
//!
//! Handles all keyboard events and translates them to messages.

use super::{
    App, Focus, GlobalMessage, InputMode, InputResult, Message, Service, ViewMode, VpcViewMode,
};
use crate::ui::components::Component;
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    /// Reset list selection to first item for current service
    pub fn reset_selection(&mut self) {
        self.get_active_service_handler_mut().reset_selection();
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
            return super::input_handlers::handle_s3_viewer_input(&mut self.services.s3, key);
        }

        // Handle S3 bucket creation modal
        if self.input_mode == InputMode::S3BucketCreation {
            use super::input_handlers::{handle_s3_bucket_creation_input, process_bucket_creation_result};
            let result = handle_s3_bucket_creation_input(&mut self.services.s3, key);
            let (exit_modal, message) = process_bucket_creation_result(result);
            if exit_modal {
                self.input_mode = InputMode::Normal;
            }
            return message;
        }

        // Handle action log popup navigation
        if self.action_log_expanded {
            use super::input_handlers::{handle_action_log_input, ActionLogState};
            let mut state = ActionLogState {
                expanded: &mut self.action_log_expanded,
                selected_index: &mut self.action_log_selected_index,
                detail_scroll: &mut self.action_log_detail_scroll,
                log_len: self.action_log.len(),
            };
            handle_action_log_input(&mut state, key);
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

        // Handle ECS service editor modal
        if self.input_mode == InputMode::EcsServiceEditor {
            match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.services.ecs.reset_service_editor();
                }
                KeyCode::Tab | KeyCode::Down => {
                    // Cycle through fields: 0=task_def, 1=cpu, 2=memory, 3=force_deploy
                    self.services.ecs.service_editor.active_field =
                        (self.services.ecs.service_editor.active_field + 1) % 4;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    // Cycle backwards
                    self.services.ecs.service_editor.active_field =
                        (self.services.ecs.service_editor.active_field + 3) % 4;
                }
                KeyCode::Char(' ') => {
                    // Toggle force deploy if on that field
                    if self.services.ecs.service_editor.active_field == 3 {
                        self.services.ecs.service_editor.toggle_force_deploy();
                    }
                }
                KeyCode::Backspace => match self.services.ecs.service_editor.active_field {
                    0 => {
                        self.services.ecs.service_editor.task_def.pop();
                    }
                    1 => {
                        self.services.ecs.service_editor.cpu.pop();
                    }
                    2 => {
                        self.services.ecs.service_editor.memory.pop();
                    }
                    _ => {}
                },
                KeyCode::Char(c) => {
                    match self.services.ecs.service_editor.active_field {
                        0 => self.services.ecs.service_editor.task_def.push(c),
                        1 => {
                            // Only allow digits for CPU
                            if c.is_ascii_digit() {
                                self.services.ecs.service_editor.cpu.push(c);
                            }
                        }
                        2 => {
                            // Only allow digits for memory
                            if c.is_ascii_digit() {
                                self.services.ecs.service_editor.memory.push(c);
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Enter => {
                    // Submit the update
                    if let (Some(cluster_arn), Some(service_name)) = (
                        self.services.ecs.selected_cluster_arn.clone(),
                        self.services.ecs.service_editor.service_name.clone(),
                    ) {
                        let task_def = if self.services.ecs.service_editor.task_def.is_empty() {
                            None
                        } else {
                            Some(self.services.ecs.service_editor.task_def.clone())
                        };
                        let cpu = if self.services.ecs.service_editor.cpu.is_empty() {
                            None
                        } else {
                            Some(self.services.ecs.service_editor.cpu.clone())
                        };
                        let memory = if self.services.ecs.service_editor.memory.is_empty() {
                            None
                        } else {
                            Some(self.services.ecs.service_editor.memory.clone())
                        };
                        let force_deploy = self.services.ecs.service_editor.force_deploy;

                        // Only submit if at least one field has a value
                        if task_def.is_some() || cpu.is_some() || memory.is_some() {
                            self.input_mode = InputMode::Normal;
                            self.services.ecs.reset_service_editor();
                            return Some(Message::ecs_update_service(
                                cluster_arn,
                                service_name,
                                task_def,
                                cpu,
                                memory,
                                force_deploy,
                            ));
                        } else {
                            self.error_message =
                                Some("Please specify at least one change".to_string());
                        }
                    }
                }
                _ => {}
            }
            return None;
        }

        // Handle ECS task definition selector modal
        if self.input_mode == InputMode::EcsTaskDefSelector {
            match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.services.ecs.reset_task_def_selector();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.services.ecs.task_def_selector.nav_down();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.services.ecs.task_def_selector.nav_up();
                }
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    // Toggle force new deployment
                    self.services.ecs.task_def_selector.toggle_force_deploy();
                }
                KeyCode::PageDown => {
                    self.services.ecs.task_def_selector.scroll_down(5);
                }
                KeyCode::PageUp => {
                    self.services.ecs.task_def_selector.scroll_up(5);
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    // Scroll detail pane down
                    self.services.ecs.task_def_selector.scroll_down(1);
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    // Scroll detail pane up
                    self.services.ecs.task_def_selector.scroll_up(1);
                }
                KeyCode::Enter => {
                    // Submit the selection - request confirmation
                    if let (Some(cluster_arn), Some(service_name), Some(task_def)) = (
                        self.services.ecs.selected_cluster_arn.clone(),
                        self.services.ecs.task_def_selector.service_name.clone(),
                        self.services
                            .ecs
                            .task_def_selector
                            .selected()
                            .map(|td| td.task_definition_arn.clone()),
                    ) {
                        let force_deploy = self.services.ecs.task_def_selector.force_deploy;
                        self.input_mode = InputMode::Normal;
                        self.services.ecs.reset_task_def_selector();

                        // Request confirmation for the task definition change
                        return self.request_action(Message::ecs_update_service(
                            cluster_arn,
                            service_name,
                            Some(task_def),
                            None,
                            None,
                            force_deploy,
                        ));
                    }
                }
                _ => {}
            }
            return None;
        }

        // Handle global search modal
        if self.input_mode == InputMode::GlobalSearch {
            match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.global_search.clear();
                }
                KeyCode::Down | KeyCode::Char('j')
                    if key.modifiers.is_empty() || self.global_search.query.is_empty() =>
                {
                    self.global_search.nav_down();
                }
                KeyCode::Up | KeyCode::Char('k')
                    if key.modifiers.is_empty() || self.global_search.query.is_empty() =>
                {
                    self.global_search.nav_up();
                }
                KeyCode::Enter => {
                    if let Some(result) = self.global_search.selected() {
                        let service = result.service;
                        let resource_id = result.primary_id.clone();
                        self.input_mode = InputMode::Normal;
                        self.global_search.clear();
                        return Some(Message::goto_search_result(service, resource_id));
                    }
                }
                KeyCode::Backspace => {
                    self.global_search.query.pop();
                    self.refresh_global_search();
                }
                KeyCode::Char(c) => {
                    self.global_search.query.push(c);
                    self.refresh_global_search();
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
                let result = self.get_active_service_handler_mut().handle_input(key);

                match result {
                    InputResult::Message(msg) => return Some(msg),
                    InputResult::Action(action) => return self.request_action(action),
                    InputResult::OpenInputMode(mode) => {
                        self.input_mode = mode;
                        // For EcsTaskDefSelector, we need to trigger loading task definitions
                        if mode == InputMode::EcsTaskDefSelector {
                            // Get the family from the current service's task definition
                            if let Some(service) = self
                                .services
                                .ecs
                                .services
                                .get(self.services.ecs.list_state.selected().unwrap_or(0))
                            {
                                if let Some(td_arn) = &service.task_definition {
                                    // Extract family from ARN: arn:aws:ecs:region:account:task-definition/family:revision
                                    let family = td_arn
                                        .split('/')
                                        .next_back()
                                        .and_then(|f| f.split(':').next())
                                        .map(|f| f.to_string());

                                    if let Some(family) = family {
                                        return Some(
                                            Message::ecs_load_task_definitions_for_selector(family),
                                        );
                                    }
                                }
                            }
                        }
                        return None;
                    }
                    InputResult::None => {
                        // ESC: If service handler didn't consume it, switch focus to sidebar
                        if key.code == KeyCode::Esc {
                            self.focus = Focus::Sidebar;
                            self.sidebar.is_focused = true;
                            return None;
                        }
                    }
                }
            }
        }

        // Global keys
        match key.code {
            KeyCode::Char('r') => Some(Message::refresh()),
            KeyCode::Char('y') => self.handle_copy(),
            KeyCode::Char('q') => Some(Message::quit()),
            KeyCode::Char('?') | KeyCode::Char('S') => Some(Message::open_global_search()),
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
            KeyCode::Char('e') => Some(Message::navigate(Service::ECS)), // Using 'e' for ECS
            KeyCode::Char('c') => Some(Message::navigate(Service::ECR)), // Using 'c' for ECR (Container Registry)
            KeyCode::Char('A') => Some(Message::toggle_action_log()),
            KeyCode::Char('d') => Some(Message::toggle_detail_panel()),
            KeyCode::Char('D') => Some(Message::Global(GlobalMessage::ToggleDetailFullscreen)),
            KeyCode::PageUp => Some(Message::Global(GlobalMessage::DetailScrollUp)),
            KeyCode::PageDown => Some(Message::Global(GlobalMessage::DetailScrollDown)),
            _ => None,
        }
    }

    fn handle_copy(&self) -> Option<Message> {
        self.get_active_service_handler()
            .get_copiable_text()
            .map(Message::copy_to_clipboard)
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
            Service::ECR => self.auto_select_ecr(),
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
                BackupViewMode::RecoveryPoints => !self.services.backup.recovery_points.is_empty(),
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

    fn auto_select_ecr(&mut self) {
        if self.services.ecr.list_state.selected().is_none() {
            use crate::app::EcrViewMode;
            let has_items = match self.services.ecr.view_mode {
                EcrViewMode::Repositories => !self.services.ecr.repositories.is_empty(),
                EcrViewMode::Images => !self.services.ecr.images.is_empty(),
            };
            if has_items {
                self.services.ecr.list_state.select(Some(0));
            }
        }
    }

    // ====================================
    // Service-specific input handlers
    // ====================================

    // ====================================
    // Global Search helpers
    // ====================================

    /// Refresh global search results based on current query
    pub fn refresh_global_search(&mut self) {
        let all_results = self.collect_all_search_results();
        self.global_search.filter(&all_results);
    }

    /// Collect all searchable resources from all services
    fn collect_all_search_results(&self) -> Vec<super::global_search::SearchResult> {
        use super::global_search::SearchResult;
        use super::Service;

        let mut results = Vec::new();

        // EC2 Instances
        for instance in &self.services.ec2.instances {
            let name = instance.name.clone().unwrap_or_default();
            let mut result = SearchResult::new(Service::EC2, "EC2 Instance", &instance.instance_id);
            if !name.is_empty() {
                result = result.with_secondary(name);
            }
            if !instance.tags.is_empty() {
                result = result.with_tags(instance.tags.clone());
            }
            results.push(result);
        }

        // S3 Buckets
        for bucket in &self.services.s3.buckets {
            results.push(SearchResult::new(Service::S3, "S3 Bucket", &bucket.name));
        }

        // RDS Instances
        for instance in &self.services.rds.instances {
            let mut result = SearchResult::new(
                Service::RDS,
                "RDS Instance",
                &instance.db_instance_identifier,
            );
            result = result.with_secondary(instance.engine.clone());
            results.push(result);
        }

        // DynamoDB Tables
        for table in &self.services.dynamodb.tables {
            results.push(SearchResult::new(
                Service::DynamoDB,
                "DynamoDB Table",
                &table.table_name,
            ));
        }

        // Lambda Functions
        for func in &self.services.lambda.functions {
            let mut result =
                SearchResult::new(Service::Lambda, "Lambda Function", &func.function_name);
            if let Some(ref desc) = func.description {
                if !desc.is_empty() {
                    result = result.with_secondary(desc.clone());
                }
            }
            results.push(result);
        }

        // VPCs
        for vpc in &self.services.vpc.vpcs {
            let name = vpc.name.clone().unwrap_or_default();
            let mut result = SearchResult::new(Service::VPC, "VPC", &vpc.vpc_id);
            if !name.is_empty() {
                result = result.with_secondary(name);
            }
            results.push(result);
        }

        // Subnets
        for subnet in &self.services.vpc.subnets {
            let name = subnet.name.clone().unwrap_or_default();
            let mut result = SearchResult::new(Service::VPC, "Subnet", &subnet.subnet_id);
            if !name.is_empty() {
                result = result.with_secondary(name);
            }
            results.push(result);
        }

        // Security Groups
        for sg in &self.services.vpc.security_groups {
            let mut result = SearchResult::new(Service::VPC, "Security Group", &sg.group_id);
            result = result.with_secondary(sg.group_name.clone());
            results.push(result);
        }

        // IAM Users
        for user in &self.services.iam.users {
            results.push(SearchResult::new(Service::IAM, "IAM User", &user.user_name));
        }

        // IAM Roles
        for role in &self.services.iam.roles {
            results.push(SearchResult::new(Service::IAM, "IAM Role", &role.role_name));
        }

        // IAM Policies
        for policy in &self.services.iam.policies {
            results.push(SearchResult::new(
                Service::IAM,
                "IAM Policy",
                &policy.policy_name,
            ));
        }

        // Backup Vaults
        for vault in &self.services.backup.vaults {
            results.push(SearchResult::new(
                Service::Backup,
                "Backup Vault",
                &vault.backup_vault_name,
            ));
        }

        // CloudTrail Trails
        for trail in &self.services.cloudtrail.trails {
            results.push(SearchResult::new(
                Service::CloudTrail,
                "CloudTrail Trail",
                &trail.name,
            ));
        }

        // Secrets Manager Secrets
        for secret in &self.services.secretsmanager.secrets {
            let mut result = SearchResult::new(Service::SecretsManager, "Secret", &secret.name);
            if let Some(ref desc) = secret.description {
                if !desc.is_empty() {
                    result = result.with_secondary(desc.clone());
                }
            }
            results.push(result);
        }

        // ECS Clusters
        for cluster in &self.services.ecs.clusters {
            results.push(SearchResult::new(
                Service::ECS,
                "ECS Cluster",
                &cluster.cluster_name,
            ));
        }

        // ECS Services
        for service in &self.services.ecs.services {
            results.push(SearchResult::new(
                Service::ECS,
                "ECS Service",
                &service.service_name,
            ));
        }

        // ECR Repositories
        for repo in &self.services.ecr.repositories {
            results.push(SearchResult::new(
                Service::ECR,
                "ECR Repository",
                &repo.repository_name,
            ));
        }

        results
    }
}
