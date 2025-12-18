//! AWS service traits and macros
//!
//! Provides common abstractions to reduce boilerplate across AWS service implementations.

use crate::error::AppResult;
use std::future::Future;
use std::pin::Pin;

/// Trait for AWS services that support listing resources.
pub trait AwsService<T>: Send + Sync {
    fn list<'a>(&'a self) -> Pin<Box<dyn Future<Output = AppResult<Vec<T>>> + Send + 'a>>;
}

/// Trait for AWS services that support deleting a resource by ID
#[allow(dead_code)] // Infrastructure for future use
pub trait DeletableResource: Send + Sync {
    /// The service name for display (e.g., "EC2", "RDS")
    fn service_name(&self) -> &'static str;

    /// Delete a resource by its identifier
    fn delete<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;
}

/// Macro to generate a simple AWS service struct with a `new(client: Client) -> Self` constructor.
///
/// This reduces boilerplate since every AWS service struct follows the same pattern.
///
/// # Example
/// ```ignore
/// aws_service_struct!(Ec2Service, aws_sdk_ec2::Client);
/// ```
/// Generates:
/// ```ignore
/// pub struct Ec2Service {
///     client: aws_sdk_ec2::Client,
/// }
///
/// impl Ec2Service {
///     pub fn new(client: aws_sdk_ec2::Client) -> Self {
///         Self { client }
///     }
/// }
/// ```
#[macro_export]
macro_rules! aws_service_struct {
    ($name:ident, $client_type:ty) => {
        pub struct $name {
            client: $client_type,
        }

        impl $name {
            pub fn new(client: $client_type) -> Self {
                Self { client }
            }
        }
    };
}

