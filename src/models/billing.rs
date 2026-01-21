//! Billing models for AWS Billing service

use serde::{Deserialize, Serialize};

/// Represents an AWS Billing View
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingView {
    pub arn: String,
    pub name: String,
    pub owner_account_id: String,
    pub description: Option<String>,
}

impl BillingView {
    pub fn from_aws(view: &aws_sdk_billing::types::BillingViewListElement) -> Self {
        Self {
            arn: view.arn().unwrap_or_default().to_string(),
            name: view.name().unwrap_or_default().to_string(),
            owner_account_id: view.owner_account_id().unwrap_or_default().to_string(),
            description: view.description().map(|s| s.to_string()),
        }
    }
}

impl crate::models::Filterable for BillingView {
    fn matches_filter(&self, filter: &str) -> bool {
        self.name.to_lowercase().contains(filter)
            || self.owner_account_id.to_lowercase().contains(filter)
    }
}
