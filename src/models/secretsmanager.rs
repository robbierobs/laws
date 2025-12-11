use aws_sdk_secretsmanager::types::SecretListEntry;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Secret {
    pub name: String,
    pub arn: Option<String>,
    pub description: Option<String>,
    pub created_date: Option<String>,
    pub last_changed_date: Option<String>,
    pub last_accessed_date: Option<String>,
    pub deleted_date: Option<String>,
    pub secret_versions_to_stages: Option<HashMap<String, Vec<String>>>,
}

impl Secret {
    pub fn from_aws(item: &SecretListEntry) -> Self {
        Self {
            name: item.name().unwrap_or_default().to_string(),
            arn: item.arn().map(|s| s.to_string()),
            description: item.description().map(|s| s.to_string()),
            created_date: item.created_date().map(|d| d.to_string()),
            last_changed_date: item.last_changed_date().map(|d| d.to_string()),
            last_accessed_date: item.last_accessed_date().map(|d| d.to_string()),
            deleted_date: item.deleted_date().map(|d| d.to_string()),
            secret_versions_to_stages: item.secret_versions_to_stages().map(|map| {
                map.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            }),
        }
    }
}

impl crate::models::Filterable for Secret {
    fn matches_filter(&self, filter: &str) -> bool {
        self.name.to_lowercase().contains(filter)
            || self.description.as_deref().unwrap_or("").to_lowercase().contains(filter)
            || self.arn.as_deref().unwrap_or("").to_lowercase().contains(filter)
    }
}
