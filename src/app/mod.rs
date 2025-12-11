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

mod events;
pub mod filtered_list;
mod input;
mod messages;
mod service_state;
mod state;
pub mod task_manager;
pub mod update;
mod view_mode;

// Re-export everything needed by other modules
pub use messages::{
    BackupViewMode, CloudTrailViewMode, DynamoDbViewMode, Focus, GlobalMessage, IamViewMode,
    InputMode, Message, Service, ServiceAction, VpcViewMode,
};
pub use state::App;
pub use view_mode::ViewMode;
// Re-export from service_state (for external use - suppressed unused warning)
#[allow(unused_imports)]
pub use service_state::ServiceStates;
// Re-export TaskManager (for external use)
#[allow(unused_imports)]
pub use task_manager::TaskManager;
// Re-export FilteredList (for external use)
#[allow(unused_imports)]
pub use filtered_list::FilteredList;

use crossterm::event::KeyEvent;

/// Result from service input handlers
#[derive(Debug)]
pub enum InputResult {
    None,
    Message(Message),
    Action(Message), // Action that needs confirmation
}

/// Trait for service state that handles its own input
pub trait ServiceInputHandler {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult;
}
