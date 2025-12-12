//! Application state and logic module
//!
//! This module is split into several submodules for maintainability:
//! - `messages`: Enums for services, messages, focus, and input modes
//! - `state`: App struct definition and constructors
//! - `service_state`: Per-service state structs consolidating service-specific fields
//! - `update`: Message handling (reducer/update function)
//! - `input`: Keyboard input handling
//! - `events`: AWS event handling
//! - `task_manager`: Async task tracking and cancellation
//! - `filtered_list`: Generic filtered list with caching
//! - `navigation`: Reusable list navigation helpers
//! - `ecs_modals`: ECS-specific modal state structs

pub mod ecs_modals;
mod events;
pub mod filtered_list;
mod input;
pub mod messages;
pub mod navigation;
pub mod states;
#[allow(deprecated)]
pub mod service_state;
mod state;
pub mod task_manager;
pub mod update;
mod view_mode;

// Re-export everything needed by other modules
pub use messages::{
    BackupViewMode, CloudTrailViewMode, DynamoDbViewMode, EcsViewMode, EcrViewMode, Focus, GlobalMessage,
    IamViewMode, InputMode, Message, Service, ServiceAction, VpcViewMode,
};
pub use state::App;
pub use view_mode::ViewMode;
// Re-export from service_state (for external use - suppressed unused warning)
#[allow(unused_imports)]
pub use service_state::ServiceStates;

/// Type alias for bounded event sender (used throughout update handlers)
pub type EventSender = crate::event::EventSender;
// Re-export TaskManager (for external use)
#[allow(unused_imports)]
pub use task_manager::TaskManager;
// Re-export FilteredList (for external use)
#[allow(unused_imports)]
pub use filtered_list::FilteredList;
// Re-export navigation helpers
pub use navigation::TableStateExt;

use crossterm::event::KeyEvent;

/// Result from service input handlers
#[derive(Debug)]
pub enum InputResult {
    None,
    Message(Message),
    Action(Message), // Action that needs confirmation
    OpenInputMode(InputMode), // Open a specific input mode (modal)
}

/// Trait for service state that handles its own input
pub trait ServiceInputHandler {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult;
    fn reset_selection(&mut self);
    fn get_copiable_text(&self) -> Option<String>;
}
