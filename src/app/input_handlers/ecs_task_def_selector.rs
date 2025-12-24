//! ECS Task Definition Selector modal input handling

use crate::app::states::ecs::EcsState;
use crate::app::Message;
use crossterm::event::{KeyCode, KeyEvent};

/// Result of handling ECS task definition selector input
pub enum EcsTaskDefSelectorResult {
    /// Stay in modal, no message
    Continue,
    /// Exit modal, no message (cancelled)
    Cancel,
    /// Exit modal with update service message (needs confirmation)
    SelectWithConfirmation(Message),
}

/// Handle keyboard input for the ECS task definition selector modal.
pub fn handle_ecs_task_def_selector_input(
    ecs_state: &mut EcsState,
    key: KeyEvent,
) -> EcsTaskDefSelectorResult {
    match key.code {
        KeyCode::Esc => {
            ecs_state.reset_task_def_selector();
            EcsTaskDefSelectorResult::Cancel
        }
        KeyCode::Down | KeyCode::Char('j') => {
            ecs_state.task_def_selector.nav_down();
            EcsTaskDefSelectorResult::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            ecs_state.task_def_selector.nav_up();
            EcsTaskDefSelectorResult::Continue
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            // Toggle force new deployment
            ecs_state.task_def_selector.toggle_force_deploy();
            EcsTaskDefSelectorResult::Continue
        }
        KeyCode::PageDown => {
            ecs_state.task_def_selector.scroll_down(5);
            EcsTaskDefSelectorResult::Continue
        }
        KeyCode::PageUp => {
            ecs_state.task_def_selector.scroll_up(5);
            EcsTaskDefSelectorResult::Continue
        }
        KeyCode::Char('l') | KeyCode::Right => {
            // Scroll detail pane down
            ecs_state.task_def_selector.scroll_down(1);
            EcsTaskDefSelectorResult::Continue
        }
        KeyCode::Char('h') | KeyCode::Left => {
            // Scroll detail pane up
            ecs_state.task_def_selector.scroll_up(1);
            EcsTaskDefSelectorResult::Continue
        }
        KeyCode::Enter => {
            // Submit the selection - request confirmation
            if let (Some(cluster_arn), Some(service_name), Some(task_def)) = (
                ecs_state.selected_cluster_arn.clone(),
                ecs_state.task_def_selector.service_name.clone(),
                ecs_state
                    .task_def_selector
                    .selected()
                    .map(|td| td.task_definition_arn.clone()),
            ) {
                let force_deploy = ecs_state.task_def_selector.force_deploy;
                ecs_state.reset_task_def_selector();

                EcsTaskDefSelectorResult::SelectWithConfirmation(Message::ecs_update_service(
                    cluster_arn,
                    service_name,
                    Some(task_def),
                    None,
                    None,
                    force_deploy,
                ))
            } else {
                EcsTaskDefSelectorResult::Continue
            }
        }
        _ => EcsTaskDefSelectorResult::Continue,
    }
}
