//! Backup-specific actions

use super::super::confirmable::ConfirmableAction;

/// Backup-specific actions
#[derive(Debug, Clone)]
pub enum BackupAction {
    LoadRecoveryPoints(String),
    LeaveVault,
    /// Open the filter modal (Jobs view)
    OpenFilterModal,
    #[allow(dead_code)] // TODO: Wire up backup filter modal
    /// Close the filter modal without applying
    CloseFilterModal,
    /// Apply filters from modal
    ApplyFilters,
    /// Clear all filters
    ClearFilters,
}

impl ConfirmableAction for BackupAction {
    fn confirmation_description(&self) -> String {
        "Backup operation".to_string()
    }
}
