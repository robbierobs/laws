use crate::error::AppResult;
use crate::models::s3::{S3Bucket, S3BucketDetails};
use crate::utils::error::{format_s3_error, format_sdk_error};
use aws_sdk_s3::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(S3Service, Client);

const S3_MAX_KEYS_PER_PAGE: usize = 1000;

fn connection_error_message(bucket_name: &str, error: &str) -> Option<String> {
    let lower = error.to_lowercase();
    let patterns = [
        "connection refused",
        "connection error",
        "connect error",
        "failed to connect",
        "connection reset",
        "connection aborted",
        "timed out",
        "timeout",
        "dns",
        "failed to lookup address",
        "name or service not known",
        "no such host",
        "could not resolve",
        "os error 111",
    ];

    if patterns.iter().any(|pattern| lower.contains(pattern)) {
        Some(format!(
            "Connection failed while listing objects in bucket '{}'. If you're using a custom endpoint (LocalStack/MinIO), verify --endpoint-url or AWS_ENDPOINT_URL. LocalStack often needs 127.0.0.1 instead of localhost (IPv6).",
            bucket_name
        ))
    } else {
        None
    }
}

impl S3Service {

    pub async fn list_buckets(&self) -> AppResult<Vec<S3Bucket>> {
        let response = self
            .client
            .list_buckets()
            .send()
            .await
            .map_err(|e| format_sdk_error("S3", "list_buckets", "all", e))?;

        let buckets = response
            .buckets()
            .iter()
            .map(|b| S3Bucket::from_aws(b.clone()))
            .collect();

        Ok(buckets)
    }

    pub async fn list_objects(
        &self,
        bucket_name: &str,
        max_keys: usize,
    ) -> AppResult<Vec<crate::models::s3::S3Object>> {
        if max_keys == 0 {
            return Ok(Vec::new());
        }

        let mut objects = Vec::new();
        let mut continuation_token: Option<String> = None;
        let mut remaining = max_keys;

        loop {
            let page_size = remaining.min(S3_MAX_KEYS_PER_PAGE) as i32;
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(bucket_name)
                .max_keys(page_size);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let result = request.send().await;

            match result {
                Ok(response) => {
                    for obj in response.contents() {
                        objects.push(crate::models::s3::S3Object::from_aws(obj.clone()));
                    }

                    let fetched = response.contents().len();
                    remaining = remaining.saturating_sub(fetched);
                    if remaining == 0 {
                        break;
                    }

                    if response.is_truncated() == Some(true) {
                        if let Some(token) = response.next_continuation_token() {
                            continuation_token = Some(token.to_string());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    let debug_str = format!("{:?}", e);
                    if let Some(message) = connection_error_message(bucket_name, &debug_str) {
                        return Err(crate::error::AppError::aws_api("S3", message));
                    }

                    return Err(format_s3_error(bucket_name, e));
                }
            }
        }

        Ok(objects)
    }

    /// Fetch bucket details asynchronously (versioning, encryption, object count)
    pub async fn get_bucket_details(&self, bucket_name: &str) -> S3BucketDetails {
        let mut details = S3BucketDetails::default();

        let (versioning_result, encryption_result, tagging_result) = tokio::join!(
            self.client
                .get_bucket_versioning()
                .bucket(bucket_name)
                .send(),
            self.client
                .get_bucket_encryption()
                .bucket(bucket_name)
                .send(),
            self.client
                .get_bucket_tagging()
                .bucket(bucket_name)
                .send(),
        );

        // Get versioning status
        match versioning_result {
            Ok(resp) => {
                details.versioning_enabled = resp
                    .status()
                    .map(|s| s == &aws_sdk_s3::types::BucketVersioningStatus::Enabled);
            }
            Err(e) => {
                tracing::warn!("Failed to get versioning for bucket {}: {}", bucket_name, e);
            }
        }

        // Get encryption configuration
        match encryption_result {
            Ok(resp) => {
                if let Some(config) = resp.server_side_encryption_configuration() {
                    let encryption_types: Vec<String> = config
                        .rules()
                        .iter()
                        .filter_map(|rule| {
                            rule.apply_server_side_encryption_by_default()
                                .map(|sse| sse.sse_algorithm().as_str().to_string())
                        })
                        .collect();
                    if !encryption_types.is_empty() {
                        details.encryption = Some(encryption_types.join(", "));
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to get encryption for bucket {}: {}", bucket_name, e);
            }
        }

        // Get bucket tagging
        match tagging_result {
            Ok(resp) => {
                details.tags = resp
                    .tag_set()
                    .iter()
                    .map(|t| (t.key().to_string(), t.value().to_string()))
                    .collect();
                details.tags.sort_by(|a, b| a.0.cmp(&b.0));
            }
            Err(e) => {
                tracing::warn!("Failed to get tags for bucket {}: {}", bucket_name, e);
            }
        }

        // Get object count and total size
        let mut object_count: i64 = 0;
        let mut total_size: i64 = 0;
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self.client.list_objects_v2().bucket(bucket_name);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            match request.send().await {
                Ok(resp) => {
                    object_count += resp.key_count().unwrap_or(0) as i64;
                    for obj in resp.contents() {
                        total_size += obj.size().unwrap_or(0);
                    }

                    if resp.is_truncated() == Some(true) {
                        continuation_token = resp.next_continuation_token().map(|s| s.to_string());
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        details.object_count = Some(object_count);
        details.total_size = Some(total_size);
        details.loading = false;

        details
    }

    pub async fn delete_object(&self, bucket_name: &str, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| format_sdk_error("S3", "delete_object", key, e))?;
        Ok(())
    }

    /// Create a new S3 bucket
    pub async fn create_bucket(&self, bucket_name: &str, region: &str) -> AppResult<()> {
        let mut request = self.client.create_bucket().bucket(bucket_name);

        // For regions other than us-east-1, we need to specify LocationConstraint
        if region != "us-east-1" {
            use aws_sdk_s3::types::{BucketLocationConstraint, CreateBucketConfiguration};
            let constraint = BucketLocationConstraint::from(region);
            let config = CreateBucketConfiguration::builder()
                .location_constraint(constraint)
                .build();
            request = request.create_bucket_configuration(config);
        }

        request
            .send()
            .await
            .map_err(|e| format_sdk_error("S3", "create_bucket", bucket_name, e))?;
        Ok(())
    }

    /// Delete an S3 bucket (must be empty)
    pub async fn delete_bucket(&self, bucket_name: &str) -> AppResult<()> {
        self.client
            .delete_bucket()
            .bucket(bucket_name)
            .send()
            .await
            .map_err(|e| format_sdk_error("S3", "delete_bucket", bucket_name, e))?;
        Ok(())
    }

    /// Download an object from S3 and return its contents as bytes
    pub async fn get_object(&self, bucket_name: &str, key: &str) -> AppResult<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| format_sdk_error("S3", "get_object", key, e))?;

        let bytes = response
            .body
            .collect()
            .await
            .map_err(|e| format_sdk_error("S3", "read_body", key, e))?
            .into_bytes()
            .to_vec();

        Ok(bytes)
    }

    /// Upload (put) an object to S3
    pub async fn put_object(&self, bucket_name: &str, key: &str, data: Vec<u8>) -> AppResult<()> {
        self.client
            .put_object()
            .bucket(bucket_name)
            .key(key)
            .body(data.into())
            .send()
            .await
            .map_err(|e| format_sdk_error("S3", "put_object", key, e))?;
        Ok(())
    }
}

impl crate::aws::traits::AwsService<S3Bucket> for S3Service {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<S3Bucket>>> + Send + 'a>>
    {
        Box::pin(self.list_buckets())
    }
}

#[cfg(test)]
mod tests {
    use super::connection_error_message;

    #[test]
    fn test_connection_error_message_matches_common_failure() {
        let message = connection_error_message("my-bucket", "Connection refused");
        assert!(message.is_some());
        let msg = message.expect("expected message");
        assert!(msg.contains("my-bucket"));
        assert!(msg.contains("127.0.0.1"));
    }

    #[test]
    fn test_connection_error_message_returns_none_for_service_errors() {
        let message = connection_error_message("my-bucket", "NoSuchBucket");
        assert!(message.is_none());
    }
}
