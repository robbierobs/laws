use crate::error::AppResult;
use crate::models::secretsmanager::Secret;
use crate::utils::error::format_sdk_error;
use aws_sdk_secretsmanager::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(SecretsManagerService, Client);

impl SecretsManagerService {

    pub async fn list_secrets(&self) -> AppResult<Vec<Secret>> {
        let mut secrets = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.list_secrets();
            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("SecretsManager", "list_secrets", "all", e))?;

            for secret in response.secret_list() {
                secrets.push(Secret::from_aws(secret));
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(secrets)
    }
    pub async fn get_secret_value(&self, arn: &str) -> AppResult<String> {
        let response = self
            .client
            .get_secret_value()
            .secret_id(arn)
            .send()
            .await
            .map_err(|e| format_sdk_error("SecretsManager", "get_secret_value", arn, e))?;

        if let Some(secret_string) = response.secret_string() {
            Ok(secret_string.to_string())
        } else {
            // Binary secrets are not supported for now, return placeholder
            Ok("[Binary secret value not supported]".to_string())
        }

    }

    pub async fn delete_secret(&self, arn: &str) -> AppResult<()> {
        self.client
            .delete_secret()
            .secret_id(arn)
            .force_delete_without_recovery(true)
            .send()
            .await
            .map_err(|e| format_sdk_error("SecretsManager", "delete_secret", arn, e))?;
        Ok(())
    }
}

impl crate::aws::traits::AwsService<Secret> for SecretsManagerService {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<Secret>>> + Send + 'a>> {
        Box::pin(self.list_secrets())
    }
}
