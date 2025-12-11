//! Centralized error formatting utilities for AWS SDK errors

use crate::error::AppError;
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
/// * `error` - The SDK error implementing Debug
///
/// # Returns
/// An anyhow::Error wrapping an AppError with a formatted error message
pub fn format_sdk_error<E: Debug>(
    service: &str,
    action: &str,
    resource_id: &str,
    error: E,
) -> anyhow::Error {
    let debug_str = format!("{:?}", error);

    let message = if debug_str.contains("Unhandled") || debug_str.contains("unhandled") {
        format!(
            "{} for '{}': Not supported (LocalStack limitation?)",
            action, resource_id
        )
    } else {
        // Try to extract service error code from debug representation
        let code = debug_str.split('{').next().unwrap_or("Unknown").trim();

        if !code.is_empty() && code != "Unknown" {
            format!("{} failed for '{}': {}", action, resource_id, code)
        } else {
            format!("{} failed for '{}': {:?}", action, resource_id, error)
        }
    };

    AppError::aws_api(service, message).into()
}

/// Format an S3 error with detailed context
///
/// Handles common S3 error patterns including region redirects, access denied, etc.
pub fn format_s3_error(bucket_name: &str, error: impl std::fmt::Display + Debug) -> anyhow::Error {
    let err_str = format!("{:?}", error);

    let message = if err_str.contains("PermanentRedirect") || err_str.contains("301") {
        format!(
            "Bucket '{}' is in a different region. Try running with --region <bucket-region>",
            bucket_name
        )
    } else if err_str.contains("NoSuchBucket") {
        format!("Bucket '{}' does not exist", bucket_name)
    } else if err_str.contains("AccessDenied") {
        format!(
            "Access denied to bucket '{}'. Check your IAM permissions.",
            bucket_name
        )
    } else if err_str.contains("InvalidAccessKeyId") || err_str.contains("SignatureDoesNotMatch") {
        format!(
            "Invalid AWS credentials when accessing bucket '{}'",
            bucket_name
        )
    } else if err_str.contains("XmlDecodeError") || err_str.contains("invalid XML") {
        format!(
            "Bucket '{}' returned invalid XML response. This may be a LocalStack limitation.",
            bucket_name
        )
    } else if err_str.contains("Unhandled") {
        format!(
            "Bucket '{}' returned an unhandled error. This may be a LocalStack limitation.",
            bucket_name
        )
    } else {
        let display_msg = format!("{}", error);
        if display_msg.is_empty() || display_msg == "service error" {
            format!("Error for '{}': {:?}", bucket_name, error)
        } else {
            format!("Error for '{}': {}", bucket_name, display_msg)
        }
    };

    AppError::aws_api("S3", message).into()
}
