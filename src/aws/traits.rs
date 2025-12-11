use crate::error::AppResult;
use std::future::Future;
use std::pin::Pin;

/// Trait for AWS services that support listing resources.
pub trait AwsService<T>: Send + Sync {
    fn list<'a>(&'a self) -> Pin<Box<dyn Future<Output = AppResult<Vec<T>>> + Send + 'a>>;
}
