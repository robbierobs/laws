//! Keyboard input handling
//!
//! Handles all keyboard events and translates them to messages.

use super::{
    App, Focus, GlobalMessage, InputMode, InputResult, Message, Service, ViewMode, VpcViewMode,
};
use crate::app::global_search::Searchable;
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
            use super::input_handlers::{handle_s3_bucket_creation_input, S3BucketCreationResult};
            match handle_s3_bucket_creation_input(&mut self.services.s3, key) {
                S3BucketCreationResult::Continue => {}
                S3BucketCreationResult::Cancel => {
                    self.input_mode = InputMode::Normal;
                }
                S3BucketCreationResult::Create(name) => {
                    self.input_mode = InputMode::Normal;
                    return Some(Message::s3_create_bucket(name));
                }
            }
            return None;
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
            use super::input_handlers::{handle_profile_selection_input, ProfileSwitcherResult};
            match handle_profile_selection_input(&mut self.profile_switcher, key) {
                ProfileSwitcherResult::Continue => {}
                ProfileSwitcherResult::Cancel => {
                    return Some(Message::cancel_profile_switcher());
                }
                ProfileSwitcherResult::ProfileSelected => {
                    self.input_mode = InputMode::ProfileSwitcherRegion;
                    // Pre-select current region in region list
                    let current_region = &self.region;
                    let filtered = self.profile_switcher.filtered_regions();
                    if let Some(idx) = filtered.iter().position(|r| *r == current_region) {
                        self.profile_switcher.region_switcher_index = idx;
                    } else {
                        self.profile_switcher.region_switcher_index = 0;
                    }
                }
                _ => {}
            }
            return None;
        }

        // Handle profile switcher - region selection
        if self.input_mode == InputMode::ProfileSwitcherRegion {
            use super::input_handlers::{handle_region_selection_input, ProfileSwitcherResult};
            match handle_region_selection_input(&mut self.profile_switcher, key) {
                ProfileSwitcherResult::Continue => {}
                ProfileSwitcherResult::Cancel => {
                    return Some(Message::cancel_profile_switcher());
                }
                ProfileSwitcherResult::Switch {
                    profile,
                    region,
                    read_only,
                } => {
                    return Some(Message::switch_profile_region(profile, region, read_only));
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
            use super::input_handlers::{handle_ecs_service_editor_input, EcsEditorResult};
            match handle_ecs_service_editor_input(&mut self.services.ecs, key) {
                EcsEditorResult::Continue => {}
                EcsEditorResult::Cancel => {
                    self.input_mode = InputMode::Normal;
                }
                EcsEditorResult::Update(msg) => {
                    self.input_mode = InputMode::Normal;
                    return Some(msg);
                }
                EcsEditorResult::Error(err) => {
                    self.error_message = Some(err);
                }
            }
            return None;
        }

        // Handle ECS task definition selector modal
        if self.input_mode == InputMode::EcsTaskDefSelector {
            use super::input_handlers::{handle_ecs_task_def_selector_input, EcsTaskDefSelectorResult};
            match handle_ecs_task_def_selector_input(&mut self.services.ecs, key) {
                EcsTaskDefSelectorResult::Continue => {}
                EcsTaskDefSelectorResult::Cancel => {
                    self.input_mode = InputMode::Normal;
                }
                EcsTaskDefSelectorResult::SelectWithConfirmation(msg) => {
                    self.input_mode = InputMode::Normal;
                    // Request confirmation for the task definition change
                    return self.request_action(msg);
                }
            }
            return None;
        }

        // Handle global search modal
        if self.input_mode == InputMode::GlobalSearch {
            use super::input_handlers::{handle_global_search_input, GlobalSearchInputResult};
            match handle_global_search_input(&mut self.global_search, key) {
                GlobalSearchInputResult::Continue => {}
                GlobalSearchInputResult::Cancel => {
                    self.input_mode = InputMode::Normal;
                }
                GlobalSearchInputResult::Select(service, resource_id) => {
                    self.input_mode = InputMode::Normal;
                    return Some(Message::goto_search_result(service, resource_id));
                }
                GlobalSearchInputResult::QueryChanged => {
                    self.refresh_global_search();
                }
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
        self.services.get_mut(self.current_service).auto_select_first();
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
        self.services.get_search_results()
    }
}
