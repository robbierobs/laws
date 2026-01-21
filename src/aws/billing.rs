//! AWS Billing service wrapper

use crate::error::AppResult;
use crate::models::billing::BillingView;
use crate::utils::error::format_sdk_error;
use aws_sdk_billing::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(BillingService, Client);

impl BillingService {
    /// List all billing views
    pub async fn list_billing_views(&self) -> AppResult<Vec<BillingView>> {
        let response = self
            .client
            .list_billing_views()
            .send()
            .await
            .map_err(|e| format_sdk_error("Billing", "list_billing_views", "all", e))?;

        let views = response
            .billing_views()
            .iter()
            .map(BillingView::from_aws)
            .collect();

        Ok(views)
    }
}

impl crate::aws::traits::AwsService<BillingView> for BillingService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AppResult<Vec<BillingView>>> + Send + 'a>,
    > {
        Box::pin(self.list_billing_views())
    }
}
