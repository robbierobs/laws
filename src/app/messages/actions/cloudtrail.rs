//! CloudTrail-specific actions

use super::super::confirmable::ConfirmableAction;

/// Parameters for CloudTrail event lookup
#[derive(Debug, Clone, Default)]
pub struct CloudTrailLookupParams {
    /// Start time for event lookup (ISO 8601 format, e.g., "2024-01-15T00:00:00Z")
    pub start_time: Option<String>,
    /// End time for event lookup (ISO 8601 format)
    pub end_time: Option<String>,
    /// Filter by event source (e.g., "s3.amazonaws.com", "ec2.amazonaws.com")
    pub event_source: Option<String>,
    /// Filter by event name (e.g., "CreateBucket", "RunInstances")
    pub event_name: Option<String>,
    /// Filter by username
    pub username: Option<String>,
    /// Filter by resource type (e.g., "AWS::S3::Bucket")
    pub resource_type: Option<String>,
    /// Filter by resource name
    pub resource_name: Option<String>,
    /// Whether to show read-only events (None = all, Some(true) = read-only, Some(false) = write-only)
    pub read_only: Option<bool>,
}

impl CloudTrailLookupParams {
    pub fn is_empty(&self) -> bool {
        self.start_time.is_none()
            && self.end_time.is_none()
            && self.event_source.is_none()
            && self.event_name.is_none()
            && self.username.is_none()
            && self.resource_type.is_none()
            && self.resource_name.is_none()
            && self.read_only.is_none()
    }
}

/// CloudTrail-specific actions
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved variants for future UI improvements
pub enum CloudTrailAction {
    ShowEventDetails(String),
    CloseEventDetails,
    DeleteTrail(String),
    /// Load more events (pagination) using the stored next_token
    LoadMoreEvents,
    /// Open the event filter modal
    OpenFilterModal,
    /// Close the event filter modal without applying
    CloseFilterModal,
    /// Apply filters and reload events
    ApplyFilters(CloudTrailLookupParams),
    /// Clear all filters and reload events
    ClearFilters,
    /// Refresh events with current filters
    RefreshEvents,
}

impl ConfirmableAction for CloudTrailAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::DeleteTrail(name) => format!("Delete CloudTrail trail {}", name),
            _ => "CloudTrail operation".to_string(),
        }
    }
}
