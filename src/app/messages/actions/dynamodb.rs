//! DynamoDB-specific actions

use super::super::confirmable::ConfirmableAction;
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

impl ConfirmableAction for DynamoDbAction {
    fn confirmation_description(&self) -> String {
        match self {
            Self::DeleteItem { table_name, .. } => {
                format!("Delete item from DynamoDB table {}", table_name)
            }
            _ => "DynamoDB operation".to_string(),
        }
    }
}
