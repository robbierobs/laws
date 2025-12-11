//! Centralized error formatting utilities for AWS SDK errors

use std::fmt::Debug;

/// Formats AWS SDK errors with service error extraction
/// 
/// This function attempts to extract the service error code from SDK errors
/// and provides consistent error messages across all AWS services.
/// 
/// # Arguments
/// * `service` - The AWS service name (e.g., "EC2", "RDS", "Lambda")
/// * `action` - The action being performed (e.g., "start", "stop", "list_functions")
/// * `resource_id` - The resource identifier (e.g., instance ID, table name, or "all")
/// * `error` - The SDK error implementing both Debug and Display
/// 
/// # Returns
/// An anyhow::Error with a formatted error message
pub fn format_sdk_error<E: Debug>(
    service: &str,
    action: &str,
    resource_id: &str,
    error: E,
) -> anyhow::Error {
    // Try to extract service error code from debug representation
    let debug_str = format!("{:?}", error);
    let code = debug_str
        .split('{')
        .next()
        .unwrap_or("Unknown")
        .trim()
        .to_string();
    
    let display_str = format!("{:?}", error);
    
    let msg = if display_str.contains("Unhandled") || display_str.contains("unhandled") {
        format!(
            "{} {} for '{}': Not supported (LocalStack limitation?)",
            service, action, resource_id
        )
    } else if code != "Unknown" && !code.is_empty() {
        format!("{} {} failed for '{}': {}", service, action, resource_id, code)
    } else {
        format!("{} {} failed for '{}': {:?}", service, action, resource_id, error)
    };
    
    anyhow::anyhow!(msg)
}
