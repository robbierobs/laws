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

mod messages;
mod state;
mod service_state;
mod update;
mod input;
mod events;
pub mod task_manager;

// Re-export everything needed by other modules
pub use messages::{
    Service, Message, GlobalMessage, ServiceAction,
    Focus, InputMode, 
    VpcViewMode, IamViewMode, BackupViewMode, CloudTrailViewMode, DynamoDbViewMode,
};
pub use state::App;
// Re-export from service_state (only what's needed externally)
pub use service_state::ServiceStates;
// Re-export TaskManager
pub use task_manager::TaskManager;
