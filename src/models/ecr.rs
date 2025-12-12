use serde::{Deserialize, Serialize};

use crate::models::Filterable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcrRepository {
    pub repository_name: String,
    pub repository_arn: Option<String>,
    pub repository_uri: Option<String>,
    pub created_at: Option<String>,
    pub image_tag_mutability: Option<String>,
}

impl EcrRepository {
    pub fn from_aws(repo: &aws_sdk_ecr::types::Repository) -> Self {
        Self {
            repository_name: repo.repository_name().unwrap_or_default().to_string(),
            repository_arn: repo.repository_arn().map(|s| s.to_string()),
            repository_uri: repo.repository_uri().map(|s| s.to_string()),
            created_at: repo.created_at().map(|d| d.to_string()),
            image_tag_mutability: repo
                .image_tag_mutability()
                .map(|m| m.as_str().to_string()),
        }
    }
}

impl Filterable for EcrRepository {
    fn matches_filter(&self, filter: &str) -> bool {
        self.repository_name.to_lowercase().contains(filter)
            || self
                .repository_uri
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(filter)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcrImage {
    pub image_digest: String,
    pub image_tags: Vec<String>,
    pub image_pushed_at: Option<String>,
    pub image_size_in_bytes: Option<i64>,
    pub image_uri: Option<String>, // Constructed usually
}

impl EcrImage {
    pub fn from_aws(image: &aws_sdk_ecr::types::ImageDetail) -> Self {
        Self {
            image_digest: image.image_digest().unwrap_or_default().to_string(),
            image_tags: image
                .image_tags()
                .iter()
                .map(|t| t.to_string())
                .collect(),
            image_pushed_at: image.image_pushed_at().map(|d| d.to_string()),
            image_size_in_bytes: image.image_size_in_bytes(),
            image_uri: None, // Can be constructed if we have repo URI
        }
    }
    
    #[allow(dead_code)] // Helper for display, may be used in future
    pub fn main_tag(&self) -> String {
        self.image_tags.first().cloned().unwrap_or_else(|| "<untagged>".to_string())
    }
}

impl Filterable for EcrImage {
    fn matches_filter(&self, filter: &str) -> bool {
        self.image_digest.to_lowercase().contains(filter)
            || self.image_tags.iter().any(|t| t.to_lowercase().contains(filter))
    }
}
