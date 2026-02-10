use crate::error::AppResult;
use crate::models::dynamodb::{
    DynamoDbTable, GlobalSecondaryIndex, KeyAttribute, LocalSecondaryIndex,
};
use crate::utils::error::format_sdk_error;
use aws_sdk_dynamodb::Client;

// Use macro to generate struct and constructor
crate::aws_service_struct!(DynamoDbService, Client);

impl DynamoDbService {

    pub async fn list_tables(&self) -> AppResult<Vec<DynamoDbTable>> {
        // First, get all table names with pagination
        let mut table_names = Vec::new();
        let mut last_evaluated: Option<String> = None;

        loop {
            let mut request = self.client.list_tables();
            if let Some(token) = last_evaluated {
                request = request.exclusive_start_table_name(token);
            }

            let list_response = request
                .send()
                .await
                .map_err(|e| format_sdk_error("DynamoDB", "list_tables", "all", e))?;

            table_names.extend(list_response.table_names().iter().cloned());

            last_evaluated = list_response.last_evaluated_table_name().map(|s| s.to_string());
            if last_evaluated.is_none() {
                break;
            }
        }

        let mut tables = Vec::new();

        // For each table, get detailed information
        for table_name in &table_names {
            match self.describe_table(table_name).await {
                Ok(table) => tables.push(table),
                Err(e) => {
                    tracing::warn!("Failed to describe table {}: {}", table_name, e);
                }
            }
        }

        Ok(tables)
    }

    pub async fn describe_table(&self, table_name: &str) -> AppResult<DynamoDbTable> {
        let response = self
            .client
            .describe_table()
            .table_name(table_name)
            .send()
            .await
            .map_err(|e| format_sdk_error("DynamoDB", "describe_table", table_name, e))?;

        let table = response
            .table()
            .ok_or_else(|| crate::error::AppError::not_found("Table", table_name))?;

        // Parse key schema
        let mut partition_key: Option<KeyAttribute> = None;
        let mut sort_key: Option<KeyAttribute> = None;

        for key in table.key_schema() {
            let key_type = key.key_type().as_str().to_string();
            let attr_name = key.attribute_name().to_string();

            // Find attribute type from attribute definitions
            let attr_type = table
                .attribute_definitions()
                .iter()
                .find(|a| a.attribute_name() == attr_name.as_str())
                .map(|a| a.attribute_type().as_str().to_string())
                .unwrap_or_else(|| "S".to_string());

            let key_attr = KeyAttribute {
                name: attr_name,
                key_type: key_type.clone(),
                attribute_type: attr_type,
            };

            if key_type == "HASH" {
                partition_key = Some(key_attr);
            } else if key_type == "RANGE" {
                sort_key = Some(key_attr);
            }
        }

        // Parse billing mode
        let billing_mode = table
            .billing_mode_summary()
            .and_then(|b| b.billing_mode())
            .map(|m| m.as_str().to_string());

        // Parse provisioned throughput
        let (read_capacity_units, write_capacity_units) = table
            .provisioned_throughput()
            .map(|pt| (pt.read_capacity_units(), pt.write_capacity_units()))
            .unwrap_or((None, None));

        // Parse global secondary indexes
        let global_secondary_indexes: Vec<GlobalSecondaryIndex> = table
            .global_secondary_indexes()
            .iter()
            .map(|gsi| {
                let key_schema: Vec<String> = gsi
                    .key_schema()
                    .iter()
                    .map(|k| format!("{} ({})", k.attribute_name(), k.key_type().as_str()))
                    .collect();

                GlobalSecondaryIndex {
                    index_name: gsi.index_name().unwrap_or("-").to_string(),
                    key_schema: key_schema.join(", "),
                    projection_type: gsi
                        .projection()
                        .and_then(|p| p.projection_type())
                        .map(|t| t.as_str().to_string()),
                    index_status: gsi.index_status().map(|s| s.as_str().to_string()),
                    item_count: gsi.item_count(),
                }
            })
            .collect();

        // Parse local secondary indexes
        let local_secondary_indexes: Vec<LocalSecondaryIndex> = table
            .local_secondary_indexes()
            .iter()
            .map(|lsi| {
                let key_schema: Vec<String> = lsi
                    .key_schema()
                    .iter()
                    .map(|k| format!("{} ({})", k.attribute_name(), k.key_type().as_str()))
                    .collect();

                LocalSecondaryIndex {
                    index_name: lsi.index_name().unwrap_or("-").to_string(),
                    key_schema: key_schema.join(", "),
                    projection_type: lsi
                        .projection()
                        .and_then(|p| p.projection_type())
                        .map(|t| t.as_str().to_string()),
                    item_count: lsi.item_count(),
                }
            })
            .collect();

        // Parse stream specification
        let (stream_enabled, stream_view_type) = table
            .stream_specification()
            .map(|s| {
                (
                    s.stream_enabled(),
                    s.stream_view_type().map(|t| t.as_str().to_string()),
                )
            })
            .unwrap_or((false, None));

        Ok(DynamoDbTable {
            table_name: table.table_name().unwrap_or_default().to_string(),
            table_status: table
                .table_status()
                .map(|s| s.as_str().to_string())
                .unwrap_or_default(),
            item_count: table.item_count(),
            table_size_bytes: table.table_size_bytes(),
            creation_date_time: table.creation_date_time().map(|t| t.to_string()),
            partition_key,
            sort_key,
            billing_mode,
            read_capacity_units,
            write_capacity_units,
            global_secondary_indexes,
            local_secondary_indexes,
            stream_enabled,
            stream_view_type,
            table_arn: table.table_arn().map(|s| s.to_string()),
            deletion_protection: table.deletion_protection_enabled().unwrap_or(false),
            table_class: table
                .table_class_summary()
                .and_then(|c| c.table_class())
                .map(|c| c.as_str().to_string()),
        })
    }

    /// Scan items from a table (limited to first N items)
    pub async fn scan_items(
        &self,
        table_name: &str,
        limit: i32,
    ) -> AppResult<Vec<crate::models::dynamodb::DynamoDbItem>> {
        use crate::models::dynamodb::DynamoDbItem;

        let response = self
            .client
            .scan()
            .table_name(table_name)
            .limit(limit)
            .send()
            .await
            .map_err(|e| format_sdk_error("DynamoDB", "scan", table_name, e))?;

        let items: Vec<DynamoDbItem> = response
            .items()
            .iter()
            .map(|item| {
                let attributes: std::collections::HashMap<String, String> = item
                    .iter()
                    .map(|(k, v)| (k.clone(), format_attribute_value(v)))
                    .collect();
                DynamoDbItem { attributes }
            })
            .collect();

        Ok(items)
    }

    /// Delete an item from a table
    pub async fn delete_item(
        &self,
        table_name: &str,
        key: std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>,
    ) -> AppResult<()> {
        self.client
            .delete_item()
            .table_name(table_name)
            .set_key(Some(key))
            .send()
            .await
            .map_err(|e| format_sdk_error("DynamoDB", "delete_item", table_name, e))?;

        Ok(())
    }
}

/// Format an AttributeValue to a displayable string
fn format_attribute_value(av: &aws_sdk_dynamodb::types::AttributeValue) -> String {
    match av {
        aws_sdk_dynamodb::types::AttributeValue::S(s) => s.clone(),
        aws_sdk_dynamodb::types::AttributeValue::N(n) => n.clone(),
        aws_sdk_dynamodb::types::AttributeValue::B(b) => {
            format!("<binary {} bytes>", b.as_ref().len())
        }
        aws_sdk_dynamodb::types::AttributeValue::Bool(b) => b.to_string(),
        aws_sdk_dynamodb::types::AttributeValue::Null(_) => "NULL".to_string(),
        aws_sdk_dynamodb::types::AttributeValue::L(list) => {
            format!(
                "[{}]",
                list.iter()
                    .map(format_attribute_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        aws_sdk_dynamodb::types::AttributeValue::M(map) => {
            let entries: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_attribute_value(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        aws_sdk_dynamodb::types::AttributeValue::Ss(ss) => format!("[{}]", ss.join(", ")),
        aws_sdk_dynamodb::types::AttributeValue::Ns(ns) => format!("[{}]", ns.join(", ")),
        aws_sdk_dynamodb::types::AttributeValue::Bs(_) => "<binary set>".to_string(),
        _ => "<unknown>".to_string(),
    }
}

impl crate::aws::traits::AwsService<DynamoDbTable> for DynamoDbService {
    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AppResult<Vec<DynamoDbTable>>> + Send + 'a>,
    > {
        Box::pin(self.list_tables())
    }
}
