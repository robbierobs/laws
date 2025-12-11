//! IAM update handlers
//!
//! Handles IAM-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::{App, IamViewMode};
use crate::event::{AwsEvent, Event};

impl App {
    pub(super) fn handle_drill_down_iam_user(&mut self, event_tx: crate::app::EventSender) {
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
                    tx.send(Event::Aws(AwsEvent::IamUserPoliciesLoaded(policies)))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::IAM_POLICIES, handle);
    }

    pub(super) fn handle_drill_down_iam_role(&mut self, event_tx: crate::app::EventSender) {
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
                    tx.send(Event::Aws(AwsEvent::IamRolePoliciesLoaded(policies)))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::IAM_POLICIES, handle);
    }

    pub(super) fn handle_drill_down_iam_policy(&mut self, event_tx: crate::app::EventSender) {
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
                    tx.send(Event::Aws(AwsEvent::IamPolicyDocumentLoaded(doc)))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::IAM_POLICIES, handle);
    }

    pub(super) fn handle_exit_iam_drill_down(&mut self) {
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
    pub(super) fn handle_delete_iam_user(&mut self, user_name: String, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };
        self.action_log.push(format!("Deleting IAM User: {}", user_name));
        let client = clients.iam.clone();
        tokio::spawn(async move {
            let service = crate::aws::iam::IamService::new(client);
            match service.delete_user(&user_name).await {
                Ok(_) => {
                    event_tx.send(Event::Aws(AwsEvent::ActionCompleted(
                        format!("User {} deleted", user_name)
                    ))).await.ok();
                    // Trigger refresh
                    event_tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                }
                Err(e) => {
                    event_tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });
    }

    pub(super) fn handle_delete_iam_role(&mut self, role_name: String, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };
        self.action_log.push(format!("Deleting IAM Role: {}", role_name));
        let client = clients.iam.clone();
        tokio::spawn(async move {
            let service = crate::aws::iam::IamService::new(client);
            match service.delete_role(&role_name).await {
                Ok(_) => {
                    event_tx.send(Event::Aws(AwsEvent::ActionCompleted(
                        format!("Role {} deleted", role_name)
                    ))).await.ok();
                    // Trigger refresh
                    event_tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                }
                Err(e) => {
                    event_tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });
    }

    pub(super) fn handle_delete_iam_policy(&mut self, policy_arn: String, event_tx: crate::app::EventSender) {
        let Some(clients) = &self.aws_clients else {
            return;
        };
        self.action_log.push(format!("Deleting IAM Policy: {}", policy_arn));
        let client = clients.iam.clone();
        tokio::spawn(async move {
            let service = crate::aws::iam::IamService::new(client);
            match service.delete_policy(&policy_arn).await {
                Ok(_) => {
                    event_tx.send(Event::Aws(AwsEvent::ActionCompleted(
                        format!("Policy {} deleted", policy_arn)
                    ))).await.ok();
                    // Trigger refresh
                    event_tx.send(crate::event::Event::Message(crate::app::Message::refresh())).await.ok();
                }
                Err(e) => {
                    event_tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).await.ok();
                }
            }
        });
    }
}
