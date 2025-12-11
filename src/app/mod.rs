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

mod messages;
mod state;
mod service_state;
mod update;
mod input;
mod events;
pub mod task_manager;
pub mod filtered_list;

// Re-export everything needed by other modules
pub use messages::{
    Service, Message, GlobalMessage, ServiceAction,
    Focus, InputMode, 
    VpcViewMode, IamViewMode, BackupViewMode, CloudTrailViewMode, DynamoDbViewMode,
};
pub use state::App;
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
