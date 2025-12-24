//! IAM update handlers
//!
//! Handles IAM-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::{App, IamViewMode};
use crate::app::messages::IamAction;
use crate::event::{AwsEvent, Event};

impl App {
    /// Main entry point for IAM actions
    pub(super) fn handle_iam_action(
        &mut self,
        action: IamAction,
        event_tx: crate::app::EventSender,
    ) {
        match action {
            IamAction::DrillDownUser => {
                self.handle_drill_down_iam_user(event_tx);
            }
            IamAction::DrillDownRole => {
                self.handle_drill_down_iam_role(event_tx);
            }
            IamAction::DrillDownPolicy => {
                self.handle_drill_down_iam_policy(event_tx);
            }
            IamAction::ExitDrillDown => {
                self.handle_exit_iam_drill_down();
            }
            IamAction::DeleteUser(name) => {
                self.handle_delete_iam_user(name, event_tx);
            }
            IamAction::DeleteRole(name) => {
                self.handle_delete_iam_role(name, event_tx);
            }
            IamAction::DeletePolicy(arn) => {
                self.handle_delete_iam_policy(arn, event_tx);
            }
        }
    }

    fn handle_drill_down_iam_user(&mut self, event_tx: crate::app::EventSender) {
        let Some(idx) = self.services.iam.list_state.selected() else {
            return;
        };

        let Some(user) = self.services.iam.users.get(idx) else {
            return;
        };

        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.services.iam.selected_entity_name = Some(user.user_name.clone());
        self.loading = true;

        let client = clients.iam.clone();
        let tx = event_tx;
        let name = user.user_name.clone();

        let handle = tokio::spawn(async move {
            let service = crate::aws::iam::IamService::new(client);
            match service.list_attached_user_policies(&name).await {
                Ok(policies) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::IamUserPoliciesLoaded(policies))))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::IAM_POLICIES, handle);
    }

    fn handle_drill_down_iam_role(&mut self, event_tx: crate::app::EventSender) {
        let Some(idx) = self.services.iam.list_state.selected() else {
            return;
        };

        let Some(role) = self.services.iam.roles.get(idx) else {
            return;
        };

        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.services.iam.selected_entity_name = Some(role.role_name.clone());
        self.loading = true;

        let client = clients.iam.clone();
        let tx = event_tx;
        let name = role.role_name.clone();

        let handle = tokio::spawn(async move {
            let service = crate::aws::iam::IamService::new(client);
            match service.list_attached_role_policies(&name).await {
                Ok(policies) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::IamRolePoliciesLoaded(policies))))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::IAM_POLICIES, handle);
    }

    fn handle_drill_down_iam_policy(&mut self, event_tx: crate::app::EventSender) {
        let Some(idx) = self.services.iam.list_state.selected() else {
            return;
        };

        let policy = if self.services.iam.view_mode == IamViewMode::Policies {
            self.services.iam.policies.get(idx)
        } else {
            self.services.iam.current_policies.get(idx)
        };

        self.services.iam.previous_view_mode = self.services.iam.view_mode;

        let Some(p) = policy else {
            return;
        };

        let Some(arn) = &p.arn else {
            return;
        };

        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.services.iam.selected_entity_name = Some(p.policy_name.clone());
        self.loading = true;

        let client = clients.iam.clone();
        let tx = event_tx;
        let arn = arn.clone();

        let handle = tokio::spawn(async move {
            let service = crate::aws::iam::IamService::new(client);
            match service.get_policy_version(&arn).await {
                Ok(doc) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::IamPolicyDocumentLoaded(doc))))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::IAM_POLICIES, handle);
    }

    fn handle_exit_iam_drill_down(&mut self) {
        match self.services.iam.view_mode {
            IamViewMode::UserAttachedPolicies => {
                self.services.iam.view_mode = IamViewMode::Users;
            }
            IamViewMode::RoleAttachedPolicies => {
                self.services.iam.view_mode = IamViewMode::Roles;
            }
            IamViewMode::PolicyDocument => {
                self.services.iam.view_mode = self.services.iam.previous_view_mode;
            }
            _ => {}
        }

        self.services.iam.current_policies.clear();
        self.services.iam.current_policy_document.clear();
        self.services.iam.selected_entity_name = None;
        self.services.iam.list_state.select(Some(0));
    }
    fn perform_iam_delete<F, Fut>(
        &mut self,
        log_desc: String,
        success_msg: String,
        event_tx: crate::app::EventSender,
        action: F,
    ) where
        F: FnOnce(crate::aws::iam::IamService) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = crate::error::AppResult<()>> + Send,
    {
        self.action_log.push(format!("Deleting IAM {}", log_desc));
        
        self.spawn_aws_task(event_tx, task_keys::IAM_ACTION, move |clients, tx| async move {
            let service = crate::aws::iam::IamService::new(clients.iam.clone());
            match action(service).await {
                Ok(_) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(success_msg)))).await.ok();
                    // Trigger refresh
                    tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
    }

    fn handle_delete_iam_user(&mut self, user_name: String, event_tx: crate::app::EventSender) {
        let name = user_name.clone();
        self.perform_iam_delete(
            format!("User: {}", user_name),
            format!("User {} deleted", user_name),
            event_tx,
            move |service| async move { service.delete_user(&name).await },
        );
    }

    fn handle_delete_iam_role(&mut self, role_name: String, event_tx: crate::app::EventSender) {
        let name = role_name.clone();
        self.perform_iam_delete(
            format!("Role: {}", role_name),
            format!("Role {} deleted", role_name),
            event_tx,
            move |service| async move { service.delete_role(&name).await },
        );
    }

    fn handle_delete_iam_policy(&mut self, policy_arn: String, event_tx: crate::app::EventSender) {
        let arn = policy_arn.clone();
        self.perform_iam_delete(
            format!("Policy: {}", policy_arn),
            format!("Policy {} deleted", policy_arn),
            event_tx,
            move |service| async move { service.delete_policy(&arn).await },
        );
    }
}
