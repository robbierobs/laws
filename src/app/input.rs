//! Keyboard input handling
//!
//! Handles all keyboard events and translates them to messages.

use super::{App, Focus, GlobalMessage, InputMode, InputResult, Message, Service};
use crate::app::global_search::Searchable;
use crate::ui::components::Component;
use crossterm::event::{KeyCode, KeyEvent};

enum KeyHandling {
    NotHandled,
    Handled(Option<Message>),
}

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

        if let KeyHandling::Handled(message) = self.handle_modal_inputs(key) {
            return message;
        }

        if let KeyHandling::Handled(message) = self.handle_focus_toggle(key) {
            return message;
        }

        if let KeyHandling::Handled(message) = self.handle_focus_input(key) {
            return message;
        }

        self.handle_global_key(key)
    }

    fn handle_copy(&self) -> Option<Message> {
        self.get_active_service_handler()
            .get_copiable_text()
            .map(Message::copy_to_clipboard)
    }

    fn handle_modal_inputs(&mut self, key: KeyEvent) -> KeyHandling {
        if self.services.s3.show_object_viewer {
            return KeyHandling::Handled(super::input_handlers::handle_s3_viewer_input(
                &mut self.services.s3,
                key,
            ));
        }

        let handlers = [
            Self::handle_s3_bucket_creation_input_mode,
            Self::handle_action_log_input_mode,
            Self::handle_confirmation_input_mode,
            Self::handle_profile_selection_input_mode,
            Self::handle_profile_region_input_mode,
            Self::handle_filter_input_mode,
            Self::handle_ecs_service_editor_input_mode,
            Self::handle_ecs_task_def_selector_input_mode,
            Self::handle_global_search_input_mode,
            Self::handle_cloudtrail_filter_input_mode,
            Self::handle_ecr_filter_input_mode,
            Self::handle_backup_filter_input_mode,
        ];

        for handler in handlers {
            if let KeyHandling::Handled(result) = handler(self, key) {
                return KeyHandling::Handled(result);
            }
        }

        KeyHandling::NotHandled
    }

    fn handle_s3_bucket_creation_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::S3BucketCreation {
            return KeyHandling::NotHandled;
        }

        use super::input_handlers::{handle_s3_bucket_creation_input, S3BucketCreationResult};
        match handle_s3_bucket_creation_input(&mut self.services.s3, key) {
            S3BucketCreationResult::Continue => KeyHandling::Handled(None),
            S3BucketCreationResult::Cancel => {
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(None)
            }
            S3BucketCreationResult::Create(name) => {
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(Some(Message::s3_create_bucket(name)))
            }
        }
    }

    fn handle_action_log_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if !self.action_log_expanded {
            return KeyHandling::NotHandled;
        }

        use super::input_handlers::{handle_action_log_input, ActionLogState};
        let mut state = ActionLogState {
            expanded: &mut self.action_log_expanded,
            selected_index: &mut self.action_log_selected_index,
            detail_scroll: &mut self.action_log_detail_scroll,
            log_len: self.action_log.len(),
        };
        handle_action_log_input(&mut state, key);
        KeyHandling::Handled(None)
    }

    fn handle_confirmation_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if !self.show_confirmation {
            return KeyHandling::NotHandled;
        }

        let message = match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(Message::confirm_action()),
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                Some(Message::cancel_action())
            }
            _ => None,
        };

        KeyHandling::Handled(message)
    }

    fn handle_profile_selection_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::ProfileSwitcherProfile {
            return KeyHandling::NotHandled;
        }

        use super::input_handlers::{handle_profile_selection_input, ProfileSwitcherResult};
        match handle_profile_selection_input(&mut self.profile_switcher, key) {
            ProfileSwitcherResult::Continue => KeyHandling::Handled(None),
            ProfileSwitcherResult::Cancel => {
                KeyHandling::Handled(Some(Message::cancel_profile_switcher()))
            }
            ProfileSwitcherResult::ProfileSelected => {
                self.input_mode = InputMode::ProfileSwitcherRegion;
                // Pre-select current region in region list
                let current_region = &self.region;
                let filtered = self.profile_switcher.filtered_regions();
                self.profile_switcher.region_switcher_index = filtered
                    .iter()
                    .position(|r| *r == current_region)
                    .unwrap_or(0);
                KeyHandling::Handled(None)
            }
            ProfileSwitcherResult::Switch { .. } => KeyHandling::Handled(None),
        }
    }

    fn handle_profile_region_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::ProfileSwitcherRegion {
            return KeyHandling::NotHandled;
        }

        use super::input_handlers::{handle_region_selection_input, ProfileSwitcherResult};
        match handle_region_selection_input(&mut self.profile_switcher, key) {
            ProfileSwitcherResult::Continue | ProfileSwitcherResult::ProfileSelected => {
                KeyHandling::Handled(None)
            }
            ProfileSwitcherResult::Cancel => {
                KeyHandling::Handled(Some(Message::cancel_profile_switcher()))
            }
            ProfileSwitcherResult::Switch {
                profile,
                region,
                read_only,
            } => KeyHandling::Handled(Some(Message::switch_profile_region(
                profile, region, read_only,
            ))),
        }
    }

    fn handle_filter_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::Filtering {
            return KeyHandling::NotHandled;
        }

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

        KeyHandling::Handled(None)
    }

    fn handle_ecs_service_editor_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::EcsServiceEditor {
            return KeyHandling::NotHandled;
        }

        use super::input_handlers::{handle_ecs_service_editor_input, EcsEditorResult};
        match handle_ecs_service_editor_input(&mut self.services.ecs, key) {
            EcsEditorResult::Continue => KeyHandling::Handled(None),
            EcsEditorResult::Cancel => {
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(None)
            }
            EcsEditorResult::Update(msg) => {
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(Some(msg))
            }
            EcsEditorResult::Error(err) => {
                self.error_message = Some(err);
                KeyHandling::Handled(None)
            }
        }
    }

    fn handle_ecs_task_def_selector_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::EcsTaskDefSelector {
            return KeyHandling::NotHandled;
        }

        use super::input_handlers::{handle_ecs_task_def_selector_input, EcsTaskDefSelectorResult};
        match handle_ecs_task_def_selector_input(&mut self.services.ecs, key) {
            EcsTaskDefSelectorResult::Continue => KeyHandling::Handled(None),
            EcsTaskDefSelectorResult::Cancel => {
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(None)
            }
            EcsTaskDefSelectorResult::SelectWithConfirmation(msg) => {
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(self.request_action(msg))
            }
        }
    }

    fn handle_global_search_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::GlobalSearch {
            return KeyHandling::NotHandled;
        }

        use super::input_handlers::{handle_global_search_input, GlobalSearchInputResult};
        match handle_global_search_input(&mut self.global_search, key) {
            GlobalSearchInputResult::Continue => KeyHandling::Handled(None),
            GlobalSearchInputResult::Cancel => {
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(None)
            }
            GlobalSearchInputResult::Select(service, resource_id) => {
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(Some(Message::goto_search_result(service, resource_id)))
            }
            GlobalSearchInputResult::QueryChanged => {
                self.refresh_global_search();
                KeyHandling::Handled(None)
            }
        }
    }

    fn handle_cloudtrail_filter_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::CloudTrailEventFilter {
            return KeyHandling::NotHandled;
        }

        use crate::app::messages::{CloudTrailAction, ServiceAction};
        use crate::app::states::cloudtrail::filter_state_to_params;

        let config = &self.services.cloudtrail.filter_config;
        let state = &mut self.services.cloudtrail.filter_modal;
        let total_fields = config.fields.len();

        match key.code {
            KeyCode::Esc => {
                state.close();
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(None)
            }
            KeyCode::Enter => {
                let params = filter_state_to_params(state);
                state.close();
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(Some(Message::Service(ServiceAction::CloudTrail(
                    CloudTrailAction::ApplyFilters(params),
                ))))
            }
            KeyCode::Tab | KeyCode::Down => {
                state.next_field(total_fields);
                KeyHandling::Handled(None)
            }
            KeyCode::BackTab | KeyCode::Up => {
                state.prev_field(total_fields);
                KeyHandling::Handled(None)
            }
            KeyCode::Char(c) => {
                state.handle_char(c, config);
                KeyHandling::Handled(None)
            }
            KeyCode::Backspace => {
                state.handle_backspace(config);
                KeyHandling::Handled(None)
            }
            _ => KeyHandling::Handled(None),
        }
    }

    fn handle_ecr_filter_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::EcrImageFilter {
            return KeyHandling::NotHandled;
        }

        let config = &self.services.ecr.filter_config;
        let state = &mut self.services.ecr.filter_modal;
        let total_fields = config.fields.len();

        match key.code {
            KeyCode::Esc => {
                state.close();
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(None)
            }
            KeyCode::Enter => KeyHandling::Handled(Some(Message::ecr_apply_filters())),
            KeyCode::Tab | KeyCode::Down => {
                state.next_field(total_fields);
                KeyHandling::Handled(None)
            }
            KeyCode::BackTab | KeyCode::Up => {
                state.prev_field(total_fields);
                KeyHandling::Handled(None)
            }
            KeyCode::Char(c) => {
                state.handle_char(c, config);
                KeyHandling::Handled(None)
            }
            KeyCode::Backspace => {
                state.handle_backspace(config);
                KeyHandling::Handled(None)
            }
            _ => KeyHandling::Handled(None),
        }
    }

    fn handle_backup_filter_input_mode(&mut self, key: KeyEvent) -> KeyHandling {
        if self.input_mode != InputMode::BackupJobFilter {
            return KeyHandling::NotHandled;
        }

        let config = &self.services.backup.jobs_filter_config;
        let state = &mut self.services.backup.jobs_filter_modal;
        let total_fields = config.fields.len();

        match key.code {
            KeyCode::Esc => {
                state.close();
                self.input_mode = InputMode::Normal;
                KeyHandling::Handled(None)
            }
            KeyCode::Enter => KeyHandling::Handled(Some(Message::backup_apply_filters())),
            KeyCode::Tab | KeyCode::Down => {
                state.next_field(total_fields);
                KeyHandling::Handled(None)
            }
            KeyCode::BackTab | KeyCode::Up => {
                state.prev_field(total_fields);
                KeyHandling::Handled(None)
            }
            KeyCode::Char(c) => {
                state.handle_char(c, config);
                KeyHandling::Handled(None)
            }
            KeyCode::Backspace => {
                state.handle_backspace(config);
                KeyHandling::Handled(None)
            }
            _ => KeyHandling::Handled(None),
        }
    }

    fn handle_focus_toggle(&mut self, key: KeyEvent) -> KeyHandling {
        if key.code != KeyCode::Tab {
            return KeyHandling::NotHandled;
        }

        self.toggle_focus();
        KeyHandling::Handled(None)
    }

    fn handle_focus_input(&mut self, key: KeyEvent) -> KeyHandling {
        match self.focus {
            Focus::Sidebar => self.handle_sidebar_input(key),
            Focus::Main => self.handle_main_input(key),
        }
    }

    fn handle_sidebar_input(&mut self, key: KeyEvent) -> KeyHandling {
        if matches!(key.code, KeyCode::Char('S') | KeyCode::Char('?')) {
            return KeyHandling::Handled(Some(Message::open_global_search()));
        }

        if let Some(msg) = self.sidebar.handle_key(key) {
            return KeyHandling::Handled(Some(msg));
        }

        KeyHandling::NotHandled
    }

    fn handle_main_input(&mut self, key: KeyEvent) -> KeyHandling {
        if key.code == KeyCode::Char('/') {
            self.input_mode = InputMode::Filtering;
            self.filter_input.clear();
            self.reset_selection();
            return KeyHandling::Handled(None);
        }

        if self.services.get(self.current_service).can_cycle_view() {
            match key.code {
                KeyCode::Char('v') => {
                    return KeyHandling::Handled(Some(Message::cycle_view_mode()))
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    return KeyHandling::Handled(Some(Message::next_view()))
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    return KeyHandling::Handled(Some(Message::previous_view()))
                }
                _ => {}
            }
        }

        let result = self.get_active_service_handler_mut().handle_input(key);

        match result {
            InputResult::Message(msg) => KeyHandling::Handled(Some(msg)),
            InputResult::Action(action) => KeyHandling::Handled(self.request_action(action)),
            InputResult::OpenInputMode(mode) => {
                self.input_mode = mode;
                if mode == InputMode::EcsTaskDefSelector {
                    if let Some(message) = self.ecs_task_def_selector_message() {
                        return KeyHandling::Handled(Some(message));
                    }
                }
                KeyHandling::Handled(None)
            }
            InputResult::None => {
                if key.code == KeyCode::Esc {
                    self.focus = Focus::Sidebar;
                    self.sidebar.is_focused = true;
                    return KeyHandling::Handled(None);
                }
                KeyHandling::NotHandled
            }
        }
    }

    fn handle_global_key(&self, key: KeyEvent) -> Option<Message> {
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
            KeyCode::Char('e') => Some(Message::navigate(Service::ECS)),
            KeyCode::Char('c') => Some(Message::navigate(Service::ECR)),
            KeyCode::Char('A') => Some(Message::toggle_action_log()),
            KeyCode::Char('d') => Some(Message::toggle_detail_panel()),
            KeyCode::Char('D') => Some(Message::Global(GlobalMessage::ToggleDetailFullscreen)),
            KeyCode::PageUp => Some(Message::Global(GlobalMessage::DetailScrollUp)),
            KeyCode::PageDown => Some(Message::Global(GlobalMessage::DetailScrollDown)),
            _ => None,
        }
    }

    fn ecs_task_def_selector_message(&self) -> Option<Message> {
        let service = self
            .services
            .ecs
            .services
            .get(self.services.ecs.list_state.selected().unwrap_or(0))?;
        let td_arn = service.task_definition.as_ref()?;

        td_arn
            .split('/')
            .next_back()
            .and_then(|family| family.split(':').next())
            .map(|family| Message::ecs_load_task_definitions_for_selector(family.to_string()))
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
        self.services
            .get_mut(self.current_service)
            .auto_select_first();
    }

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
