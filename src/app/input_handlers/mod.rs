//! Modal-specific input handlers
//!
//! This module contains extracted input handlers for various modal dialogs.
//! Each modal handler returns `Option<Message>` to indicate whether the event
//! was handled (None = consumed, Some(msg) = message to process).

mod s3_viewer;

pub use s3_viewer::handle_s3_viewer_input;
