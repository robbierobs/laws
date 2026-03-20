//! AWS Budgets service wrapper

use crate::error::AppResult;
use crate::models::budgets::{Budget, BudgetNotification, BudgetSubscriber};
use crate::utils::error::format_sdk_error;
use aws_sdk_budgets::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(BudgetsService, Client);

impl BudgetsService {
    /// List all budgets for the account
    /// Note: Requires account ID which we'll retrieve from STS
    pub async fn list_budgets(&self, account_id: &str) -> AppResult<Vec<Budget>> {
        let response = self
            .client
            .describe_budgets()
            .account_id(account_id)
            .send()
            .await
            .map_err(|e| format_sdk_error("Budgets", "describe_budgets", account_id, e))?;

        let budgets = response
            .budgets()
            .iter()
            .map(Budget::from_aws)
            .collect();

        Ok(budgets)
    }

    /// Get notifications for a specific budget
    pub async fn describe_notifications_for_budget(
        &self,
        account_id: &str,
        budget_name: &str,
    ) -> AppResult<Vec<BudgetNotification>> {
        let response = self
            .client
            .describe_notifications_for_budget()
            .account_id(account_id)
            .budget_name(budget_name)
            .send()
            .await
            .map_err(|e| {
                format_sdk_error("Budgets", "describe_notifications_for_budget", budget_name, e)
            })?;

        let notifications = response
            .notifications()
            .iter()
            .map(BudgetNotification::from_aws)
            .collect();

        Ok(notifications)
    }

    /// Get subscribers for a specific notification
    #[allow(dead_code)] // TODO: Wire up budget subscriber notifications
    pub async fn describe_subscribers_for_notification(
        &self,
        account_id: &str,
        budget_name: &str,
        notification: &aws_sdk_budgets::types::Notification,
    ) -> AppResult<Vec<BudgetSubscriber>> {
        let response = self
            .client
            .describe_subscribers_for_notification()
            .account_id(account_id)
            .budget_name(budget_name)
            .notification(notification.clone())
            .send()
            .await
            .map_err(|e| {
                format_sdk_error(
                    "Budgets",
                    "describe_subscribers_for_notification",
                    budget_name,
                    e,
                )
            })?;

        let subscribers = response
            .subscribers()
            .iter()
            .map(BudgetSubscriber::from_aws)
            .collect();

        Ok(subscribers)
    }
}

impl crate::aws::traits::AwsService<Budget> for BudgetsService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AppResult<Vec<Budget>>> + Send + 'a>,
    > {
        // Note: This requires account_id, so we can't use the simple trait pattern
        // We'll need to handle this differently in the state refresh
        Box::pin(async move {
            // Return empty for now - actual list will use list_budgets with account_id
            Ok(Vec::new())
        })
    }
}
