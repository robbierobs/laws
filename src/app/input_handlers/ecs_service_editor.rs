//! ECS Service Editor modal input handling

use crate::app::states::ecs::EcsState;
use crate::app::Message;
use crossterm::event::{KeyCode, KeyEvent};

/// Result of handling ECS service editor input
pub enum EcsEditorResult {
    /// Stay in modal, no message
    Continue,
    /// Exit modal, no message (cancelled)
    Cancel,
    /// Exit modal with update service message
    Update(Message),
    /// Show error message, stay in modal
    Error(String),
}

/// Handle keyboard input for the ECS service editor modal.
pub fn handle_ecs_service_editor_input(ecs_state: &mut EcsState, key: KeyEvent) -> EcsEditorResult {
    match key.code {
        KeyCode::Esc => {
            ecs_state.reset_service_editor();
            EcsEditorResult::Cancel
        }
        KeyCode::Tab | KeyCode::Down => {
            ecs_state.service_editor.next_field();
            EcsEditorResult::Continue
        }
        KeyCode::BackTab | KeyCode::Up => {
            ecs_state.service_editor.prev_field();
            EcsEditorResult::Continue
        }
        KeyCode::Char(' ') => {
            // Toggle force deploy if on that field
            if ecs_state.service_editor.active_field == 3 {
                ecs_state.service_editor.toggle_force_deploy();
            }
            EcsEditorResult::Continue
        }
        KeyCode::Backspace => {
            match ecs_state.service_editor.active_field {
                0 => {
                    ecs_state.service_editor.task_def.pop();
                }
                1 => {
                    ecs_state.service_editor.cpu.pop();
                }
                2 => {
                    ecs_state.service_editor.memory.pop();
                }
                _ => {}
            }
            EcsEditorResult::Continue
        }
        KeyCode::Char(c) => {
            match ecs_state.service_editor.active_field {
                0 => ecs_state.service_editor.task_def.push(c),
                1 => {
                    // Only allow digits for CPU
                    if c.is_ascii_digit() {
                        ecs_state.service_editor.cpu.push(c);
                    }
                }
                2 => {
                    // Only allow digits for memory
                    if c.is_ascii_digit() {
                        ecs_state.service_editor.memory.push(c);
                    }
                }
                _ => {}
            }
            EcsEditorResult::Continue
        }
        KeyCode::Enter => {
            // Submit the update
            if let (Some(cluster_arn), Some(service_name)) = (
                ecs_state.selected_cluster_arn.clone(),
                ecs_state.service_editor.service_name.clone(),
            ) {
                let task_def = if ecs_state.service_editor.task_def.is_empty() {
                    None
                } else {
                    Some(ecs_state.service_editor.task_def.clone())
                };
                let cpu = if ecs_state.service_editor.cpu.is_empty() {
                    None
                } else {
                    Some(ecs_state.service_editor.cpu.clone())
                };
                let memory = if ecs_state.service_editor.memory.is_empty() {
                    None
                } else {
                    Some(ecs_state.service_editor.memory.clone())
                };
                let force_deploy = ecs_state.service_editor.force_deploy;

                // Only submit if at least one field has a value
                if task_def.is_some() || cpu.is_some() || memory.is_some() {
                    ecs_state.reset_service_editor();
                    EcsEditorResult::Update(Message::ecs_update_service(
                        cluster_arn,
                        service_name,
                        task_def,
                        cpu,
                        memory,
                        force_deploy,
                    ))
                } else {
                    EcsEditorResult::Error("Please specify at least one change".to_string())
                }
            } else {
                EcsEditorResult::Continue
            }
        }
        _ => EcsEditorResult::Continue,
    }
}
