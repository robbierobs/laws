//! RDS-specific actions

/// RDS-specific actions
#[derive(Debug, Clone)]
pub enum RdsAction {
    Start(String),
    Stop(String),
    Reboot(String),
    Delete(String),
}
