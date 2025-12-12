//! AWS event handling
//!
//! Processes async events from AWS service calls and updates application state.

use super::{App, IamViewMode};
use crate::event::AwsEvent;

impl App {
    /// Handle async AWS events and update state accordingly
    pub fn handle_aws_event(&mut self, event: AwsEvent) {
        match event {
            AwsEvent::Ec2InstancesLoaded(instances) => {
                self.services.ec2.instances = instances;
                self.loading = false;
            }
            AwsEvent::S3BucketsLoaded(buckets) => {
                self.services.s3.buckets = buckets;
                self.loading = false;
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
            }
            AwsEvent::DynamoDbTablesLoaded(tables) => {
                self.services.dynamodb.tables = tables;
                self.loading = false;
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
            }
            AwsEvent::SubnetsLoaded(subnets) => {
                self.services.vpc.subnets = subnets;
                self.loading = false;
            }
            AwsEvent::SecurityGroupsLoaded(sgs) => {
                self.services.vpc.security_groups = sgs;
                self.loading = false;
            }
            AwsEvent::IamRolesLoaded(roles) => {
                self.services.iam.roles = roles;
                self.loading = false;
            }
            AwsEvent::IamUsersLoaded(users) => {
                self.services.iam.users = users;
                self.loading = false;
            }
            AwsEvent::IamPoliciesLoaded(policies) => {
                self.services.iam.policies = policies;
                self.loading = false;
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
            }
            AwsEvent::CloudTrailEventsLoaded(events) => {
                self.services.cloudtrail.events = events;
                self.loading = false;
            }
            AwsEvent::SecretsManagerSecretsLoaded(secrets) => {
                self.services.secretsmanager.secrets = secrets;
                self.loading = false;
            }
            AwsEvent::SecretsManagerSecretValueLoaded(value) => {
                self.services.secretsmanager.secret_value = Some(value);
                self.services.secretsmanager.show_secret_modal = true;
                self.loading = false;
            }
            AwsEvent::EcsClustersLoaded(clusters) => {
                self.services.ecs.clusters = clusters;
                self.loading = false;
            }
            AwsEvent::EcsServicesLoaded(services) => {
                self.services.ecs.services = services;
                if !self.services.ecs.services.is_empty() {
                    self.services.ecs.list_state.select(Some(0));
                }
                self.loading = false;
            }
            AwsEvent::EcsTasksLoaded(tasks) => {
                self.services.ecs.tasks = tasks;
                if !self.services.ecs.tasks.is_empty() {
                    self.services.ecs.list_state.select(Some(0));
                }
                self.loading = false;
            }
            AwsEvent::EcsTaskDefinitionLoaded(task_definition) => {
                self.services.ecs.current_task_definition = Some(task_definition);
                self.loading = false;
            }
            AwsEvent::S3ObjectDownloaded { key, path } => {
                self.loading = false;
                self.action_log
                    .push(format!("[SUCCESS] Downloaded '{}' to {}", key, path));
            }
            AwsEvent::S3ObjectOpened { key, path, content } => {
                self.loading = false;
                // Store the opened content for display in popup
                self.services.s3.opened_object_key = Some(key.clone());
                self.services.s3.opened_object_content = content;
                self.services.s3.opened_object_path = Some(path.clone());
                self.services.s3.show_object_viewer = true;
                self.services.s3.viewer_scroll_offset = 0;
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
                let short_arn = new_arn.split('/').last().unwrap_or(&new_arn);
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
                    self.services.ecs.task_definitions_list_state.select(Some(0));
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
                self.error_message = Some(e.clone());
                self.action_log.push(format!("[ERROR] {}", e));
                // Reset S3 bucket view on error so user can try again
                if self.services.s3.current_bucket.is_some() {
                    self.services.s3.current_bucket = None;
                    self.services.s3.objects.clear();
                }
            }
        }
    }
}
