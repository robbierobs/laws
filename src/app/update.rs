//! Message update/reducer function
//! 
//! Handles all application messages and updates state accordingly.

use tokio::sync::mpsc;
use crate::event::{Event, AwsEvent};
use super::{App, Message, GlobalMessage, ServiceAction, Service, VpcViewMode, IamViewMode, DynamoDbViewMode};
use super::messages::{Ec2Action, S3Action, RdsAction, DynamoDbAction, VpcAction, IamAction};
use super::task_manager::task_keys;

impl App {
    /// Main message handler - processes messages and updates application state
    pub fn update<'a>(&'a mut self, message: Message, event_tx: mpsc::UnboundedSender<Event>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            match message {
                Message::Global(global) => self.handle_global_message(global, event_tx).await,
                Message::Service(service) => self.handle_service_action(service, event_tx).await,
            }
        })
    }

    /// Handle global (non-service-specific) messages
    async fn handle_global_message(&mut self, message: GlobalMessage, event_tx: mpsc::UnboundedSender<Event>) {
        use super::InputMode;
        match message {
            GlobalMessage::Quit => self.should_quit = true,
            GlobalMessage::Navigate(service) => {
                self.current_service = service;
                self.sidebar.select_service(service);
                self.update(Message::refresh(), event_tx).await;
            }
            GlobalMessage::ConfirmAction => {
                if let Some(action) = self.pending_action.take() {
                    self.show_confirmation = false;
                    self.update(action, event_tx).await;
                }
            }
            GlobalMessage::CancelAction => {
                self.pending_action = None;
                self.show_confirmation = false;
            }
            GlobalMessage::RefreshData => {
                self.handle_refresh_data(event_tx.clone()).await;
            }
            GlobalMessage::ToggleDetailPanel => {
                self.detail_panel_visible = !self.detail_panel_visible;
                // Exit fullscreen when hiding
                if !self.detail_panel_visible {
                    self.detail_panel_fullscreen = false;
                }
                // Reset scroll when toggling
                self.detail_scroll_offset = 0;
            }
            GlobalMessage::ToggleDetailFullscreen => {
                self.detail_panel_fullscreen = !self.detail_panel_fullscreen;
                // Ensure detail panel is visible when entering fullscreen
                if self.detail_panel_fullscreen {
                    self.detail_panel_visible = true;
                }
                // Reset scroll when toggling fullscreen
                self.detail_scroll_offset = 0;
            }
            GlobalMessage::DetailScrollUp => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(5);
            }
            GlobalMessage::DetailScrollDown => {
                self.detail_scroll_offset = self.detail_scroll_offset.saturating_add(5);
            }
            GlobalMessage::ToggleActionLog => {
                self.action_log_expanded = !self.action_log_expanded;
            }
            GlobalMessage::CycleViewMode | GlobalMessage::NextView => {
                self.handle_cycle_view_mode(true);
            }
            GlobalMessage::PreviousView => {
                self.handle_cycle_view_mode(false);
            }
            GlobalMessage::OpenProfileSwitcher => {
                // Pre-select current profile
                if let Some(current) = &self.profile {
                    if let Some(idx) = self.available_profiles.iter().position(|p| p == current) {
                        self.profile_switcher_index = idx;
                    }
                } else {
                    // Default profile is at index 0
                    self.profile_switcher_index = 0;
                }
                // Initialize pending_read_only to current state
                self.pending_read_only = self.read_only;
                self.input_mode = InputMode::ProfileSwitcherProfile;
            }
            GlobalMessage::CancelProfileSwitcher => {
                self.input_mode = InputMode::Normal;
                self.pending_profile = None;
                self.profile_filter.clear();
                self.region_filter.clear();
                self.profile_filter_active = false;
                self.region_filter_active = false;
            }
            GlobalMessage::SwitchProfileRegion { profile, region, read_only } => {
                self.input_mode = InputMode::Normal;
                
                // Check if profile uses SSO and run login if needed
                let profile_name = profile.clone().unwrap_or_else(|| "default".to_string());
                if crate::utils::aws_profiles::is_sso_profile(&profile_name) {
                    self.action_log.push(format!("Running SSO login for profile: {}", profile_name));
                    // Run SSO login in background
                    let sso_result = tokio::process::Command::new("aws")
                        .args(["sso", "login", "--profile", &profile_name])
                        .output()
                        .await;
                    
                    match sso_result {
                        Ok(output) => {
                            if output.status.success() {
                                self.action_log.push(format!("SSO login successful for profile: {}", profile_name));
                            } else {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                self.action_log.push(format!("SSO login warning: {}", stderr.trim()));
                            }
                        }
                        Err(e) => {
                            self.action_log.push(format!("SSO login error: {}", e));
                        }
                    }
                }
                
                // Create new AWS clients with the new profile and region
                self.loading = true;
                let new_clients = crate::aws::client::AwsClients::new(
                    profile.as_deref(),
                    Some(region.as_str()),
                    None, // Keep existing endpoint_url handling
                ).await;
                
                match new_clients {
                    Ok(clients) => {
                        self.aws_clients = Some(clients);
                        self.profile = profile;
                        self.region = region.clone();
                        self.read_only = read_only;
                        self.pending_read_only = read_only;
                        
                        // Update profile/region indices
                        if let Some(idx) = self.available_profiles.iter().position(|p| {
                            self.profile.as_ref().map_or(p == "default", |prof| p == prof)
                        }) {
                            self.profile_switcher_index = idx;
                        }
                        if let Some(idx) = self.available_regions.iter().position(|r| r == &region) {
                            self.region_switcher_index = idx;
                        }
                        
                        // Clear all service data to force refresh
                        self.services = super::service_state::ServiceStates::new();
                        
                        let ro_status = if read_only { " [READ-ONLY]" } else { "" };
                        self.action_log.push(format!(
                            "Switched to profile: {}, region: {}{}", 
                            self.profile.as_deref().unwrap_or("default"),
                            self.region,
                            ro_status
                        ));
                        
                        // Refresh current service data
                        self.update(Message::refresh(), event_tx).await;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to switch profile: {}", e));
                        self.action_log.push(format!("Failed to switch profile: {}", e));
                        self.loading = false;
                    }
                }
            }
        }
    }

    /// Handle service-specific actions
    async fn handle_service_action(&mut self, action: ServiceAction, event_tx: mpsc::UnboundedSender<Event>) {
        match action {
            // EC2 actions
            ServiceAction::Ec2(Ec2Action::Start(id)) => {
                self.handle_ec2_action("start", id, event_tx);
            }
            ServiceAction::Ec2(Ec2Action::Stop(id)) => {
                self.handle_ec2_action("stop", id, event_tx);
            }
            ServiceAction::Ec2(Ec2Action::Reboot(id)) => {
                self.handle_ec2_action("reboot", id, event_tx);
            }
            
            // S3 actions
            ServiceAction::S3(S3Action::LoadObjects(bucket)) => {
                self.handle_load_s3_objects(bucket, event_tx);
            }
            ServiceAction::S3(S3Action::LoadBucketDetails(bucket)) => {
                self.handle_load_bucket_details(bucket, event_tx);
            }
            ServiceAction::S3(S3Action::DeleteObject { bucket, key }) => {
                self.handle_delete_s3_object(bucket, key, event_tx);
            }
            ServiceAction::S3(S3Action::DownloadObject { bucket, key }) => {
                self.handle_download_s3_object(bucket, key, false, event_tx);
            }
            ServiceAction::S3(S3Action::OpenObject { bucket, key }) => {
                self.handle_download_s3_object(bucket, key, true, event_tx);
            }
            ServiceAction::S3(S3Action::LeaveBucket) => {
                self.services.s3.current_bucket = None;
                self.services.s3.objects.clear();
            }
            
            // RDS actions
            ServiceAction::Rds(RdsAction::Start(id)) => {
                self.handle_rds_action("start", id, event_tx);
            }
            ServiceAction::Rds(RdsAction::Stop(id)) => {
                self.handle_rds_action("stop", id, event_tx);
            }
            ServiceAction::Rds(RdsAction::Reboot(id)) => {
                self.handle_rds_action("reboot", id, event_tx);
            }
            
            // DynamoDB actions
            ServiceAction::DynamoDb(DynamoDbAction::DrillDownTable) => {
                self.handle_drill_down_dynamodb_table(event_tx);
            }
            ServiceAction::DynamoDb(DynamoDbAction::ExitDrillDown) => {
                self.services.dynamodb.view_mode = DynamoDbViewMode::Tables;
                self.services.dynamodb.current_table = None;
                self.services.dynamodb.items.clear();
                self.services.dynamodb.list_state.select(Some(0));
            }
            ServiceAction::DynamoDb(DynamoDbAction::LoadItems(table_name)) => {
                self.handle_load_dynamodb_items(table_name, event_tx);
            }
            ServiceAction::DynamoDb(DynamoDbAction::DeleteItem { table_name, key_attrs }) => {
                self.handle_delete_dynamodb_item(table_name, key_attrs, event_tx);
            }
            
            // VPC actions
            ServiceAction::Vpc(VpcAction::DrillDownSecurityGroup) => {
                self.handle_drill_down_security_group();
            }
            ServiceAction::Vpc(VpcAction::ExitSecurityGroupRules) => {
                self.services.vpc.view_mode = VpcViewMode::SecurityGroups;
                self.services.vpc.selected_sg_id = None;
                self.services.vpc.current_sg_rules.clear();
                self.services.vpc.list_state.select(Some(0));
            }
            ServiceAction::Vpc(VpcAction::ToggleSgRulesDirection) => {
                self.handle_toggle_sg_rules_direction();
            }
            
            // IAM actions
            ServiceAction::Iam(IamAction::DrillDownUser) => {
                self.handle_drill_down_iam_user(event_tx);
            }
            ServiceAction::Iam(IamAction::DrillDownRole) => {
                self.handle_drill_down_iam_role(event_tx);
            }
            ServiceAction::Iam(IamAction::DrillDownPolicy) => {
                self.handle_drill_down_iam_policy(event_tx);
            }
            ServiceAction::Iam(IamAction::ExitDrillDown) => {
                self.handle_exit_iam_drill_down();
            }
            
            // Lambda, Backup, CloudTrail - no actions currently supported
            ServiceAction::Lambda(_) => {}
            ServiceAction::Backup(_) => {}
            ServiceAction::CloudTrail(_) => {}
        }
    }

    // ===============================
    // Helper methods for update logic
    // ===============================

    async fn handle_refresh_data(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            match self.current_service {
                Service::EC2 => {
                    let client = clients.ec2.clone();
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::ec2::Ec2Service::new(client);
                        match service.list_instances().await {
                            Ok(instances) => { tx.send(Event::Aws(AwsEvent::Ec2InstancesLoaded(instances))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::EC2_REFRESH, handle);
                }
                Service::S3 => {
                    let client = clients.s3.clone();
                    let tx = event_tx.clone();
                    tokio::spawn(async move {
                        let service = crate::aws::s3::S3Service::new(client.clone());
                        match service.list_buckets().await {
                            Ok(buckets) => {
                                let bucket_names: Vec<String> = buckets.iter().map(|b| b.name.clone()).collect();
                                tx.send(Event::Aws(AwsEvent::S3BucketsLoaded(buckets))).ok();
                                
                                // Load details with rate limiting
                                let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(3));
                                for bucket_name in bucket_names.into_iter().take(20) {
                                    let permit = semaphore.clone().acquire_owned().await;
                                    if permit.is_err() { break; }
                                    
                                    let client_clone = client.clone();
                                    let tx_clone = tx.clone();
                                    let name = bucket_name.clone();
                                    
                                    tokio::spawn(async move {
                                        let _permit = permit;
                                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                        let service = crate::aws::s3::S3Service::new(client_clone);
                                        let details = service.get_bucket_details(&name).await;
                                        tx_clone.send(Event::Aws(AwsEvent::S3BucketDetailsLoaded { bucket_name: name, details })).ok();
                                    });
                                }
                            }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    // Also refresh objects if inside a bucket
                    if let Some(bucket) = &self.services.s3.current_bucket {
                        let bucket_name = bucket.clone();
                        let client = clients.s3.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::s3::S3Service::new(client);
                            match service.list_objects(&bucket_name).await {
                                Ok(objects) => { tx.send(Event::Aws(AwsEvent::S3ObjectsLoaded(objects))).ok(); }
                                Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                            }
                        });
                    }
                }
                Service::RDS => {
                    let client = clients.rds.clone();
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::rds::RdsService::new(client);
                        match service.list_instances().await {
                            Ok(instances) => { tx.send(Event::Aws(AwsEvent::RdsInstancesLoaded(instances))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::RDS_REFRESH, handle);
                }
                Service::DynamoDB => {
                    let client = clients.dynamodb.clone();
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::dynamodb::DynamoDbService::new(client);
                        match service.list_tables().await {
                            Ok(tables) => { tx.send(Event::Aws(AwsEvent::DynamoDbTablesLoaded(tables))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::DYNAMODB_REFRESH, handle);
                }
                Service::Lambda => {
                    let client = clients.lambda.clone();
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::lambda::LambdaService::new(client);
                        match service.list_functions().await {
                            Ok(functions) => { tx.send(Event::Aws(AwsEvent::LambdaFunctionsLoaded(functions))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::LAMBDA_REFRESH, handle);
                }
                Service::VPC => {
                    let client = clients.ec2.clone();
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::vpc::VpcService::new(client);
                        match service.list_vpcs().await {
                            Ok(vpcs) => { tx.send(Event::Aws(AwsEvent::VpcsLoaded(vpcs))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                        match service.list_subnets(None).await {
                            Ok(subnets) => { tx.send(Event::Aws(AwsEvent::SubnetsLoaded(subnets))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                        match service.list_security_groups(None).await {
                            Ok(sgs) => { tx.send(Event::Aws(AwsEvent::SecurityGroupsLoaded(sgs))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::VPC_REFRESH, handle);
                }
                Service::IAM => {
                    let client = clients.iam.clone();
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::iam::IamService::new(client);
                        match service.list_users().await {
                            Ok(users) => { tx.send(Event::Aws(AwsEvent::IamUsersLoaded(users))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                        match service.list_roles().await {
                            Ok(roles) => { tx.send(Event::Aws(AwsEvent::IamRolesLoaded(roles))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                        match service.list_policies().await {
                            Ok(policies) => { tx.send(Event::Aws(AwsEvent::IamPoliciesLoaded(policies))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::IAM_REFRESH, handle);
                }
                Service::Backup => {
                    let client = clients.backup.clone();
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::backup::BackupService::new(client);
                        match service.list_backup_vaults().await {
                            Ok(vaults) => { tx.send(Event::Aws(AwsEvent::BackupVaultsLoaded(vaults))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                        match service.list_backup_plans().await {
                            Ok(plans) => { tx.send(Event::Aws(AwsEvent::BackupPlansLoaded(plans))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                        match service.list_backup_jobs().await {
                            Ok(jobs) => { tx.send(Event::Aws(AwsEvent::BackupJobsLoaded(jobs))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::BACKUP_REFRESH, handle);
                }
                Service::CloudTrail => {
                    let client = clients.cloudtrail.clone();
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::cloudtrail::CloudTrailService::new(client);
                        match service.list_trails().await {
                            Ok(trails) => { tx.send(Event::Aws(AwsEvent::CloudTrailTrailsLoaded(trails))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                        match service.lookup_events(50).await {
                            Ok(events) => { tx.send(Event::Aws(AwsEvent::CloudTrailEventsLoaded(events))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::CLOUDTRAIL_REFRESH, handle);
                }
            }
        }
    }

    fn handle_ec2_action(&mut self, action: &str, id: String, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.ec2.clone();
            let tx = event_tx;
            let action = action.to_string();
            let handle = tokio::spawn(async move {
                let service = crate::aws::ec2::Ec2Service::new(client);
                let result = match action.as_str() {
                    "start" => service.start_instance(&id).await,
                    "stop" => service.stop_instance(&id).await,
                    "reboot" => service.reboot_instance(&id).await,
                    _ => return,
                };
                match result {
                    Ok(_) => { tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("{}ed instance {}", action.trim_end_matches('e'), id)))).ok(); }
                    Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                }
            });
            self.tasks.spawn(task_keys::EC2_ACTION, handle);
        }
    }

    fn handle_rds_action(&mut self, action: &str, id: String, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.rds.clone();
            let tx = event_tx;
            let action = action.to_string();
            let handle = tokio::spawn(async move {
                let service = crate::aws::rds::RdsService::new(client);
                let result = match action.as_str() {
                    "start" => service.start_instance(&id).await,
                    "stop" => service.stop_instance(&id).await,
                    "reboot" => service.reboot_instance(&id).await,
                    _ => return,
                };
                match result {
                    Ok(_) => { tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("{}ed RDS instance {}", action.trim_end_matches('e'), id)))).ok(); }
                    Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                }
            });
            self.tasks.spawn(task_keys::RDS_ACTION, handle);
        }
    }

    fn handle_load_s3_objects(&mut self, bucket: String, event_tx: mpsc::UnboundedSender<Event>) {
        self.services.s3.current_bucket = Some(bucket.clone());
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.s3.clone();
            let tx = event_tx;
            let handle = tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                match service.list_objects(&bucket).await {
                    Ok(objects) => { tx.send(Event::Aws(AwsEvent::S3ObjectsLoaded(objects))).ok(); }
                    Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                }
            });
            self.tasks.spawn(task_keys::S3_OBJECTS, handle);
        }
    }

    fn handle_delete_s3_object(&mut self, bucket: String, key: String, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.s3.clone();
            let tx = event_tx;
            let handle = tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                match service.delete_object(&bucket, &key).await {
                    Ok(_) => { tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Deleted object {}/{}", bucket, key)))).ok(); }
                    Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                }
            });
            self.tasks.spawn(task_keys::S3_ACTION, handle);
        }
    }

    fn handle_download_s3_object(&mut self, bucket: String, key: String, open_mode: bool, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.s3.clone();
            let tx = event_tx;
            let handle = tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                match service.get_object(&bucket, &key).await {
                    Ok(bytes) => {
                        // Get filename from key
                        let filename = key.split('/').last().unwrap_or(&key).to_string();
                        
                        // Determine target directory
                        let target_dir = if open_mode {
                            std::env::temp_dir().join("lazy_aws")
                        } else {
                            dirs::download_dir().unwrap_or_else(|| {
                                dirs::home_dir()
                                    .map(|h| h.join("Downloads"))
                                    .unwrap_or_else(std::env::temp_dir)
                            })
                        };
                        
                        // Create directory if it doesn't exist
                        if let Err(e) = std::fs::create_dir_all(&target_dir) {
                            tx.send(Event::Aws(AwsEvent::Error(format!("Failed to create directory: {}", e)))).ok();
                            return;
                        }
                        
                        let file_path = target_dir.join(&filename);
                        
                        // Write file
                        match std::fs::write(&file_path, &bytes) {
                            Ok(_) => {
                                let path_str = file_path.to_string_lossy().to_string();
                                if open_mode {
                                    // For open mode, try to read content if it's a text file
                                    let content = if bytes.len() < 1024 * 1024 { // < 1MB
                                        String::from_utf8(bytes).ok()
                                    } else {
                                        None
                                    };
                                    tx.send(Event::Aws(AwsEvent::S3ObjectOpened { 
                                        key: key.clone(), 
                                        path: path_str,
                                        content,
                                    })).ok();
                                } else {
                                    tx.send(Event::Aws(AwsEvent::S3ObjectDownloaded { 
                                        key: key.clone(), 
                                        path: path_str,
                                    })).ok();
                                }
                            }
                            Err(e) => {
                                tx.send(Event::Aws(AwsEvent::Error(format!("Failed to write file: {}", e)))).ok();
                            }
                        }
                    }
                    Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                }
            });
            self.tasks.spawn(task_keys::S3_ACTION, handle);
        }
    }

    fn handle_load_bucket_details(&mut self, bucket_name: String, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            let mut loading_details = crate::models::s3::S3BucketDetails::default();
            loading_details.loading = true;
            self.services.s3.bucket_details.insert(bucket_name.clone(), loading_details);
            self.detail_loading = true;
            
            let client = clients.s3.clone();
            let tx = event_tx;
            let bucket = bucket_name.clone();
            let handle = tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                let details = service.get_bucket_details(&bucket).await;
                tx.send(Event::Aws(AwsEvent::S3BucketDetailsLoaded { bucket_name: bucket, details })).ok();
            });
            self.tasks.spawn(task_keys::S3_DETAILS, handle);
        }
    }

    fn handle_cycle_view_mode(&mut self, forward: bool) {
        match self.current_service {
            Service::Backup => {
                self.services.backup.view_mode = if forward { self.services.backup.view_mode.next() } else { self.services.backup.view_mode.previous() };
                self.services.backup.list_state.select(None);
            }
            Service::CloudTrail => {
                self.services.cloudtrail.view_mode = if forward { self.services.cloudtrail.view_mode.next() } else { self.services.cloudtrail.view_mode.previous() };
                self.services.cloudtrail.list_state.select(None);
            }
            Service::VPC => {
                self.services.vpc.view_mode = if forward { self.services.vpc.view_mode.next() } else { self.services.vpc.view_mode.previous() };
                self.services.vpc.list_state.select(None);
            }
            Service::IAM => {
                self.services.iam.view_mode = if forward { self.services.iam.view_mode.next() } else { self.services.iam.view_mode.previous() };
                self.services.iam.list_state.select(None);
            }
            _ => {}
        }
    }

    fn handle_drill_down_security_group(&mut self) {
        if let Some(idx) = self.services.vpc.list_state.selected() {
            if let Some(sg) = self.services.vpc.security_groups.get(idx) {
                self.services.vpc.selected_sg_id = Some(sg.group_id.clone());
                self.services.vpc.sg_rules_inbound = true;
                self.services.vpc.current_sg_rules = sg.inbound_rules.clone();
                self.services.vpc.view_mode = VpcViewMode::SecurityGroupRules;
                self.services.vpc.list_state.select(Some(0));
            }
        }
    }

    fn handle_toggle_sg_rules_direction(&mut self) {
        if let Some(sg_id) = &self.services.vpc.selected_sg_id.clone() {
            if let Some(sg) = self.services.vpc.security_groups.iter().find(|s| &s.group_id == sg_id) {
                self.services.vpc.sg_rules_inbound = !self.services.vpc.sg_rules_inbound;
                self.services.vpc.current_sg_rules = if self.services.vpc.sg_rules_inbound {
                    sg.inbound_rules.clone()
                } else {
                    sg.outbound_rules.clone()
                };
                self.services.vpc.list_state.select(Some(0));
            }
        }
    }

    fn handle_drill_down_dynamodb_table(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.services.dynamodb.list_state.selected() {
            if let Some(table) = self.services.dynamodb.tables.get(idx) {
                let table_name = table.table_name.clone();
                self.services.dynamodb.current_table = Some(table_name.clone());
                self.services.dynamodb.view_mode = DynamoDbViewMode::Items;
                self.services.dynamodb.items.clear();
                self.services.dynamodb.item_list_state.select(None);
                
                // Load items
                if let Some(clients) = &self.aws_clients {
                    self.loading = true;
                    let client = clients.dynamodb.clone();
                    let tx = event_tx;
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::dynamodb::DynamoDbService::new(client);
                        match service.scan_items(&table_name, 100).await {
                            Ok(items) => { tx.send(Event::Aws(AwsEvent::DynamoDbItemsLoaded(items))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::DYNAMODB_ITEMS, handle);
                }
            }
        }
    }

    fn handle_load_dynamodb_items(&mut self, table_name: String, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.dynamodb.clone();
            let tx = event_tx;
            let handle = tokio::spawn(async move {
                let service = crate::aws::dynamodb::DynamoDbService::new(client);
                match service.scan_items(&table_name, 100).await {
                    Ok(items) => { tx.send(Event::Aws(AwsEvent::DynamoDbItemsLoaded(items))).ok(); }
                    Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                }
            });
            self.tasks.spawn(task_keys::DYNAMODB_ITEMS, handle);
        }
    }

    fn handle_delete_dynamodb_item(&mut self, table_name: String, key_attrs: std::collections::HashMap<String, String>, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            // Get key schema from current table to figure out which attributes are keys
            let table = self.services.dynamodb.tables.iter().find(|t| t.table_name == table_name);
            if let Some(t) = table {
                let pk_name = t.partition_key.as_ref().map(|k| k.name.clone());
                let sk_name = t.sort_key.as_ref().map(|k| k.name.clone());
                let pk_type = t.partition_key.as_ref().map(|k| k.attribute_type.clone());
                let sk_type = t.sort_key.as_ref().map(|k| k.attribute_type.clone());
                
                self.loading = true;
                let client = clients.dynamodb.clone();
                let tx = event_tx;
                let tbl = table_name.clone();
                
                tokio::spawn(async move {
                    use aws_sdk_dynamodb::types::AttributeValue;
                    
                    let mut key = std::collections::HashMap::new();
                    
                    // Build key from attributes
                    if let Some(pk) = pk_name {
                        if let Some(val) = key_attrs.get(&pk) {
                            let av = match pk_type.as_deref() {
                                Some("N") => AttributeValue::N(val.clone()),
                                _ => AttributeValue::S(val.clone()),
                            };
                            key.insert(pk, av);
                        }
                    }
                    if let Some(sk) = sk_name {
                        if let Some(val) = key_attrs.get(&sk) {
                            let av = match sk_type.as_deref() {
                                Some("N") => AttributeValue::N(val.clone()),
                                _ => AttributeValue::S(val.clone()),
                            };
                            key.insert(sk, av);
                        }
                    }
                    
                    let service = crate::aws::dynamodb::DynamoDbService::new(client);
                    match service.delete_item(&tbl, key).await {
                        Ok(_) => { 
                            tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Deleted item from {}", tbl)))).ok();
                        }
                        Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                    }
                });
            }
        }
    }

    fn handle_drill_down_iam_user(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.services.iam.list_state.selected() {
            if let Some(user) = self.services.iam.users.get(idx) {
                if let Some(clients) = &self.aws_clients {
                    self.services.iam.selected_entity_name = Some(user.user_name.clone());
                    self.loading = true;
                    let client = clients.iam.clone();
                    let tx = event_tx;
                    let name = user.user_name.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::iam::IamService::new(client);
                        match service.list_attached_user_policies(&name).await {
                            Ok(policies) => { tx.send(Event::Aws(AwsEvent::IamUserPoliciesLoaded(policies))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::IAM_POLICIES, handle);
                }
            }
        }
    }

    fn handle_drill_down_iam_role(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.services.iam.list_state.selected() {
            if let Some(role) = self.services.iam.roles.get(idx) {
                if let Some(clients) = &self.aws_clients {
                    self.services.iam.selected_entity_name = Some(role.role_name.clone());
                    self.loading = true;
                    let client = clients.iam.clone();
                    let tx = event_tx;
                    let name = role.role_name.clone();
                    let handle = tokio::spawn(async move {
                        let service = crate::aws::iam::IamService::new(client);
                        match service.list_attached_role_policies(&name).await {
                            Ok(policies) => { tx.send(Event::Aws(AwsEvent::IamRolePoliciesLoaded(policies))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                    self.tasks.spawn(task_keys::IAM_POLICIES, handle);
                }
            }
        }
    }

    fn handle_drill_down_iam_policy(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.services.iam.list_state.selected() {
            let policy = if self.services.iam.view_mode == IamViewMode::Policies {
                self.services.iam.policies.get(idx)
            } else {
                self.services.iam.current_policies.get(idx)
            };
            
            self.services.iam.previous_view_mode = self.services.iam.view_mode;

            if let Some(p) = policy {
                if let Some(arn) = &p.arn {
                    if let Some(clients) = &self.aws_clients {
                        self.services.iam.selected_entity_name = Some(p.policy_name.clone());
                        self.loading = true;
                        let client = clients.iam.clone();
                        let tx = event_tx;
                        let arn = arn.clone();
                        let handle = tokio::spawn(async move {
                            let service = crate::aws::iam::IamService::new(client);
                            match service.get_policy_version(&arn).await {
                                Ok(doc) => { tx.send(Event::Aws(AwsEvent::IamPolicyDocumentLoaded(doc))).ok(); }
                                Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                            }
                        });
                        self.tasks.spawn(task_keys::IAM_POLICIES, handle);
                    }
                }
            }
        }
    }

    fn handle_exit_iam_drill_down(&mut self) {
        match self.services.iam.view_mode {
            IamViewMode::UserAttachedPolicies => self.services.iam.view_mode = IamViewMode::Users,
            IamViewMode::RoleAttachedPolicies => self.services.iam.view_mode = IamViewMode::Roles,
            IamViewMode::PolicyDocument => self.services.iam.view_mode = self.services.iam.previous_view_mode,
            _ => {}
        }
        self.services.iam.current_policies.clear();
        self.services.iam.current_policy_document.clear();
        self.services.iam.selected_entity_name = None;
        self.services.iam.list_state.select(Some(0));
    }
}
