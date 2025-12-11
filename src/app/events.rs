//! AWS event handling
//! 
//! Processes async events from AWS service calls and updates application state.

use crate::event::AwsEvent;
use super::{App, IamViewMode};

impl App {
    /// Handle async AWS events and update state accordingly
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
            AwsEvent::S3BucketDetailsLoaded { bucket_name, details } => {
                self.s3_bucket_details.insert(bucket_name, details);
                self.detail_loading = false;
            }
            AwsEvent::RdsInstancesLoaded(instances) => {
                self.rds_instances = instances;
                self.loading = false;
            }
            AwsEvent::DynamoDbTablesLoaded(tables) => {
                self.dynamodb_tables = tables;
                self.loading = false;
            }
            AwsEvent::DynamoDbItemsLoaded(items) => {
                self.dynamodb_items = items;
                if !self.dynamodb_items.is_empty() {
                    self.dynamodb_item_list_state.select(Some(0));
                }
                self.loading = false;
            }
            AwsEvent::LambdaFunctionsLoaded(functions) => {
                self.lambda_functions = functions;
                self.loading = false;
            }
            AwsEvent::VpcsLoaded(vpcs) => {
                self.vpcs = vpcs;
                self.loading = false;
            }
            AwsEvent::SubnetsLoaded(subnets) => {
                self.subnets = subnets;
                self.loading = false;
            }
            AwsEvent::SecurityGroupsLoaded(sgs) => {
                self.security_groups = sgs;
                self.loading = false;
            }
            AwsEvent::IamRolesLoaded(roles) => {
                self.iam_roles = roles;
                self.loading = false;
            }
            AwsEvent::IamUsersLoaded(users) => {
                self.iam_users = users;
                self.loading = false;
            }
            AwsEvent::IamPoliciesLoaded(policies) => {
                self.iam_policies = policies;
                self.loading = false;
            }
            AwsEvent::IamUserPoliciesLoaded(policies) => {
                self.current_iam_policies = policies;
                self.iam_view_mode = IamViewMode::UserAttachedPolicies;
                self.iam_list_state.select(Some(0));
                self.loading = false;
            }
            AwsEvent::IamRolePoliciesLoaded(policies) => {
                self.current_iam_policies = policies;
                self.iam_view_mode = IamViewMode::RoleAttachedPolicies;
                self.iam_list_state.select(Some(0));
                self.loading = false;
            }
            AwsEvent::IamPolicyDocumentLoaded(doc) => {
                self.current_policy_document = doc;
                self.iam_view_mode = IamViewMode::PolicyDocument;
                self.loading = false;
            }
            AwsEvent::BackupVaultsLoaded(vaults) => {
                self.backup_vaults = vaults;
                self.loading = false;
            }
            AwsEvent::BackupPlansLoaded(plans) => {
                self.backup_plans = plans;
                self.loading = false;
            }
            AwsEvent::BackupJobsLoaded(jobs) => {
                self.backup_jobs = jobs;
                self.loading = false;
            }
            AwsEvent::CloudTrailTrailsLoaded(trails) => {
                self.cloudtrail_trails = trails;
                self.loading = false;
            }
            AwsEvent::CloudTrailEventsLoaded(events) => {
                self.cloudtrail_events = events;
                self.loading = false;
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
                if self.current_bucket.is_some() {
                    self.current_bucket = None;
                    self.s3_objects.clear();
                }
            }
        }
    }
}
