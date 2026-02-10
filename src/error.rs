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

    /// Editor operation failed
    #[error("Editor '{editor}' failed with status code: {status_code:?}")]
    EditorFailed {
        editor: String,
        status_code: Option<i32>,
    },

    /// Configuration error
    #[allow(dead_code)] // Error variant for future use
    #[error("Configuration error: {0}")]
    Config(String),

    /// Task cancelled
    #[allow(dead_code)] // Error variant for future use
    #[error("Task cancelled: {0}")]
    Cancelled(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Credential error (expired/invalid credentials)
    #[error("Credential error: {message}")]
    CredentialError { message: String, is_expired: bool },
}

/// Convenience type alias for Results with AppError
pub type AppResult<T> = Result<T, AppError>;

#[allow(dead_code)] // Error helper methods - API completeness
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

    /// Create a credential error
    pub fn credential_error(message: impl Into<String>, is_expired: bool) -> Self {
        Self::CredentialError {
            message: message.into(),
            is_expired,
        }
    }

    /// Get a user-friendly message for display in the UI
    pub fn user_message(&self) -> String {
        match self {
            Self::AwsApi {
                service, message, ..
            } => {
                format!("{}: {}", service, message)
            }
            Self::NotFound {
                resource_type,
                resource_id,
            } => {
                format!("{} '{}' not found", resource_type, resource_id)
            }
            Self::Validation(msg) => msg.clone(),
            Self::Io(e) => format!("IO error: {}", e),
            Self::EditorFailed {
                editor,
                status_code,
            } => {
                format!("Editor '{}' failed with status: {:?}", editor, status_code)
            }
            Self::Config(msg) => format!("Config: {}", msg),
            Self::Cancelled(task) => format!("Cancelled: {}", task),
            Self::Internal(msg) => format!("Error: {}", msg),
            Self::CredentialError {
                message,
                is_expired,
            } => {
                if *is_expired {
                    format!(
                        "Credential error: {}. Your credentials may have expired. Try switching profile (Shift+P) to re-authenticate.",
                        message
                    )
                } else {
                    format!(
                        "Credential error: {}. Check your AWS credentials or switch profile (Shift+P) to re-authenticate.",
                        message
                    )
                }
            }
        }
    }

    /// Returns true if this error is credential-related
    pub fn is_credential_error(&self) -> bool {
        matches!(self, Self::CredentialError { .. })
    }
}

const CREDENTIAL_ERROR_PATTERNS: [&str; 5] = [
    "expiredtoken",
    "expiredtokenexception",
    "invalidclienttokenid",
    "unrecognizedclientexception",
    "authfailure",
];

/// Returns true if the message indicates a credential error
pub fn is_credential_error_message(message: &str) -> bool {
    let lower = message.to_lowercase();
    if CREDENTIAL_ERROR_PATTERNS
        .iter()
        .any(|pattern| lower.contains(pattern))
    {
        return true;
    }

    if lower.contains("accessdenied") {
        return lower.contains("sts") || lower.contains("getcalleridentity");
    }

    false
}

/// Returns true if the message indicates expired credentials
pub fn is_expired_credential_message(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("expiredtoken") || lower.contains("expiredtokenexception")
}

/// Classify a credential error message into an AppError if it matches
pub fn classify_credential_error(message: &str) -> Option<AppError> {
    if is_credential_error_message(message) {
        Some(AppError::credential_error(
            message,
            is_expired_credential_message(message),
        ))
    } else {
        None
    }
}

/// Extension trait for converting AWS SDK errors to AppError
#[allow(dead_code)] // Infrastructure for standardized error handling
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
    fn test_not_found_error() {
        let err = AppError::not_found("Instance", "i-12345");
        assert_eq!(err.to_string(), "Resource not found: Instance 'i-12345'");
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::validation("Invalid bucket name");
        assert_eq!(err.to_string(), "Invalid input: Invalid bucket name");
    }

    #[test]
    fn test_credential_error_message_expired() {
        let message = "ExpiredToken: The security token included in the request is expired";
        assert!(is_credential_error_message(message));
        assert!(is_expired_credential_message(message));
        let err = classify_credential_error(message).expect("should classify");
        assert!(err.is_credential_error());
        assert!(err.user_message().contains("may have expired"));
    }

    #[test]
    fn test_credential_error_message_access_denied_sts() {
        let message = "AccessDenied: User is not authorized to perform sts:GetCallerIdentity";
        assert!(is_credential_error_message(message));
        let err = classify_credential_error(message).expect("should classify");
        assert!(err.user_message().contains("Credential error"));
    }

    #[test]
    fn test_credential_error_message_access_denied_non_sts() {
        let message = "AccessDenied: User is not authorized to perform s3:ListBuckets";
        assert!(!is_credential_error_message(message));
        assert!(classify_credential_error(message).is_none());
    }
}
