//! ECS Modal State Components
//!
//! This module contains state structs for ECS-related modals, extracted from
//! EcsState to reduce its complexity and improve maintainability.

#![allow(dead_code)]

use crate::app::navigation::{nav_down, nav_up};
use crate::models::ecs::EcsTaskDefinition;

// ============================================================================
// Service Editor Modal State
// ============================================================================

/// State for the ECS Service Editor modal
///
/// This modal allows editing service configuration including:
/// - Task definition ARN
/// - CPU allocation
/// - Memory allocation
/// - Force new deployment flag
#[derive(Debug, Clone, Default)]
pub struct ServiceEditorState {
    /// Name of the service being edited
    pub service_name: Option<String>,
    /// Currently active input field (0=task_def, 1=cpu, 2=memory)
    pub active_field: usize,
    /// Task definition ARN input
    pub task_def: String,
    /// CPU allocation input (e.g., "256", "512", "1024")
    pub cpu: String,
    /// Memory allocation input (e.g., "512", "1024", "2048")
    pub memory: String,
    /// Whether to force a new deployment
    pub force_deploy: bool,
}

impl ServiceEditorState {
    /// Number of editable fields in the modal
    pub const FIELD_COUNT: usize = 3;
    
    /// Field indices
    pub const FIELD_TASK_DEF: usize = 0;
    pub const FIELD_CPU: usize = 1;
    pub const FIELD_MEMORY: usize = 2;

    /// Create a new service editor state
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the editor with values from a service
    pub fn init(&mut self, service_name: String, task_def: String) {
        self.service_name = Some(service_name);
        self.task_def = task_def;
        self.cpu = String::new();
        self.memory = String::new();
        self.force_deploy = true;
        self.active_field = Self::FIELD_TASK_DEF;
    }

    /// Reset the editor state
    pub fn reset(&mut self) {
        self.service_name = None;
        self.task_def.clear();
        self.cpu.clear();
        self.memory.clear();
        self.force_deploy = true;
        self.active_field = Self::FIELD_TASK_DEF;
    }

    /// Check if the modal is active
    pub fn is_active(&self) -> bool {
        self.service_name.is_some()
    }

    /// Move to the next field
    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % Self::FIELD_COUNT;
    }

    /// Move to the previous field
    pub fn prev_field(&mut self) {
        self.active_field = if self.active_field == 0 {
            Self::FIELD_COUNT - 1
        } else {
            self.active_field - 1
        };
    }

    /// Toggle the force deploy flag
    pub fn toggle_force_deploy(&mut self) {
        self.force_deploy = !self.force_deploy;
    }

    /// Get a mutable reference to the current field's value
    pub fn current_field_mut(&mut self) -> &mut String {
        match self.active_field {
            Self::FIELD_TASK_DEF => &mut self.task_def,
            Self::FIELD_CPU => &mut self.cpu,
            Self::FIELD_MEMORY => &mut self.memory,
            _ => &mut self.task_def, // fallback
        }
    }

    /// Get the label for a field
    pub fn field_label(field: usize) -> &'static str {
        match field {
            Self::FIELD_TASK_DEF => "Task Definition",
            Self::FIELD_CPU => "CPU",
            Self::FIELD_MEMORY => "Memory",
            _ => "Unknown",
        }
    }
}

// ============================================================================
// Task Definition Selector Modal State
// ============================================================================

/// State for the ECS Task Definition Selector modal
///
/// This modal allows browsing and selecting from available task definition
/// revisions, with a detail panel showing the full definition.
#[derive(Debug, Clone, Default)]
pub struct TaskDefSelectorState {
    /// Name of the service to update
    pub service_name: Option<String>,
    /// List of available task definitions
    pub list: Vec<EcsTaskDefinition>,
    /// Currently selected index in the list
    pub index: usize,
    /// Whether to force a new deployment when applying
    pub force_deploy: bool,
    /// Scroll offset in the detail panel
    pub detail_scroll: usize,
    /// Whether task definitions are currently loading
    pub loading: bool,
}

impl TaskDefSelectorState {
    /// Create a new task definition selector state
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the selector for a service
    pub fn init(&mut self, service_name: String) {
        self.service_name = Some(service_name);
        self.list.clear();
        self.index = 0;
        self.force_deploy = true;
        self.detail_scroll = 0;
        self.loading = true;
    }

    /// Reset the selector state
    pub fn reset(&mut self) {
        self.service_name = None;
        self.list.clear();
        self.index = 0;
        self.force_deploy = true;
        self.detail_scroll = 0;
        self.loading = false;
    }

    /// Check if the modal is active
    pub fn is_active(&self) -> bool {
        self.service_name.is_some()
    }

    /// Get the currently selected task definition
    pub fn selected(&self) -> Option<&EcsTaskDefinition> {
        self.list.get(self.index)
    }

    /// Navigate up in the list
    pub fn nav_up(&mut self) {
        if let Some(new_idx) = nav_up(Some(self.index), self.list.len()) {
            self.index = new_idx;
            self.detail_scroll = 0; // Reset scroll when changing selection
        }
    }

    /// Navigate down in the list
    pub fn nav_down(&mut self) {
        if let Some(new_idx) = nav_down(Some(self.index), self.list.len()) {
            self.index = new_idx;
            self.detail_scroll = 0; // Reset scroll when changing selection
        }
    }

    /// Scroll the detail panel up
    pub fn scroll_up(&mut self, amount: usize) {
        self.detail_scroll = self.detail_scroll.saturating_sub(amount);
    }

    /// Scroll the detail panel down
    pub fn scroll_down(&mut self, amount: usize) {
        self.detail_scroll = self.detail_scroll.saturating_add(amount);
    }

    /// Toggle the force deploy flag
    pub fn toggle_force_deploy(&mut self) {
        self.force_deploy = !self.force_deploy;
    }

    /// Set the list of task definitions (after loading)
    pub fn set_list(&mut self, list: Vec<EcsTaskDefinition>) {
        self.list = list;
        self.index = 0;
        self.loading = false;
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Get the number of items in the list
    pub fn len(&self) -> usize {
        self.list.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a mock task definition for testing
    fn mock_task_definition(revision: i32) -> EcsTaskDefinition {
        EcsTaskDefinition {
            task_definition_arn: format!("arn:aws:ecs:us-east-1:123456789:task-definition/test:{}", revision),
            family: "test".to_string(),
            revision,
            status: "ACTIVE".to_string(),
            task_role_arn: None,
            execution_role_arn: None,
            network_mode: Some("awsvpc".to_string()),
            cpu: Some("256".to_string()),
            memory: Some("512".to_string()),
            requires_compatibilities: vec!["FARGATE".to_string()],
            runtime_platform: None,
            container_definitions: vec![],
            volumes: vec![],
            placement_constraints: vec![],
            registered_at: None,
            registered_by: None,
            pid_mode: None,
            ipc_mode: None,
            proxy_configuration: None,
            ephemeral_storage_size_gib: None,
        }
    }

    #[test]
    fn test_service_editor_field_navigation() {
        let mut editor = ServiceEditorState::new();
        assert_eq!(editor.active_field, 0);

        editor.next_field();
        assert_eq!(editor.active_field, 1);

        editor.next_field();
        assert_eq!(editor.active_field, 2);

        editor.next_field();
        assert_eq!(editor.active_field, 0); // wraps

        editor.prev_field();
        assert_eq!(editor.active_field, 2); // wraps back
    }

    #[test]
    fn test_service_editor_reset() {
        let mut editor = ServiceEditorState::new();
        editor.init("my-service".to_string(), "arn:aws:ecs:...".to_string());
        
        assert!(editor.is_active());
        assert_eq!(editor.service_name, Some("my-service".to_string()));

        editor.reset();
        assert!(!editor.is_active());
        assert!(editor.task_def.is_empty());
    }

    #[test]
    fn test_task_def_selector_navigation() {
        let mut selector = TaskDefSelectorState::new();
        selector.init("my-service".to_string());
        
        // Simulate loading task definitions
        selector.set_list(vec![
            mock_task_definition(1),
            mock_task_definition(2),
            mock_task_definition(3),
        ]);

        assert_eq!(selector.index, 0);
        assert!(!selector.loading);

        selector.nav_down();
        assert_eq!(selector.index, 1);

        selector.nav_down();
        assert_eq!(selector.index, 2);

        selector.nav_down();
        assert_eq!(selector.index, 0); // wraps

        selector.nav_up();
        assert_eq!(selector.index, 2); // wraps back
    }

    #[test]
    fn test_task_def_selector_scroll() {
        let mut selector = TaskDefSelectorState::new();
        
        selector.scroll_down(10);
        assert_eq!(selector.detail_scroll, 10);

        selector.scroll_up(5);
        assert_eq!(selector.detail_scroll, 5);

        selector.scroll_up(10); // Should saturate at 0
        assert_eq!(selector.detail_scroll, 0);
    }
}
