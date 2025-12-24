use ratatui::widgets::TableState;
use crate::models::backup::{BackupJob, BackupPlan, BackupVault, RecoveryPoint};
use crate::app::{BackupViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::app::filter_modal::{FilterFieldConfig, FilterModalConfig, FilterModalState};
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::aws::backup::BackupService;
use crate::event::{AwsEvent, Event};
use crossterm::event::{KeyCode, KeyEvent};

/// Field to sort Backup Jobs by
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JobSortField {
    #[default]
    CreatedAt,
    State,
    ResourceType,
}

impl JobSortField {
    pub fn label(&self) -> &'static str {
        match self {
            Self::CreatedAt => "Date",
            Self::State => "State",
            Self::ResourceType => "Type",
        }
    }
    
    pub fn next(&self) -> Self {
        match self {
            Self::CreatedAt => Self::State,
            Self::State => Self::ResourceType,
            Self::ResourceType => Self::CreatedAt,
        }
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Descending,
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

/// Backup Jobs filter field IDs
pub mod filter_fields {
    pub const JOB_STATE: &str = "job_state";
    pub const RESOURCE_TYPE: &str = "resource_type";
}

/// Get the Backup Jobs filter modal configuration
pub fn backup_jobs_filter_config() -> FilterModalConfig {
    FilterModalConfig::new("Backup Job Filters")
        .field(FilterFieldConfig::cycle(filter_fields::JOB_STATE, "Job State",
            vec!["All".into(), "COMPLETED".into(), "RUNNING".into(), "FAILED".into(), "PENDING".into()]))
        .field(FilterFieldConfig::text(filter_fields::RESOURCE_TYPE, "Resource Type")
            .placeholder("e.g., EBS, RDS, DynamoDB"))
        .dimensions(50, 35)
}

/// Filter parameters for Backup Jobs
#[derive(Default, Clone)]
pub struct BackupJobFilters {
    /// Job state: 0 = All, 1+ = specific states
    pub state_index: usize,
    /// Resource type search
    pub resource_type: String,
}

impl BackupJobFilters {
    pub fn from_modal_state(state: &FilterModalState) -> Self {
        Self {
            state_index: state.get_cycle_index(filter_fields::JOB_STATE),
            resource_type: state.get_text(filter_fields::RESOURCE_TYPE),
        }
    }
    
    pub fn state_filter(&self) -> Option<&'static str> {
        match self.state_index {
            1 => Some("COMPLETED"),
            2 => Some("RUNNING"),
            3 => Some("FAILED"),
            4 => Some("PENDING"),
            _ => None,
        }
    }
    
    pub fn matches(&self, job: &BackupJob) -> bool {
        // State filter
        if let Some(expected_state) = self.state_filter() {
            if job.state.to_uppercase() != expected_state {
                return false;
            }
        }
        
        // Resource type filter
        if !self.resource_type.is_empty() {
            let search = self.resource_type.to_lowercase();
            let job_type = job.resource_type.as_deref().unwrap_or("").to_lowercase();
            if !job_type.contains(&search) {
                return false;
            }
        }
        
        true
    }
    
    pub fn is_empty(&self) -> bool {
        self.state_index == 0 && self.resource_type.is_empty()
    }
}

/// State for AWS Backup service
pub struct BackupState {
    pub vaults: Vec<BackupVault>,
    pub plans: Vec<BackupPlan>,
    pub jobs: Vec<BackupJob>,
    pub recovery_points: Vec<RecoveryPoint>,
    pub list_state: TableState,
    pub view_mode: BackupViewMode,
    
    // Sort state for Jobs
    pub job_sort_field: JobSortField,
    pub job_sort_direction: SortDirection,
    
    // Filter state for Jobs (using generic filter modal)
    pub jobs_filter_config: FilterModalConfig,
    pub jobs_filter_modal: FilterModalState,
    pub jobs_current_filters: BackupJobFilters,
}

impl Default for BackupState {
    fn default() -> Self {
        let config = backup_jobs_filter_config();
        let modal_state = FilterModalState::new(&config);
        Self {
            vaults: Vec::new(),
            plans: Vec::new(),
            jobs: Vec::new(),
            recovery_points: Vec::new(),
            list_state: TableState::default(),
            view_mode: BackupViewMode::default(),
            job_sort_field: JobSortField::default(),
            job_sort_direction: SortDirection::default(),
            jobs_filter_config: config,
            jobs_filter_modal: modal_state,
            jobs_current_filters: BackupJobFilters::default(),
        }
    }
}

impl BackupState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sort jobs based on current sort field and direction
    pub fn sort_jobs(&mut self) {
        let ascending = self.job_sort_direction == SortDirection::Ascending;
        
        self.jobs.sort_by(|a, b| {
            let cmp = match self.job_sort_field {
                JobSortField::CreatedAt => {
                    a.creation_date.cmp(&b.creation_date)
                }
                JobSortField::State => {
                    a.state.cmp(&b.state)
                }
                JobSortField::ResourceType => {
                    a.resource_type.cmp(&b.resource_type)
                }
            };
            
            if ascending { cmp } else { cmp.reverse() }
        });
    }

    /// Get the currently selected vault, if any
    pub fn selected_vault(&self) -> Option<&BackupVault> {
        if self.view_mode == BackupViewMode::Vaults {
            self.list_state.selected().and_then(|i| self.vaults.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected plan, if any
    pub fn selected_plan(&self) -> Option<&BackupPlan> {
        if self.view_mode == BackupViewMode::Plans {
            self.list_state.selected().and_then(|i| self.plans.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected job, if any
    pub fn selected_job(&self) -> Option<&BackupJob> {
        if self.view_mode == BackupViewMode::Jobs {
            self.list_state.selected().and_then(|i| self.jobs.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected recovery point, if any
    pub fn selected_recovery_point(&self) -> Option<&RecoveryPoint> {
        if self.view_mode == BackupViewMode::RecoveryPoints {
            self.list_state.selected().and_then(|i| self.recovery_points.get(i))
        } else {
            None
        }
    }
}

impl crate::app::global_search::Searchable for BackupState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        // Backup Vaults
        for vault in &self.vaults {
            results.push(SearchResult::new(
                Service::Backup,
                "Backup Vault",
                &vault.backup_vault_name,
            ));
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for BackupState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        self.view_mode = BackupViewMode::Vaults;
        if let Some(idx) = self.vaults.iter().position(|v| v.backup_vault_name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for BackupState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.view_mode {
            BackupViewMode::Vaults => self.vaults.len(),
            BackupViewMode::Plans => self.plans.len(),
            BackupViewMode::Jobs => self.jobs.len(),
            BackupViewMode::RecoveryPoints => self.recovery_points.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter => {
                if self.view_mode == BackupViewMode::Vaults {
                    if let Some(vault) = self.selected_vault() {
                        return InputResult::Message(Message::backup_load_recovery_points(
                            vault.backup_vault_name.clone(),
                        ));
                    }
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                if self.view_mode == BackupViewMode::RecoveryPoints {
                    // Go back to vaults
                    return InputResult::Message(Message::backup_leave_vault());
                }
            }
            // Cycle sort field (Jobs view only)
            KeyCode::Char('s') if self.view_mode == BackupViewMode::Jobs => {
                self.job_sort_field = self.job_sort_field.next();
                self.sort_jobs();
            }
            // Toggle sort direction (Jobs view only)
            KeyCode::Char('S') if self.view_mode == BackupViewMode::Jobs => {
                self.job_sort_direction = self.job_sort_direction.toggle();
                self.sort_jobs();
            }
            // Open filter modal (Jobs view only)
            KeyCode::Char('F') if self.view_mode == BackupViewMode::Jobs => {
                return InputResult::Message(Message::backup_open_filter_modal());
            }
            // Clear filters (Jobs view only, when filters active)
            KeyCode::Char('c') if self.view_mode == BackupViewMode::Jobs && !self.jobs_current_filters.is_empty() => {
                return InputResult::Message(Message::backup_clear_filters());
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
            BackupViewMode::Vaults => self.selected_vault().map(|v| v.backup_vault_name.clone()),
            BackupViewMode::Plans => self.selected_plan().map(|p| p.backup_plan_id.clone()),
            BackupViewMode::Jobs => self.selected_job().map(|j| j.backup_job_id.clone()),
            BackupViewMode::RecoveryPoints => self.selected_recovery_point().map(|rp| rp.recovery_point_arn.clone()),
        }
    }
}

impl ServiceInternal for BackupState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.backup.clone();
        let handle = tokio::spawn(async move {
            let service = BackupService::new(client);
            match service.list_backup_vaults().await {
                Ok(vaults) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::BackupVaultsLoaded(vaults))))
                        .await
                        .ok();
                }
                Err(e) => {
                    if report_errors {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                            .await
                            .ok();
                    }
                }
            }
            if let Ok(plans) = service.list_backup_plans().await {
                tx.send(Event::Aws(Box::new(AwsEvent::BackupPlansLoaded(plans))))
                    .await
                    .ok();
            }
            if let Ok(jobs) = service.list_backup_jobs().await {
                tx.send(Event::Aws(Box::new(AwsEvent::BackupJobsLoaded(jobs))))
                    .await
                    .ok();
            }
        });
        tasks.spawn(task_keys::BACKUP_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.vaults.clear();
        self.plans.clear();
        self.jobs.clear();
        self.recovery_points.clear();
        self.view_mode = BackupViewMode::Vaults;
        self.list_state.select(Some(0));
        // Reset sort state
        self.job_sort_field = JobSortField::default();
        self.job_sort_direction = SortDirection::default();
        // Reset filter state
        self.jobs_filter_modal.close();
        self.jobs_filter_modal.clear_all();
        self.jobs_current_filters = BackupJobFilters::default();
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() {
            let has_items = match self.view_mode {
                BackupViewMode::Vaults => !self.vaults.is_empty(),
                BackupViewMode::Plans => !self.plans.is_empty(),
                BackupViewMode::Jobs => !self.jobs.is_empty(),
                BackupViewMode::RecoveryPoints => !self.recovery_points.is_empty(),
            };
            if has_items {
                self.list_state.select(Some(0));
            }
        }
    }

    fn can_cycle_view(&self) -> bool {
        true // Backup supports view mode cycling in all tabs
    }
}
