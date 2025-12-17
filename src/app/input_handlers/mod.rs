//! Modal-specific input handlers
//!
//! This module contains extracted input handlers for various modal dialogs.
//! Each modal handler returns `Option<Message>` to indicate whether the event
//! was handled (None = consumed, Some(msg) = message to process).

mod action_log;
mod ecs_service_editor;
mod ecs_task_def_selector;
mod global_search;
mod s3_bucket_creation;
mod s3_viewer;

pub use action_log::{handle_action_log_input, ActionLogState};
pub use ecs_service_editor::{handle_ecs_service_editor_input, EcsEditorResult};
pub use ecs_task_def_selector::{handle_ecs_task_def_selector_input, EcsTaskDefSelectorResult};
pub use global_search::{handle_global_search_input, GlobalSearchInputResult};
pub use s3_bucket_creation::{handle_s3_bucket_creation_input, process_bucket_creation_result};
pub use s3_viewer::handle_s3_viewer_input;
