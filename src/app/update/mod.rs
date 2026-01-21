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
mod budgets;
mod ecr;
mod ecs;
pub mod refresh;
mod s3;
mod secretsmanager;
mod view_mode;
mod vpc;

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
    /// 
    /// Each service has a single entry point handler that processes all
    /// actions for that service.
    fn handle_service_action(
        &mut self,
        action: ServiceAction,
        event_tx: crate::app::EventSender,
    ) {
        match action {
            ServiceAction::Ec2(action) => self.handle_ec2_action(action, event_tx),
            ServiceAction::S3(action) => self.handle_s3_action(action, event_tx),
            ServiceAction::Rds(action) => self.handle_rds_action(action, event_tx),
            ServiceAction::DynamoDb(action) => self.handle_dynamodb_action(action, event_tx),
            ServiceAction::Vpc(action) => self.handle_vpc_action(action, event_tx),
            ServiceAction::Iam(action) => self.handle_iam_action(action, event_tx),
            ServiceAction::Lambda(action) => self.handle_lambda_action(action, event_tx),
            ServiceAction::Backup(action) => self.handle_backup_action(action, event_tx),
            ServiceAction::CloudTrail(action) => self.handle_cloudtrail_action(action, event_tx),
            ServiceAction::SecretsManager(action) => self.handle_secretsmanager_action(action, event_tx),
            ServiceAction::Ecs(action) => self.handle_ecs_action(action, event_tx),
            ServiceAction::Ecr(action) => self.handle_ecr_action(action, event_tx),
            ServiceAction::Budgets(action) => self.handle_budgets_action(action, event_tx),
        }
    }
}

