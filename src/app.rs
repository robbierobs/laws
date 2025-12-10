use tokio::sync::mpsc;
use ratatui::Frame;
use crate::event::{Event, AwsEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    EC2,
    S3,
    RDS,
    DynamoDB,
    Lambda,
    VPC,
    IAM,
    Backup,
    CloudTrail,
}

impl Service {
    pub fn as_str(&self) -> &str {
        match self {
            Service::EC2 => "EC2",
            Service::S3 => "S3",
            Service::RDS => "RDS",
            Service::DynamoDB => "DynamoDB",
            Service::Lambda => "Lambda",
            Service::VPC => "VPC",
            Service::IAM => "IAM",
            Service::Backup => "Backup",
            Service::CloudTrail => "CloudTrail",
        }
    }

    pub fn iterator() -> impl Iterator<Item = Self> {
        [
            Self::EC2,
            Self::S3,
            Self::RDS,
            Self::DynamoDB,
            Self::Lambda,
            Self::VPC,
            Self::IAM,
            Self::Backup,
            Self::CloudTrail,
        ]
        .iter()
        .copied()
    }
}

pub enum Message {
    // Navigation
    NavigateToService(Service),
    NavigateBack,

    // Selection
    SelectNext,
    SelectPrevious,
    SelectItem(usize),

    // Actions
    RefreshData,
    ConfirmAction,
    CancelAction,
    StartInstance(String),
    StopInstance(String),
    RebootInstance(String),
    TerminateInstance(String),
    LoadS3Objects(String),
    LoadBucketDetails(String),
    LeaveS3Bucket,
    // RDS actions
    StartRdsInstance(String),
    StopRdsInstance(String),
    RebootRdsInstance(String),

    // Async results
    // DataLoaded(ServiceData),
    // ActionCompleted(ActionResult),
    Error(String),

    // UI
    ToggleDetailPanel,
    ShowHelp,
    Quit,
}

use crate::ui::components::sidebar::Sidebar;
use crate::ui::components::Component;
use crossterm::event::{KeyCode, KeyEvent};

// ... (Service enum remains)

pub enum Focus {
    Sidebar,
    Main,
}

use crate::aws::client::AwsClients;
use crate::models::dynamodb::DynamoDbTable;
use crate::models::ec2::Ec2Instance;
use crate::models::lambda::LambdaFunction;
use crate::models::rds::RdsInstance;
use crate::models::s3::{S3Bucket, S3BucketDetails};

use ratatui::widgets::TableState;
use std::collections::HashMap;

pub struct App {
    pub should_quit: bool,
    pub current_service: Service,
    pub sidebar: Sidebar,
    pub focus: Focus,
    pub aws_clients: Option<AwsClients>,
    pub profile: Option<String>,
    pub region: String,
    pub ec2_instances: Vec<Ec2Instance>,
    pub ec2_list_state: TableState,
    pub s3_buckets: Vec<S3Bucket>,
    pub s3_list_state: TableState,
    pub current_bucket: Option<String>,
    pub s3_objects: Vec<crate::models::s3::S3Object>,
    pub s3_object_list_state: TableState,
    pub loading: bool,
    pub should_refresh: bool,
    pub error_message: Option<String>,
    // Detail panel state
    pub detail_panel_visible: bool,
    pub s3_bucket_details: HashMap<String, S3BucketDetails>,
    pub detail_loading: bool,
    // RDS state
    pub rds_instances: Vec<RdsInstance>,
    pub rds_list_state: TableState,
    // DynamoDB state
    pub dynamodb_tables: Vec<DynamoDbTable>,
    pub dynamodb_list_state: TableState,
    // Lambda state
    pub lambda_functions: Vec<LambdaFunction>,
    pub lambda_list_state: TableState,
}

impl App {
    pub fn new(aws_clients: Option<AwsClients>, profile: Option<String>, region: String) -> Self {
        Self {
            should_quit: false,
            current_service: Service::EC2,
            sidebar: Sidebar::new(),
            focus: Focus::Sidebar,
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
            lambda_functions: Vec::new(),
            lambda_list_state: TableState::default(),
        }
    }

    pub fn update<'a>(&'a mut self, message: Message, event_tx: mpsc::UnboundedSender<Event>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            match message {
                Message::Quit => self.should_quit = true,
                Message::NavigateToService(service) => {
                    self.current_service = service;
                    // Trigger data refresh when switching services
                    self.update(Message::RefreshData, event_tx).await;
                }
                Message::RefreshData => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        match self.current_service {
                            Service::EC2 => {
                                let client = clients.ec2.clone();
                                let tx = event_tx.clone();
                                tokio::spawn(async move {
                                    let service = crate::aws::ec2::Ec2Service::new(client);
                                    match service.list_instances().await {
                                        Ok(instances) => {
                                            tx.send(Event::Aws(AwsEvent::Ec2InstancesLoaded(instances))).ok();
                                        }
                                        Err(e) => {
                                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                        }
                                    }
                                });
                            }
                            Service::S3 => {
                                let client = clients.s3.clone();
                                let tx = event_tx.clone();
                                tokio::spawn(async move {
                                    let service = crate::aws::s3::S3Service::new(client);
                                    match service.list_buckets().await {
                                        Ok(buckets) => {
                                            tx.send(Event::Aws(AwsEvent::S3BucketsLoaded(buckets))).ok();
                                        }
                                        Err(e) => {
                                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                        }
                                    }
                                });
                            }
                            Service::RDS => {
                                let client = clients.rds.clone();
                                let tx = event_tx.clone();
                                tokio::spawn(async move {
                                    let service = crate::aws::rds::RdsService::new(client);
                                    match service.list_instances().await {
                                        Ok(instances) => {
                                            tx.send(Event::Aws(AwsEvent::RdsInstancesLoaded(instances))).ok();
                                        }
                                        Err(e) => {
                                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                        }
                                    }
                                });
                            }
                            Service::DynamoDB => {
                                let client = clients.dynamodb.clone();
                                let tx = event_tx.clone();
                                tokio::spawn(async move {
                                    let service = crate::aws::dynamodb::DynamoDbService::new(client);
                                    match service.list_tables().await {
                                        Ok(tables) => {
                                            tx.send(Event::Aws(AwsEvent::DynamoDbTablesLoaded(tables))).ok();
                                        }
                                        Err(e) => {
                                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                        }
                                    }
                                });
                            }
                            Service::Lambda => {
                                let client = clients.lambda.clone();
                                let tx = event_tx.clone();
                                tokio::spawn(async move {
                                    let service = crate::aws::lambda::LambdaService::new(client);
                                    match service.list_functions().await {
                                        Ok(functions) => {
                                            tx.send(Event::Aws(AwsEvent::LambdaFunctionsLoaded(functions))).ok();
                                        }
                                        Err(e) => {
                                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                        }
                                    }
                                });
                            }
                            _ => {
                                // TODO: Implement other services
                                self.loading = false;
                            }
                        }
                    }
                }
                Message::StartInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.ec2.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::ec2::Ec2Service::new(client);
                            match service.start_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Started instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                Message::StopInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.ec2.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::ec2::Ec2Service::new(client);
                            match service.stop_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Stopped instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                Message::RebootInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.ec2.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::ec2::Ec2Service::new(client);
                            match service.reboot_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Rebooted instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                Message::LoadS3Objects(bucket) => {
                    self.current_bucket = Some(bucket.clone());
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.s3.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::s3::S3Service::new(client);
                            match service.list_objects(&bucket).await {
                                Ok(objects) => {
                                    tx.send(Event::Aws(AwsEvent::S3ObjectsLoaded(objects))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                Message::LeaveS3Bucket => {
                    self.current_bucket = None;
                    self.s3_objects.clear();
                }
                Message::LoadBucketDetails(bucket_name) => {
                    if let Some(clients) = &self.aws_clients {
                        // Mark bucket details as loading
                        let mut loading_details = crate::models::s3::S3BucketDetails::default();
                        loading_details.loading = true;
                        self.s3_bucket_details.insert(bucket_name.clone(), loading_details);
                        self.detail_loading = true;
                        
                        let client = clients.s3.clone();
                        let tx = event_tx.clone();
                        let bucket = bucket_name.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::s3::S3Service::new(client);
                            let details = service.get_bucket_details(&bucket).await;
                            tx.send(Event::Aws(AwsEvent::S3BucketDetailsLoaded { 
                                bucket_name: bucket, 
                                details 
                            })).ok();
                        });
                    }
                }
                Message::ToggleDetailPanel => {
                    self.detail_panel_visible = !self.detail_panel_visible;
                }
                // RDS actions
                Message::StartRdsInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.rds.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::rds::RdsService::new(client);
                            match service.start_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Started RDS instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                Message::StopRdsInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.rds.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::rds::RdsService::new(client);
                            match service.stop_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Stopped RDS instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                Message::RebootRdsInstance(id) => {
                    if let Some(clients) = &self.aws_clients {
                        self.loading = true;
                        let client = clients.rds.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::rds::RdsService::new(client);
                            match service.reboot_instance(&id).await {
                                Ok(_) => {
                                    tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Rebooted RDS instance {}", id)))).ok();
                                }
                                Err(e) => {
                                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                                }
                            }
                        });
                    }
                }
                _ => {}
            }
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Message> {
        // Clear any error message on keypress
        self.error_message = None;
        
        if key.code == KeyCode::Tab {
            self.toggle_focus();
            return None;
        }

        // Route to focused component
        match self.focus {
            Focus::Sidebar => {
                if let Some(msg) = self.sidebar.handle_key(key) {
                    return Some(msg);
                }
            }
            Focus::Main => {
                match self.current_service {
                    Service::EC2 => {
                        match key.code {
                            KeyCode::Down | KeyCode::Char('j') => {
                                if !self.ec2_instances.is_empty() {
                                    let i = match self.ec2_list_state.selected() {
                                        Some(i) => {
                                            if i >= self.ec2_instances.len() - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    self.ec2_list_state.select(Some(i));
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if !self.ec2_instances.is_empty() {
                                    let i = match self.ec2_list_state.selected() {
                                        Some(i) => {
                                            if i == 0 {
                                                self.ec2_instances.len() - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    self.ec2_list_state.select(Some(i));
                                }
                            }
                            KeyCode::Char('s') => {
                                if let Some(i) = self.ec2_list_state.selected() {
                                    if let Some(instance) = self.ec2_instances.get(i) {
                                        return Some(Message::StartInstance(instance.instance_id.clone()));
                                    }
                                }
                            }
                            KeyCode::Char('S') => {
                                if let Some(i) = self.ec2_list_state.selected() {
                                    if let Some(instance) = self.ec2_instances.get(i) {
                                        return Some(Message::StopInstance(instance.instance_id.clone()));
                                    }
                                }
                            }
                            KeyCode::Char('R') => {
                                if let Some(i) = self.ec2_list_state.selected() {
                                    if let Some(instance) = self.ec2_instances.get(i) {
                                        return Some(Message::RebootInstance(instance.instance_id.clone()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Service::S3 => {
                        if self.current_bucket.is_some() {
                            // Object list navigation
                            match key.code {
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if !self.s3_objects.is_empty() {
                                        let i = match self.s3_object_list_state.selected() {
                                            Some(i) => {
                                                if i >= self.s3_objects.len() - 1 {
                                                    0
                                                } else {
                                                    i + 1
                                                }
                                            }
                                            None => 0,
                                        };
                                        self.s3_object_list_state.select(Some(i));
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if !self.s3_objects.is_empty() {
                                        let i = match self.s3_object_list_state.selected() {
                                            Some(i) => {
                                                if i == 0 {
                                                    self.s3_objects.len() - 1
                                                } else {
                                                    i - 1
                                                }
                                            }
                                            None => 0,
                                        };
                                        self.s3_object_list_state.select(Some(i));
                                    }
                                }
                                KeyCode::Esc | KeyCode::Backspace => {
                                    return Some(Message::LeaveS3Bucket);
                                }
                                _ => {}
                            }
                        } else {
                            // Bucket list navigation
                            match key.code {
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if !self.s3_buckets.is_empty() {
                                        let i = match self.s3_list_state.selected() {
                                            Some(i) => {
                                                if i >= self.s3_buckets.len() - 1 {
                                                    0
                                                } else {
                                                    i + 1
                                                }
                                            }
                                            None => 0,
                                        };
                                        self.s3_list_state.select(Some(i));
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if !self.s3_buckets.is_empty() {
                                        let i = match self.s3_list_state.selected() {
                                            Some(i) => {
                                                if i == 0 {
                                                    self.s3_buckets.len() - 1
                                                } else {
                                                    i - 1
                                                }
                                            }
                                            None => 0,
                                        };
                                        self.s3_list_state.select(Some(i));
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(i) = self.s3_list_state.selected() {
                                        if let Some(bucket) = self.s3_buckets.get(i) {
                                            return Some(Message::LoadS3Objects(bucket.name.clone()));
                                        }
                                    }
                                }
                                KeyCode::Char('i') => {
                                    // Load detailed bucket info
                                    if let Some(i) = self.s3_list_state.selected() {
                                        if let Some(bucket) = self.s3_buckets.get(i) {
                                            return Some(Message::LoadBucketDetails(bucket.name.clone()));
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Service::RDS => {
                        match key.code {
                            KeyCode::Down | KeyCode::Char('j') => {
                                if !self.rds_instances.is_empty() {
                                    let i = match self.rds_list_state.selected() {
                                        Some(i) => {
                                            if i >= self.rds_instances.len() - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    self.rds_list_state.select(Some(i));
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if !self.rds_instances.is_empty() {
                                    let i = match self.rds_list_state.selected() {
                                        Some(i) => {
                                            if i == 0 {
                                                self.rds_instances.len() - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    self.rds_list_state.select(Some(i));
                                }
                            }
                            KeyCode::Char('s') => {
                                if let Some(i) = self.rds_list_state.selected() {
                                    if let Some(instance) = self.rds_instances.get(i) {
                                        return Some(Message::StartRdsInstance(instance.db_instance_identifier.clone()));
                                    }
                                }
                            }
                            KeyCode::Char('S') => {
                                if let Some(i) = self.rds_list_state.selected() {
                                    if let Some(instance) = self.rds_instances.get(i) {
                                        return Some(Message::StopRdsInstance(instance.db_instance_identifier.clone()));
                                    }
                                }
                            }
                            KeyCode::Char('R') => {
                                if let Some(i) = self.rds_list_state.selected() {
                                    if let Some(instance) = self.rds_instances.get(i) {
                                        return Some(Message::RebootRdsInstance(instance.db_instance_identifier.clone()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Service::DynamoDB => {
                        match key.code {
                            KeyCode::Down | KeyCode::Char('j') => {
                                if !self.dynamodb_tables.is_empty() {
                                    let i = match self.dynamodb_list_state.selected() {
                                        Some(i) => {
                                            if i >= self.dynamodb_tables.len() - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    self.dynamodb_list_state.select(Some(i));
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if !self.dynamodb_tables.is_empty() {
                                    let i = match self.dynamodb_list_state.selected() {
                                        Some(i) => {
                                            if i == 0 {
                                                self.dynamodb_tables.len() - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    self.dynamodb_list_state.select(Some(i));
                                }
                            }
                            _ => {}
                        }
                    }
                    Service::Lambda => {
                        match key.code {
                            KeyCode::Down | KeyCode::Char('j') => {
                                if !self.lambda_functions.is_empty() {
                                    let i = match self.lambda_list_state.selected() {
                                        Some(i) => {
                                            if i >= self.lambda_functions.len() - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    self.lambda_list_state.select(Some(i));
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if !self.lambda_functions.is_empty() {
                                    let i = match self.lambda_list_state.selected() {
                                        Some(i) => {
                                            if i == 0 {
                                                self.lambda_functions.len() - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    self.lambda_list_state.select(Some(i));
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Global keys
        match key.code {
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('1') => Some(Message::NavigateToService(Service::EC2)),
            KeyCode::Char('2') => Some(Message::NavigateToService(Service::S3)),
            KeyCode::Char('3') => Some(Message::NavigateToService(Service::RDS)),
            KeyCode::Char('4') => Some(Message::NavigateToService(Service::DynamoDB)),
            KeyCode::Char('5') => Some(Message::NavigateToService(Service::Lambda)),
            KeyCode::Char('6') => Some(Message::NavigateToService(Service::VPC)),
            KeyCode::Char('7') => Some(Message::NavigateToService(Service::IAM)),
            KeyCode::Char('8') => Some(Message::NavigateToService(Service::Backup)),
            KeyCode::Char('9') => Some(Message::NavigateToService(Service::CloudTrail)),
            KeyCode::Char('d') => Some(Message::ToggleDetailPanel),
            _ => None,
        }
    }

    fn toggle_focus(&mut self) {
        match self.focus {
            Focus::Sidebar => {
                self.focus = Focus::Main;
                self.sidebar.is_focused = false;
            }
            Focus::Main => {
                self.focus = Focus::Sidebar;
                self.sidebar.is_focused = true;
            }
        }
    }

    pub fn handle_aws_event(&mut self, event: AwsEvent) {
        match event {
            AwsEvent::Ec2InstancesLoaded(instances) => {
                self.ec2_instances = instances;
                self.loading = false;
            }
            AwsEvent::S3BucketsLoaded(buckets) => {
                self.s3_buckets = buckets;
                self.loading = false;
            }
            AwsEvent::S3ObjectsLoaded(objects) => {
                self.s3_objects = objects;
                self.loading = false;
            }
            AwsEvent::RdsInstancesLoaded(instances) => {
                self.rds_instances = instances;
                self.loading = false;
            }
            AwsEvent::DynamoDbTablesLoaded(tables) => {
                self.dynamodb_tables = tables;
                self.loading = false;
            }
            AwsEvent::LambdaFunctionsLoaded(functions) => {
                self.lambda_functions = functions;
                self.loading = false;
            }
            AwsEvent::ActionCompleted(msg) => {
                self.loading = false;
                // TODO: Show success message via a notification system
                eprintln!("Action completed: {}", msg);
                
                // Trigger refresh
                // Since we are in a synchronous method, we can't await. 
                // But we can spawn a task if we had a handle, or just set a flag.
                // For now, let's assume the next tick or user interaction will pick it up? 
                // No, we need to actively trigger it.
                // A common pattern is to have an `Action` queue or similar.
                // Or, we can just send a message to the event loop if we had the sender here.
                // But we don't have the sender in `handle_aws_event`.
                // Let's add a `should_refresh` flag to App and check it in `on_tick` or `update`.
                self.should_refresh = true;
            }
            AwsEvent::Error(e) => {
                self.loading = false;
                self.detail_loading = false;
                self.error_message = Some(e);
                // Reset S3 bucket view on error so user can try again
                if self.current_bucket.is_some() {
                    self.current_bucket = None;
                    self.s3_objects.clear();
                }
            }
            AwsEvent::S3BucketDetailsLoaded { bucket_name, details } => {
                self.s3_bucket_details.insert(bucket_name, details);
                self.detail_loading = false;
            }
        }
    }

    pub fn on_tick(&mut self) {
        // Handle tick
    }

    pub fn render(&mut self, frame: &mut Frame) {
        crate::ui::render::render(frame, self);
    }
}
