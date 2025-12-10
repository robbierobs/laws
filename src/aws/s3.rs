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
}
