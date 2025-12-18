use ratatui::widgets::TableState;
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::app::{CloudTrailViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
use crossterm::event::{KeyCode, KeyEvent};

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

impl crate::app::global_search::Searchable for CloudTrailState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        // CloudTrail Trails
        for trail in &self.trails {
            results.push(SearchResult::new(
                Service::CloudTrail,
                "CloudTrail Trail",
                &trail.name,
            ));
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for CloudTrailState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        self.view_mode = CloudTrailViewMode::Trails;
        if let Some(idx) = self.trails.iter().position(|t| t.name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
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

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn get_copiable_text(&self) -> Option<String> {
        match self.view_mode {
            CloudTrailViewMode::Trails => self.selected_trail().map(|t| t.name.clone()),
            CloudTrailViewMode::Events => self.selected_event().and_then(|e| e.event_id.clone()),
        }
    }
}

impl ServiceInternal for CloudTrailState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.cloudtrail.clone();
        let handle = spawn_list_task(
            tx.clone(),
            move || async move {
                crate::aws::cloudtrail::CloudTrailService::new(client)
                    .list_trails()
                    .await
            },
            AwsEvent::CloudTrailTrailsLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::CLOUDTRAIL_REFRESH, handle);

        // Also refresh events
        let client_events = clients.cloudtrail.clone();
        let handle_events = spawn_list_task(
            tx,
            move || async move {
                crate::aws::cloudtrail::CloudTrailService::new(client_events)
                    .lookup_events(50)
                    .await
            },
            AwsEvent::CloudTrailEventsLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::CLOUDTRAIL_REFRESH, handle_events);
    }

    fn clear(&mut self) {
        self.trails.clear();
        self.events.clear();
        self.selected_event_detail = None;
        self.show_detail_modal = false;
        self.view_mode = CloudTrailViewMode::Trails;
        self.list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() {
            let has_items = match self.view_mode {
                CloudTrailViewMode::Trails => !self.trails.is_empty(),
                CloudTrailViewMode::Events => !self.events.is_empty(),
            };
            if has_items {
                self.list_state.select(Some(0));
            }
        }
    }
}
