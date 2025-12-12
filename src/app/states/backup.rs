use ratatui::widgets::TableState;
use crate::models::backup::{BackupJob, BackupPlan, BackupVault, RecoveryPoint};
use crate::app::{BackupViewMode, InputResult, Message, ServiceInputHandler, TableStateExt};
use crossterm::event::{KeyCode, KeyEvent};

/// State for AWS Backup service
#[derive(Default)]
pub struct BackupState {
    pub vaults: Vec<BackupVault>,
    pub plans: Vec<BackupPlan>,
    pub jobs: Vec<BackupJob>,
    pub recovery_points: Vec<RecoveryPoint>,
    pub list_state: TableState,
    pub view_mode: BackupViewMode,
}

impl BackupState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected vault, if any
    pub fn selected_vault(&self) -> Option<&BackupVault> {
        if self.view_mode == BackupViewMode::Vaults {
            self.list_state.selected().and_then(|i| self.vaults.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected plan, if any
    pub fn selected_plan(&self) -> Option<&BackupPlan> {
        if self.view_mode == BackupViewMode::Plans {
            self.list_state.selected().and_then(|i| self.plans.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected job, if any
    pub fn selected_job(&self) -> Option<&BackupJob> {
        if self.view_mode == BackupViewMode::Jobs {
            self.list_state.selected().and_then(|i| self.jobs.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected recovery point, if any
    pub fn selected_recovery_point(&self) -> Option<&RecoveryPoint> {
        if self.view_mode == BackupViewMode::RecoveryPoints {
            self.list_state.selected().and_then(|i| self.recovery_points.get(i))
        } else {
            None
        }
    }
}

impl ServiceInputHandler for BackupState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        let len = match self.view_mode {
            BackupViewMode::Vaults => self.vaults.len(),
            BackupViewMode::Plans => self.plans.len(),
            BackupViewMode::Jobs => self.jobs.len(),
            BackupViewMode::RecoveryPoints => self.recovery_points.len(),
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(len),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(len),
            KeyCode::Enter => {
                if self.view_mode == BackupViewMode::Vaults {
                    if let Some(vault) = self.selected_vault() {
                        return InputResult::Message(Message::backup_load_recovery_points(
                            vault.backup_vault_name.clone(),
                        ));
                    }
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                if self.view_mode == BackupViewMode::RecoveryPoints {
                    // Go back to vaults
                    return InputResult::Message(Message::backup_leave_vault());
                }
            }
            _ => {}
        }
        InputResult::None
    }
}
