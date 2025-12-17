//! Modal-specific input handlers
//!
//! This module contains extracted input handlers for various modal dialogs.
//! Each modal handler returns `Option<Message>` to indicate whether the event
//! was handled (None = consumed, Some(msg) = message to process).

mod action_log;
mod s3_bucket_creation;
mod s3_viewer;

pub use action_log::{handle_action_log_input, ActionLogState};
pub use s3_bucket_creation::{handle_s3_bucket_creation_input, process_bucket_creation_result};
pub use s3_viewer::handle_s3_viewer_input;
