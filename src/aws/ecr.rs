use crate::error::AppResult;
use crate::models::ecr::{EcrImage, EcrRepository};
use crate::utils::error::format_sdk_error;
use aws_sdk_ecr::Client;

pub struct EcrService {
    client: Client,
}

impl EcrService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_repositories(&self) -> AppResult<Vec<EcrRepository>> {
        let response = self
            .client
            .describe_repositories()
            .send()
            .await
            .map_err(|e| format_sdk_error("ECR", "describe_repositories", "all", e))?;

        let repos = response
            .repositories()
            .iter()
            .map(|r| EcrRepository::from_aws(r))
            .collect();

        Ok(repos)
    }

    pub async fn describe_images(&self, repository_name: &str) -> AppResult<Vec<EcrImage>> {
        let response = self
            .client
            .describe_images()
            .repository_name(repository_name)
            .send()
            .await
            .map_err(|e| format_sdk_error("ECR", "describe_images", repository_name, e))?;

        let images = response
            .image_details()
            .iter()
            .map(|i| EcrImage::from_aws(i))
            .collect();

        Ok(images)
    }
}

impl crate::aws::traits::AwsService<EcrRepository> for EcrService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AppResult<Vec<EcrRepository>>> + Send + 'a>,
    > {
        Box::pin(self.list_repositories())
    }
}
