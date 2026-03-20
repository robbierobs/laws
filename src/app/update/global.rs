//! Global message handling
//!
//! Handles application-wide messages like navigation, refresh, profile switching.

use crate::app::task_manager::task_keys;
use crate::app::{App, GlobalMessage, InputMode, Service};
use crate::event::{AwsEvent, Event, ProfileRegionSwitchedData};

impl App {
    /// Handle global (non-service-specific) messages
    pub(super) fn handle_global_message(
        &mut self,
        message: GlobalMessage,
        event_tx: crate::app::EventSender,
    ) {
        match message {
            GlobalMessage::Quit => self.should_quit = true,
            GlobalMessage::Navigate(service) => {
                self.current_service = service;
                self.sidebar.select_service(service);
                self.handle_refresh_data(event_tx);
            }
            GlobalMessage::ConfirmAction => {
                if let Some(action) = self.pending_action.take() {
                    self.show_confirmation = false;
                    self.update(action, event_tx);
                }
            }
            GlobalMessage::CancelAction => {
                self.pending_action = None;
                self.show_confirmation = false;
            }
            GlobalMessage::RefreshData => {
                self.handle_refresh_data(event_tx.clone());
            }
            GlobalMessage::ToggleDetailPanel => {
                self.detail_panel_visible = !self.detail_panel_visible;
                if !self.detail_panel_visible {
                    self.detail_panel_fullscreen = false;
                }
                self.detail_scroll_offset = 0;
            }
            GlobalMessage::ToggleDetailFullscreen => {
                self.detail_panel_fullscreen = !self.detail_panel_fullscreen;
                if self.detail_panel_fullscreen {
                    self.detail_panel_visible = true;
                }
                self.detail_scroll_offset = 0;
            }
            GlobalMessage::DetailScrollUp => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(5);
            }
            GlobalMessage::DetailScrollDown => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_add(5);
            }
            GlobalMessage::ToggleActionLog => {
                self.action_log_expanded = !self.action_log_expanded;
            }
            GlobalMessage::CycleViewMode | GlobalMessage::NextView => {
                self.handle_cycle_view_mode(true);
            }
            GlobalMessage::PreviousView => {
                self.handle_cycle_view_mode(false);
            }
            GlobalMessage::OpenProfileSwitcher => {
                self.handle_open_profile_switcher();
            }
            GlobalMessage::CancelProfileSwitcher => {
                self.handle_cancel_profile_switcher();
            }
            GlobalMessage::SwitchProfileRegion {
                profile,
                region,
                read_only,
            } => {
                self.handle_switch_profile_region(profile, region, read_only, event_tx);
            }
            GlobalMessage::CopyToClipboard(text) => {
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => {
                        if let Err(e) = clipboard.set_text(text.clone()) {
                            self.action_log
                                .push(format!("Failed to copy to clipboard: {}", e));
                        } else {
                            self.action_log
                                .push(format!("Copied to clipboard: {}", text));
                            // Also send a notification event if we want a popup, but action log is fine for now
                        }
                    }
                    Err(e) => {
                        self.action_log
                            .push(format!("Failed to access clipboard: {}", e));
                    }
                }
            }
            GlobalMessage::OpenGlobalSearch => {
                self.input_mode = InputMode::GlobalSearch;
                self.global_search.clear();
                self.global_search.loading = true;
                // Trigger data load for all services
                self.refresh_all_services_for_search(event_tx.clone());
                // Pre-populate with all currently loaded results
                self.refresh_global_search();
            }
            GlobalMessage::CloseGlobalSearch => {
                self.input_mode = InputMode::Normal;
                self.global_search.clear();
            }
            GlobalMessage::GotoSearchResult {
                service,
                resource_id,
            } => {
                // Navigate to the service
                self.current_service = service;
                self.sidebar.select_service(service);

                // Try to select the resource in the appropriate service state
                self.select_resource_by_id(service, &resource_id);

                self.action_log.push(format!(
                    "Navigated to {} - {}",
                    service.as_str(),
                    resource_id
                ));

                // Refresh data for the service
                self.handle_refresh_data(event_tx);
            }
        }
    }

    fn handle_open_profile_switcher(&mut self) {
        if let Some(current) = &self.profile {
            if let Some(idx) = self
                .profile_switcher
                .available_profiles
                .iter()
                .position(|p| p == current)
            {
                self.profile_switcher.profile_switcher_index = idx;
            }
        } else {
            self.profile_switcher.profile_switcher_index = 0;
        }
        self.profile_switcher.pending_read_only = self.read_only;
        self.input_mode = InputMode::ProfileSwitcherProfile;
    }

    fn handle_cancel_profile_switcher(&mut self) {
        self.input_mode = InputMode::Normal;
        self.profile_switcher.pending_profile = None;
        self.profile_switcher.reset_filters();
    }

    /// Spawn a background task to switch profile/region
    ///
    /// This performs SSO login (if needed) and AWS client creation in a background
    /// task, then sends a ProfileRegionSwitched event when complete.
    fn handle_switch_profile_region(
        &mut self,
        profile: Option<String>,
        region: String,
        read_only: bool,
        event_tx: crate::app::EventSender,
    ) {
        self.input_mode = InputMode::Normal;
        self.loading = true;

        let profile_name = profile.clone().unwrap_or_else(|| "default".to_string());
        self.action_log.push(format!(
            "Switching to profile: {}, region: {}...",
            profile_name, region
        ));

        // Clone values for async task
        let profile_clone = profile.clone();
        let region_clone = region.clone();
        let is_sso = crate::utils::aws_profiles::is_sso_profile(&profile_name);

        let handle = tokio::spawn(async move {
            let mut sso_messages = Vec::new();

            // Check if profile uses SSO and run login if needed
            if is_sso {
                // First check if existing credentials are valid
                let creds_check = tokio::process::Command::new("aws")
                    .args(["sts", "get-caller-identity", "--profile", &profile_name])
                    .output()
                    .await;

                let needs_login = match creds_check {
                    Ok(output) => !output.status.success(),
                    Err(_) => true,
                };

                if needs_login {
                    sso_messages.push(format!("Running SSO login for profile: {}", profile_name));
                    let sso_result = tokio::process::Command::new("aws")
                        .args(["sso", "login", "--profile", &profile_name])
                        .output()
                        .await;

                    match sso_result {
                        Ok(output) => {
                            if output.status.success() {
                                sso_messages.push(format!(
                                    "SSO login successful for profile: {}",
                                    profile_name
                                ));
                            } else {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                sso_messages.push(format!("SSO login warning: {}", stderr.trim()));
                            }
                        }
                        Err(e) => {
                            sso_messages.push(format!("SSO login error: {}", e));
                        }
                    }
                } else {
                    sso_messages.push(format!(
                        "SSO credentials valid for profile: {}",
                        profile_name
                    ));
                }
            }

            // Create new AWS clients with the new profile and region
            let new_clients = crate::aws::client::AwsClients::new(
                profile_clone.as_deref(),
                Some(region_clone.as_str()),
                None,
            )
            .await;

            match new_clients {
                Ok(clients) => {
                    event_tx
                        .send(Event::Aws(Box::new(AwsEvent::ProfileRegionSwitched(
                            Box::new(ProfileRegionSwitchedData {
                                clients,
                                profile: profile_clone,
                                region: region_clone,
                                read_only,
                                sso_messages,
                            }),
                        ))))
                        .await
                        .ok();
                }
                Err(e) => {
                    event_tx
                        .send(Event::Aws(Box::new(AwsEvent::ProfileRegionSwitchFailed(
                            e.to_string(),
                        ))))
                        .await
                        .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::PROFILE_SWITCH, handle);
    }

    pub(super) fn handle_refresh_data(&mut self, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        self.services.get_mut(self.current_service).refresh(
            event_tx,
            clients,
            &mut self.tasks,
            &self.config,
            true,
        );
    }

    /// Refresh all services in parallel for global search
    /// This ensures we have data from all services for comprehensive search results
    fn refresh_all_services_for_search(&mut self, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.action_log
            .push("Loading all services for global search...".to_string());

        for service in self.services.get_all_mut() {
            service.refresh(
                event_tx.clone(),
                clients,
                &mut self.tasks,
                &self.config,
                false,
            );
        }
    }

    /// Select a resource by its ID within the appropriate service state
    fn select_resource_by_id(&mut self, service: Service, resource_id: &str) {
        // Delegate to the service states' select_by_service_and_id method
        self.services.select_by_service_and_id(service, resource_id);

        // Set focus to main pane so user can immediately interact with the selection
        self.focus = super::super::Focus::Main;
        self.sidebar.is_focused = false;
    }
}
