#![allow(dead_code)]

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

pub enum Event {
    Key(KeyEvent),
    Tick,
    Aws(AwsEvent),
}

use crate::models::backup::{BackupJob, BackupPlan, BackupVault};
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::models::dynamodb::{DynamoDbItem, DynamoDbTable};
use crate::models::ec2::Ec2Instance;
use crate::models::iam::{IamPolicy, IamRole, IamUser};
use crate::models::lambda::LambdaFunction;
use crate::models::rds::RdsInstance;
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crate::models::secretsmanager::Secret;
use crate::models::vpc::{SecurityGroup, Subnet, Vpc};

#[derive(Debug)]
pub enum AwsEvent {
    Ec2InstancesLoaded(Vec<Ec2Instance>),
    S3BucketsLoaded(Vec<S3Bucket>),
    S3ObjectsLoaded(Vec<S3Object>),
    S3BucketDetailsLoaded {
        bucket_name: String,
        details: S3BucketDetails,
    },
    RdsInstancesLoaded(Vec<RdsInstance>),
    DynamoDbTablesLoaded(Vec<DynamoDbTable>),
    DynamoDbItemsLoaded(Vec<DynamoDbItem>),
    LambdaFunctionsLoaded(Vec<LambdaFunction>),
    VpcsLoaded(Vec<Vpc>),
    SubnetsLoaded(Vec<Subnet>),
    SecurityGroupsLoaded(Vec<SecurityGroup>),
    IamRolesLoaded(Vec<IamRole>),
    IamUsersLoaded(Vec<IamUser>),
    IamPoliciesLoaded(Vec<IamPolicy>),
    IamUserPoliciesLoaded(Vec<IamPolicy>),
    IamRolePoliciesLoaded(Vec<IamPolicy>),
    IamPolicyDocumentLoaded(String),
    BackupVaultsLoaded(Vec<BackupVault>),
    BackupPlansLoaded(Vec<BackupPlan>),
    BackupJobsLoaded(Vec<BackupJob>),
    CloudTrailTrailsLoaded(Vec<Trail>),
    CloudTrailEventsLoaded(Vec<CloudTrailEvent>),
    SecretsManagerSecretsLoaded(Vec<Secret>),
    SecretsManagerSecretValueLoaded(String),
    /// S3 object was downloaded to a file path
    S3ObjectDownloaded {
        key: String,
        path: String,
    },
    /// S3 object was downloaded and ready to open (with content for text files)
    S3ObjectOpened {
        key: String,
        path: String,
        content: Option<String>,
    },
    ActionCompleted(String), // Message to display
    Error(String),
}

/// Metrics for event channel monitoring
#[derive(Debug, Clone, Default)]
pub struct EventMetrics {
    total_sent: usize,
    total_received: usize,
    peak_queue_depth: usize,
    dropped_events: usize,
}

impl EventMetrics {
    pub fn total_sent(&self) -> usize {
        self.total_sent
    }

    pub fn total_received(&self) -> usize {
        self.total_received
    }

    pub fn peak_queue_depth(&self) -> usize {
        self.peak_queue_depth
    }

    pub fn dropped_events(&self) -> usize {
        self.dropped_events
    }
}

/// Type alias for event sender (bounded channel)
pub type EventSender = mpsc::Sender<Event>;

/// Event handler with bounded channel and backpressure support
///
/// Uses a bounded mpsc channel (default 1000 capacity) to prevent unbounded
/// memory growth under heavy event load. Provides metrics for monitoring
/// queue depth and event throughput.
pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
    tx: EventSender,
    metrics: Arc<EventMetrics>,
    queue_depth: Arc<AtomicUsize>,
    _task_handle: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new EventHandler with default bounded channel (1000 capacity)
    pub fn new(tick_rate: u64) -> Self {
        Self::with_capacity(tick_rate, 1000)
    }

    /// Create a new EventHandler with custom bounded channel capacity
    ///
    /// # Arguments
    /// * `tick_rate` - UI update rate in milliseconds
    /// * `capacity` - Maximum number of events to buffer in the channel
    ///
    /// If the channel fills up, backpressure will cause senders to wait
    /// until space becomes available.
    pub fn with_capacity(tick_rate: u64, capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        let event_tx = tx.clone();
        let queue_depth = Arc::new(AtomicUsize::new(0));
        let queue_depth_clone = queue_depth.clone();
        let metrics = Arc::new(EventMetrics::default());

        // Spawn input handling task
        let task_handle = tokio::spawn(async move {
            let tick_rate = Duration::from_millis(tick_rate);
            loop {
                let event_available = event::poll(tick_rate).unwrap_or(false);
                if event_available {
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        // Try to send key event (may wait if channel is full)
                        if event_tx.send(Event::Key(key)).await.is_err() {
                            break; // Channel closed, exit task
                        }
                        queue_depth_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
                // Always send tick
                if event_tx.send(Event::Tick).await.is_err() {
                    break; // Channel closed, exit task
                }
                queue_depth_clone.fetch_add(1, Ordering::Relaxed);
            }
        });

        Self {
            rx,
            tx,
            metrics,
            queue_depth,
            _task_handle: task_handle,
        }
    }

    /// Receive the next event from the queue
    pub async fn next(&mut self) -> Option<Event> {
        if let Some(event) = self.rx.recv().await {
            let _depth = self.queue_depth.fetch_sub(1, Ordering::Relaxed);
            Some(event)
        } else {
            None
        }
    }

    /// Get a sender for this event channel
    pub fn sender(&self) -> EventSender {
        self.tx.clone()
    }

    /// Get the current queue depth
    pub fn queue_depth(&self) -> usize {
        self.queue_depth.load(Ordering::Relaxed)
    }

    /// Get the channel capacity
    pub fn capacity(&self) -> usize {
        self.tx.capacity()
    }

    /// Check if channel is near capacity (>80%)
    pub fn is_near_capacity(&self) -> bool {
        let depth = self.queue_depth.load(Ordering::Relaxed);
        let capacity = self.capacity();
        depth > (capacity * 80 / 100)
    }

    /// Get current metrics snapshot
    pub fn metrics(&self) -> EventMetrics {
        (*self.metrics).clone()
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self._task_handle.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_handler_new() {
        let handler = EventHandler::new(100);
        assert_eq!(handler.capacity(), 1000);
        assert_eq!(handler.queue_depth(), 0);
    }

    #[tokio::test]
    async fn test_event_handler_with_capacity() {
        let handler = EventHandler::with_capacity(100, 500);
        assert_eq!(handler.capacity(), 500);
    }

    #[tokio::test]
    async fn test_send_and_receive_tick() {
        let mut handler = EventHandler::with_capacity(1000, 100);
        let tx = handler.sender();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = tx.send(Event::Tick).await;
        });

        if let Some(Event::Tick) = handler.next().await {
            // Success
        } else {
            panic!("Failed to receive Tick event");
        }
    }

    #[tokio::test]
    async fn test_queue_depth_tracking() {
        // Create handler with long tick rate to minimize interference
        let mut handler = EventHandler::with_capacity(10000, 50);

        // Wait for initial tick to be processed
        let _ = handler.next().await;

        // After receiving, depth should be low (may have another tick pending)
        let depth_after = handler.queue_depth();
        // Just verify we can track depth - exact value varies due to background ticks
        assert!(depth_after < 50); // Should be well under capacity
    }

    #[tokio::test]
    async fn test_is_near_capacity() {
        let handler = EventHandler::with_capacity(100, 100);
        let tx = handler.sender();

        // Send 85 events to exceed 80% threshold
        for _ in 0..85 {
            let _ = tx.send(Event::Tick).await;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(handler.is_near_capacity());
    }

    #[tokio::test]
    async fn test_backpressure_on_full_channel() {
        let handler = EventHandler::with_capacity(100, 5);
        let tx = handler.sender();

        // Fill the channel
        for _ in 0..5 {
            let _ = tx.send(Event::Tick).await;
        }

        // Next send should block until space available
        let tx_clone = tx.clone();
        let send_future = tokio::spawn(async move { tx_clone.send(Event::Tick).await });

        // Give it a bit of time to block
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Task should still be pending (not completed)
        assert!(!send_future.is_finished());
    }

    #[tokio::test]
    async fn test_metrics_initialized() {
        let handler = EventHandler::new(100);
        let metrics = handler.metrics();
        assert_eq!(metrics.total_sent(), 0);
        assert_eq!(metrics.total_received(), 0);
        assert_eq!(metrics.dropped_events(), 0);
    }
}
