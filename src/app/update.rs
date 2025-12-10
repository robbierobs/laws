//! Message update/reducer function
//! 
//! Handles all application messages and updates state accordingly.

use tokio::sync::mpsc;
use crate::event::{Event, AwsEvent};
use super::{App, Message, Service};

impl App {
    /// Main message handler - processes messages and updates application state
    pub fn update<'a>(&'a mut self, message: Message, event_tx: mpsc::UnboundedSender<Event>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            match message {
                Message::Quit => self.should_quit = true,
                Message::NavigateToService(service) => {
                    self.current_service = service;
                    self.sidebar.select_service(service);
                    self.update(Message::RefreshData, event_tx).await;
                }
                Message::ConfirmAction => {
                    if let Some(action) = self.pending_action.take() {
                        self.show_confirmation = false;
                        self.update(action, event_tx).await;
                    }
                }
                Message::CancelAction => {
                    self.pending_action = None;
                    self.show_confirmation = false;
                }
                Message::RefreshData => {
                    self.handle_refresh_data(event_tx.clone()).await;
                }
                Message::StartInstance(id) => {
                    self.handle_ec2_action("start", id, event_tx.clone());
                }
                Message::StopInstance(id) => {
                    self.handle_ec2_action("stop", id, event_tx.clone());
                }
                Message::RebootInstance(id) => {
                    self.handle_ec2_action("reboot", id, event_tx.clone());
                }
                Message::LoadS3Objects(bucket) => {
                    self.handle_load_s3_objects(bucket, event_tx.clone());
                }
                Message::DeleteS3Object(bucket, key) => {
                    self.handle_delete_s3_object(bucket, key, event_tx.clone());
                }
                Message::LeaveS3Bucket => {
                    self.current_bucket = None;
                    self.s3_objects.clear();
                }
                Message::LoadBucketDetails(bucket_name) => {
                    self.handle_load_bucket_details(bucket_name, event_tx.clone());
                }
                Message::ToggleDetailPanel => {
                    self.detail_panel_visible = !self.detail_panel_visible;
                }
                Message::ToggleActionLog => {
                    self.action_log_expanded = !self.action_log_expanded;
                }
                Message::CycleViewMode | Message::NextView => {
                    self.handle_cycle_view_mode(true);
                }
                Message::PreviousView => {
                    self.handle_cycle_view_mode(false);
                }
                Message::StartRdsInstance(id) => {
                    self.handle_rds_action("start", id, event_tx.clone());
                }
                Message::StopRdsInstance(id) => {
                    self.handle_rds_action("stop", id, event_tx.clone());
                }
                Message::RebootRdsInstance(id) => {
                    self.handle_rds_action("reboot", id, event_tx.clone());
                }
                Message::DrillDownSecurityGroup => {
                    self.handle_drill_down_security_group();
                }
                Message::ExitSecurityGroupRules => {
                    self.vpc_view_mode = 2;
                    self.selected_sg_id = None;
                    self.current_sg_rules.clear();
                    self.vpc_list_state.select(Some(0));
                }
                Message::DrillDownIamUser => {
                    self.handle_drill_down_iam_user(event_tx.clone());
                }
                Message::DrillDownIamRole => {
                    self.handle_drill_down_iam_role(event_tx.clone());
                }
                Message::DrillDownIamPolicy => {
                    self.handle_drill_down_iam_policy(event_tx.clone());
                }
                Message::ExitIamDrillDown => {
                    self.handle_exit_iam_drill_down();
                }
            }
        })
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
                    tokio::spawn(async move {
                        let service = crate::aws::ec2::Ec2Service::new(client);
                        match service.list_instances().await {
                            Ok(instances) => { tx.send(Event::Aws(AwsEvent::Ec2InstancesLoaded(instances))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
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
                    if let Some(bucket) = &self.current_bucket {
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
                    tokio::spawn(async move {
                        let service = crate::aws::rds::RdsService::new(client);
                        match service.list_instances().await {
                            Ok(instances) => { tx.send(Event::Aws(AwsEvent::RdsInstancesLoaded(instances))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                }
                Service::DynamoDB => {
                    let client = clients.dynamodb.clone();
                    let tx = event_tx.clone();
                    tokio::spawn(async move {
                        let service = crate::aws::dynamodb::DynamoDbService::new(client);
                        match service.list_tables().await {
                            Ok(tables) => { tx.send(Event::Aws(AwsEvent::DynamoDbTablesLoaded(tables))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                }
                Service::Lambda => {
                    let client = clients.lambda.clone();
                    let tx = event_tx.clone();
                    tokio::spawn(async move {
                        let service = crate::aws::lambda::LambdaService::new(client);
                        match service.list_functions().await {
                            Ok(functions) => { tx.send(Event::Aws(AwsEvent::LambdaFunctionsLoaded(functions))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                }
                Service::VPC => {
                    let client = clients.ec2.clone();
                    let tx = event_tx.clone();
                    tokio::spawn(async move {
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
                }
                Service::IAM => {
                    let client = clients.iam.clone();
                    let tx = event_tx.clone();
                    tokio::spawn(async move {
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
                }
                Service::Backup => {
                    let client = clients.backup.clone();
                    let tx = event_tx.clone();
                    tokio::spawn(async move {
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
                }
                Service::CloudTrail => {
                    let client = clients.cloudtrail.clone();
                    let tx = event_tx.clone();
                    tokio::spawn(async move {
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
            tokio::spawn(async move {
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
        }
    }

    fn handle_rds_action(&mut self, action: &str, id: String, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.rds.clone();
            let tx = event_tx;
            let action = action.to_string();
            tokio::spawn(async move {
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
        }
    }

    fn handle_load_s3_objects(&mut self, bucket: String, event_tx: mpsc::UnboundedSender<Event>) {
        self.current_bucket = Some(bucket.clone());
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.s3.clone();
            let tx = event_tx;
            tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                match service.list_objects(&bucket).await {
                    Ok(objects) => { tx.send(Event::Aws(AwsEvent::S3ObjectsLoaded(objects))).ok(); }
                    Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                }
            });
        }
    }

    fn handle_delete_s3_object(&mut self, bucket: String, key: String, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            self.loading = true;
            let client = clients.s3.clone();
            let tx = event_tx;
            tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                match service.delete_object(&bucket, &key).await {
                    Ok(_) => { tx.send(Event::Aws(AwsEvent::ActionCompleted(format!("Deleted object {}/{}", bucket, key)))).ok(); }
                    Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                }
            });
        }
    }

    fn handle_load_bucket_details(&mut self, bucket_name: String, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(clients) = &self.aws_clients {
            let mut loading_details = crate::models::s3::S3BucketDetails::default();
            loading_details.loading = true;
            self.s3_bucket_details.insert(bucket_name.clone(), loading_details);
            self.detail_loading = true;
            
            let client = clients.s3.clone();
            let tx = event_tx;
            let bucket = bucket_name.clone();
            tokio::spawn(async move {
                let service = crate::aws::s3::S3Service::new(client);
                let details = service.get_bucket_details(&bucket).await;
                tx.send(Event::Aws(AwsEvent::S3BucketDetailsLoaded { bucket_name: bucket, details })).ok();
            });
        }
    }

    fn handle_cycle_view_mode(&mut self, forward: bool) {
        match self.current_service {
            Service::Backup => {
                self.backup_view_mode = if forward { (self.backup_view_mode + 1) % 3 } else { (self.backup_view_mode + 2) % 3 };
                self.backup_list_state.select(None);
            }
            Service::CloudTrail => {
                self.cloudtrail_view_mode = if forward { (self.cloudtrail_view_mode + 1) % 2 } else { (self.cloudtrail_view_mode + 1) % 2 };
                self.cloudtrail_list_state.select(None);
            }
            Service::VPC => {
                self.vpc_view_mode = if forward { (self.vpc_view_mode + 1) % 3 } else { (self.vpc_view_mode + 2) % 3 };
                self.vpc_list_state.select(None);
            }
            Service::IAM => {
                self.iam_view_mode = if forward { (self.iam_view_mode + 1) % 3 } else { (self.iam_view_mode + 2) % 3 };
                self.iam_list_state.select(None);
            }
            _ => {}
        }
    }

    fn handle_drill_down_security_group(&mut self) {
        if let Some(idx) = self.vpc_list_state.selected() {
            if let Some(sg) = self.security_groups.get(idx) {
                self.selected_sg_id = Some(sg.group_id.clone());
                self.current_sg_rules = sg.inbound_rules.clone();
                self.vpc_view_mode = 3;
                self.vpc_list_state.select(Some(0));
            }
        }
    }

    fn handle_drill_down_iam_user(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.iam_list_state.selected() {
            if let Some(user) = self.iam_users.get(idx) {
                if let Some(clients) = &self.aws_clients {
                    self.selected_iam_entity_name = Some(user.user_name.clone());
                    self.loading = true;
                    let client = clients.iam.clone();
                    let tx = event_tx;
                    let name = user.user_name.clone();
                    tokio::spawn(async move {
                        let service = crate::aws::iam::IamService::new(client);
                        match service.list_attached_user_policies(&name).await {
                            Ok(policies) => { tx.send(Event::Aws(AwsEvent::IamUserPoliciesLoaded(policies))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                }
            }
        }
    }

    fn handle_drill_down_iam_role(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.iam_list_state.selected() {
            if let Some(role) = self.iam_roles.get(idx) {
                if let Some(clients) = &self.aws_clients {
                    self.selected_iam_entity_name = Some(role.role_name.clone());
                    self.loading = true;
                    let client = clients.iam.clone();
                    let tx = event_tx;
                    let name = role.role_name.clone();
                    tokio::spawn(async move {
                        let service = crate::aws::iam::IamService::new(client);
                        match service.list_attached_role_policies(&name).await {
                            Ok(policies) => { tx.send(Event::Aws(AwsEvent::IamRolePoliciesLoaded(policies))).ok(); }
                            Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                        }
                    });
                }
            }
        }
    }

    fn handle_drill_down_iam_policy(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.iam_list_state.selected() {
            let policy = if self.iam_view_mode == 2 {
                self.iam_policies.get(idx)
            } else {
                self.current_iam_policies.get(idx)
            };
            
            self.previous_iam_view_mode = self.iam_view_mode;

            if let Some(p) = policy {
                if let Some(arn) = &p.arn {
                    if let Some(clients) = &self.aws_clients {
                        self.selected_iam_entity_name = Some(p.policy_name.clone());
                        self.loading = true;
                        let client = clients.iam.clone();
                        let tx = event_tx;
                        let arn = arn.clone();
                        tokio::spawn(async move {
                            let service = crate::aws::iam::IamService::new(client);
                            match service.get_policy_version(&arn).await {
                                Ok(doc) => { tx.send(Event::Aws(AwsEvent::IamPolicyDocumentLoaded(doc))).ok(); }
                                Err(e) => { tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(); }
                            }
                        });
                    }
                }
            }
        }
    }

    fn handle_exit_iam_drill_down(&mut self) {
        match self.iam_view_mode {
            3 => self.iam_view_mode = 0,
            4 => self.iam_view_mode = 1,
            5 => self.iam_view_mode = self.previous_iam_view_mode,
            _ => {}
        }
        self.current_iam_policies.clear();
        self.current_policy_document.clear();
        self.selected_iam_entity_name = None;
        self.iam_list_state.select(Some(0));
    }
}
