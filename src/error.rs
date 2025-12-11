//! Application error types
//!
//! Provides structured error handling with context for AWS API errors,
//! validation errors, and other failure modes.

#![allow(dead_code)]

use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    /// AWS API error with service context
    #[error("AWS {service} error: {message}")]
    AwsApi {
        service: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Resource not found
    #[error("Resource not found: {resource_type} '{resource_id}'")]
    NotFound {
        resource_type: String,
        resource_id: String,
    },

    /// Invalid user input or configuration
    #[error("Invalid input: {0}")]
    Validation(String),

    /// IO error (file operations, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Task cancelled
    #[error("Task cancelled: {0}")]
    Cancelled(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience type alias for Results with AppError
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Create an AWS API error
    pub fn aws_api(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::AwsApi {
            service: service.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create an AWS API error with source
    pub fn aws_api_with_source(
        service: impl Into<String>,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::AwsApi {
            service: service.into(),
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a not found error
    pub fn not_found(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create a cancelled error
    pub fn cancelled(task: impl Into<String>) -> Self {
        Self::Cancelled(task.into())
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Get a user-friendly message for display in the UI
    pub fn user_message(&self) -> String {
        match self {
            Self::AwsApi { service, message, .. } => {
                format!("{}: {}", service, message)
            }
            Self::NotFound { resource_type, resource_id } => {
                format!("{} '{}' not found", resource_type, resource_id)
            }
            Self::Validation(msg) => msg.clone(),
            Self::Io(e) => format!("IO error: {}", e),
            Self::Config(msg) => format!("Config: {}", msg),
            Self::Cancelled(task) => format!("Cancelled: {}", task),
            Self::Internal(msg) => format!("Error: {}", msg),
        }
    }
}

/// Extension trait for converting AWS SDK errors to AppError
pub trait AwsErrorExt<T> {
    fn map_aws_err(self, service: &str) -> AppResult<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> AwsErrorExt<T> for Result<T, E> {
    fn map_aws_err(self, service: &str) -> AppResult<T> {
        self.map_err(|e| AppError::aws_api_with_source(service, e.to_string(), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_api_error() {
        let err = AppError::aws_api("EC2", "DescribeInstances failed");
        assert_eq!(err.to_string(), "AWS EC2 error: DescribeInstances failed");
        assert_eq!(err.user_message(), "EC2: DescribeInstances failed");
    }

    #[test]
    fn test_aws_api_error_with_source() {
        let source_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
        let err = AppError::aws_api_with_source("S3", "Bucket not found", source_err);
        assert!(err.to_string().contains("AWS S3 error"));
        assert!(err.to_string().contains("Bucket not found"));
    }

    #[test]
    fn test_not_found_error() {
        let err = AppError::not_found("Instance", "i-12345");
        assert_eq!(err.to_string(), "Resource not found: Instance 'i-12345'");
        assert_eq!(err.user_message(), "Instance 'i-12345' not found");
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::validation("Invalid bucket name");
        assert_eq!(err.to_string(), "Invalid input: Invalid bucket name");
        assert_eq!(err.user_message(), "Invalid bucket name");
    }
    
    #[test]
    fn test_config_error() {
        let err = AppError::config("Missing required field");
        assert_eq!(err.to_string(), "Configuration error: Missing required field");
        assert_eq!(err.user_message(), "Config: Missing required field");
    }
    
    #[test]
    fn test_cancelled_error() {
        let err = AppError::cancelled("fetch_s3_objects");
        assert_eq!(err.to_string(), "Task cancelled: fetch_s3_objects");
        assert_eq!(err.user_message(), "Cancelled: fetch_s3_objects");
    }
    
    #[test]
    fn test_internal_error() {
        let err = AppError::internal("Unexpected state");
        assert_eq!(err.to_string(), "Internal error: Unexpected state");
        assert_eq!(err.user_message(), "Error: Unexpected state");
    }
    
    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let err: AppError = io_err.into();
        assert!(matches!(err, AppError::Io(_)));
        assert!(err.to_string().contains("IO error"));
    }
    
    #[test]
    fn test_aws_error_ext() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found"
        ));
        
        let app_result = result.map_aws_err("EC2");
        assert!(app_result.is_err());
        
        if let Err(err) = app_result {
            assert!(matches!(err, AppError::AwsApi { .. }));
            assert!(err.to_string().contains("EC2"));
        }
    }
    
    #[test]
    fn test_user_message_formatting() {
        // Test that user messages are consistently formatted
        let errors = vec![
            AppError::aws_api("S3", "Bucket access denied"),
            AppError::not_found("Bucket", "my-bucket"),
            AppError::validation("Invalid region"),
            AppError::config("Missing config file"),
            AppError::cancelled("long_task"),
            AppError::internal("Bug found"),
        ];
        
        for err in errors {
            let msg = err.user_message();
            assert!(!msg.is_empty(), "User message should not be empty");
            // User messages should not contain internal error details
            assert!(!msg.contains("Error::"), "User message should be user-friendly");
        }
    }
}
