//! Task Manager for async task tracking and cancellation
//!
//! Provides centralized management of background async tasks to prevent
//! race conditions and enable graceful cancellation when switching services.

#![allow(dead_code)]

use std::collections::HashMap;
use tokio::task::JoinHandle;

/// Manages active async tasks with cancellation support
/// 
/// Tasks are keyed by a string identifier (e.g., "ec2_refresh", "s3_objects").
/// When a new task is spawned with the same key, any existing task with that
/// key is cancelled first.
pub struct TaskManager {
    active_tasks: HashMap<String, JoinHandle<()>>,
}

impl TaskManager {
    /// Create a new TaskManager
    pub fn new() -> Self {
        Self {
            active_tasks: HashMap::new(),
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
        }
        
        self.active_tasks.insert(key, handle);
    }

    /// Cancel a specific task by key
    pub fn cancel(&mut self, key: &str) {
        if let Some(handle) = self.active_tasks.remove(key) {
            handle.abort();
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
            }
        }
    }

    /// Cancel all active tasks
    pub fn cancel_all(&mut self) {
        for (_, handle) in self.active_tasks.drain() {
            handle.abort();
        }
    }

    /// Check if a task with the given key is active
    pub fn is_active(&self, key: &str) -> bool {
        self.active_tasks.contains_key(key)
    }

    /// Get the number of active tasks
    pub fn active_count(&self) -> usize {
        self.active_tasks.len()
    }

    /// Clean up completed tasks (optional maintenance)
    pub fn cleanup_completed(&mut self) {
        self.active_tasks.retain(|_, handle| !handle.is_finished());
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
    
    // Action tasks
    pub const EC2_ACTION: &str = "ec2:action";
    pub const RDS_ACTION: &str = "rds:action";
    pub const S3_ACTION: &str = "s3:action";
    pub const DYNAMODB_ACTION: &str = "dynamodb:action";
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
}
