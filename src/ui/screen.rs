//! Screen trait for service-specific rendering
//!
//! This trait enables polymorphic dispatch for service screens.

use ratatui::{layout::Rect, Frame};
use crossterm::event::KeyEvent;
use crate::app::{App, InputResult};

/// Trait for service screens that can render a list and optional detail panel
pub trait Screen {
    /// Render the screen content
    ///
    /// # Arguments
    /// * `frame` - The frame to render to
    /// * `list_area` - Area for the main list/table view
    /// * `detail_area` - Optional area for detail panel
    /// * `app` - Mutable reference to app state (for TableState updates)
    fn render(&self, frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App);
}

/// Optional trait for screens that want to handle their own input
/// 
/// This allows screens to encapsulate their input handling logic,
/// reducing the complexity of the main input handler and improving testability.
/// 
/// # Example
/// ```ignore
/// impl InputHandler for Ec2Screen {
///     fn handle_key(&self, key: KeyEvent, app: &mut App) -> InputResult {
///         match key.code {
///             KeyCode::Char('s') => {
///                 if let Some(instance_id) = app.services.ec2.selected_instance_id() {
///                     InputResult::Action(Message::ec2_start(instance_id))
///                 } else {
///                     InputResult::None
///                 }
///             }
///             _ => InputResult::None,
///         }
///     }
/// }
/// ```
pub trait InputHandler {
    /// Handle keyboard input for this screen
    /// 
    /// Returns an InputResult indicating what action should be taken.
    fn handle_key(&self, key: KeyEvent, app: &mut App) -> InputResult;
}
