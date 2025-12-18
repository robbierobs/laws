use ratatui::widgets::TableState;
use crate::models::ecs::{EcsCluster, EcsService, EcsTask, EcsTaskDefinition};
use crate::app::{EcsViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
use crate::app::ecs_modals::{ServiceEditorState, TaskDefSelectorState};
use crossterm::event::{KeyCode, KeyEvent};

/// State for ECS service
#[derive(Default)]
pub struct EcsState {
    // Data
    pub clusters: Vec<EcsCluster>,
    pub services: Vec<EcsService>,
    pub tasks: Vec<EcsTask>,
    pub current_task_definition: Option<EcsTaskDefinition>,

    // Navigation state
    pub list_state: TableState,
    pub view_mode: EcsViewMode,

    // Drill-down tracking
    pub selected_cluster_arn: Option<String>,
    pub selected_service_arn: Option<String>,
    pub selected_service_name: Option<String>,

    // Detail panel scroll
    pub detail_scroll_offset: usize,

    // Task definition selector modal (legacy - to be removed)
    pub show_task_definition_selector: bool,
    pub task_definitions_list: Vec<String>,
    pub task_definitions_list_state: TableState,

    // Pending edit operation (family, path) - for synchronous editor handling
    pub pending_edit: Option<(String, String)>,

    // Modal states (refactored)
    pub service_editor: ServiceEditorState,
    pub task_def_selector: TaskDefSelectorState,
}

impl EcsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected cluster (only valid in Clusters view)
    pub fn selected_cluster(&self) -> Option<&EcsCluster> {
        if self.view_mode != EcsViewMode::Clusters {
            return None;
        }
        self.list_state
            .selected()
            .and_then(|i| self.clusters.get(i))
    }

    /// Get the currently selected service (only valid in Services view)
    pub fn selected_service(&self) -> Option<&EcsService> {
        if self.view_mode != EcsViewMode::Services {
            return None;
        }
        self.list_state
            .selected()
            .and_then(|i| self.services.get(i))
    }

    /// Get the currently selected task (only valid in Tasks view)
    pub fn selected_task(&self) -> Option<&EcsTask> {
        if self.view_mode != EcsViewMode::Tasks {
            return None;
        }
        self.list_state.selected().and_then(|i| self.tasks.get(i))
    }

    /// Get count for current view
    fn current_list_len(&self) -> usize {
        match self.view_mode {
            EcsViewMode::Clusters => self.clusters.len(),
            EcsViewMode::Services => self.services.len(),
            EcsViewMode::Tasks => self.tasks.len(),
            EcsViewMode::TaskDefinition => 0,
        }
    }

    /// Clear tasks and services when navigating back
    pub fn clear_services(&mut self) {
        self.services.clear();
        self.selected_service_arn = None;
        self.selected_service_name = None;
    }

    pub fn clear_tasks(&mut self) {
        self.tasks.clear();
        self.current_task_definition = None;
    }

    /// Prepare the service editor modal with current service values
    pub fn prepare_service_editor(&mut self) {
        // Extract values from selected service first to avoid borrow conflict
        let (service_name, task_def) = {
            if let Some(service) = self.selected_service() {
                (
                    service.service_name.clone(),
                    service.task_definition.clone().unwrap_or_default(),
                )
            } else {
                return;
            }
        };
        
        self.service_editor.init(service_name, task_def);
    }

    /// Reset the service editor state
    pub fn reset_service_editor(&mut self) {
        self.service_editor.reset();
    }

    /// Prepare the task definition selector modal
    pub fn prepare_task_def_selector(&mut self) {
        // Extract service name to avoid borrow conflict
        let service_name = {
            if let Some(service) = self.selected_service() {
                service.service_name.clone()
            } else {
                return;
            }
        };
        
        self.task_def_selector.init(service_name);
    }

    /// Reset the task definition selector state
    pub fn reset_task_def_selector(&mut self) {
        self.task_def_selector.reset();
    }

    /// Get the currently selected task definition in the selector
    #[allow(dead_code)] // For potential future use
    pub fn selected_task_def_in_selector(&self) -> Option<&EcsTaskDefinition> {
        self.task_def_selector.selected()
    }

    /// Navigate up in task definition selector
    #[allow(dead_code)] // May be used for keyboard navigation
    pub fn task_def_selector_up(&mut self) {
        self.task_def_selector.nav_up();
    }

    /// Navigate down in task definition selector
    #[allow(dead_code)] // May be used for keyboard navigation
    pub fn task_def_selector_down(&mut self) {
        self.task_def_selector.nav_down();
    }
}

impl crate::app::global_search::Searchable for EcsState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();

        // ECS Clusters
        for cluster in &self.clusters {
            results.push(SearchResult::new(
                Service::ECS,
                "ECS Cluster",
                &cluster.cluster_name,
            ));
        }

        // ECS Services
        for service in &self.services {
            results.push(SearchResult::new(
                Service::ECS,
                "ECS Service",
                &service.service_name,
            ));
        }

        results
    }
}

impl crate::app::global_search::AutoSelectable for EcsState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        // Check clusters first, then services
        if let Some(idx) = self.clusters.iter().position(|c| c.cluster_name == resource_id) {
            self.view_mode = EcsViewMode::Clusters;
            self.list_state.select(Some(idx));
            return true;
        }
        if let Some(idx) = self.services.iter().position(|s| s.service_name == resource_id) {
            self.view_mode = EcsViewMode::Services;
            self.list_state.select(Some(idx));
            return true;
        }
        false
    }
}

impl ServiceInputHandler for EcsState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match self.view_mode {
            EcsViewMode::Clusters => self.handle_clusters_input(key),
            EcsViewMode::Services => self.handle_services_input(key),
            EcsViewMode::Tasks => self.handle_tasks_input(key),
            EcsViewMode::TaskDefinition => self.handle_task_definition_input(key),
        }
    }

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn get_copiable_text(&self) -> Option<String> {
        match self.view_mode {
            EcsViewMode::Clusters => self.selected_cluster().map(|c| c.cluster_arn.clone()),
            EcsViewMode::Services => self.selected_service().map(|s| s.service_arn.clone()),
            EcsViewMode::Tasks => self.selected_task().map(|t| t.task_arn.clone()),
            EcsViewMode::TaskDefinition => self.current_task_definition.as_ref().map(|td| td.task_definition_arn.clone()),
        }
    }
}

impl ServiceInternal for EcsState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.ecs.clone();
        let handle = spawn_list_task(
            tx,
            move || async move { crate::aws::ecs::EcsClient::new(client).list_clusters().await },
            AwsEvent::EcsClustersLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::ECS_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.clusters.clear();
        self.services.clear();
        self.tasks.clear();
        self.current_task_definition = None;
        self.selected_cluster_arn = None;
        self.selected_service_arn = None;
        self.selected_service_name = None;
        self.view_mode = EcsViewMode::Clusters;
        self.list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() && !self.clusters.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}

impl EcsState {
    fn handle_clusters_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.current_list_len());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.current_list_len());
            }
            KeyCode::Enter => {
                if let Some(cluster) = self.selected_cluster() {
                    return InputResult::Message(Message::ecs_view_services(
                        cluster.cluster_arn.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_services_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.current_list_len());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.current_list_len());
            }
            KeyCode::Enter => {
                // Drill into tasks for this service
                if let Some(service) = self.selected_service() {
                    return InputResult::Message(Message::ecs_view_tasks(
                        service.service_arn.clone(),
                    ));
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                return InputResult::Message(Message::ecs_back_to_clusters());
            }
            // 't' - View task definition for this service
            KeyCode::Char('t') => {
                if let Some(service) = self.selected_service() {
                    if let Some(td) = &service.task_definition {
                        return InputResult::Message(Message::ecs_view_task_definition(td.clone()));
                    }
                }
            }
            // 'd' - Force new deployment
            KeyCode::Char('d') => {
                if let (Some(cluster_arn), Some(service)) =
                    (&self.selected_cluster_arn, self.selected_service())
                {
                    return InputResult::Action(Message::ecs_force_new_deployment(
                        cluster_arn.clone(),
                        service.service_name.clone(),
                    ));
                }
            }
            // '+' - Scale up
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if let (Some(cluster_arn), Some(service)) =
                    (&self.selected_cluster_arn, self.selected_service())
                {
                    let new_count = service.desired_count + 1;
                    return InputResult::Action(Message::ecs_update_desired_count(
                        cluster_arn.clone(),
                        service.service_name.clone(),
                        new_count,
                    ));
                }
            }
            // '-' - Scale down
            KeyCode::Char('-') => {
                if let (Some(cluster_arn), Some(service)) =
                    (&self.selected_cluster_arn, self.selected_service())
                {
                    let new_count = (service.desired_count - 1).max(0);
                    return InputResult::Action(Message::ecs_update_desired_count(
                        cluster_arn.clone(),
                        service.service_name.clone(),
                        new_count,
                    ));
                }
            }
            // 'e' - Edit service (modify task definition, CPU, memory)
            KeyCode::Char('e') => {
                if self.selected_service().is_some() {
                    self.prepare_service_editor();
                    return InputResult::OpenInputMode(crate::app::InputMode::EcsServiceEditor);
                }
            }
            // 'T' - Select task definition (browse task definitions with details)
            KeyCode::Char('T') => {
                if self.selected_service().is_some() {
                    self.prepare_task_def_selector();
                    return InputResult::OpenInputMode(crate::app::InputMode::EcsTaskDefSelector);
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_tasks_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.nav_down(self.current_list_len());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.nav_up(self.current_list_len());
            }
            KeyCode::Esc | KeyCode::Backspace => {
                return InputResult::Message(Message::ecs_back_to_services());
            }
            // 't' - View task definition for this task
            KeyCode::Char('t') | KeyCode::Enter => {
                if let Some(task) = self.selected_task() {
                    return InputResult::Message(Message::ecs_view_task_definition(
                        task.task_definition_arn.clone(),
                    ));
                }
            }
            // 'S' - Stop task
            KeyCode::Char('S') => {
                if let (Some(cluster_arn), Some(task)) =
                    (&self.selected_cluster_arn, self.selected_task())
                {
                    return InputResult::Action(Message::ecs_stop_task(
                        cluster_arn.clone(),
                        task.task_arn.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }

    fn handle_task_definition_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => {
                return InputResult::Message(Message::ecs_back_to_tasks());
            }
            // Scroll in detail view
            KeyCode::Down | KeyCode::Char('j') => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_add(10);
            }
            KeyCode::PageUp => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.detail_scroll_offset = 0;
            }
            // 'E' - Edit task definition
            KeyCode::Char('E') => {
                if let Some(td) = &self.current_task_definition {
                    return InputResult::Action(Message::ecs_edit_task_definition(
                        td.task_definition_arn.clone(),
                    ));
                }
            }
            // 'X' - Deregister task definition
            KeyCode::Char('X') | KeyCode::Delete => {
                if let Some(td) = &self.current_task_definition {
                    return InputResult::Action(Message::ecs_deregister_task_definition(
                        td.task_definition_arn.clone(),
                    ));
                }
            }
            _ => {}
        }
        InputResult::None
    }
}
