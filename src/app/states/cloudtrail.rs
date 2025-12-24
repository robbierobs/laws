use ratatui::widgets::TableState;
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::app::{CloudTrailViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::app::messages::CloudTrailLookupParams;
use crate::app::filter_modal::{FilterFieldConfig, FilterModalConfig, FilterModalState};
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

/// CloudTrail filter field IDs (for type-safe access)
pub mod filter_fields {
    pub const START_DATE: &str = "start_date";
    pub const START_TIME: &str = "start_time";
    pub const END_DATE: &str = "end_date";
    pub const END_TIME: &str = "end_time";
    pub const EVENT_SOURCE: &str = "event_source";
    pub const EVENT_NAME: &str = "event_name";
    pub const USERNAME: &str = "username";
    pub const READ_ONLY: &str = "read_only";
}

/// Get the CloudTrail filter modal configuration
pub fn cloudtrail_filter_config() -> FilterModalConfig {
    FilterModalConfig::new("CloudTrail Event Filters")
        .field(FilterFieldConfig::date(filter_fields::START_DATE, "Start Date"))
        .field(FilterFieldConfig::time(filter_fields::START_TIME, "Start Time"))
        .field(FilterFieldConfig::date(filter_fields::END_DATE, "End Date"))
        .field(FilterFieldConfig::time(filter_fields::END_TIME, "End Time"))
        .field(FilterFieldConfig::text(filter_fields::EVENT_SOURCE, "Event Source")
            .placeholder("e.g., s3.amazonaws.com"))
        .field(FilterFieldConfig::text(filter_fields::EVENT_NAME, "Event Name")
            .placeholder("e.g., CreateBucket"))
        .field(FilterFieldConfig::text(filter_fields::USERNAME, "Username"))
        .field(FilterFieldConfig::cycle(filter_fields::READ_ONLY, "Read Only",
            vec!["All".into(), "Read Only".into(), "Write Only".into()]))
        .dimensions(60, 80)
}

/// State for CloudTrail service
pub struct CloudTrailState {
    pub trails: Vec<Trail>,
    /// Events with pagination handled by PaginatedList
    pub events: crate::app::pagination::PaginatedList<CloudTrailEvent>,
    pub list_state: TableState,
    pub view_mode: CloudTrailViewMode,
    pub selected_event_detail: Option<String>,
    pub show_detail_modal: bool,
    
    // Filter state (using generic filter modal)
    /// Current active filters
    pub current_filters: CloudTrailLookupParams,
    /// Filter modal configuration (static)
    pub filter_config: FilterModalConfig,
    /// Filter modal runtime state
    pub filter_modal: FilterModalState,
    
    // Sort state
    /// Current sort field
    pub sort_field: EventSortField,
    /// Sort direction
    pub sort_direction: SortDirection,
}

impl Default for CloudTrailState {
    fn default() -> Self {
        let config = cloudtrail_filter_config();
        let modal_state = FilterModalState::new(&config);
        Self {
            trails: Vec::new(),
            events: Default::default(),
            list_state: TableState::default(),
            view_mode: CloudTrailViewMode::default(),
            selected_event_detail: None,
            show_detail_modal: false,
            current_filters: CloudTrailLookupParams::default(),
            filter_config: config,
            filter_modal: modal_state,
            sort_field: EventSortField::default(),
            sort_direction: SortDirection::default(),
        }
    }
}

/// Convert filter modal state to CloudTrailLookupParams
pub fn filter_state_to_params(state: &FilterModalState) -> CloudTrailLookupParams {
    let start_date = state.get_text(filter_fields::START_DATE);
    let start_time = state.get_text(filter_fields::START_TIME);
    let end_date = state.get_text(filter_fields::END_DATE);
    let end_time = state.get_text(filter_fields::END_TIME);
    
    let start_iso = build_iso_time(&start_date, &start_time);
    let end_iso = build_iso_time(&end_date, &end_time);
    
    let event_source = state.get_text(filter_fields::EVENT_SOURCE);
    let event_name = state.get_text(filter_fields::EVENT_NAME);
    let username = state.get_text(filter_fields::USERNAME);
    
    // Cycle: 0 = All, 1 = Read Only, 2 = Write Only
    let read_only = match state.get_cycle_index(filter_fields::READ_ONLY) {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    };
    
    CloudTrailLookupParams {
        start_time: start_iso,
        end_time: end_iso,
        event_source: if event_source.is_empty() { None } else { Some(event_source) },
        event_name: if event_name.is_empty() { None } else { Some(event_name) },
        username: if username.is_empty() { None } else { Some(username) },
        resource_type: None,
        resource_name: None,
        read_only,
    }
}

/// Populate filter modal state from CloudTrailLookupParams
pub fn params_to_filter_state(params: &CloudTrailLookupParams, state: &mut FilterModalState) {
    if let Some(ref start) = params.start_time {
        let (date, time) = parse_iso_time(start);
        state.set_text(filter_fields::START_DATE, date);
        state.set_text(filter_fields::START_TIME, time);
    }
    if let Some(ref end) = params.end_time {
        let (date, time) = parse_iso_time(end);
        state.set_text(filter_fields::END_DATE, date);
        state.set_text(filter_fields::END_TIME, time);
    }
    if let Some(ref source) = params.event_source {
        state.set_text(filter_fields::EVENT_SOURCE, source.clone());
    }
    if let Some(ref name) = params.event_name {
        state.set_text(filter_fields::EVENT_NAME, name.clone());
    }
    if let Some(ref user) = params.username {
        state.set_text(filter_fields::USERNAME, user.clone());
    }
    // Set cycle index: None = 0 (All), Some(true) = 1 (Read), Some(false) = 2 (Write)
    let read_only_idx = match params.read_only {
        None => 0,
        Some(true) => 1,
        Some(false) => 2,
    };
    state.set_cycle_index(filter_fields::READ_ONLY, read_only_idx);
}

fn build_iso_time(date: &str, time: &str) -> Option<String> {
    if date.is_empty() {
        return None;
    }
    let time_part = if time.is_empty() { "00:00" } else { time };
    Some(format!("{}T{}:00Z", date, time_part))
}

fn parse_iso_time(iso: &str) -> (String, String) {
    if iso.contains('T') {
        let parts: Vec<&str> = iso.split('T').collect();
        let date = parts[0].to_string();
        let time = parts.get(1)
            .map(|t| t.trim_end_matches('Z').trim_end_matches(":00"))
            .unwrap_or("")
            .to_string();
        (date, time)
    } else {
        (String::new(), String::new())
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
            self.list_state.selected().and_then(|i| self.events.items.get(i))
        } else {
            None
        }
    }
    
    /// Sort events based on current sort field and direction
    pub fn sort_events(&mut self) {
        let ascending = self.sort_direction == SortDirection::Ascending;
        
        self.events.items.sort_by(|a, b| {
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
            KeyCode::Char('L') if self.view_mode == CloudTrailViewMode::Events && self.events.has_more => {
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
        self.events.clear(); // PaginatedList.clear() resets all pagination state
        self.selected_event_detail = None;
        self.show_detail_modal = false;
        self.view_mode = CloudTrailViewMode::Trails;
        self.list_state.select(Some(0));
        // Reset filter state
        self.current_filters = CloudTrailLookupParams::default();
        self.filter_modal.close();
        self.filter_modal.clear_all();
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
