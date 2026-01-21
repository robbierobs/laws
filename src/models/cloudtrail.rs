use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trail {
    pub name: String,
    pub s3_bucket_name: Option<String>,
    pub s3_key_prefix: Option<String>,
    pub home_region: Option<String>,
    pub is_multi_region_trail: bool,
    pub is_organization_trail: bool,
    pub log_file_validation_enabled: bool,
    pub include_global_service_events: bool,
    pub trail_arn: Option<String>,
    pub has_custom_event_selectors: bool,
    pub has_insight_selectors: bool,
    pub kms_key_id: Option<String>,
    pub cloudwatch_logs_log_group_arn: Option<String>,
}

impl Trail {
    pub fn from_aws(trail: &aws_sdk_cloudtrail::types::Trail) -> Self {
        Self {
            name: trail.name().unwrap_or_default().to_string(),
            s3_bucket_name: trail.s3_bucket_name().map(|s| s.to_string()),
            s3_key_prefix: trail.s3_key_prefix().map(|s| s.to_string()),
            home_region: trail.home_region().map(|s| s.to_string()),
            is_multi_region_trail: trail.is_multi_region_trail().unwrap_or(false),
            is_organization_trail: trail.is_organization_trail().unwrap_or(false),
            log_file_validation_enabled: trail.log_file_validation_enabled().unwrap_or(false),
            include_global_service_events: trail.include_global_service_events().unwrap_or(false),
            trail_arn: trail.trail_arn().map(|s| s.to_string()),
            has_custom_event_selectors: trail.has_custom_event_selectors().unwrap_or(false),
            has_insight_selectors: trail.has_insight_selectors().unwrap_or(false),
            kms_key_id: trail.kms_key_id().map(|s| s.to_string()),
            cloudwatch_logs_log_group_arn: trail
                .cloud_watch_logs_log_group_arn()
                .map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTrailEvent {
    pub event_id: Option<String>,
    pub event_name: Option<String>,
    pub event_source: Option<String>,
    pub event_time: Option<String>,
    pub username: Option<String>,
    pub resources: Vec<String>,
    pub read_only: Option<String>,
}

impl CloudTrailEvent {
    pub fn from_aws(event: &aws_sdk_cloudtrail::types::Event) -> Self {
        let resources: Vec<String> = event
            .resources()
            .iter()
            .filter_map(|r| r.resource_name().map(|s| s.to_string()))
            .collect();

        Self {
            event_id: event.event_id().map(|s| s.to_string()),
            event_name: event.event_name().map(|s| s.to_string()),
            event_source: event.event_source().map(|s| s.to_string()),
            event_time: event.event_time().map(|t| t.to_string()),
            username: event.username().map(|s| s.to_string()),
            resources,
            read_only: event.read_only().map(|s| s.to_string()),
        }
    }
}

impl crate::models::Filterable for Trail {
    fn matches_filter(&self, filter: &str) -> bool {
        self.name.to_lowercase().contains(filter)
            || self
                .s3_bucket_name
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}

impl crate::models::Filterable for CloudTrailEvent {
    fn matches_filter(&self, filter: &str) -> bool {
        self.event_name
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains(filter)
            || self
                .event_source
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
            || self
                .username
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}
