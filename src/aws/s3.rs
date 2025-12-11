use crate::models::s3::{S3Bucket, S3BucketDetails};
use aws_sdk_s3::Client;

pub struct S3Service {
    client: Client,
}

impl S3Service {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_buckets(&self) -> anyhow::Result<Vec<S3Bucket>> {
        let response = self.client
            .list_buckets()
            .send()
            .await?;

        let buckets = response
            .buckets()
            .iter()
            .map(|b| S3Bucket::from_aws(b.clone()))
            .collect();

        Ok(buckets)
    }

    pub async fn list_objects(&self, bucket_name: &str) -> anyhow::Result<Vec<crate::models::s3::S3Object>> {
        let result = self.client
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
            Err(e) => {
                // Extract more details from the error
                let msg = if let Some(svc_err) = e.as_service_error() {
                    let code = svc_err.meta().code().unwrap_or("");
                    let message = svc_err.meta().message().unwrap_or("");
                    
                    // Check for redirect error (bucket in different region)
                    if code == "PermanentRedirect" || code == "301" {
                        format!("Bucket '{}' is in a different region. Try running with --region <bucket-region>", bucket_name)
                    } else if !code.is_empty() && !message.is_empty() {
                        // Both code and message available
                        format!("S3 error for '{}': {} - {}", bucket_name, code, message)
                    } else if !code.is_empty() {
                        // Only code available
                        format!("S3 error for '{}': {}", bucket_name, code)
                    } else if !message.is_empty() {
                        // Only message available
                        format!("S3 error for '{}': {}", bucket_name, message)
                    } else {
                        // Neither code nor message - use full debug output
                        format!("S3 error for '{}': {:?}", bucket_name, svc_err)
                    }
                } else {
                    // Not a service error - check for common patterns
                    let err_str = format!("{:?}", e);
                    if err_str.contains("PermanentRedirect") || err_str.contains("301") {
                        format!("Bucket '{}' is in a different region. Try running with --region <bucket-region>", bucket_name)
                    } else if err_str.contains("NoSuchBucket") {
                        format!("Bucket '{}' does not exist", bucket_name)
                    } else if err_str.contains("AccessDenied") {
                        format!("Access denied to bucket '{}'. Check your IAM permissions.", bucket_name)
                    } else if err_str.contains("InvalidAccessKeyId") || err_str.contains("SignatureDoesNotMatch") {
                        format!("Invalid AWS credentials when accessing bucket '{}'", bucket_name)
                    } else if err_str.contains("XmlDecodeError") || err_str.contains("invalid XML") {
                        // LocalStack compatibility issue
                        format!("Bucket '{}' returned invalid XML response. This may be a LocalStack limitation - try upgrading LocalStack or checking if the bucket exists.", bucket_name)
                    } else if err_str.contains("Unhandled") {
                        // Generic unhandled error - likely LocalStack
                        format!("Bucket '{}' returned an unhandled error. This may be a LocalStack limitation.", bucket_name)
                    } else {
                        // Use Display for cleaner output, but include Debug if empty
                        let display_msg = format!("{}", e);
                        if display_msg.is_empty() || display_msg == "service error" {
                            format!("S3 error for '{}': {:?}", bucket_name, e)
                        } else {
                            format!("S3 error for '{}': {}", bucket_name, display_msg)
                        }
                    }
                };
                Err(anyhow::anyhow!(msg))
            }
        }
    }

    /// Fetch bucket details asynchronously (versioning, encryption, object count)
    pub async fn get_bucket_details(&self, bucket_name: &str) -> S3BucketDetails {
        let mut details = S3BucketDetails::default();

        // Get versioning status
        if let Ok(resp) = self.client
            .get_bucket_versioning()
            .bucket(bucket_name)
            .send()
            .await
        {
            details.versioning_enabled = resp.status()
                .map(|s| s == &aws_sdk_s3::types::BucketVersioningStatus::Enabled);
        }

        // Get encryption configuration
        if let Ok(resp) = self.client
            .get_bucket_encryption()
            .bucket(bucket_name)
            .send()
            .await
        {
            if let Some(config) = resp.server_side_encryption_configuration() {
                let encryption_types: Vec<String> = config.rules()
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

        // Get object count and total size
        let mut object_count: i64 = 0;
        let mut total_size: i64 = 0;
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self.client
                .list_objects_v2()
                .bucket(bucket_name);
            
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
    pub async fn delete_object(&self, bucket_name: &str, key: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    /// Download an object from S3 and return its contents as bytes
    pub async fn get_object(&self, bucket_name: &str, key: &str) -> anyhow::Result<Vec<u8>> {
        let response = self.client
            .get_object()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get object s3://{}/{}: {}", bucket_name, key, e))?;

        let bytes = response
            .body
            .collect()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read object body: {}", e))?
            .into_bytes()
            .to_vec();

        Ok(bytes)
    }
}


impl crate::aws::traits::AwsService<S3Bucket> for S3Service {
    fn list<'a>(&'a self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<S3Bucket>>> + Send + 'a>> {
        Box::pin(self.list_buckets())
    }
}
