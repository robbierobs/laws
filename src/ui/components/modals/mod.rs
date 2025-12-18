//! Modal rendering components
//!
//! This module provides all modal dialogs used in the application.
//! Each modal is in its own submodule for better organization.
//!
//! ## Module Structure
//!
//! - `confirmation` - Confirmation dialog for destructive actions
//! - `object_viewer` - S3 object content viewer with text/hex modes
//! - `s3` - S3 bucket creation modal
//! - `profile_switcher` - AWS profile and region selection modals
//! - `ecs` - ECS service editor and task definition selector
//! - `global_search` - Cross-service resource search modal
//! - `helpers` - Common utilities (centering, hex dump formatting)

pub mod confirmation;
pub mod ecs;
pub mod global_search;
pub mod helpers;
pub mod object_viewer;
pub mod profile_switcher;
pub mod s3;

// Re-export commonly used functions at the module level for backwards compatibility
pub use confirmation::render as render_confirmation_modal;
pub use ecs::{render_service_editor as render_ecs_service_editor_modal, render_task_def_selector as render_task_def_selector_modal};
pub use global_search::render as render_global_search_modal;
#[allow(unused_imports)]
pub use helpers::{centered_rect, centered_rect_fixed};
pub use object_viewer::render as render_object_viewer_modal;
pub use profile_switcher::{render_profile_switcher as render_profile_switcher_modal, render_region_switcher as render_region_switcher_modal};
pub use s3::render_bucket_creation as render_s3_bucket_creation_modal;
