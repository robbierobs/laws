//! CloudTrail update handlers
//!
//! Handles CloudTrail-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::App;
use crate::app::messages::{CloudTrailAction, CloudTrailLookupParams};
use crate::app::states::cloudtrail::FilterModalInputs;
use crate::event::{AwsEvent, Event};

impl App {
    /// Main entry point for CloudTrail actions
    pub(super) fn handle_cloudtrail_action(
        &mut self,
        action: CloudTrailAction,
        event_tx: crate::app::EventSender,
    ) {
        match action {
            CloudTrailAction::ShowEventDetails(json) => {
                self.services.cloudtrail.selected_event_detail = Some(json);
                self.services.cloudtrail.show_detail_modal = true;
            }
            CloudTrailAction::CloseEventDetails => {
                self.services.cloudtrail.show_detail_modal = false;
                self.services.cloudtrail.selected_event_detail = None;
            }
            CloudTrailAction::DeleteTrail(name) => {
                self.handle_delete_trail(name, event_tx);
            }
            CloudTrailAction::LoadMoreEvents => {
                self.handle_load_more_events(event_tx);
            }
            CloudTrailAction::OpenFilterModal => {
                // Initialize modal inputs from current filters
                self.services.cloudtrail.filter_modal_inputs = 
                    FilterModalInputs::from_params(&self.services.cloudtrail.current_filters);
                self.services.cloudtrail.filter_modal_selected_field = 0;
                self.services.cloudtrail.show_filter_modal = true;
                self.input_mode = crate::app::InputMode::CloudTrailEventFilter;
            }
            CloudTrailAction::CloseFilterModal => {
                self.services.cloudtrail.show_filter_modal = false;
                self.input_mode = crate::app::InputMode::Normal;
            }
            CloudTrailAction::ApplyFilters(params) => {
                self.handle_apply_filters(params, event_tx);
            }
            CloudTrailAction::ClearFilters => {
                self.handle_clear_filters(event_tx);
            }
            CloudTrailAction::RefreshEvents => {
                self.handle_refresh_cloudtrail_events(event_tx, false);
            }
        }
    }

    fn handle_delete_trail(&mut self, name: String, event_tx: crate::app::EventSender) {
        self.action_log.push(format!("Deleting CloudTrail: {}", name));
        
        self.spawn_aws_task(event_tx, task_keys::CLOUDTRAIL_ACTION, move |clients, tx| async move {
            let service = crate::aws::cloudtrail::CloudTrailService::new(clients.cloudtrail.clone());
            match service.delete_trail(&name).await {
                Ok(_) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(
                        format!("Trail {} deleted", name)
                    )))).await.ok();
                    // Trigger refresh
                    tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
    }

    fn handle_load_more_events(&mut self, event_tx: crate::app::EventSender) {
        let next_token = match &self.services.cloudtrail.next_token {
            Some(token) => token.clone(),
            None => return, // No more events to load
        };

        self.services.cloudtrail.loading_more = true;
        let filters = self.services.cloudtrail.current_filters.clone();
        let max_results = self.config.max_cloudtrail_events as i32;
        
        self.action_log.push("Loading more CloudTrail events...".to_string());

        self.spawn_aws_task(event_tx, task_keys::CLOUDTRAIL_ACTION, move |clients, tx| async move {
            let service = crate::aws::cloudtrail::CloudTrailService::new(clients.cloudtrail.clone());
            let params = if filters.is_empty() { None } else { Some(&filters) };
            
            match service.lookup_events(max_results, params, Some(&next_token)).await {
                Ok(result) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::CloudTrailEventsLoaded {
                        events: result.events,
                        next_token: result.next_token,
                        append: true, // Append to existing events
                    }))).await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
    }

    fn handle_apply_filters(&mut self, params: CloudTrailLookupParams, event_tx: crate::app::EventSender) {
        // Store the new filters
        self.services.cloudtrail.current_filters = params.clone();
        // Clear existing events and pagination state
        self.services.cloudtrail.events.clear();
        self.services.cloudtrail.next_token = None;
        self.services.cloudtrail.has_more_events = false;
        self.services.cloudtrail.list_state.select(None);
        
        self.action_log.push("Applying CloudTrail event filters...".to_string());
        self.handle_refresh_cloudtrail_events(event_tx, false);
    }

    fn handle_clear_filters(&mut self, event_tx: crate::app::EventSender) {
        // Clear filters
        self.services.cloudtrail.current_filters = CloudTrailLookupParams::default();
        self.services.cloudtrail.filter_modal_inputs = FilterModalInputs::default();
        // Clear existing events and pagination state
        self.services.cloudtrail.events.clear();
        self.services.cloudtrail.next_token = None;
        self.services.cloudtrail.has_more_events = false;
        self.services.cloudtrail.list_state.select(None);
        
        self.action_log.push("Cleared CloudTrail event filters".to_string());
        self.handle_refresh_cloudtrail_events(event_tx, false);
    }

    /// Refresh CloudTrail events with current filters
    fn handle_refresh_cloudtrail_events(&mut self, event_tx: crate::app::EventSender, _report_errors: bool) {
        let filters = self.services.cloudtrail.current_filters.clone();
        let max_results = self.config.max_cloudtrail_events as i32;

        self.spawn_aws_task(event_tx, task_keys::CLOUDTRAIL_REFRESH, move |clients, tx| async move {
            let service = crate::aws::cloudtrail::CloudTrailService::new(clients.cloudtrail.clone());
            let params = if filters.is_empty() { None } else { Some(&filters) };
            
            match service.lookup_events(max_results, params, None).await {
                Ok(result) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::CloudTrailEventsLoaded {
                        events: result.events,
                        next_token: result.next_token,
                        append: false, // Replace events
                    }))).await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
    }
}
