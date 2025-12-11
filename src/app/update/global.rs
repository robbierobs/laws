//! Global message handling
//!
//! Handles application-wide messages like navigation, refresh, profile switching.

use super::super::task_manager::task_keys;
use super::super::{App, GlobalMessage, InputMode, Message, Service};
use crate::aws::traits::AwsService;
use crate::event::{AwsEvent, Event};
use tokio::sync::mpsc;

impl App {
    /// Handle global (non-service-specific) messages
    pub(super) async fn handle_global_message(
        &mut self,
        message: GlobalMessage,
        event_tx: mpsc::UnboundedSender<Event>,
    ) {
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
                if !self.detail_panel_visible {
                    self.detail_panel_fullscreen = false;
                }
                self.detail_scroll_offset = 0;
            }
            GlobalMessage::ToggleDetailFullscreen => {
                self.detail_panel_fullscreen = !self.detail_panel_fullscreen;
                if self.detail_panel_fullscreen {
                    self.detail_panel_visible = true;
                }
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
                self.handle_open_profile_switcher();
            }
            GlobalMessage::CancelProfileSwitcher => {
                self.handle_cancel_profile_switcher();
            }
            GlobalMessage::SwitchProfileRegion {
                profile,
                region,
                read_only,
            } => {
                self.handle_switch_profile_region(profile, region, read_only, event_tx)
                    .await;
            }
        }
    }

    fn handle_open_profile_switcher(&mut self) {
        if let Some(current) = &self.profile {
            if let Some(idx) = self.available_profiles.iter().position(|p| p == current) {
                self.profile_switcher_index = idx;
            }
        } else {
            self.profile_switcher_index = 0;
        }
        self.pending_read_only = self.read_only;
        self.input_mode = InputMode::ProfileSwitcherProfile;
    }

    fn handle_cancel_profile_switcher(&mut self) {
        self.input_mode = InputMode::Normal;
        self.pending_profile = None;
        self.profile_filter.clear();
        self.region_filter.clear();
        self.profile_filter_active = false;
        self.region_filter_active = false;
    }

    async fn handle_switch_profile_region(
        &mut self,
        profile: Option<String>,
        region: String,
        read_only: bool,
        event_tx: mpsc::UnboundedSender<Event>,
    ) {
        self.input_mode = InputMode::Normal;

        // Check if profile uses SSO and run login if needed
        let profile_name = profile.clone().unwrap_or_else(|| "default".to_string());
        if crate::utils::aws_profiles::is_sso_profile(&profile_name) {
            self.action_log
                .push(format!("Running SSO login for profile: {}", profile_name));
            let sso_result = tokio::process::Command::new("aws")
                .args(["sso", "login", "--profile", &profile_name])
                .output()
                .await;

            match sso_result {
                Ok(output) => {
                    if output.status.success() {
                        self.action_log.push(format!(
                            "SSO login successful for profile: {}",
                            profile_name
                        ));
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        self.action_log
                            .push(format!("SSO login warning: {}", stderr.trim()));
                    }
                }
                Err(e) => {
                    self.action_log.push(format!("SSO login error: {}", e));
                }
            }
        }

        // Create new AWS clients with the new profile and region
        self.loading = true;
        let new_clients =
            crate::aws::client::AwsClients::new(profile.as_deref(), Some(region.as_str()), None)
                .await;

        match new_clients {
            Ok(clients) => {
                self.aws_clients = Some(clients);
                self.profile = profile;
                self.region = region.clone();
                self.read_only = read_only;
                self.pending_read_only = read_only;

                // Update profile/region indices
                if let Some(idx) = self.available_profiles.iter().position(|p| {
                    self.profile
                        .as_ref()
                        .map_or(p == "default", |prof| p == prof)
                }) {
                    self.profile_switcher_index = idx;
                }
                if let Some(idx) = self.available_regions.iter().position(|r| r == &region) {
                    self.region_switcher_index = idx;
                }

                // Clear all service data to force refresh
                self.services = super::super::service_state::ServiceStates::new();

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
                self.action_log
                    .push(format!("Failed to switch profile: {}", e));
                self.loading = false;
            }
        }
    }

    pub(super) async fn handle_refresh_data(&mut self, event_tx: mpsc::UnboundedSender<Event>) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;

        match self.current_service {
            Service::EC2 => {
                let client = clients.ec2.clone();
                let tx = event_tx.clone();
                let handle = tokio::spawn(async move {
                    let service = crate::aws::ec2::Ec2Service::new(client);
                    match service.list().await {
                        Ok(instances) => {
                            tx.send(Event::Aws(AwsEvent::Ec2InstancesLoaded(instances)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                });
                self.tasks.spawn(task_keys::EC2_REFRESH, handle);
            }
            Service::S3 => {
                let clients = clients.clone();
                self.refresh_s3(&clients, event_tx).await;
            }
            Service::RDS => {
                let client = clients.rds.clone();
                let tx = event_tx.clone();
                let handle = tokio::spawn(async move {
                    let service = crate::aws::rds::RdsService::new(client);
                    match service.list_instances().await {
                        Ok(instances) => {
                            tx.send(Event::Aws(AwsEvent::RdsInstancesLoaded(instances)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
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
                        Ok(tables) => {
                            tx.send(Event::Aws(AwsEvent::DynamoDbTablesLoaded(tables)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
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
                        Ok(functions) => {
                            tx.send(Event::Aws(AwsEvent::LambdaFunctionsLoaded(functions)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
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
                        Ok(vpcs) => {
                            tx.send(Event::Aws(AwsEvent::VpcsLoaded(vpcs))).ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                    match service.list_subnets(None).await {
                        Ok(subnets) => {
                            tx.send(Event::Aws(AwsEvent::SubnetsLoaded(subnets))).ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                    match service.list_security_groups(None).await {
                        Ok(sgs) => {
                            tx.send(Event::Aws(AwsEvent::SecurityGroupsLoaded(sgs)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
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
                        Ok(users) => {
                            tx.send(Event::Aws(AwsEvent::IamUsersLoaded(users))).ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                    match service.list_roles().await {
                        Ok(roles) => {
                            tx.send(Event::Aws(AwsEvent::IamRolesLoaded(roles))).ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                    match service.list_policies().await {
                        Ok(policies) => {
                            tx.send(Event::Aws(AwsEvent::IamPoliciesLoaded(policies)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
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
                        Ok(vaults) => {
                            tx.send(Event::Aws(AwsEvent::BackupVaultsLoaded(vaults)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                    match service.list_backup_plans().await {
                        Ok(plans) => {
                            tx.send(Event::Aws(AwsEvent::BackupPlansLoaded(plans))).ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                    match service.list_backup_jobs().await {
                        Ok(jobs) => {
                            tx.send(Event::Aws(AwsEvent::BackupJobsLoaded(jobs))).ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                });
                self.tasks.spawn(task_keys::BACKUP_REFRESH, handle);
            }
            Service::CloudTrail => {
                let client = clients.cloudtrail.clone();
                let tx = event_tx.clone();
                let limit = self.config.max_cloudtrail_events as i32;
                let handle = tokio::spawn(async move {
                    let service = crate::aws::cloudtrail::CloudTrailService::new(client);
                    match service.list_trails().await {
                        Ok(trails) => {
                            tx.send(Event::Aws(AwsEvent::CloudTrailTrailsLoaded(trails)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                    match service.lookup_events(limit).await {
                        Ok(events) => {
                            tx.send(Event::Aws(AwsEvent::CloudTrailEventsLoaded(events)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                });
                self.tasks.spawn(task_keys::CLOUDTRAIL_REFRESH, handle);
            }
            Service::SecretsManager => {
                let client = clients.secretsmanager.clone();
                let tx = event_tx.clone();
                let handle = tokio::spawn(async move {
                    let service = crate::aws::secretsmanager::SecretsManagerService::new(client);
                    match service.list_secrets().await {
                        Ok(secrets) => {
                            tx.send(Event::Aws(AwsEvent::SecretsManagerSecretsLoaded(secrets)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                        }
                    }
                });
                self.tasks.spawn(task_keys::SECRETSMANAGER_REFRESH, handle);
            }
        }
    }

    async fn refresh_s3(
        &mut self,
        clients: &crate::aws::client::AwsClients,
        event_tx: mpsc::UnboundedSender<Event>,
    ) {
        let client = clients.s3.clone();
        let tx = event_tx.clone();
        let concurrency = self.config.s3_detail_concurrency;
        let delay = self.config.s3_detail_delay_ms;

        tokio::spawn(async move {
            let service = crate::aws::s3::S3Service::new(client.clone());
            match service.list_buckets().await {
                Ok(buckets) => {
                    let bucket_names: Vec<String> =
                        buckets.iter().map(|b| b.name.clone()).collect();
                    tx.send(Event::Aws(AwsEvent::S3BucketsLoaded(buckets))).ok();

                    // Load details with rate limiting
                    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
                    for bucket_name in bucket_names.into_iter().take(20) {
                        let permit = semaphore.clone().acquire_owned().await;
                        if permit.is_err() {
                            break;
                        }

                        let client_clone = client.clone();
                        let tx_clone = tx.clone();
                        let name = bucket_name.clone();

                        tokio::spawn(async move {
                            let _permit = permit;
                            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                            let service = crate::aws::s3::S3Service::new(client_clone);
                            let details = service.get_bucket_details(&name).await;
                            tx_clone
                                .send(Event::Aws(AwsEvent::S3BucketDetailsLoaded {
                                    bucket_name: name,
                                    details,
                                }))
                                .ok();
                        });
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok();
                }
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
}
