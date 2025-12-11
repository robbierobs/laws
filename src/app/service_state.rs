//! Service-specific state structs
//!
//! This module consolidates service-specific state that was previously scattered
//! across 50+ fields in the App struct. Each service has its own state struct
//! containing all data, list states, and view modes specific to that service.

use ratatui::widgets::TableState;
use std::collections::HashMap;

use crate::models::backup::{BackupJob, BackupPlan, BackupVault};
use crate::models::cloudtrail::{CloudTrailEvent, Trail};
use crate::models::dynamodb::{DynamoDbItem, DynamoDbTable};
use crate::models::ec2::Ec2Instance;
use crate::models::iam::{IamPolicy, IamRole, IamUser};
use crate::models::lambda::LambdaFunction;
use crate::models::rds::RdsInstance;
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crate::models::vpc::{SecurityGroup, SecurityGroupRule, Subnet, Vpc};

use super::{BackupViewMode, CloudTrailViewMode, DynamoDbViewMode, IamViewMode, VpcViewMode};

// ============================================================================
// EC2 State
// ============================================================================

/// State for EC2 service
#[derive(Default)]
pub struct Ec2State {
    pub instances: Vec<Ec2Instance>,
    pub list_state: TableState,
}

impl Ec2State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected instance, if any
    pub fn selected_instance(&self) -> Option<&Ec2Instance> {
        self.list_state
            .selected()
            .and_then(|i| self.instances.get(i))
    }

    /// Get the instance ID of the currently selected instance
    pub fn selected_instance_id(&self) -> Option<String> {
        self.selected_instance().map(|i| i.instance_id.clone())
    }
}

// ============================================================================
// S3 State
// ============================================================================

/// State for S3 service
#[derive(Default)]
pub struct S3State {
    pub buckets: Vec<S3Bucket>,
    pub list_state: TableState,
    pub current_bucket: Option<String>,
    pub objects: Vec<S3Object>,
    pub object_list_state: TableState,
    pub bucket_details: HashMap<String, S3BucketDetails>,
}

impl S3State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're currently viewing objects inside a bucket
    pub fn is_viewing_objects(&self) -> bool {
        self.current_bucket.is_some()
    }

    /// Get the currently selected bucket, if any
    pub fn selected_bucket(&self) -> Option<&S3Bucket> {
        self.list_state
            .selected()
            .and_then(|i| self.buckets.get(i))
    }

    /// Get the currently selected object, if any
    pub fn selected_object(&self) -> Option<&S3Object> {
        self.object_list_state
            .selected()
            .and_then(|i| self.objects.get(i))
    }
}

// ============================================================================
// RDS State
// ============================================================================

/// State for RDS service
#[derive(Default)]
pub struct RdsState {
    pub instances: Vec<RdsInstance>,
    pub list_state: TableState,
}

impl RdsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected instance, if any
    pub fn selected_instance(&self) -> Option<&RdsInstance> {
        self.list_state
            .selected()
            .and_then(|i| self.instances.get(i))
    }

    /// Get the instance ID of the currently selected instance
    pub fn selected_instance_id(&self) -> Option<String> {
        self.selected_instance().map(|i| i.db_instance_identifier.clone())
    }
}

// ============================================================================
// DynamoDB State
// ============================================================================

/// State for DynamoDB service
#[derive(Default)]
pub struct DynamoDbState {
    pub tables: Vec<DynamoDbTable>,
    pub list_state: TableState,
    pub view_mode: DynamoDbViewMode,
    pub current_table: Option<String>,
    pub items: Vec<DynamoDbItem>,
    pub item_list_state: TableState,
}

impl DynamoDbState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're currently viewing items inside a table
    pub fn is_viewing_items(&self) -> bool {
        self.current_table.is_some()
    }

    /// Get the currently selected table, if any
    pub fn selected_table(&self) -> Option<&DynamoDbTable> {
        self.list_state
            .selected()
            .and_then(|i| self.tables.get(i))
    }

    /// Get the currently selected item, if any
    pub fn selected_item(&self) -> Option<&DynamoDbItem> {
        self.item_list_state
            .selected()
            .and_then(|i| self.items.get(i))
    }
}

// ============================================================================
// Lambda State
// ============================================================================

/// State for Lambda service
#[derive(Default)]
pub struct LambdaState {
    pub functions: Vec<LambdaFunction>,
    pub list_state: TableState,
}

impl LambdaState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected function, if any
    pub fn selected_function(&self) -> Option<&LambdaFunction> {
        self.list_state
            .selected()
            .and_then(|i| self.functions.get(i))
    }
}

// ============================================================================
// VPC State
// ============================================================================

/// State for VPC service
#[derive(Default)]
pub struct VpcState {
    pub vpcs: Vec<Vpc>,
    pub subnets: Vec<Subnet>,
    pub security_groups: Vec<SecurityGroup>,
    pub list_state: TableState,
    pub view_mode: VpcViewMode,
    pub current_sg_rules: Vec<SecurityGroupRule>,
    pub selected_sg_id: Option<String>,
    pub sg_rules_inbound: bool,
}

impl VpcState {
    pub fn new() -> Self {
        Self {
            sg_rules_inbound: true, // Default to showing inbound rules
            ..Self::default()
        }
    }

    /// Get the currently selected VPC, if any
    pub fn selected_vpc(&self) -> Option<&Vpc> {
        if self.view_mode == VpcViewMode::Vpcs {
            self.list_state.selected().and_then(|i| self.vpcs.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected subnet, if any
    pub fn selected_subnet(&self) -> Option<&Subnet> {
        if self.view_mode == VpcViewMode::Subnets {
            self.list_state
                .selected()
                .and_then(|i| self.subnets.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected security group, if any
    pub fn selected_security_group(&self) -> Option<&SecurityGroup> {
        if self.view_mode == VpcViewMode::SecurityGroups {
            self.list_state
                .selected()
                .and_then(|i| self.security_groups.get(i))
        } else {
            None
        }
    }
}

// ============================================================================
// IAM State
// ============================================================================

/// State for IAM service
#[derive(Default)]
pub struct IamState {
    pub roles: Vec<IamRole>,
    pub users: Vec<IamUser>,
    pub policies: Vec<IamPolicy>,
    pub list_state: TableState,
    pub view_mode: IamViewMode,
    pub previous_view_mode: IamViewMode,
    pub current_policies: Vec<IamPolicy>,
    pub current_policy_document: String,
    pub selected_entity_name: Option<String>,
}

impl IamState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected user, if any
    pub fn selected_user(&self) -> Option<&IamUser> {
        if self.view_mode == IamViewMode::Users {
            self.list_state.selected().and_then(|i| self.users.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected role, if any
    pub fn selected_role(&self) -> Option<&IamRole> {
        if self.view_mode == IamViewMode::Roles {
            self.list_state.selected().and_then(|i| self.roles.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected policy, if any
    pub fn selected_policy(&self) -> Option<&IamPolicy> {
        if self.view_mode == IamViewMode::Policies {
            self.list_state
                .selected()
                .and_then(|i| self.policies.get(i))
        } else {
            None
        }
    }
}

// ============================================================================
// Backup State
// ============================================================================

/// State for AWS Backup service
#[derive(Default)]
pub struct BackupState {
    pub vaults: Vec<BackupVault>,
    pub plans: Vec<BackupPlan>,
    pub jobs: Vec<BackupJob>,
    pub list_state: TableState,
    pub view_mode: BackupViewMode,
}

impl BackupState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected vault, if any
    pub fn selected_vault(&self) -> Option<&BackupVault> {
        if self.view_mode == BackupViewMode::Vaults {
            self.list_state
                .selected()
                .and_then(|i| self.vaults.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected plan, if any
    pub fn selected_plan(&self) -> Option<&BackupPlan> {
        if self.view_mode == BackupViewMode::Plans {
            self.list_state.selected().and_then(|i| self.plans.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected job, if any
    pub fn selected_job(&self) -> Option<&BackupJob> {
        if self.view_mode == BackupViewMode::Jobs {
            self.list_state.selected().and_then(|i| self.jobs.get(i))
        } else {
            None
        }
    }
}

// ============================================================================
// CloudTrail State
// ============================================================================

/// State for CloudTrail service
#[derive(Default)]
pub struct CloudTrailState {
    pub trails: Vec<Trail>,
    pub events: Vec<CloudTrailEvent>,
    pub list_state: TableState,
    pub view_mode: CloudTrailViewMode,
}

impl CloudTrailState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected trail, if any
    pub fn selected_trail(&self) -> Option<&Trail> {
        if self.view_mode == CloudTrailViewMode::Trails {
            self.list_state
                .selected()
                .and_then(|i| self.trails.get(i))
        } else {
            None
        }
    }

    /// Get the currently selected event, if any
    pub fn selected_event(&self) -> Option<&CloudTrailEvent> {
        if self.view_mode == CloudTrailViewMode::Events {
            self.list_state
                .selected()
                .and_then(|i| self.events.get(i))
        } else {
            None
        }
    }
}

// ============================================================================
// Service States Container
// ============================================================================

/// Container for all service-specific states
/// 
/// This consolidates what was previously 50+ individual fields in the App struct
/// into a single organized structure with clear ownership.
#[derive(Default)]
pub struct ServiceStates {
    pub ec2: Ec2State,
    pub s3: S3State,
    pub rds: RdsState,
    pub dynamodb: DynamoDbState,
    pub lambda: LambdaState,
    pub vpc: VpcState,
    pub iam: IamState,
    pub backup: BackupState,
    pub cloudtrail: CloudTrailState,
}

impl ServiceStates {
    pub fn new() -> Self {
        Self {
            ec2: Ec2State::new(),
            s3: S3State::new(),
            rds: RdsState::new(),
            dynamodb: DynamoDbState::new(),
            lambda: LambdaState::new(),
            vpc: VpcState::new(),
            iam: IamState::new(),
            backup: BackupState::new(),
            cloudtrail: CloudTrailState::new(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ec2_state_new() {
        let state = Ec2State::new();
        assert!(state.instances.is_empty());
        assert!(state.list_state.selected().is_none());
    }

    #[test]
    fn test_ec2_state_selected_instance() {
        use crate::models::ec2::InstanceState;

        let mut state = Ec2State::new();
        assert!(state.selected_instance().is_none());

        // Add an instance and select it
        state.instances.push(Ec2Instance {
            instance_id: "i-1234567890abcdef0".to_string(),
            name: Some("test-instance".to_string()),
            instance_type: "t2.micro".to_string(),
            state: InstanceState::Running,
            public_ip: None,
            private_ip: Some("10.0.0.1".to_string()),
            launch_time: None,
            vpc_id: None,
            subnet_id: None,
            availability_zone: None,
            architecture: None,
            platform: None,
            ami_id: None,
            key_name: None,
            monitoring_state: None,
            security_groups: vec![],
        });
        state.list_state.select(Some(0));

        assert!(state.selected_instance().is_some());
        assert_eq!(state.selected_instance_id(), Some("i-1234567890abcdef0".to_string()));
    }

    #[test]
    fn test_s3_state_viewing_objects() {
        let mut state = S3State::new();
        assert!(!state.is_viewing_objects());

        state.current_bucket = Some("my-bucket".to_string());
        assert!(state.is_viewing_objects());
    }

    #[test]
    fn test_vpc_state_new_defaults() {
        let state = VpcState::new();
        assert!(state.sg_rules_inbound); // Should default to true
        assert_eq!(state.view_mode, VpcViewMode::Vpcs);
    }

    #[test]
    fn test_service_states_new() {
        let states = ServiceStates::new();
        assert!(states.ec2.instances.is_empty());
        assert!(states.s3.buckets.is_empty());
        assert!(states.rds.instances.is_empty());
        assert!(states.dynamodb.tables.is_empty());
        assert!(states.lambda.functions.is_empty());
        assert!(states.vpc.vpcs.is_empty());
        assert!(states.iam.users.is_empty());
        assert!(states.backup.vaults.is_empty());
        assert!(states.cloudtrail.trails.is_empty());
    }

    #[test]
    fn test_dynamodb_state_viewing_items() {
        let mut state = DynamoDbState::new();
        assert!(!state.is_viewing_items());

        state.current_table = Some("my-table".to_string());
        assert!(state.is_viewing_items());
    }
}
