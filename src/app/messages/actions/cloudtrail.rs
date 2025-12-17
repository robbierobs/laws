//! CloudTrail-specific actions

/// CloudTrail-specific actions
#[derive(Debug, Clone)]
pub enum CloudTrailAction {
    ShowEventDetails(String),
    CloseEventDetails,
    DeleteTrail(String),
}
