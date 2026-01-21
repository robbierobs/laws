use crate::error::AppResult;
use crate::models::s3::{S3Bucket, S3BucketDetails};
use crate::utils::error::{format_s3_error, format_sdk_error};
use aws_sdk_s3::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(S3Service, Client);

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
    ) -> AppResult<Vec<crate::models::s3::S3Object>> {
        let result = self
            .client
            .list_objects_v2()
            .bucket(bucket_name)
            .send()
            .await;

        match result {
            Ok(response) => {
                let objects = response
                    .contents()
                    .iter()
                    .map(|o| crate::models::s3::S3Object::from_aws(o.clone()))
                    .collect();
                Ok(objects)
            }
            Err(e) => Err(format_s3_error(bucket_name, e)),
        }
    }

    /// Fetch bucket details asynchronously (versioning, encryption, object count)
    pub async fn get_bucket_details(&self, bucket_name: &str) -> S3BucketDetails {
        let mut details = S3BucketDetails::default();

        // Get versioning status
        match self
            .client
            .get_bucket_versioning()
            .bucket(bucket_name)
            .send()
            .await
        {
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
        match self
            .client
            .get_bucket_encryption()
            .bucket(bucket_name)
            .send()
            .await
        {
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
        match self
            .client
            .get_bucket_tagging()
            .bucket(bucket_name)
            .send()
            .await
        {
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
