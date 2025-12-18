//! DynamoDB-specific actions

use std::collections::HashMap;

/// DynamoDB-specific actions
#[derive(Debug, Clone)]
pub enum DynamoDbAction {
    DrillDownTable,
    ExitDrillDown,
    LoadItems(String),
    DeleteItem {
        table_name: String,
        key_attrs: HashMap<String, String>,
    },
}
