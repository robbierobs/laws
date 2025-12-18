//! EC2-specific actions

/// EC2-specific actions
#[derive(Debug, Clone)]
pub enum Ec2Action {
    Start(String),
    Stop(String),
    Reboot(String),
    Terminate(String),
}
