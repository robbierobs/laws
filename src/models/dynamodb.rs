use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamoDbTable {
    pub table_name: String,
    pub table_status: String,
    pub item_count: Option<i64>,
    pub table_size_bytes: Option<i64>,
    pub creation_date_time: Option<String>,
    // Key schema
    pub partition_key: Option<KeyAttribute>,
    pub sort_key: Option<KeyAttribute>,
    // Billing
    pub billing_mode: Option<String>,
    pub read_capacity_units: Option<i64>,
    pub write_capacity_units: Option<i64>,
    // Indexes
    pub global_secondary_indexes: Vec<GlobalSecondaryIndex>,
    pub local_secondary_indexes: Vec<LocalSecondaryIndex>,
    // Stream
    pub stream_enabled: bool,
    pub stream_view_type: Option<String>,
    // Other
    pub table_arn: Option<String>,
    pub deletion_protection: bool,
    pub table_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAttribute {
    pub name: String,
    pub key_type: String,       // HASH or RANGE
    pub attribute_type: String, // S, N, B
}

/// Represents a single item (row) in a DynamoDB table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamoDbItem {
    /// Map of attribute name to string representation of value
    pub attributes: HashMap<String, String>,
}

impl DynamoDbItem {
    /// Get the value of an attribute by name
    pub fn get(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Get the first few attribute values for display
    #[allow(dead_code)]
    pub fn preview(&self, max_attrs: usize) -> String {
        let mut parts: Vec<String> = self
            .attributes
            .iter()
            .take(max_attrs)
            .map(|(k, v)| {
                let truncated = if v.len() > 30 {
                    format!("{}...", &v[..27])
                } else {
                    v.clone()
                };
                format!("{}={}", k, truncated)
            })
            .collect();
        if self.attributes.len() > max_attrs {
            parts.push("...".to_string());
        }
        parts.join(", ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSecondaryIndex {
    pub index_name: String,
    pub key_schema: String,
    pub projection_type: Option<String>,
    pub index_status: Option<String>,
    pub item_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSecondaryIndex {
    pub index_name: String,
    pub key_schema: String,
    pub projection_type: Option<String>,
    pub item_count: Option<i64>,
}

impl DynamoDbTable {
    pub fn state_color(&self) -> ratatui::style::Color {
        use crate::ui::theme::THEME;
        match self.table_status.to_uppercase().as_str() {
            "ACTIVE" => THEME.success,
            "CREATING" | "UPDATING" => THEME.warning,
            "DELETING" => THEME.error,
            "INACCESSIBLE_ENCRYPTION_CREDENTIALS" | "ARCHIVING" | "ARCHIVED" => THEME.error,
            _ => THEME.muted,
        }
    }

    pub fn format_size(&self) -> String {
        if let Some(size) = self.table_size_bytes {
            const KB: i64 = 1024;
            const MB: i64 = KB * 1024;
            const GB: i64 = MB * 1024;

            if size >= GB {
                format!("{:.2} GB", size as f64 / GB as f64)
            } else if size >= MB {
                format!("{:.2} MB", size as f64 / MB as f64)
            } else if size >= KB {
                format!("{:.2} KB", size as f64 / KB as f64)
            } else {
                format!("{} B", size)
            }
        } else {
            "-".to_string()
        }
    }
}

impl crate::models::Filterable for DynamoDbTable {
    fn matches_filter(&self, filter: &str) -> bool {
        self.table_name.to_lowercase().contains(filter)
    }
}

impl crate::models::Filterable for DynamoDbItem {
    fn matches_filter(&self, filter: &str) -> bool {
        // Filter across all attribute values
        self.attributes
            .values()
            .any(|v| v.to_lowercase().contains(filter))
    }
}
