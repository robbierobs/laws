use crate::error::AppResult;
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::utils::error::format_sdk_error;
use crate::app::messages::CloudTrailLookupParams;
use aws_sdk_cloudtrail::Client;
use aws_sdk_cloudtrail::types::{LookupAttribute, LookupAttributeKey};

// Use macro to generate struct and constructor
crate::aws_service_struct!(CloudTrailService, Client);

/// Result from looking up CloudTrail events
#[derive(Debug)]
pub struct CloudTrailEventsResult {
    pub events: Vec<CloudTrailEvent>,
    pub next_token: Option<String>,
}

impl CloudTrailService {

    pub async fn list_trails(&self) -> AppResult<Vec<Trail>> {
        let response = self
            .client
            .describe_trails()
            .send()
            .await
            .map_err(|e| format_sdk_error("CloudTrail", "describe_trails", "all", e))?;

        let trails = response
            .trail_list()
            .iter()
            .map(Trail::from_aws)
            .collect();

        Ok(trails)
    }

    /// Lookup CloudTrail events with optional filtering and pagination
    pub async fn lookup_events(
        &self,
        max_results: i32,
        params: Option<&CloudTrailLookupParams>,
        next_token: Option<&str>,
    ) -> AppResult<CloudTrailEventsResult> {
        let mut request = self.client.lookup_events().max_results(max_results);
        
        // Add pagination token if provided
        if let Some(token) = next_token {
            request = request.next_token(token);
        }
        
        // Add date filters if provided
        if let Some(params) = params {
            if let Some(ref start) = params.start_time {
                if let Ok(dt) = parse_iso_datetime(start) {
                    request = request.start_time(dt);
                }
            }
            if let Some(ref end) = params.end_time {
                if let Ok(dt) = parse_iso_datetime(end) {
                    request = request.end_time(dt);
                }
            }
            
            // Add lookup attributes (AWS allows only one at a time, so we prioritize)
            if let Some(ref event_name) = params.event_name {
                let attr = LookupAttribute::builder()
                    .attribute_key(LookupAttributeKey::EventName)
                    .attribute_value(event_name)
                    .build()
                    .map_err(|e| crate::error::AppError::validation(format!("Failed to build lookup attribute: {}", e)))?;
                request = request.lookup_attributes(attr);
            } else if let Some(ref event_source) = params.event_source {
                let attr = LookupAttribute::builder()
                    .attribute_key(LookupAttributeKey::EventSource)
                    .attribute_value(event_source)
                    .build()
                    .map_err(|e| crate::error::AppError::validation(format!("Failed to build lookup attribute: {}", e)))?;
                request = request.lookup_attributes(attr);
            } else if let Some(ref username) = params.username {
                let attr = LookupAttribute::builder()
                    .attribute_key(LookupAttributeKey::Username)
                    .attribute_value(username)
                    .build()
                    .map_err(|e| crate::error::AppError::validation(format!("Failed to build lookup attribute: {}", e)))?;
                request = request.lookup_attributes(attr);
            } else if let Some(ref resource_type) = params.resource_type {
                let attr = LookupAttribute::builder()
                    .attribute_key(LookupAttributeKey::ResourceType)
                    .attribute_value(resource_type)
                    .build()
                    .map_err(|e| crate::error::AppError::validation(format!("Failed to build lookup attribute: {}", e)))?;
                request = request.lookup_attributes(attr);
            } else if let Some(ref resource_name) = params.resource_name {
                let attr = LookupAttribute::builder()
                    .attribute_key(LookupAttributeKey::ResourceName)
                    .attribute_value(resource_name)
                    .build()
                    .map_err(|e| crate::error::AppError::validation(format!("Failed to build lookup attribute: {}", e)))?;
                request = request.lookup_attributes(attr);
            } else if let Some(read_only) = params.read_only {
                let value = if read_only { "true" } else { "false" };
                let attr = LookupAttribute::builder()
                    .attribute_key(LookupAttributeKey::ReadOnly)
                    .attribute_value(value)
                    .build()
                    .map_err(|e| crate::error::AppError::validation(format!("Failed to build lookup attribute: {}", e)))?;
                request = request.lookup_attributes(attr);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format_sdk_error("CloudTrail", "lookup_events", "all", e))?;

        let events = response
            .events()
            .iter()
            .map(CloudTrailEvent::from_aws)
            .collect();

        Ok(CloudTrailEventsResult {
            events,
            next_token: response.next_token().map(|s| s.to_string()),
        })
    }

    pub async fn delete_trail(&self, name: &str) -> AppResult<()> {
        self.client
            .delete_trail()
            .name(name)
            .send()
            .await
            .map_err(|e| format_sdk_error("CloudTrail", "delete_trail", name, e))?;
        Ok(())
    }
}

impl crate::aws::traits::AwsService<Trail> for CloudTrailService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<Trail>>> + Send + 'a>>
    {
        Box::pin(self.list_trails())
    }
}

impl crate::aws::traits::DeletableResource for CloudTrailService {
    fn service_name(&self) -> &'static str {
        "CloudTrail"
    }

    fn delete<'a>(
        &'a self,
        id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.delete_trail(id))
    }
}

/// Parse an ISO 8601 datetime string into an AWS DateTime
fn parse_iso_datetime(s: &str) -> Result<aws_smithy_types::DateTime, String> {
    // Try parsing as RFC 3339 format (which ISO 8601 is compatible with)
    aws_smithy_types::DateTime::from_str(s, aws_smithy_types::date_time::Format::DateTime)
        .map_err(|e| format!("Failed to parse datetime '{}': {}", s, e))
}
