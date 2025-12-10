use crate::models::s3::S3Bucket;
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
                    let code = svc_err.meta().code().unwrap_or("Unknown");
                    let message = svc_err.meta().message().unwrap_or("No message");
                    
                    // Check for redirect error (bucket in different region)
                    if code == "PermanentRedirect" || code == "301" {
                        format!("Bucket '{}' is in a different region. Try running with --region <bucket-region>", bucket_name)
                    } else {
                        format!("S3 error for '{}': {} - {}", bucket_name, code, message)
                    }
                } else {
                    // Check if it's a redirect by looking at the error string
                    let err_str = format!("{:?}", e);
                    if err_str.contains("PermanentRedirect") || err_str.contains("301") {
                        format!("Bucket '{}' is in a different region. Try running with --region <bucket-region>", bucket_name)
                    } else {
                        format!("S3 error for '{}': {}", bucket_name, e)
                    }
                };
                Err(anyhow::anyhow!(msg))
            }
        }
    }
}
