//! Message update/reducer function
//!
//! Handles all application messages and updates state accordingly.
//! Split into submodules by domain for maintainability.

mod cloudtrail;
mod dynamodb;
mod ec2;
mod global;
mod iam;
pub mod instance_actions;

mod lambda;
mod rds;

mod backup;
mod ecr;
mod ecs;
pub mod refresh;
mod s3;
mod secretsmanager;
mod view_mode;
mod vpc;

use super::messages::{
    DynamoDbAction, Ec2Action, IamAction, RdsAction, S3Action, SecretsManagerAction, VpcAction,
};
use super::{App, Message, ServiceAction};

impl App {
    /// Main message handler - processes messages and updates application state
    /// 
    /// This is now fully synchronous. All I/O operations are spawned as background
    /// tasks that communicate back via the event channel.
    pub fn update(
        &mut self,
        message: Message,
        event_tx: crate::app::EventSender,
    ) {
        match message {
            Message::Global(global) => self.handle_global_message(global, event_tx),
            Message::Service(service) => self.handle_service_action(service, event_tx),
        }
    }

    /// Handle service-specific actions - dispatches to service modules
    fn handle_service_action(
        &mut self,
        action: ServiceAction,
        event_tx: crate::app::EventSender,
    ) {
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
            ServiceAction::Ec2(Ec2Action::Terminate(id)) => {
                self.handle_ec2_action("terminate", id, event_tx);
            }

            // S3 actions
            ServiceAction::S3(S3Action::LoadObjects(bucket)) => {
                self.handle_load_s3_objects(bucket, event_tx);
            }
            ServiceAction::S3(S3Action::LoadBucketDetails(bucket)) => {
                self.handle_load_bucket_details(bucket, event_tx);
            }
            ServiceAction::S3(S3Action::CreateBucket(name)) => {
                self.handle_create_s3_bucket(name, event_tx);
            }
            ServiceAction::S3(S3Action::DeleteBucket(name)) => {
                self.handle_delete_s3_bucket(name, event_tx);
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
            ServiceAction::S3(S3Action::EditObject { bucket, key }) => {
                self.handle_edit_s3_object(bucket, key, event_tx);
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
            ServiceAction::Rds(RdsAction::Delete(id)) => {
                self.handle_rds_action("terminate", id, event_tx);
            }

            // DynamoDB actions
            ServiceAction::DynamoDb(DynamoDbAction::DrillDownTable) => {
                self.handle_drill_down_dynamodb_table(event_tx);
            }
            ServiceAction::DynamoDb(DynamoDbAction::ExitDrillDown) => {
                self.handle_dynamodb_exit_drilldown();
            }
            ServiceAction::DynamoDb(DynamoDbAction::LoadItems(table_name)) => {
                self.handle_load_dynamodb_items(table_name, event_tx);
            }
            ServiceAction::DynamoDb(DynamoDbAction::DeleteItem {
                table_name,
                key_attrs,
            }) => {
                self.handle_delete_dynamodb_item(table_name, key_attrs, event_tx);
            }

            // VPC actions
            ServiceAction::Vpc(VpcAction::DrillDownSecurityGroup) => {
                self.handle_drill_down_security_group();
            }
            ServiceAction::Vpc(VpcAction::ExitSecurityGroupRules) => {
                self.handle_vpc_exit_sg_rules();
            }
            ServiceAction::Vpc(VpcAction::ToggleSgRulesDirection) => {
                self.handle_toggle_sg_rules_direction();
            }
            ServiceAction::Vpc(VpcAction::DeleteSecurityGroup(id)) => {
                self.handle_delete_security_group(id, event_tx);
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
            ServiceAction::Iam(IamAction::DeleteUser(name)) => {
                self.handle_delete_iam_user(name, event_tx);
            }
            ServiceAction::Iam(IamAction::DeleteRole(name)) => {
                self.handle_delete_iam_role(name, event_tx);
            }
            ServiceAction::Iam(IamAction::DeletePolicy(arn)) => {
                self.handle_delete_iam_policy(arn, event_tx);
            }

            // SecretsManager actions
            ServiceAction::SecretsManager(SecretsManagerAction::GetSecretValue(arn)) => {
                self.handle_get_secret_value(arn, event_tx);
            }
            ServiceAction::SecretsManager(SecretsManagerAction::CloseSecretValue) => {
                self.services.secretsmanager.show_secret_modal = false;
                self.services.secretsmanager.secret_value = None;
            }
            ServiceAction::SecretsManager(SecretsManagerAction::DeleteSecret(arn)) => {
                self.handle_delete_secret(arn, event_tx);
            }

            // No-op actions for services without mutations
            // Lambda actions
            ServiceAction::Lambda(crate::app::messages::LambdaAction::InvokeFunction(name)) => {
                self.handle_invoke_lambda(name, event_tx);
            }
            ServiceAction::Lambda(crate::app::messages::LambdaAction::DeleteFunction(name)) => {
                self.handle_delete_lambda(name, event_tx);
            }
            ServiceAction::Lambda(crate::app::messages::LambdaAction::LoadFunctionDetails(
                name,
            )) => {
                self.handle_load_function_details(name, event_tx);
            }
            ServiceAction::Backup(action) => {
                self.handle_backup_action(action, event_tx);
            }
            ServiceAction::CloudTrail(action) => match action {
                crate::app::messages::CloudTrailAction::ShowEventDetails(json) => {
                    self.services.cloudtrail.selected_event_detail = Some(json);
                    self.services.cloudtrail.show_detail_modal = true;
                }
                crate::app::messages::CloudTrailAction::CloseEventDetails => {
                    self.services.cloudtrail.show_detail_modal = false;
                    self.services.cloudtrail.selected_event_detail = None;
                }
                crate::app::messages::CloudTrailAction::DeleteTrail(name) => {
                    self.handle_delete_trail(name, event_tx);
                }
            },
            ServiceAction::Ecs(action) => {
                self.handle_ecs_action(action, event_tx);
            }
            ServiceAction::Ecr(action) => {
                self.handle_ecr_action(action, event_tx);
            }
        }
    }
}
