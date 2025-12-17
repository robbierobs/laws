//! Global message handling
//!
//! Handles application-wide messages like navigation, refresh, profile switching.

use super::super::task_manager::task_keys;
use super::super::{App, GlobalMessage, InputMode, Service};
use super::refresh::spawn_list_task;
use crate::aws::traits::AwsService;
use crate::event::{AwsEvent, Event};

impl App {
    /// Handle global (non-service-specific) messages
    pub(super) fn handle_global_message(
        &mut self,
        message: GlobalMessage,
        event_tx: crate::app::EventSender,
    ) {
        match message {
            GlobalMessage::Quit => self.should_quit = true,
            GlobalMessage::Navigate(service) => {
                self.current_service = service;
                self.sidebar.select_service(service);
                self.handle_refresh_data(event_tx);
            }
            GlobalMessage::ConfirmAction => {
                if let Some(action) = self.pending_action.take() {
                    self.show_confirmation = false;
                    self.update(action, event_tx);
                }
            }
            GlobalMessage::CancelAction => {
                self.pending_action = None;
                self.show_confirmation = false;
            }
            GlobalMessage::RefreshData => {
                self.handle_refresh_data(event_tx.clone());
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
                self.handle_switch_profile_region(profile, region, read_only, event_tx);
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
                self.global_search.loading = true;
                // Trigger data load for all services
                self.refresh_all_services_for_search(event_tx.clone());
                // Pre-populate with all currently loaded results
                self.refresh_global_search();
            }
            GlobalMessage::CloseGlobalSearch => {
                self.input_mode = InputMode::Normal;
                self.global_search.clear();
            }
            GlobalMessage::GotoSearchResult {
                service,
                resource_id,
            } => {
                // Navigate to the service
                self.current_service = service;
                self.sidebar.select_service(service);

                // Try to select the resource in the appropriate service state
                self.select_resource_by_id(service, &resource_id);

                self.action_log.push(format!(
                    "Navigated to {} - {}",
                    service.as_str(),
                    resource_id
                ));

                // Refresh data for the service
                self.handle_refresh_data(event_tx);
            }
        }
    }

    fn handle_open_profile_switcher(&mut self) {
        if let Some(current) = &self.profile {
            if let Some(idx) = self
                .profile_switcher
                .available_profiles
                .iter()
                .position(|p| p == current)
            {
                self.profile_switcher.profile_switcher_index = idx;
            }
        } else {
            self.profile_switcher.profile_switcher_index = 0;
        }
        self.profile_switcher.pending_read_only = self.read_only;
        self.input_mode = InputMode::ProfileSwitcherProfile;
    }

    fn handle_cancel_profile_switcher(&mut self) {
        self.input_mode = InputMode::Normal;
        self.profile_switcher.pending_profile = None;
        self.profile_switcher.reset_filters();
    }

    /// Spawn a background task to switch profile/region
    /// 
    /// This performs SSO login (if needed) and AWS client creation in a background
    /// task, then sends a ProfileRegionSwitched event when complete.
    fn handle_switch_profile_region(
        &mut self,
        profile: Option<String>,
        region: String,
        read_only: bool,
        event_tx: crate::app::EventSender,
    ) {
        self.input_mode = InputMode::Normal;
        self.loading = true;
        
        let profile_name = profile.clone().unwrap_or_else(|| "default".to_string());
        self.action_log.push(format!(
            "Switching to profile: {}, region: {}...",
            profile_name, region
        ));
        
        // Clone values for async task
        let profile_clone = profile.clone();
        let region_clone = region.clone();
        let is_sso = crate::utils::aws_profiles::is_sso_profile(&profile_name);
        
        let handle = tokio::spawn(async move {
            let mut sso_messages = Vec::new();
            
            // Check if profile uses SSO and run login if needed
            if is_sso {
                sso_messages.push(format!("Running SSO login for profile: {}", profile_name));
                let sso_result = tokio::process::Command::new("aws")
                    .args(["sso", "login", "--profile", &profile_name])
                    .output()
                    .await;

                match sso_result {
                    Ok(output) => {
                        if output.status.success() {
                            sso_messages.push(format!(
                                "SSO login successful for profile: {}",
                                profile_name
                            ));
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            sso_messages.push(format!("SSO login warning: {}", stderr.trim()));
                        }
                    }
                    Err(e) => {
                        sso_messages.push(format!("SSO login error: {}", e));
                    }
                }
            }

            // Create new AWS clients with the new profile and region
            let new_clients = crate::aws::client::AwsClients::new(
                profile_clone.as_deref(),
                Some(region_clone.as_str()),
                None,
            )
            .await;

            match new_clients {
                Ok(clients) => {
                    event_tx
                        .send(Event::Aws(AwsEvent::ProfileRegionSwitched {
                            clients,
                            profile: profile_clone,
                            region: region_clone,
                            read_only,
                            sso_messages,
                        }))
                        .await
                        .ok();
                }
                Err(e) => {
                    event_tx
                        .send(Event::Aws(AwsEvent::ProfileRegionSwitchFailed(
                            e.to_string(),
                        )))
                        .await
                        .ok();
                }
            }
        });
        
        self.tasks.spawn(task_keys::PROFILE_SWITCH, handle);
    }

    pub(super) fn handle_refresh_data(&mut self, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;

        match self.current_service {
            Service::EC2 => {
                let client = clients.ec2.clone();
                let handle = spawn_list_task(
                    event_tx,
                    move || async move { crate::aws::ec2::Ec2Service::new(client).list().await },
                    AwsEvent::Ec2InstancesLoaded,
                    true,
                );
                self.tasks.spawn(task_keys::EC2_REFRESH, handle);
            }
            Service::S3 => {
                let clients = clients.clone();
                self.handle_refresh_s3(&clients, event_tx);
            }
            Service::RDS => {
                let client = clients.rds.clone();
                let handle = spawn_list_task(
                    event_tx,
                    move || async move {
                        crate::aws::rds::RdsService::new(client)
                            .list_instances()
                            .await
                    },
                    AwsEvent::RdsInstancesLoaded,
                    true,
                );
                self.tasks.spawn(task_keys::RDS_REFRESH, handle);
            }
            Service::DynamoDB => {
                let client = clients.dynamodb.clone();
                let handle = spawn_list_task(
                    event_tx,
                    move || async move {
                        crate::aws::dynamodb::DynamoDbService::new(client)
                            .list_tables()
                            .await
                    },
                    AwsEvent::DynamoDbTablesLoaded,
                    true,
                );
                self.tasks.spawn(task_keys::DYNAMODB_REFRESH, handle);
            }
            Service::Lambda => {
                let client = clients.lambda.clone();
                let handle = spawn_list_task(
                    event_tx,
                    move || async move {
                        crate::aws::lambda::LambdaService::new(client)
                            .list_functions()
                            .await
                    },
                    AwsEvent::LambdaFunctionsLoaded,
                    true,
                );
                self.tasks.spawn(task_keys::LAMBDA_REFRESH, handle);
            }
            Service::VPC => {
                let client = clients.ec2.clone();
                self.refresh_vpc(client, event_tx);
            }
            Service::IAM => {
                let client = clients.iam.clone();
                self.refresh_iam(client, event_tx);
            }
            Service::Backup => {
                let client = clients.backup.clone();
                self.refresh_backup(client, event_tx);
            }
            Service::CloudTrail => {
                let client = clients.cloudtrail.clone();
                let tx = event_tx.clone();
                let limit = self.config.max_cloudtrail_events as i32;
                let handle = tokio::spawn(async move {
                    let service = crate::aws::cloudtrail::CloudTrailService::new(client);
                    if let Ok(trails) = service.list_trails().await {
                        tx.send(Event::Aws(AwsEvent::CloudTrailTrailsLoaded(trails)))
                            .await
                            .ok();
                    }
                    if let Ok(events) = service.lookup_events(limit).await {
                        tx.send(Event::Aws(AwsEvent::CloudTrailEventsLoaded(events)))
                            .await
                            .ok();
                    }
                });
                self.tasks.spawn(task_keys::CLOUDTRAIL_REFRESH, handle);
            }
            Service::SecretsManager => {
                let client = clients.secretsmanager.clone();
                let handle = spawn_list_task(
                    event_tx,
                    move || async move {
                        crate::aws::secretsmanager::SecretsManagerService::new(client)
                            .list_secrets()
                            .await
                    },
                    AwsEvent::SecretsManagerSecretsLoaded,
                    true,
                );
                self.tasks.spawn(task_keys::SECRETSMANAGER_REFRESH, handle);
            }
            Service::ECS => {
                let client = clients.ecs.clone();
                let handle = spawn_list_task(
                    event_tx,
                    move || async move {
                        crate::aws::ecs::EcsClient::new(client)
                            .list_clusters()
                            .await
                    },
                    AwsEvent::EcsClustersLoaded,
                    true,
                );
                self.tasks.spawn(task_keys::ECS_REFRESH, handle);
            }
            Service::ECR => {
                let client = clients.ecr.clone();
                let handle = spawn_list_task(
                    event_tx,
                    move || async move {
                        crate::aws::ecr::EcrService::new(client)
                            .list_repositories()
                            .await
                    },
                    AwsEvent::EcrRepositoriesLoaded,
                    true,
                );
                self.tasks.spawn(task_keys::ECR_REFRESH, handle);
            }
        }
    }

    /// Helper for VPC refresh (multiple resources)
    fn refresh_vpc(&mut self, client: aws_sdk_ec2::Client, event_tx: crate::app::EventSender) {
        let tx = event_tx;
        let handle = tokio::spawn(async move {
            let service = crate::aws::vpc::VpcService::new(client);
            if let Ok(vpcs) = service.list_vpcs().await {
                tx.send(Event::Aws(AwsEvent::VpcsLoaded(vpcs))).await.ok();
            }
            if let Ok(subnets) = service.list_subnets(None).await {
                tx.send(Event::Aws(AwsEvent::SubnetsLoaded(subnets)))
                    .await
                    .ok();
            }
            if let Ok(sgs) = service.list_security_groups(None).await {
                tx.send(Event::Aws(AwsEvent::SecurityGroupsLoaded(sgs)))
                    .await
                    .ok();
            }
        });
        self.tasks.spawn(task_keys::VPC_REFRESH, handle);
    }

    /// Helper for IAM refresh (multiple resources)
    fn refresh_iam(&mut self, client: aws_sdk_iam::Client, event_tx: crate::app::EventSender) {
        let tx = event_tx;
        let handle = tokio::spawn(async move {
            let service = crate::aws::iam::IamService::new(client);
            if let Ok(users) = service.list_users().await {
                tx.send(Event::Aws(AwsEvent::IamUsersLoaded(users)))
                    .await
                    .ok();
            }
            if let Ok(roles) = service.list_roles().await {
                tx.send(Event::Aws(AwsEvent::IamRolesLoaded(roles)))
                    .await
                    .ok();
            }
            if let Ok(policies) = service.list_policies().await {
                tx.send(Event::Aws(AwsEvent::IamPoliciesLoaded(policies)))
                    .await
                    .ok();
            }
        });
        self.tasks.spawn(task_keys::IAM_REFRESH, handle);
    }

    /// Helper for Backup refresh (multiple resources)
    fn refresh_backup(
        &mut self,
        client: aws_sdk_backup::Client,
        event_tx: crate::app::EventSender,
    ) {
        let tx = event_tx;
        let handle = tokio::spawn(async move {
            let service = crate::aws::backup::BackupService::new(client);
            if let Ok(vaults) = service.list_backup_vaults().await {
                tx.send(Event::Aws(AwsEvent::BackupVaultsLoaded(vaults)))
                    .await
                    .ok();
            }
            if let Ok(plans) = service.list_backup_plans().await {
                tx.send(Event::Aws(AwsEvent::BackupPlansLoaded(plans)))
                    .await
                    .ok();
            }
            if let Ok(jobs) = service.list_backup_jobs().await {
                tx.send(Event::Aws(AwsEvent::BackupJobsLoaded(jobs)))
                    .await
                    .ok();
            }
        });
        self.tasks.spawn(task_keys::BACKUP_REFRESH, handle);
    }

    /// Refresh all services in parallel for global search
    /// This ensures we have data from all services for comprehensive search results
    fn refresh_all_services_for_search(&mut self, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.action_log
            .push("Loading all services for global search...".to_string());

        // Simple services - use spawn_list_task with report_errors=false
        let ec2_client = clients.ec2.clone();
        let handle = spawn_list_task(
            event_tx.clone(),
            move || async move { crate::aws::ec2::Ec2Service::new(ec2_client).list().await },
            AwsEvent::Ec2InstancesLoaded,
            false,
        );
        self.tasks.spawn(task_keys::EC2_REFRESH, handle);

        let s3_client = clients.s3.clone();
        let handle = spawn_list_task(
            event_tx.clone(),
            move || async move {
                crate::aws::s3::S3Service::new(s3_client)
                    .list_buckets()
                    .await
            },
            AwsEvent::S3BucketsLoaded,
            false,
        );
        self.tasks.spawn(task_keys::S3_REFRESH, handle);

        let rds_client = clients.rds.clone();
        let handle = spawn_list_task(
            event_tx.clone(),
            move || async move {
                crate::aws::rds::RdsService::new(rds_client)
                    .list_instances()
                    .await
            },
            AwsEvent::RdsInstancesLoaded,
            false,
        );
        self.tasks.spawn(task_keys::RDS_REFRESH, handle);

        let dynamodb_client = clients.dynamodb.clone();
        let handle = spawn_list_task(
            event_tx.clone(),
            move || async move {
                crate::aws::dynamodb::DynamoDbService::new(dynamodb_client)
                    .list_tables()
                    .await
            },
            AwsEvent::DynamoDbTablesLoaded,
            false,
        );
        self.tasks.spawn(task_keys::DYNAMODB_REFRESH, handle);

        let lambda_client = clients.lambda.clone();
        let handle = spawn_list_task(
            event_tx.clone(),
            move || async move {
                crate::aws::lambda::LambdaService::new(lambda_client)
                    .list_functions()
                    .await
            },
            AwsEvent::LambdaFunctionsLoaded,
            false,
        );
        self.tasks.spawn(task_keys::LAMBDA_REFRESH, handle);

        let secretsmanager_client = clients.secretsmanager.clone();
        let handle = spawn_list_task(
            event_tx.clone(),
            move || async move {
                crate::aws::secretsmanager::SecretsManagerService::new(secretsmanager_client)
                    .list_secrets()
                    .await
            },
            AwsEvent::SecretsManagerSecretsLoaded,
            false,
        );
        self.tasks.spawn(task_keys::SECRETSMANAGER_REFRESH, handle);

        let ecs_client = clients.ecs.clone();
        let handle = spawn_list_task(
            event_tx.clone(),
            move || async move {
                crate::aws::ecs::EcsClient::new(ecs_client)
                    .list_clusters()
                    .await
            },
            AwsEvent::EcsClustersLoaded,
            false,
        );
        self.tasks.spawn(task_keys::ECS_REFRESH, handle);

        let ecr_client = clients.ecr.clone();
        let handle = spawn_list_task(
            event_tx.clone(),
            move || async move {
                crate::aws::ecr::EcrService::new(ecr_client)
                    .list_repositories()
                    .await
            },
            AwsEvent::EcrRepositoriesLoaded,
            false,
        );
        self.tasks.spawn(task_keys::ECR_REFRESH, handle);

        // Multi-resource services
        let vpc_client = clients.ec2.clone();
        let iam_client = clients.iam.clone();
        let backup_client = clients.backup.clone();
        let cloudtrail_client = clients.cloudtrail.clone();

        self.refresh_vpc(vpc_client, event_tx.clone());
        self.refresh_iam(iam_client, event_tx.clone());
        self.refresh_backup(backup_client, event_tx.clone());

        // CloudTrail (just trails for search)
        let handle = spawn_list_task(
            event_tx,
            move || async move {
                crate::aws::cloudtrail::CloudTrailService::new(cloudtrail_client)
                    .list_trails()
                    .await
            },
            AwsEvent::CloudTrailTrailsLoaded,
            false,
        );
        self.tasks.spawn(task_keys::CLOUDTRAIL_REFRESH, handle);
    }

    /// Select a resource by its ID within the appropriate service state
    fn select_resource_by_id(&mut self, service: Service, resource_id: &str) {
        match service {
            Service::EC2 => {
                if let Some(idx) = self
                    .services
                    .ec2
                    .instances
                    .iter()
                    .position(|i| i.instance_id == resource_id)
                {
                    self.services.ec2.list_state.select(Some(idx));
                }
            }
            Service::S3 => {
                // For S3, navigate to bucket list and select the bucket
                self.services.s3.current_bucket = None; // Ensure we're at bucket level
                if let Some(idx) = self
                    .services
                    .s3
                    .buckets
                    .iter()
                    .position(|b| b.name == resource_id)
                {
                    self.services.s3.list_state.select(Some(idx));
                }
            }
            Service::RDS => {
                if let Some(idx) = self
                    .services
                    .rds
                    .instances
                    .iter()
                    .position(|i| i.db_instance_identifier == resource_id)
                {
                    self.services.rds.list_state.select(Some(idx));
                }
            }
            Service::DynamoDB => {
                if let Some(idx) = self
                    .services
                    .dynamodb
                    .tables
                    .iter()
                    .position(|t| t.table_name == resource_id)
                {
                    self.services.dynamodb.list_state.select(Some(idx));
                }
            }
            Service::Lambda => {
                if let Some(idx) = self
                    .services
                    .lambda
                    .functions
                    .iter()
                    .position(|f| f.function_name == resource_id)
                {
                    self.services.lambda.list_state.select(Some(idx));
                }
            }
            Service::VPC => {
                // Need to detect resource type from ID prefix
                if resource_id.starts_with("vpc-") {
                    self.services.vpc.view_mode = super::super::VpcViewMode::Vpcs;
                    if let Some(idx) = self
                        .services
                        .vpc
                        .vpcs
                        .iter()
                        .position(|v| v.vpc_id == resource_id)
                    {
                        self.services.vpc.list_state.select(Some(idx));
                    }
                } else if resource_id.starts_with("subnet-") {
                    self.services.vpc.view_mode = super::super::VpcViewMode::Subnets;
                    if let Some(idx) = self
                        .services
                        .vpc
                        .subnets
                        .iter()
                        .position(|s| s.subnet_id == resource_id)
                    {
                        self.services.vpc.list_state.select(Some(idx));
                    }
                } else if resource_id.starts_with("sg-") {
                    self.services.vpc.view_mode = super::super::VpcViewMode::SecurityGroups;
                    if let Some(idx) = self
                        .services
                        .vpc
                        .security_groups
                        .iter()
                        .position(|s| s.group_id == resource_id)
                    {
                        self.services.vpc.list_state.select(Some(idx));
                    }
                }
            }
            Service::IAM => {
                // Check users first, then roles, then policies
                if let Some(idx) = self
                    .services
                    .iam
                    .users
                    .iter()
                    .position(|u| u.user_name == resource_id)
                {
                    self.services.iam.view_mode = super::super::IamViewMode::Users;
                    self.services.iam.list_state.select(Some(idx));
                } else if let Some(idx) = self
                    .services
                    .iam
                    .roles
                    .iter()
                    .position(|r| r.role_name == resource_id)
                {
                    self.services.iam.view_mode = super::super::IamViewMode::Roles;
                    self.services.iam.list_state.select(Some(idx));
                } else if let Some(idx) = self
                    .services
                    .iam
                    .policies
                    .iter()
                    .position(|p| p.policy_name == resource_id)
                {
                    self.services.iam.view_mode = super::super::IamViewMode::Policies;
                    self.services.iam.list_state.select(Some(idx));
                }
            }
            Service::Backup => {
                self.services.backup.view_mode = super::super::BackupViewMode::Vaults;
                if let Some(idx) = self
                    .services
                    .backup
                    .vaults
                    .iter()
                    .position(|v| v.backup_vault_name == resource_id)
                {
                    self.services.backup.list_state.select(Some(idx));
                }
            }
            Service::CloudTrail => {
                self.services.cloudtrail.view_mode = super::super::CloudTrailViewMode::Trails;
                if let Some(idx) = self
                    .services
                    .cloudtrail
                    .trails
                    .iter()
                    .position(|t| t.name == resource_id)
                {
                    self.services.cloudtrail.list_state.select(Some(idx));
                }
            }
            Service::SecretsManager => {
                if let Some(idx) = self
                    .services
                    .secretsmanager
                    .secrets
                    .iter()
                    .position(|s| s.name == resource_id)
                {
                    self.services.secretsmanager.list_state.select(Some(idx));
                }
            }
            Service::ECS => {
                // Check clusters first, then services
                if let Some(idx) = self
                    .services
                    .ecs
                    .clusters
                    .iter()
                    .position(|c| c.cluster_name == resource_id)
                {
                    self.services.ecs.view_mode = super::super::EcsViewMode::Clusters;
                    self.services.ecs.list_state.select(Some(idx));
                } else if let Some(idx) = self
                    .services
                    .ecs
                    .services
                    .iter()
                    .position(|s| s.service_name == resource_id)
                {
                    self.services.ecs.view_mode = super::super::EcsViewMode::Services;
                    self.services.ecs.list_state.select(Some(idx));
                }
            }
            Service::ECR => {
                self.services.ecr.view_mode = super::super::EcrViewMode::Repositories;
                if let Some(idx) = self
                    .services
                    .ecr
                    .repositories
                    .iter()
                    .position(|r| r.repository_name == resource_id)
                {
                    self.services.ecr.list_state.select(Some(idx));
                }
            }
        }

        // Set focus to main pane so user can immediately interact with the selection
        self.focus = super::super::Focus::Main;
        self.sidebar.is_focused = false;
    }
}
