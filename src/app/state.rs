//! App struct definition and constructors

use ratatui::widgets::TableState;
use std::collections::HashMap;

use crate::aws::client::AwsClients;
use crate::models::backup::{BackupVault, BackupPlan, BackupJob};
use crate::models::cloudtrail::{Trail, CloudTrailEvent};
use crate::models::dynamodb::{DynamoDbTable, DynamoDbItem};
use crate::models::ec2::Ec2Instance;
use crate::models::iam::{IamRole, IamUser, IamPolicy};
use crate::models::lambda::LambdaFunction;
use crate::models::rds::RdsInstance;
use crate::models::s3::{S3Bucket, S3BucketDetails, S3Object};
use crate::models::vpc::{Vpc, Subnet, SecurityGroup, SecurityGroupRule};
use crate::ui::components::sidebar::Sidebar;

use super::{Service, Message, Focus, InputMode, VpcViewMode, IamViewMode, BackupViewMode, CloudTrailViewMode, DynamoDbViewMode};

/// Main application state
pub struct App {
    pub should_quit: bool,
    pub current_service: Service,
    pub sidebar: Sidebar,
    pub focus: Focus,
    pub input_mode: InputMode,
    pub filter_input: String,
    pub aws_clients: Option<AwsClients>,
    pub profile: Option<String>,
    pub region: String,
    
    // EC2 state
    pub ec2_instances: Vec<Ec2Instance>,
    pub ec2_list_state: TableState,
    
    // S3 state
    pub s3_buckets: Vec<S3Bucket>,
    pub s3_list_state: TableState,
    pub current_bucket: Option<String>,
    pub s3_objects: Vec<S3Object>,
    pub s3_object_list_state: TableState,
    pub s3_bucket_details: HashMap<String, S3BucketDetails>,
    
    // Loading states
    pub loading: bool,
    pub should_refresh: bool,
    pub error_message: Option<String>,
    pub detail_panel_visible: bool,
    pub detail_loading: bool,
    
    // RDS state
    pub rds_instances: Vec<RdsInstance>,
    pub rds_list_state: TableState,
    
    // DynamoDB state
    pub dynamodb_tables: Vec<DynamoDbTable>,
    pub dynamodb_list_state: TableState,
    pub dynamodb_view_mode: DynamoDbViewMode,
    pub current_dynamodb_table: Option<String>,
    pub dynamodb_items: Vec<DynamoDbItem>,
    pub dynamodb_item_list_state: TableState,
    
    // Lambda state
    pub lambda_functions: Vec<LambdaFunction>,
    pub lambda_list_state: TableState,
    
    // VPC state
    pub vpcs: Vec<Vpc>,
    pub subnets: Vec<Subnet>,
    pub security_groups: Vec<SecurityGroup>,
    pub vpc_list_state: TableState,
    pub vpc_view_mode: VpcViewMode,
    pub current_sg_rules: Vec<SecurityGroupRule>,
    pub selected_sg_id: Option<String>,
    pub sg_rules_inbound: bool, // true = showing inbound, false = showing outbound
    
    // IAM state
    pub iam_roles: Vec<IamRole>,
    pub iam_users: Vec<IamUser>,
    pub iam_policies: Vec<IamPolicy>,
    pub iam_list_state: TableState,
    pub iam_view_mode: IamViewMode,
    pub previous_iam_view_mode: IamViewMode,
    pub current_iam_policies: Vec<IamPolicy>,
    pub current_policy_document: String,
    pub selected_iam_entity_name: Option<String>,
    
    // Backup state
    pub backup_vaults: Vec<BackupVault>,
    pub backup_plans: Vec<BackupPlan>,
    pub backup_jobs: Vec<BackupJob>,
    pub backup_list_state: TableState,
    pub backup_view_mode: BackupViewMode,
    
    // CloudTrail state
    pub cloudtrail_trails: Vec<Trail>,
    pub cloudtrail_events: Vec<CloudTrailEvent>,
    pub cloudtrail_list_state: TableState,
    pub cloudtrail_view_mode: CloudTrailViewMode,
    
    // Config and State
    pub read_only: bool,
    pub pending_action: Option<Message>,
    pub show_confirmation: bool,
    
    // Action Log
    pub action_log: Vec<String>,
    pub action_log_expanded: bool,
}

impl App {
    pub fn new(aws_clients: Option<AwsClients>, profile: Option<String>, region: String, read_only: bool) -> Self {
        Self {
            should_quit: false,
            current_service: Service::EC2,
            sidebar: Sidebar::new(),
            focus: Focus::Sidebar,
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            aws_clients,
            profile,
            region,
            ec2_instances: Vec::new(),
            ec2_list_state: TableState::default(),
            s3_buckets: Vec::new(),
            s3_list_state: TableState::default(),
            current_bucket: None,
            s3_objects: Vec::new(),
            s3_object_list_state: TableState::default(),
            loading: false,
            should_refresh: false,
            error_message: None,
            detail_panel_visible: true,
            s3_bucket_details: HashMap::new(),
            detail_loading: false,
            rds_instances: Vec::new(),
            rds_list_state: TableState::default(),
            dynamodb_tables: Vec::new(),
            dynamodb_list_state: TableState::default(),
            dynamodb_view_mode: DynamoDbViewMode::default(),
            current_dynamodb_table: None,
            dynamodb_items: Vec::new(),
            dynamodb_item_list_state: TableState::default(),
            lambda_functions: Vec::new(),
            lambda_list_state: TableState::default(),
            vpcs: Vec::new(),
            subnets: Vec::new(),
            security_groups: Vec::new(),
            vpc_list_state: TableState::default(),
            vpc_view_mode: VpcViewMode::default(),
            current_sg_rules: Vec::new(),
            selected_sg_id: None,
            sg_rules_inbound: true,
            iam_roles: Vec::new(),
            iam_users: Vec::new(),
            iam_policies: Vec::new(),
            iam_list_state: TableState::default(),
            iam_view_mode: IamViewMode::default(),
            previous_iam_view_mode: IamViewMode::default(),
            current_iam_policies: Vec::new(),
            current_policy_document: String::new(),
            selected_iam_entity_name: None,
            backup_vaults: Vec::new(),
            backup_plans: Vec::new(),
            backup_jobs: Vec::new(),
            backup_list_state: TableState::default(),
            backup_view_mode: BackupViewMode::default(),
            cloudtrail_trails: Vec::new(),
            cloudtrail_events: Vec::new(),
            cloudtrail_list_state: TableState::default(),
            cloudtrail_view_mode: CloudTrailViewMode::default(),
            read_only,
            pending_action: None,
            show_confirmation: false,
            action_log: Vec::new(),
            action_log_expanded: false,
        }
    }
    
    pub fn on_tick(&mut self) {
        // Handle tick events if needed
    }
    
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        crate::ui::render::render(frame, self);
        
        if self.show_confirmation {
            if let Some(action) = &self.pending_action {
                let description = match action {
                    Message::StartInstance(id) => format!("Start EC2 Instance {}", id),
                    Message::StopInstance(id) => format!("Stop EC2 Instance {}", id),
                    Message::RebootInstance(id) => format!("Reboot EC2 Instance {}", id),
                    Message::StartRdsInstance(id) => format!("Start RDS Instance {}", id),
                    Message::StopRdsInstance(id) => format!("Stop RDS Instance {}", id),
                    Message::RebootRdsInstance(id) => format!("Reboot RDS Instance {}", id),
                    Message::DeleteS3Object(bucket, key) => format!("Delete S3 Object s3://{}/{}", bucket, key),
                    Message::DeleteDynamoDbItem(table, _) => format!("Delete item from DynamoDB table {}", table),
                    _ => "Unknown Action".to_string(),
                };
                crate::ui::components::modal::render_confirmation_modal(frame, frame.area(), &description);
            }
        }
    }
}
