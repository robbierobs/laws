//! Task Manager for async task tracking and cancellation
//!
//! Provides centralized management of background async tasks to prevent
//! race conditions and enable graceful cancellation when switching services.
//! Includes timeout support and metrics tracking.

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

/// Task metadata for tracking and metrics
#[derive(Debug, Clone)]
struct TaskMetadata {
    created_at: Instant,
    timeout: Option<Duration>,
}

/// Manages active async tasks with cancellation, timeout, and metrics support
///
/// Tasks are keyed by a string identifier (e.g., "ec2_refresh", "s3_objects").
/// When a new task is spawned with the same key, any existing task with that
/// key is cancelled first.
pub struct TaskManager {
    active_tasks: HashMap<String, JoinHandle<()>>,
    task_metadata: HashMap<String, TaskMetadata>,
    metrics: TaskMetrics,
}

/// Metrics for task performance monitoring
#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    total_spawned: u64,
    total_completed: u64,
    total_cancelled: u64,
    total_timed_out: u64,
}

impl TaskMetrics {
    /// Get the number of total tasks spawned
    pub fn total_spawned(&self) -> u64 {
        self.total_spawned
    }

    /// Get the number of total tasks completed
    pub fn total_completed(&self) -> u64 {
        self.total_completed
    }

    /// Get the number of total tasks cancelled
    pub fn total_cancelled(&self) -> u64 {
        self.total_cancelled
    }

    /// Get the number of total tasks timed out
    pub fn total_timed_out(&self) -> u64 {
        self.total_timed_out
    }
}

impl TaskManager {
    /// Create a new TaskManager
    pub fn new() -> Self {
        Self {
            active_tasks: HashMap::new(),
            task_metadata: HashMap::new(),
            metrics: TaskMetrics::default(),
        }
    }

    /// Spawn a new task with the given key
    ///
    /// If a task with the same key exists, it will be cancelled first.
    /// This prevents race conditions when the user rapidly switches between
    /// services or refreshes data.
    pub fn spawn(&mut self, key: impl Into<String>, handle: JoinHandle<()>) {
        let key = key.into();

        // Cancel existing task with this key if present
        if let Some(existing) = self.active_tasks.remove(&key) {
            existing.abort();
            self.metrics.total_cancelled += 1;
        }
        self.task_metadata.remove(&key);

        self.active_tasks.insert(key.clone(), handle);
        self.task_metadata.insert(
            key,
            TaskMetadata {
                created_at: Instant::now(),
                timeout: None,
            },
        );

        self.metrics.total_spawned += 1;
    }

    /// Spawn a new task with a timeout
    ///
    /// If the task doesn't complete within the timeout duration, it will be automatically cancelled.
    /// The task's metadata will record the timeout for monitoring purposes.
    pub fn spawn_with_timeout(
        &mut self,
        key: impl Into<String>,
        handle: JoinHandle<()>,
        timeout: Duration,
    ) {
        let key = key.into();

        // Cancel existing task with this key if present
        if let Some(existing) = self.active_tasks.remove(&key) {
            existing.abort();
            self.metrics.total_cancelled += 1;
        }
        self.task_metadata.remove(&key);

        self.active_tasks.insert(key.clone(), handle);
        self.task_metadata.insert(
            key,
            TaskMetadata {
                created_at: Instant::now(),
                timeout: Some(timeout),
            },
        );

        self.metrics.total_spawned += 1;
    }

    /// Cancel a specific task by key
    pub fn cancel(&mut self, key: &str) {
        if let Some(handle) = self.active_tasks.remove(key) {
            handle.abort();
            self.task_metadata.remove(key);
            self.metrics.total_cancelled += 1;
        }
    }

    /// Cancel all tasks for a specific service
    ///
    /// Cancels all tasks whose keys start with the given prefix.
    pub fn cancel_service(&mut self, service_prefix: &str) {
        let keys_to_cancel: Vec<String> = self
            .active_tasks
            .keys()
            .filter(|k| k.starts_with(service_prefix))
            .cloned()
            .collect();

        for key in keys_to_cancel {
            if let Some(handle) = self.active_tasks.remove(&key) {
                handle.abort();
                self.task_metadata.remove(&key);
                self.metrics.total_cancelled += 1;
            }
        }
    }

    /// Cancel all active tasks
    pub fn cancel_all(&mut self) {
        for (_, handle) in self.active_tasks.drain() {
            handle.abort();
            self.metrics.total_cancelled += 1;
        }
        self.task_metadata.clear();
    }

    /// Check if a task with the given key is active
    pub fn is_active(&self, key: &str) -> bool {
        self.active_tasks.contains_key(key)
    }

    /// Get the number of active tasks
    pub fn active_count(&self) -> usize {
        self.active_tasks.len()
    }

    /// Check for timed-out tasks and cancel them
    ///
    /// This should be called periodically (e.g., in the main event loop tick)
    /// to enforce timeouts on tasks that exceed their configured duration.
    pub fn check_timeouts(&mut self) {
        let now = Instant::now();
        let keys_to_timeout: Vec<String> = self
            .task_metadata
            .iter()
            .filter_map(|(key, meta)| {
                if let Some(timeout) = meta.timeout {
                    if now.duration_since(meta.created_at) > timeout {
                        return Some(key.clone());
                    }
                }
                None
            })
            .collect();

        for key in keys_to_timeout {
            if let Some(handle) = self.active_tasks.remove(&key) {
                handle.abort();
                self.task_metadata.remove(&key);
                self.metrics.total_timed_out += 1;
            }
        }
    }

    /// Clean up completed tasks (optional maintenance)
    ///
    /// Removes references to tasks that have finished executing.
    pub fn cleanup_completed(&mut self) {
        let keys_to_remove: Vec<String> = self
            .active_tasks
            .iter()
            .filter_map(|(key, handle)| {
                if handle.is_finished() {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            self.active_tasks.remove(&key);
            self.task_metadata.remove(&key);
            self.metrics.total_completed += 1;
        }
    }

    /// Get a snapshot of current metrics
    pub fn metrics(&self) -> TaskMetrics {
        self.metrics.clone()
    }

    /// Get elapsed time for a task
    pub fn elapsed(&self, key: &str) -> Option<Duration> {
        self.task_metadata
            .get(key)
            .map(|meta| meta.created_at.elapsed())
    }

    /// Get timeout for a task
    pub fn timeout(&self, key: &str) -> Option<Option<Duration>> {
        self.task_metadata.get(key).map(|meta| meta.timeout)
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Task keys for common operations
pub mod task_keys {
    // Service refresh tasks
    pub const EC2_REFRESH: &str = "ec2:refresh";
    pub const S3_REFRESH: &str = "s3:refresh";
    pub const S3_OBJECTS: &str = "s3:objects";
    pub const S3_DETAILS: &str = "s3:details";
    pub const RDS_REFRESH: &str = "rds:refresh";
    pub const DYNAMODB_REFRESH: &str = "dynamodb:refresh";
    pub const DYNAMODB_ITEMS: &str = "dynamodb:items";
    pub const LAMBDA_REFRESH: &str = "lambda:refresh";
    pub const VPC_REFRESH: &str = "vpc:refresh";
    pub const IAM_REFRESH: &str = "iam:refresh";
    pub const IAM_POLICIES: &str = "iam:policies";
    pub const BACKUP_REFRESH: &str = "backup:refresh";
    pub const CLOUDTRAIL_REFRESH: &str = "cloudtrail:refresh";
    pub const SECRETSMANAGER_REFRESH: &str = "secretsmanager:refresh";

    // Action tasks
    pub const EC2_ACTION: &str = "ec2:action";
    pub const RDS_ACTION: &str = "rds:action";
    pub const S3_ACTION: &str = "s3:action";
    pub const DYNAMODB_ACTION: &str = "dynamodb:action";
    pub const SECRETSMANAGER_ACTION: &str = "secretsmanager:action";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_manager_new() {
        let tm = TaskManager::new();
        assert_eq!(tm.active_count(), 0);
    }

    #[tokio::test]
    async fn test_spawn_and_cancel() {
        let mut tm = TaskManager::new();

        let handle = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        });

        tm.spawn("test_task", handle);
        assert!(tm.is_active("test_task"));
        assert_eq!(tm.active_count(), 1);

        tm.cancel("test_task");
        assert!(!tm.is_active("test_task"));
        assert_eq!(tm.active_count(), 0);
    }

    #[tokio::test]
    async fn test_spawn_replaces_existing() {
        let mut tm = TaskManager::new();

        let handle1 = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        });
        tm.spawn("task1", handle1);

        let handle2 = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        });
        tm.spawn("task1", handle2);

        // Should still be only 1 task
        assert_eq!(tm.active_count(), 1);
    }

    #[tokio::test]
    async fn test_cancel_service() {
        let mut tm = TaskManager::new();

        for i in 0..3 {
            let handle = tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            });
            tm.spawn(format!("s3:task{}", i), handle);
        }

        let ec2_handle = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        });
        tm.spawn("ec2:refresh", ec2_handle);

        assert_eq!(tm.active_count(), 4);

        tm.cancel_service("s3:");
        assert_eq!(tm.active_count(), 1);
        assert!(tm.is_active("ec2:refresh"));
    }

    #[tokio::test]
    async fn test_metrics_spawn() {
        let mut tm = TaskManager::new();

        for i in 0..3 {
            let handle = tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            });
            tm.spawn(format!("task{}", i), handle);
        }

        let metrics = tm.metrics();
        assert_eq!(metrics.total_spawned(), 3);
    }

    #[tokio::test]
    async fn test_metrics_cancel() {
        let mut tm = TaskManager::new();

        let handle = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        });
        tm.spawn("task1", handle);

        tm.cancel("task1");

        let metrics = tm.metrics();
        assert_eq!(metrics.total_cancelled(), 1);
    }

    #[tokio::test]
    async fn test_spawn_with_timeout() {
        let mut tm = TaskManager::new();

        let handle = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        });

        tm.spawn_with_timeout("task_timeout", handle, Duration::from_millis(100));
        assert!(tm.is_active("task_timeout"));

        tokio::time::sleep(Duration::from_millis(150)).await;
        tm.check_timeouts();

        assert!(!tm.is_active("task_timeout"));
        assert_eq!(tm.metrics().total_timed_out(), 1);
    }

    #[tokio::test]
    async fn test_elapsed() {
        let mut tm = TaskManager::new();

        let handle = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        });

        tm.spawn("task1", handle);

        tokio::time::sleep(Duration::from_millis(100)).await;

        if let Some(elapsed) = tm.elapsed("task1") {
            assert!(elapsed.as_millis() >= 100);
        }
    }

    #[tokio::test]
    async fn test_cleanup_completed() {
        let mut tm = TaskManager::new();

        let handle = tokio::spawn(async {
            // Completes immediately
        });

        tm.spawn("quick_task", handle);

        tokio::time::sleep(Duration::from_millis(50)).await;
        tm.cleanup_completed();

        let metrics = tm.metrics();
        assert_eq!(metrics.total_completed(), 1);
        assert_eq!(tm.active_count(), 0);
    }
}
