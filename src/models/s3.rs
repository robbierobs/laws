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

/// Extended bucket details fetched asynchronously
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3BucketDetails {
    pub versioning_enabled: Option<bool>,
    pub encryption: Option<String>,
    pub object_count: Option<i64>,
    pub total_size: Option<i64>,
    pub tags: Vec<(String, String)>,
    pub loading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Object {
    pub key: String,
    pub size: i64,
    pub last_modified: Option<String>,
    pub storage_class: Option<String>,
    pub etag: Option<String>,
}

impl S3Object {
    pub fn from_aws(object: aws_sdk_s3::types::Object) -> Self {
        Self {
            key: object.key().unwrap_or_default().to_string(),
            size: object.size().unwrap_or(0),
            last_modified: object.last_modified().map(|d| d.to_string()),
            storage_class: object.storage_class().map(|s| s.as_str().to_string()),
            etag: object.e_tag().map(|s| s.to_string()),
        }
    }
}

impl crate::models::Filterable for S3Bucket {
    fn matches_filter(&self, filter: &str) -> bool {
        self.name.to_lowercase().contains(filter)
    }
}

impl crate::models::Filterable for S3Object {
    fn matches_filter(&self, filter: &str) -> bool {
        self.key.to_lowercase().contains(filter)
    }
}

