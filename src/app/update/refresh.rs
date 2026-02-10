//! Generic refresh task helpers
//!
//! Provides utilities to reduce boilerplate when spawning AWS refresh tasks.
//! Each refresh task follows the same pattern:
//! 1. Clone client and event sender
//! 2. Spawn async task
//! 3. Call AWS service list operation
//! 4. Send success event or error event
//! 5. Register with TaskManager

use crate::error::classify_credential_error;
use crate::event::{AwsEvent, Event, EventSender};
use std::future::Future;
use tokio::task::JoinHandle;

/// Spawns a generic refresh task that lists resources and sends an event.
///
/// # Arguments
/// * `tx` - Event sender channel
/// * `list_fn` - Async function that returns `Result<T, E>` where E: ToString
/// * `event_builder` - Function to convert successful result into AwsEvent
/// * `report_errors` - If true, sends error events; if false, silently ignores errors
///
/// # Returns
/// A JoinHandle that can be registered with TaskManager
pub fn spawn_list_task<T, E, F, Fut>(
    tx: EventSender,
    list_fn: F,
    event_builder: fn(T) -> AwsEvent,
    report_errors: bool,
) -> JoinHandle<()>
where
    T: Send + 'static,
    E: ToString + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, E>> + Send,
{
    tokio::spawn(async move {
        match list_fn().await {
            Ok(data) => {
                tx.send(Event::Aws(Box::new(event_builder(data)))).await.ok();
            }
            Err(e) => {
                if report_errors {
                    let message = e.to_string();
                    let user_message = classify_credential_error(&message)
                        .map(|err| err.user_message())
                        .unwrap_or(message);
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(user_message))))
                        .await
                        .ok();
                }
            }
        }
    })
}

/// Spawns a generic action task that performs an operation on a single resource.
/// 
/// This is useful for operations like delete, invoke, get details, etc.
/// 
/// # Arguments
/// * `tx` - Event sender channel
/// * `action_fn` - Async function that performs the action
/// * `success_message` - Message to send on success
/// * `resource_id` - ID of the resource for error reporting
/// * `report_errors` - If true, sends error events; if false, silently ignores errors
///
/// # Returns
/// A JoinHandle that can be registered with TaskManager
#[allow(dead_code)] // Infrastructure for future use
pub fn spawn_action_task<E, F, Fut>(
    tx: EventSender,
    action_fn: F,
    success_message: String,
    report_errors: bool,
) -> JoinHandle<()>
where
    E: ToString + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), E>> + Send,
{
    tokio::spawn(async move {
        match action_fn().await {
            Ok(()) => {
                tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(success_message))))
                    .await
                    .ok();
            }
            Err(e) => {
                if report_errors {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                        .await
                        .ok();
                }
            }
        }
    })
}

/// Spawns an action task that returns a value (e.g., get secret value, invoke lambda)
#[allow(dead_code)] // Infrastructure for future use
pub fn spawn_action_with_result_task<T, E, F, Fut>(
    tx: EventSender,
    action_fn: F,
    event_builder: fn(T) -> AwsEvent,
    report_errors: bool,
) -> JoinHandle<()>
where
    T: Send + 'static,
    E: ToString + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, E>> + Send,
{
    tokio::spawn(async move {
        match action_fn().await {
            Ok(result) => {
                tx.send(Event::Aws(Box::new(event_builder(result)))).await.ok();
            }
            Err(e) => {
                if report_errors {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                        .await
                        .ok();
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_spawn_list_task_success() {
        let (tx, mut rx) = mpsc::channel(10);

        let handle = spawn_list_task(
            tx,
            || async { Ok::<_, String>(vec!["item1", "item2"]) },
            |items: Vec<&str>| AwsEvent::ActionCompleted(format!("Loaded {} items", items.len())),
            true,
        );

        handle.await.unwrap();

        if let Some(Event::Aws(aws_event)) = rx.recv().await {
            if let AwsEvent::ActionCompleted(msg) = *aws_event {
                assert_eq!(msg, "Loaded 2 items");
            } else {
                panic!("Expected ActionCompleted event");
            }
        } else {
            panic!("Expected Aws event");
        }
    }

    #[tokio::test]
    async fn test_spawn_list_task_error_reported() {
        let (tx, mut rx) = mpsc::channel(10);

        let handle = spawn_list_task(
            tx,
            || async { Err::<Vec<String>, _>("test error") },
            |_: Vec<String>| AwsEvent::ActionCompleted("success".to_string()),
            true,
        );

        handle.await.unwrap();

        if let Some(Event::Aws(aws_event)) = rx.recv().await {
            if let AwsEvent::Error(msg) = *aws_event {
                assert_eq!(msg, "test error");
            } else {
                panic!("Expected Error event");
            }
        } else {
            panic!("Expected Aws event");
        }
    }

    #[tokio::test]
    async fn test_spawn_list_task_error_silent() {
        let (tx, mut rx) = mpsc::channel(10);

        let handle = spawn_list_task(
            tx,
            || async { Err::<Vec<String>, _>("test error") },
            |_: Vec<String>| AwsEvent::ActionCompleted("success".to_string()),
            false, // Don't report errors
        );

        handle.await.unwrap();

        // Channel should be empty - no error sent
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_spawn_list_task_credential_error_classified() {
        let (tx, mut rx) = mpsc::channel(10);

        let handle = spawn_list_task(
            tx,
            || async { Err::<Vec<String>, _>("ExpiredToken: token expired") },
            |_: Vec<String>| AwsEvent::ActionCompleted("success".to_string()),
            true,
        );

        handle.await.unwrap();

        if let Some(Event::Aws(aws_event)) = rx.recv().await {
            if let AwsEvent::Error(msg) = *aws_event {
                assert!(msg.contains("Credential error"));
                assert!(msg.contains("expired"));
            } else {
                panic!("Expected Error event");
            }
        } else {
            panic!("Expected Aws event");
        }
    }
}
