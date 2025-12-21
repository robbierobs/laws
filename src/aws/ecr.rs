use crate::error::AppResult;
use crate::models::ecr::{EcrImage, EcrRepository};
use crate::utils::error::format_sdk_error;
use aws_sdk_ecr::Client;

/// Result from describing ECR images with pagination
#[derive(Debug)]
pub struct EcrImagesResult {
    pub images: Vec<EcrImage>,
    pub next_token: Option<String>,
}

// Use macro to generate struct and constructor
crate::aws_service_struct!(EcrService, Client);

impl EcrService {

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
            .map(EcrRepository::from_aws)
            .collect();

        Ok(repos)
    }

    #[allow(dead_code)] // Backward-compatible wrapper, may be used in tests
    pub async fn describe_images(&self, repository_name: &str) -> AppResult<Vec<EcrImage>> {
        // Simple version without pagination for backward compatibility
        let result = self.describe_images_paginated(repository_name, None, None, None).await?;
        Ok(result.images)
    }

    /// Describe images with pagination and optional tag status filter
    /// 
    /// # Arguments
    /// * `repository_name` - The repository to query
    /// * `max_results` - Maximum number of results (1-1000, default 100)
    /// * `next_token` - Token for pagination
    /// * `tag_status` - Filter by tag status: "TAGGED", "UNTAGGED", or "ANY" (default)
    pub async fn describe_images_paginated(
        &self,
        repository_name: &str,
        max_results: Option<i32>,
        next_token: Option<&str>,
        tag_status: Option<&str>,
    ) -> AppResult<EcrImagesResult> {
        let mut request = self
            .client
            .describe_images()
            .repository_name(repository_name);

        // Add pagination parameters
        if let Some(max) = max_results {
            request = request.max_results(max);
        }
        if let Some(token) = next_token {
            request = request.next_token(token);
        }

        // Add tag status filter if specified
        if let Some(status) = tag_status {
            use aws_sdk_ecr::types::{DescribeImagesFilter, TagStatus};
            let tag_status_enum = match status.to_uppercase().as_str() {
                "TAGGED" => TagStatus::Tagged,
                "UNTAGGED" => TagStatus::Untagged,
                _ => TagStatus::Any,
            };
            let filter = DescribeImagesFilter::builder()
                .tag_status(tag_status_enum)
                .build();
            request = request.filter(filter);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format_sdk_error("ECR", "describe_images", repository_name, e))?;

        let images = response
            .image_details()
            .iter()
            .map(EcrImage::from_aws)
            .collect();

        Ok(EcrImagesResult {
            images,
            next_token: response.next_token().map(|s| s.to_string()),
        })
    }

    /// Get ECR authorization token for docker login
    /// Returns (username, password, proxy_endpoint) tuple
    pub async fn get_authorization_token(&self) -> AppResult<(String, String, String)> {
        use base64::Engine;

        let response = self
            .client
            .get_authorization_token()
            .send()
            .await
            .map_err(|e| format_sdk_error("ECR", "get_authorization_token", "default", e))?;

        let auth_data = response
            .authorization_data()
            .first()
            .ok_or_else(|| crate::error::AppError::Internal("No authorization data returned".to_string()))?;

        let token = auth_data
            .authorization_token()
            .ok_or_else(|| crate::error::AppError::Internal("No authorization token".to_string()))?;

        let proxy_endpoint = auth_data
            .proxy_endpoint()
            .ok_or_else(|| crate::error::AppError::Internal("No proxy endpoint".to_string()))?
            .to_string();

        // Token is base64 encoded "username:password"
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(token)
            .map_err(|e| crate::error::AppError::Internal(format!("Failed to decode token: {}", e)))?;

        let decoded_str = String::from_utf8(decoded)
            .map_err(|e| crate::error::AppError::Internal(format!("Invalid UTF-8 in token: {}", e)))?;

        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(crate::error::AppError::Internal("Invalid token format".to_string()));
        }

        Ok((parts[0].to_string(), parts[1].to_string(), proxy_endpoint))
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
