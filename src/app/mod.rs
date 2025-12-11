//! Application state and logic module
//! 
//! This module is split into several submodules for maintainability:
//! - `messages`: Enums for services, messages, focus, and input modes
//! - `state`: App struct definition and constructors
//! - `update`: Message handling (reducer/update function)
//! - `input`: Keyboard input handling
//! - `events`: AWS event handling

mod messages;
mod state;
mod update;
mod input;
mod events;

// Re-export everything needed by other modules
pub use messages::{Service, Message, Focus, InputMode, VpcViewMode, IamViewMode, BackupViewMode, CloudTrailViewMode, DynamoDbViewMode};
pub use state::App;
