use ratatui::widgets::TableState;
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::app::{CloudTrailViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::app::messages::CloudTrailLookupParams;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
use crossterm::event::{KeyCode, KeyEvent};

/// Field to sort CloudTrail events by
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventSortField {
    #[default]
    Time,
    EventName,
    Source,
    Username,
}

impl EventSortField {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Time => "Time",
            Self::EventName => "Name",
            Self::Source => "Source",
            Self::Username => "User",
        }
    }
    
    pub fn next(&self) -> Self {
        match self {
            Self::Time => Self::EventName,
            Self::EventName => Self::Source,
            Self::Source => Self::Username,
            Self::Username => Self::Time,
        }
    }
}

/// Sort direction for CloudTrail events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Descending, // Most recent first (default for time-based data)
    Ascending,
}

impl SortDirection {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ascending => "↑",
            Self::Descending => "↓",
        }
    }
    
    pub fn toggle(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// State for CloudTrail service
#[derive(Default)]
pub struct CloudTrailState {
    pub trails: Vec<Trail>,
    pub events: Vec<CloudTrailEvent>,
    pub list_state: TableState,
    pub view_mode: CloudTrailViewMode,
    pub selected_event_detail: Option<String>,
    pub show_detail_modal: bool,
    
    // Pagination state
    /// Token for fetching the next page of events
    pub next_token: Option<String>,
    /// Whether there are more events to load
    pub has_more_events: bool,
    /// Whether we're currently loading more events
    pub loading_more: bool,
    
    // Filter state
    /// Current active filters
    pub current_filters: CloudTrailLookupParams,
    /// Whether the filter modal is open
    pub show_filter_modal: bool,
    /// Filter modal input state - which field is selected (0-7)
    pub filter_modal_selected_field: usize,
    /// Filter modal input buffers
    pub filter_modal_inputs: FilterModalInputs,
    
    // Sort state
    /// Current sort field
    pub sort_field: EventSortField,
    /// Sort direction
    pub sort_direction: SortDirection,
}

/// Input buffers for the filter modal
#[derive(Default, Clone)]
pub struct FilterModalInputs {
    pub start_date: String,      // YYYY-MM-DD format
    pub start_time: String,      // HH:MM format
    pub end_date: String,        // YYYY-MM-DD format
    pub end_time: String,        // HH:MM format
    pub event_source: String,
    pub event_name: String,
    pub username: String,
    pub read_only: Option<bool>, // None = all, Some(true) = read-only, Some(false) = write
}

impl FilterModalInputs {
    pub fn to_params(&self) -> CloudTrailLookupParams {
        CloudTrailLookupParams {
            start_time: self.build_iso_time(&self.start_date, &self.start_time),
            end_time: self.build_iso_time(&self.end_date, &self.end_time),
            event_source: if self.event_source.is_empty() { None } else { Some(self.event_source.clone()) },
            event_name: if self.event_name.is_empty() { None } else { Some(self.event_name.clone()) },
            username: if self.username.is_empty() { None } else { Some(self.username.clone()) },
            resource_type: None,
            resource_name: None,
            read_only: self.read_only,
        }
    }
    
    fn build_iso_time(&self, date: &str, time: &str) -> Option<String> {
        if date.is_empty() {
            return None;
        }
        let time_part = if time.is_empty() { "00:00" } else { time };
        Some(format!("{}T{}:00Z", date, time_part))
    }
    
    pub fn from_params(params: &CloudTrailLookupParams) -> Self {
        let (start_date, start_time) = Self::parse_iso_time(params.start_time.as_deref());
        let (end_date, end_time) = Self::parse_iso_time(params.end_time.as_deref());
        
        Self {
            start_date,
            start_time,
            end_date,
            end_time,
            event_source: params.event_source.clone().unwrap_or_default(),
            event_name: params.event_name.clone().unwrap_or_default(),
            username: params.username.clone().unwrap_or_default(),
            read_only: params.read_only,
        }
    }
    
    fn parse_iso_time(iso: Option<&str>) -> (String, String) {
        match iso {
            Some(s) if s.contains('T') => {
                let parts: Vec<&str> = s.split('T').collect();
                let date = parts[0].to_string();
                let time = parts.get(1)
                    .map(|t| t.trim_end_matches('Z').trim_end_matches(":00"))
                    .unwrap_or("")
                    .to_string();
                (date, time)
            }
            _ => (String::new(), String::new()),
        }
    }
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
    
    /// Sort events based on current sort field and direction
    pub fn sort_events(&mut self) {
        let ascending = self.sort_direction == SortDirection::Ascending;
        
        self.events.sort_by(|a, b| {
            let cmp = match self.sort_field {
                EventSortField::Time => {
                    // Compare by event_time (string comparison works for ISO dates)
                    a.event_time.cmp(&b.event_time)
                }
                EventSortField::EventName => {
                    a.event_name.cmp(&b.event_name)
                }
                EventSortField::Source => {
                    a.event_source.cmp(&b.event_source)
                }
                EventSortField::Username => {
                    a.username.cmp(&b.username)
                }
            };
            
            if ascending { cmp } else { cmp.reverse() }
        });
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
        // Note: Filter modal input is handled by InputMode::CloudTrailEventFilter in App::handle_key
        
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
            // Open filter modal (Events view only)
            KeyCode::Char('F') if self.view_mode == CloudTrailViewMode::Events => {
                return InputResult::Message(Message::cloudtrail_open_filter_modal());
            }
            // Load more events (Events view only)
            KeyCode::Char('L') if self.view_mode == CloudTrailViewMode::Events && self.has_more_events => {
                return InputResult::Message(Message::cloudtrail_load_more_events());
            }
            // Clear filters (Events view only) 
            KeyCode::Char('c') if self.view_mode == CloudTrailViewMode::Events && !self.current_filters.is_empty() => {
                return InputResult::Message(Message::cloudtrail_clear_filters());
            }
            // Cycle sort field (Events view only)
            KeyCode::Char('s') if self.view_mode == CloudTrailViewMode::Events => {
                self.sort_field = self.sort_field.next();
                self.sort_events();
            }
            // Toggle sort direction (Events view only)
            KeyCode::Char('S') if self.view_mode == CloudTrailViewMode::Events => {
                self.sort_direction = self.sort_direction.toggle();
                self.sort_events();
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
        config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        use crate::event::Event;
        
        // Refresh trails
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

        // Refresh events with current filters
        let client_events = clients.cloudtrail.clone();
        let filters = self.current_filters.clone();
        let max_results = config.max_cloudtrail_events as i32;
        let tx_events = tx.clone();
        
        let handle_events = tokio::spawn(async move {
            let service = crate::aws::cloudtrail::CloudTrailService::new(client_events);
            let params = if filters.is_empty() { None } else { Some(&filters) };
            
            match service.lookup_events(max_results, params, None).await {
                Ok(result) => {
                    tx_events.send(Event::Aws(Box::new(AwsEvent::CloudTrailEventsLoaded {
                        events: result.events,
                        next_token: result.next_token,
                        append: false,
                    }))).await.ok();
                }
                Err(e) => {
                    if report_errors {
                        tx_events.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                            .await.ok();
                    }
                }
            }
        });
        tasks.spawn(task_keys::CLOUDTRAIL_REFRESH, handle_events);
    }

    fn clear(&mut self) {
        self.trails.clear();
        self.events.clear();
        self.selected_event_detail = None;
        self.show_detail_modal = false;
        self.view_mode = CloudTrailViewMode::Trails;
        self.list_state.select(Some(0));
        // Reset pagination state
        self.next_token = None;
        self.has_more_events = false;
        self.loading_more = false;
        // Reset filter state
        self.current_filters = CloudTrailLookupParams::default();
        self.show_filter_modal = false;
        self.filter_modal_selected_field = 0;
        self.filter_modal_inputs = FilterModalInputs::default();
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

    fn can_cycle_view(&self) -> bool {
        true // CloudTrail supports view mode cycling in all tabs
    }
}
