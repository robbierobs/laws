use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum Event {
    Key(KeyEvent),
    Tick,
    Aws(AwsEvent),
}

use crate::models::backup::{BackupVault, BackupPlan, BackupJob};
use crate::models::cloudtrail::{Trail, CloudTrailEvent};
use crate::models::dynamodb::DynamoDbTable;
use crate::models::ec2::Ec2Instance;
use crate::models::iam::{IamRole, IamUser, IamPolicy};
use crate::models::lambda::LambdaFunction;
use crate::models::rds::RdsInstance;
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crate::models::vpc::{Vpc, Subnet, SecurityGroup};

#[derive(Debug)]
pub enum AwsEvent {
    Ec2InstancesLoaded(Vec<Ec2Instance>),
    S3BucketsLoaded(Vec<S3Bucket>),
    S3ObjectsLoaded(Vec<S3Object>),
    S3BucketDetailsLoaded { bucket_name: String, details: S3BucketDetails },
    RdsInstancesLoaded(Vec<RdsInstance>),
    DynamoDbTablesLoaded(Vec<DynamoDbTable>),
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
    ActionCompleted(String), // Message to display
    Error(String),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        // Spawn input handling task
        tokio::spawn(async move {
            let tick_rate = Duration::from_millis(tick_rate);
            loop {
                let event_available = event::poll(tick_rate).unwrap_or(false);
                if event_available {
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        event_tx.send(Event::Key(key)).unwrap_or(());
                    }
                }
                event_tx.send(Event::Tick).ok();
            }
        });

        Self { rx, _tx: tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self._tx.clone()
    }
}
