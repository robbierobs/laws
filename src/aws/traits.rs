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

/// Generic pagination helper for AWS APIs that use a next token.
pub async fn paginate<F, Fut, T, R, Extract>(
    mut fetch_page: F,
    mut extract: Extract,
) -> AppResult<Vec<T>>
where
    F: FnMut(Option<String>) -> Fut,
    Fut: Future<Output = AppResult<R>>,
    Extract: FnMut(R) -> (Vec<T>, Option<String>),
{
    let mut items = Vec::new();
    let mut next_token: Option<String> = None;

    loop {
        let response = fetch_page(next_token.take()).await?;
        let (mut page_items, token) = extract(response);
        items.append(&mut page_items);
        next_token = token;

        if next_token.is_none() {
            break;
        }
    }

    Ok(items)
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

#[cfg(test)]
mod tests {
    use super::paginate;
    use crate::error::{AppError, AppResult};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[tokio::test(flavor = "current_thread")]
    async fn test_paginate_collects_items_across_pages() {
        let calls: Rc<RefCell<Vec<Option<String>>>> = Rc::new(RefCell::new(Vec::new()));
        let pages = Rc::new(RefCell::new(vec![
            (vec!["a".to_string(), "b".to_string()], Some("next".to_string())),
            (vec!["c".to_string()], None),
        ]));

        let result = paginate(
            {
                let calls = Rc::clone(&calls);
                let pages = Rc::clone(&pages);
                move |token| {
                    let calls = Rc::clone(&calls);
                    let pages = Rc::clone(&pages);
                    async move {
                        calls.borrow_mut().push(token);
                        let (items, next) = pages.borrow_mut().remove(0);
                        Ok::<_, AppError>((items, next))
                    }
                }
            },
            |(items, next)| (items, next),
        )
        .await
        .unwrap();

        assert_eq!(result, vec!["a", "b", "c"]);
        assert_eq!(
            calls.borrow().as_slice(),
            &[None, Some("next".to_string())]
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_paginate_propagates_errors() {
        let called = Rc::new(RefCell::new(false));
        let result: AppResult<Vec<String>> = paginate(
            {
                let called = Rc::clone(&called);
                move |_token| {
                    let called = Rc::clone(&called);
                    async move {
                        if *called.borrow() {
                            Ok((vec![], None))
                        } else {
                            *called.borrow_mut() = true;
                            Err(AppError::internal("boom"))
                        }
                    }
                }
            },
            |(items, next)| (items, next),
        )
        .await;

        assert!(result.is_err());
    }
}
