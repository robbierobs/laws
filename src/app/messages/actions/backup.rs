//! Backup-specific actions

/// Backup-specific actions
#[derive(Debug, Clone)]
pub enum BackupAction {
    LoadRecoveryPoints(String),
    LeaveVault,
}
