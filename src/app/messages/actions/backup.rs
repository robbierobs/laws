//! Backup-specific actions

use super::super::confirmable::ConfirmableAction;

/// Backup-specific actions
#[derive(Debug, Clone)]
pub enum BackupAction {
    LoadRecoveryPoints(String),
    LeaveVault,
}

impl ConfirmableAction for BackupAction {
    fn confirmation_description(&self) -> String {
        "Backup operation".to_string()
    }
}
