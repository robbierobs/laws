use crate::models::secretsmanager::Secret;
use crate::utils::error::format_sdk_error;
use aws_sdk_secretsmanager::Client;

pub struct SecretsManagerService {
    client: Client,
}

impl SecretsManagerService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_secrets(&self) -> anyhow::Result<Vec<Secret>> {
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
}

impl crate::aws::traits::AwsService<Secret> for SecretsManagerService {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<Secret>>> + Send + 'a>> {
        Box::pin(self.list_secrets())
    }
}
