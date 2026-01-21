//! Budget models for AWS Budgets service

use serde::{Deserialize, Serialize};

/// Represents an AWS Budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub budget_name: String,
    pub budget_type: String,
    pub time_unit: String,
    pub budget_limit: Option<f64>,
    pub actual_spend: Option<f64>,
    pub forecasted_spend: Option<f64>,
    pub time_period_start: Option<String>,
    pub time_period_end: Option<String>,
}

impl Budget {
    pub fn from_aws(budget: &aws_sdk_budgets::types::Budget) -> Self {
        // Budget limit amount is a String that needs parsing
        let budget_limit = budget
            .budget_limit()
            .and_then(|s| s.amount().parse::<f64>().ok());

        let actual_spend = budget
            .calculated_spend()
            .and_then(|cs| cs.actual_spend())
            .and_then(|s| s.amount().parse::<f64>().ok());

        let forecasted_spend = budget
            .calculated_spend()
            .and_then(|cs| cs.forecasted_spend())
            .and_then(|s| s.amount().parse::<f64>().ok());

        Self {
            budget_name: budget.budget_name().to_string(),
            budget_type: budget.budget_type().as_str().to_string(),
            time_unit: budget.time_unit().as_str().to_string(),
            budget_limit,
            actual_spend,
            forecasted_spend,
            time_period_start: budget
                .time_period()
                .and_then(|tp| tp.start())
                .map(|d| d.to_string()),
            time_period_end: budget
                .time_period()
                .and_then(|tp| tp.end())
                .map(|d| d.to_string()),
        }
    }

    /// Calculate the percentage of budget used (actual / limit * 100)
    pub fn usage_percentage(&self) -> Option<f64> {
        match (self.actual_spend, self.budget_limit) {
            (Some(actual), Some(limit)) if limit > 0.0 => Some((actual / limit) * 100.0),
            _ => None,
        }
    }

    /// Calculate the forecasted percentage (forecasted / limit * 100)
    pub fn forecasted_percentage(&self) -> Option<f64> {
        match (self.forecasted_spend, self.budget_limit) {
            (Some(forecasted), Some(limit)) if limit > 0.0 => Some((forecasted / limit) * 100.0),
            _ => None,
        }
    }

    /// Get color based on usage percentage
    pub fn status_color(&self) -> ratatui::style::Color {
        use crate::ui::theme::THEME;
        match self.usage_percentage() {
            Some(pct) if pct >= 100.0 => THEME.error,
            Some(pct) if pct >= 80.0 => THEME.warning,
            Some(_) => THEME.success,
            None => THEME.muted,
        }
    }
}

impl crate::models::Filterable for Budget {
    fn matches_filter(&self, filter: &str) -> bool {
        self.budget_name.to_lowercase().contains(filter)
            || self.budget_type.to_lowercase().contains(filter)
    }
}

/// Represents a notification/alert for a budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetNotification {
    pub notification_type: String,
    pub comparison_operator: String,
    pub threshold: f64,
    pub threshold_type: String,
    pub notification_state: Option<String>,
}

impl BudgetNotification {
    pub fn from_aws(notification: &aws_sdk_budgets::types::Notification) -> Self {
        Self {
            notification_type: notification.notification_type().as_str().to_string(),
            comparison_operator: notification.comparison_operator().as_str().to_string(),
            threshold: notification.threshold(),
            threshold_type: notification
                .threshold_type()
                .map(|t| t.as_str().to_string())
                .unwrap_or_else(|| "PERCENTAGE".to_string()),
            notification_state: notification
                .notification_state()
                .map(|s| s.as_str().to_string()),
        }
    }

    /// Get color based on notification state
    pub fn state_color(&self) -> ratatui::style::Color {
        use crate::ui::theme::THEME;
        match self.notification_state.as_deref() {
            Some("ALARM") => THEME.error,
            Some("OK") => THEME.success,
            _ => THEME.muted,
        }
    }

    /// Format the threshold for display
    pub fn threshold_display(&self) -> String {
        if self.threshold_type == "PERCENTAGE" {
            format!("{}%", self.threshold)
        } else {
            format!("${:.2}", self.threshold)
        }
    }
}

impl crate::models::Filterable for BudgetNotification {
    fn matches_filter(&self, filter: &str) -> bool {
        self.notification_type.to_lowercase().contains(filter)
            || self.threshold_display().to_lowercase().contains(filter)
    }
}

/// Represents a subscriber to a budget notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSubscriber {
    pub subscription_type: String,
    pub address: String,
}

impl BudgetSubscriber {
    pub fn from_aws(subscriber: &aws_sdk_budgets::types::Subscriber) -> Self {
        Self {
            subscription_type: subscriber.subscription_type().as_str().to_string(),
            address: subscriber.address().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_usage_percentage() {
        let budget = Budget {
            budget_name: "Test".to_string(),
            budget_type: "COST".to_string(),
            time_unit: "MONTHLY".to_string(),
            budget_limit: Some(100.0),
            actual_spend: Some(45.0),
            forecasted_spend: Some(80.0),
            time_period_start: None,
            time_period_end: None,
        };

        assert_eq!(budget.usage_percentage(), Some(45.0));
        assert_eq!(budget.forecasted_percentage(), Some(80.0));
    }

    #[test]
    fn test_budget_usage_percentage_none_when_no_limit() {
        let budget = Budget {
            budget_name: "Test".to_string(),
            budget_type: "COST".to_string(),
            time_unit: "MONTHLY".to_string(),
            budget_limit: None,
            actual_spend: Some(45.0),
            forecasted_spend: None,
            time_period_start: None,
            time_period_end: None,
        };

        assert_eq!(budget.usage_percentage(), None);
    }

    #[test]
    fn test_notification_threshold_display() {
        let notification = BudgetNotification {
            notification_type: "ACTUAL".to_string(),
            comparison_operator: "GREATER_THAN".to_string(),
            threshold: 80.0,
            threshold_type: "PERCENTAGE".to_string(),
            notification_state: Some("OK".to_string()),
        };

        assert_eq!(notification.threshold_display(), "80%");
    }
}
