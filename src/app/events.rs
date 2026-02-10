//! AWS event handling
//!
//! Processes async events from AWS service calls and updates application state.

use super::{App, IamViewMode, InputMode};
use crate::error::classify_credential_error;
use crate::event::AwsEvent;

impl App {
    /// Update global search state after a service data load event
    fn update_global_search_for_event(&mut self) {
        if self.input_mode == InputMode::GlobalSearch {
            self.refresh_global_search();
            // Clean up finished tasks before checking if any are still active
            self.tasks.cleanup_completed();
            // Check if all refresh tasks are done
            if !self.tasks.is_any_refresh_active() {
                self.global_search.loading = false;
            }
        }
    }

    /// Handle async AWS events and update state accordingly
    pub fn handle_aws_event(&mut self, event: AwsEvent) {
        // Reset loading state for most events - saves repeating this 30+ times
        self.loading = false;

        match event {
            AwsEvent::Ec2InstancesLoaded(instances) => {
                self.services.ec2.instances = instances;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::S3BucketsLoaded(buckets) => {
                self.services.s3.buckets = buckets;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::S3ObjectsLoaded(objects) => {
                self.services.s3.objects = objects;
                self.loading = false;
            }
            AwsEvent::S3BucketDetailsLoaded {
                bucket_name,
                details,
            } => {
                self.services.s3.bucket_details.insert(bucket_name, details);
                self.detail_loading = false;
            }
            AwsEvent::RdsInstancesLoaded(instances) => {
                self.services.rds.instances = instances;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::DynamoDbTablesLoaded(tables) => {
                self.services.dynamodb.tables = tables;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::DynamoDbItemsLoaded(items) => {
                self.services.dynamodb.items = items;
                if !self.services.dynamodb.items.is_empty() {
                    self.services.dynamodb.item_list_state.select(Some(0));
                }
                self.loading = false;
            }
            AwsEvent::LambdaFunctionsLoaded(functions) => {
                self.services.lambda.functions = functions;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::LambdaFunctionDetailsLoaded {
                function_name,
                details,
            } => {
                self.services
                    .lambda
                    .function_details
                    .insert(function_name, details);
            }
            AwsEvent::VpcsLoaded(vpcs) => {
                self.services.vpc.vpcs = vpcs;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::SubnetsLoaded(subnets) => {
                self.services.vpc.subnets = subnets;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::SecurityGroupsLoaded(sgs) => {
                self.services.vpc.security_groups = sgs;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::IamRolesLoaded(roles) => {
                self.services.iam.roles = roles;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::IamUsersLoaded(users) => {
                self.services.iam.users = users;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::IamPoliciesLoaded(policies) => {
                self.services.iam.policies = policies;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::IamUserPoliciesLoaded(policies) => {
                self.services.iam.current_policies = policies;
                self.services.iam.view_mode = IamViewMode::UserAttachedPolicies;
                self.services.iam.list_state.select(Some(0));
                self.loading = false;
            }
            AwsEvent::IamRolePoliciesLoaded(policies) => {
                self.services.iam.current_policies = policies;
                self.services.iam.view_mode = IamViewMode::RoleAttachedPolicies;
                self.services.iam.list_state.select(Some(0));
                self.loading = false;
            }
            AwsEvent::IamPolicyDocumentLoaded(doc) => {
                self.services.iam.current_policy_document = doc;
                self.services.iam.view_mode = IamViewMode::PolicyDocument;
                self.loading = false;
            }
            AwsEvent::BackupVaultsLoaded(vaults) => {
                self.services.backup.vaults = vaults;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::BackupPlansLoaded(plans) => {
                self.services.backup.plans = plans;
                self.loading = false;
            }
            AwsEvent::BackupJobsLoaded(jobs) => {
                self.services.backup.jobs = jobs;
                self.loading = false;
            }
            AwsEvent::BackupRecoveryPointsLoaded(points) => {
                self.services.backup.recovery_points = points;
                if !self.services.backup.recovery_points.is_empty() {
                    self.services.backup.list_state.select(Some(0));
                }
                self.loading = false;
            }
            AwsEvent::CloudTrailTrailsLoaded(trails) => {
                self.services.cloudtrail.trails = trails;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::CloudTrailEventsLoaded {
                events,
                next_token,
                append,
            } => {
                if append {
                    // Append to existing events (load more)
                    self.services.cloudtrail.events.append(events, next_token);
                } else {
                    // Replace events (fresh load or filter applied)
                    self.services.cloudtrail.events.replace(events, next_token);
                    if !self.services.cloudtrail.events.is_empty() {
                        self.services.cloudtrail.list_state.select(Some(0));
                    }
                }
                // Apply current sort settings
                self.services.cloudtrail.sort_events();
                self.loading = false;
            }
            AwsEvent::SecretsManagerSecretsLoaded(secrets) => {
                self.services.secretsmanager.secrets = secrets;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::SecretsManagerSecretValueLoaded(value) => {
                self.services.secretsmanager.secret_value = Some(value);
                self.services.secretsmanager.show_secret_modal = true;
                self.loading = false;
            }
            AwsEvent::EcsClustersLoaded(clusters) => {
                self.services.ecs.clusters = clusters;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::EcsServicesLoaded(services) => {
                self.services.ecs.services = services;
                if !self.services.ecs.services.is_empty() {
                    self.services.ecs.list_state.select(Some(0));
                }
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::EcsTasksLoaded(tasks) => {
                self.services.ecs.tasks = tasks;
                if !self.services.ecs.tasks.is_empty() {
                    self.services.ecs.list_state.select(Some(0));
                }
                self.loading = false;
            }
            AwsEvent::EcsTaskDefinitionLoaded(task_definition) => {
                self.services.ecs.current_task_definition = Some(*task_definition);
                self.loading = false;
            }
            AwsEvent::S3ObjectDownloaded { key, path } => {
                self.loading = false;
                self.action_log
                    .push(format!("[SUCCESS] Downloaded '{}' to {}", key, path));
            }
            AwsEvent::S3ObjectOpened {
                key,
                path,
                content,
                raw_bytes,
            } => {
                self.loading = false;
                // Store the opened content for display in popup
                self.services.s3.opened_object_key = Some(key.clone());
                self.services.s3.opened_object_content = content;
                self.services.s3.opened_object_bytes = raw_bytes;
                self.services.s3.opened_object_path = Some(path.clone());
                self.services.s3.show_object_viewer = true;
                self.services.s3.viewer_scroll_offset = 0;
                self.services.s3.viewer_mode = crate::app::states::s3::ViewerMode::Text;
                self.action_log.push(format!("[SUCCESS] Opened '{}'", key));
            }
            AwsEvent::S3ObjectEdited { bucket, key } => {
                self.loading = false;
                let msg = format!("Edited and uploaded s3://{}/{}", bucket, key);
                self.action_log.push(format!("[SUCCESS] {}", msg));
                self.should_refresh = true;
            }
            AwsEvent::S3ObjectReadyForEdit { bucket, key, path } => {
                self.loading = false;
                // Store the pending edit info - will be processed synchronously
                self.services.s3.pending_edit = Some((bucket, key, path));
            }
            AwsEvent::EcsTaskDefinitionEdited { family: _, new_arn } => {
                self.loading = false;
                let short_arn = new_arn.split('/').next_back().unwrap_or(&new_arn);
                let msg = format!("Registered new task definition: {}", short_arn);
                self.action_log.push(format!("[SUCCESS] {}", msg));
                self.should_refresh = true;
                // Show the task definition selector for the family
                self.services.ecs.show_task_definition_selector = true;
            }
            AwsEvent::EcsTaskDefinitionsListed(task_defs) => {
                self.loading = false;
                self.services.ecs.task_definitions_list = task_defs;
                self.services.ecs.show_task_definition_selector = true;
                if !self.services.ecs.task_definitions_list.is_empty() {
                    self.services
                        .ecs
                        .task_definitions_list_state
                        .select(Some(0));
                }
            }
            AwsEvent::EcsTaskDefinitionsForSelectorLoaded(task_defs) => {
                self.loading = false;
                self.services.ecs.task_def_selector.set_list(task_defs);
            }
            AwsEvent::EcsTaskDefinitionReadyForEdit { family, path } => {
                self.loading = false;
                // Store the pending edit info - will be processed synchronously
                self.services.ecs.pending_edit = Some((family, path));
            }
            AwsEvent::ActionCompleted(msg) => {
                self.loading = false;
                self.action_log.push(format!("[SUCCESS] {}", msg));
                self.should_refresh = true;
            }
            AwsEvent::Error(e) => {
                self.loading = false;
                self.detail_loading = false;
                let user_message = classify_credential_error(&e)
                    .map(|err| err.user_message())
                    .unwrap_or_else(|| e.clone());
                self.error_message = Some(user_message.clone());
                self.action_log.push(format!("[ERROR] {}", user_message));
                // Reset S3 bucket view on error so user can try again
                if self.services.s3.current_bucket.is_some() {
                    self.services.s3.current_bucket = None;
                    self.services.s3.objects.clear();
                }
            }
            AwsEvent::EcrRepositoriesLoaded(repos) => {
                self.services.ecr.repositories = repos;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::EcrImagesLoaded {
                images,
                next_token,
                append,
            } => {
                if append {
                    // Append to existing images (load more)
                    self.services.ecr.images.append(images, next_token);
                } else {
                    // Replace images (fresh load or filter applied)
                    self.services.ecr.images.replace(images, next_token);
                    if !self.services.ecr.images.is_empty() {
                        self.services.ecr.list_state.select(Some(0));
                    }
                }
                // Apply current sort settings
                self.services.ecr.sort_images();
                self.loading = false;
            }
            AwsEvent::EcrImagePulled { image_uri } => {
                self.loading = false;
                self.action_log
                    .push(format!("[SUCCESS] Pulled image: {}", image_uri));
            }
            AwsEvent::EcrScanFindingsLoaded {
                image_digest,
                findings,
            } => {
                self.services.ecr.scan_findings_loading = false;
                self.services.ecr.scan_findings_digest = Some(image_digest);
                self.services.ecr.scan_findings = Some(findings);
            }
            AwsEvent::EcrScanFindingsExported { path } => {
                self.action_log
                    .push(format!("[SUCCESS] Exported scan findings to {}", path));
            }
            AwsEvent::BudgetsLoaded(budgets) => {
                self.services.budgets.budgets = budgets;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::BudgetNotificationsLoaded {
                budget_name,
                notifications,
            } => {
                self.services.budgets.notifications = notifications;
                self.services.budgets.selected_budget = Some(budget_name);
                self.services.budgets.view_mode = crate::app::BudgetsViewMode::Notifications;
                self.services.budgets.list_state.select(Some(0));
                self.loading = false;
            }
            AwsEvent::BillingViewsLoaded(views) => {
                self.services.budgets.billing_views = views;
                self.loading = false;
            }
            AwsEvent::SqsQueuesLoaded(queues) => {
                self.services.sqs.queues = queues;
                self.loading = false;
                self.update_global_search_for_event();
            }
            AwsEvent::ProfileRegionSwitched(data) => {
                // Log SSO messages
                for msg in data.sso_messages {
                    self.action_log.push(msg);
                }

                // Apply the new clients and state
                self.aws_clients = Some(data.clients);
                self.profile = data.profile;
                self.region = data.region.clone();
                self.read_only = data.read_only;
                self.profile_switcher.pending_read_only = data.read_only;

                // Update profile/region indices
                if let Some(idx) = self
                    .profile_switcher
                    .available_profiles
                    .iter()
                    .position(|p| {
                        self.profile
                            .as_ref()
                            .map_or(p == "default", |prof| p == prof)
                    })
                {
                    self.profile_switcher.profile_switcher_index = idx;
                }
                if let Some(idx) = self
                    .profile_switcher
                    .available_regions
                    .iter()
                    .position(|r| r == &data.region)
                {
                    self.profile_switcher.region_switcher_index = idx;
                }

                // Clear all service data to force refresh
                self.services.clear_all();

                let ro_status = if data.read_only { " [READ-ONLY]" } else { "" };
                self.action_log.push(format!(
                    "Switched to profile: {}, region: {}{}",
                    self.profile.as_deref().unwrap_or("default"),
                    self.region,
                    ro_status
                ));

                // Request a refresh of current service data
                self.loading = false;
                self.should_refresh = true;
            }
            AwsEvent::ProfileRegionSwitchFailed(error) => {
                self.loading = false;
                self.error_message = Some(format!("Failed to switch profile: {}", error));
                self.action_log
                    .push(format!("[ERROR] Failed to switch profile: {}", error));
            }
        }
    }
}
