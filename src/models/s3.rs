use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Bucket {
    pub name: String,
    pub creation_date: Option<String>,
    pub region: Option<String>, // Note: ListBuckets doesn't return region directly usually, but we might fetch it
}

impl S3Bucket {
    pub fn from_aws(bucket: aws_sdk_s3::types::Bucket) -> Self {
        Self {
            name: bucket.name().unwrap_or_default().to_string(),
            creation_date: bucket.creation_date().map(|d| d.to_string()),
            region: None, // Requires separate call usually
        }
    }
}
