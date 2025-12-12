//! Global message handling
//!
//! Handles application-wide messages like navigation, refresh, profile switching.

use super::super::task_manager::task_keys;
use super::super::{App, GlobalMessage, InputMode, Message, Service};
use crate::aws::traits::AwsService;
use crate::event::{AwsEvent, Event};

impl App {
    /// Handle global (non-service-specific) messages
    pub(super) async fn handle_global_message(
        &mut self,
        message: GlobalMessage,
        event_tx: crate::app::EventSender,
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
            GlobalMessage::CopyToClipboard(text) => {
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => {
                        if let Err(e) = clipboard.set_text(text.clone()) {
                            self.action_log
                                .push(format!("Failed to copy to clipboard: {}", e));
                        } else {
                            self.action_log
                                .push(format!("Copied to clipboard: {}", text));
                            // Also send a notification event if we want a popup, but action log is fine for now
                        }
                    }
                    Err(e) => {
                        self.action_log
                            .push(format!("Failed to access clipboard: {}", e));
                    }
                }
            }
            GlobalMessage::OpenGlobalSearch => {
                self.input_mode = InputMode::GlobalSearch;
                self.global_search.clear();
                // Pre-populate with all results
                self.refresh_global_search();
            }
            GlobalMessage::CloseGlobalSearch => {
                self.input_mode = InputMode::Normal;
                self.global_search.clear();
            }
            GlobalMessage::GotoSearchResult { service, resource_id } => {
                // Navigate to the service
                self.current_service = service;
                self.sidebar.select_service(service);
                
                // Try to select the resource in the appropriate service state
                self.select_resource_by_id(service, &resource_id);
                
                self.action_log.push(format!("Navigated to {} - {}", service.as_str(), resource_id));
                
                // Refresh data for the service
                self.update(Message::refresh(), event_tx).await;
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
        event_tx: crate::app::EventSender,
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
                self.services = super::super::states::ServiceStates::new();

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

    pub(super) async fn handle_refresh_data(&mut self, event_tx: crate::app::EventSender) {
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
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                });
                self.tasks.spawn(task_keys::EC2_REFRESH, handle);
            }
            Service::S3 => {
                let clients = clients.clone();
                self.handle_refresh_s3(&clients, event_tx).await;
            }
            Service::RDS => {
                let client = clients.rds.clone();
                let tx = event_tx.clone();
                let handle = tokio::spawn(async move {
                    let service = crate::aws::rds::RdsService::new(client);
                    match service.list_instances().await {
                        Ok(instances) => {
                            tx.send(Event::Aws(AwsEvent::RdsInstancesLoaded(instances)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
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
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
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
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
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
                            tx.send(Event::Aws(AwsEvent::VpcsLoaded(vpcs))).await.ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                    match service.list_subnets(None).await {
                        Ok(subnets) => {
                            tx.send(Event::Aws(AwsEvent::SubnetsLoaded(subnets)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                    match service.list_security_groups(None).await {
                        Ok(sgs) => {
                            tx.send(Event::Aws(AwsEvent::SecurityGroupsLoaded(sgs)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
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
                            tx.send(Event::Aws(AwsEvent::IamUsersLoaded(users)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                    match service.list_roles().await {
                        Ok(roles) => {
                            tx.send(Event::Aws(AwsEvent::IamRolesLoaded(roles)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                    match service.list_policies().await {
                        Ok(policies) => {
                            tx.send(Event::Aws(AwsEvent::IamPoliciesLoaded(policies)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
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
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                    match service.list_backup_plans().await {
                        Ok(plans) => {
                            tx.send(Event::Aws(AwsEvent::BackupPlansLoaded(plans)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                    match service.list_backup_jobs().await {
                        Ok(jobs) => {
                            tx.send(Event::Aws(AwsEvent::BackupJobsLoaded(jobs)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
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
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                    match service.lookup_events(limit).await {
                        Ok(events) => {
                            tx.send(Event::Aws(AwsEvent::CloudTrailEventsLoaded(events)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
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
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                });
                self.tasks.spawn(task_keys::SECRETSMANAGER_REFRESH, handle);
            }
            Service::ECS => {
                let client = clients.ecs.clone();
                let tx = event_tx.clone();
                let handle = tokio::spawn(async move {
                    let ecs_client = crate::aws::ecs::EcsClient::new(client);
                    match ecs_client.list_clusters().await {
                        Ok(clusters) => {
                            tx.send(Event::Aws(AwsEvent::EcsClustersLoaded(clusters)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                });
                self.tasks.spawn(task_keys::ECS_REFRESH, handle);
            }
            Service::ECR => {
                let client = clients.ecr.clone();
                let tx = event_tx.clone();
                let handle = tokio::spawn(async move {
                    // Create wrapper service
                    let ecr_service = crate::aws::ecr::EcrService::new(client);
                    match ecr_service.list_repositories().await {
                        Ok(repos) => {
                            tx.send(Event::Aws(AwsEvent::EcrRepositoriesLoaded(repos)))
                                .await
                                .ok();
                        }
                        Err(e) => {
                            tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                .await
                                .ok();
                        }
                    }
                });
                self.tasks.spawn(task_keys::ECR_REFRESH, handle);
            }
        }
    }

    /// Select a resource by its ID within the appropriate service state
    fn select_resource_by_id(&mut self, service: Service, resource_id: &str) {
        match service {
            Service::EC2 => {
                if let Some(idx) = self.services.ec2.instances.iter().position(|i| i.instance_id == resource_id) {
                    self.services.ec2.list_state.select(Some(idx));
                }
            }
            Service::S3 => {
                // For S3, navigate to bucket list and select the bucket
                self.services.s3.current_bucket = None; // Ensure we're at bucket level
                if let Some(idx) = self.services.s3.buckets.iter().position(|b| b.name == resource_id) {
                    self.services.s3.list_state.select(Some(idx));
                }
            }
            Service::RDS => {
                if let Some(idx) = self.services.rds.instances.iter().position(|i| i.db_instance_identifier == resource_id) {
                    self.services.rds.list_state.select(Some(idx));
                }
            }
            Service::DynamoDB => {
                if let Some(idx) = self.services.dynamodb.tables.iter().position(|t| t.table_name == resource_id) {
                    self.services.dynamodb.list_state.select(Some(idx));
                }
            }
            Service::Lambda => {
                if let Some(idx) = self.services.lambda.functions.iter().position(|f| f.function_name == resource_id) {
                    self.services.lambda.list_state.select(Some(idx));
                }
            }
            Service::VPC => {
                // Need to detect resource type from ID prefix
                if resource_id.starts_with("vpc-") {
                    self.services.vpc.view_mode = super::super::VpcViewMode::Vpcs;
                    if let Some(idx) = self.services.vpc.vpcs.iter().position(|v| v.vpc_id == resource_id) {
                        self.services.vpc.list_state.select(Some(idx));
                    }
                } else if resource_id.starts_with("subnet-") {
                    self.services.vpc.view_mode = super::super::VpcViewMode::Subnets;
                    if let Some(idx) = self.services.vpc.subnets.iter().position(|s| s.subnet_id == resource_id) {
                        self.services.vpc.list_state.select(Some(idx));
                    }
                } else if resource_id.starts_with("sg-") {
                    self.services.vpc.view_mode = super::super::VpcViewMode::SecurityGroups;
                    if let Some(idx) = self.services.vpc.security_groups.iter().position(|s| s.group_id == resource_id) {
                        self.services.vpc.list_state.select(Some(idx));
                    }
                }
            }
            Service::IAM => {
                // Check users first, then roles, then policies
                if let Some(idx) = self.services.iam.users.iter().position(|u| u.user_name == resource_id) {
                    self.services.iam.view_mode = super::super::IamViewMode::Users;
                    self.services.iam.list_state.select(Some(idx));
                } else if let Some(idx) = self.services.iam.roles.iter().position(|r| r.role_name == resource_id) {
                    self.services.iam.view_mode = super::super::IamViewMode::Roles;
                    self.services.iam.list_state.select(Some(idx));
                } else if let Some(idx) = self.services.iam.policies.iter().position(|p| p.policy_name == resource_id) {
                    self.services.iam.view_mode = super::super::IamViewMode::Policies;
                    self.services.iam.list_state.select(Some(idx));
                }
            }
            Service::Backup => {
                self.services.backup.view_mode = super::super::BackupViewMode::Vaults;
                if let Some(idx) = self.services.backup.vaults.iter().position(|v| v.backup_vault_name == resource_id) {
                    self.services.backup.list_state.select(Some(idx));
                }
            }
            Service::CloudTrail => {
                self.services.cloudtrail.view_mode = super::super::CloudTrailViewMode::Trails;
                if let Some(idx) = self.services.cloudtrail.trails.iter().position(|t| t.name == resource_id) {
                    self.services.cloudtrail.list_state.select(Some(idx));
                }
            }
            Service::SecretsManager => {
                if let Some(idx) = self.services.secretsmanager.secrets.iter().position(|s| s.name == resource_id) {
                    self.services.secretsmanager.list_state.select(Some(idx));
                }
            }
            Service::ECS => {
                // Check clusters first, then services
                if let Some(idx) = self.services.ecs.clusters.iter().position(|c| c.cluster_name == resource_id) {
                    self.services.ecs.view_mode = super::super::EcsViewMode::Clusters;
                    self.services.ecs.list_state.select(Some(idx));
                } else if let Some(idx) = self.services.ecs.services.iter().position(|s| s.service_name == resource_id) {
                    self.services.ecs.view_mode = super::super::EcsViewMode::Services;
                    self.services.ecs.list_state.select(Some(idx));
                }
            }
            Service::ECR => {
                self.services.ecr.view_mode = super::super::EcrViewMode::Repositories;
                if let Some(idx) = self.services.ecr.repositories.iter().position(|r| r.repository_name == resource_id) {
                    self.services.ecr.list_state.select(Some(idx));
                }
            }
        }
        
        // Set focus to main pane so user can immediately interact with the selection
        self.focus = super::super::Focus::Main;
        self.sidebar.is_focused = false;
    }
}
