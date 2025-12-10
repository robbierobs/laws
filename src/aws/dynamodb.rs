use crate::models::dynamodb::{DynamoDbTable, GlobalSecondaryIndex, KeyAttribute, LocalSecondaryIndex};
use aws_sdk_dynamodb::Client;

pub struct DynamoDbService {
    client: Client,
}

fn format_dynamodb_error<E: std::fmt::Debug>(e: aws_sdk_dynamodb::error::SdkError<E>, action: &str, table_name: &str) -> anyhow::Error {
    let msg = if let Some(svc_err) = e.as_service_error() {
        let code = format!("{:?}", svc_err).split('{').next().unwrap_or("Unknown").trim().to_string();
        format!("DynamoDB {} failed for '{}': {}", action, table_name, code)
    } else {
        let err_str = format!("{}", e);
        if err_str.contains("Unhandled") || err_str.contains("unhandled") {
            format!("DynamoDB {} for '{}': Not supported (LocalStack limitation?)", action, table_name)
        } else {
            format!("DynamoDB {} failed for '{}': {}", action, table_name, err_str)
        }
    };
    anyhow::anyhow!(msg)
}

impl DynamoDbService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_tables(&self) -> anyhow::Result<Vec<DynamoDbTable>> {
        // First, get the list of table names
        let list_response = self.client
            .list_tables()
            .send()
            .await
            .map_err(|e| format_dynamodb_error(e, "list_tables", "all"))?;

        let table_names = list_response.table_names();
        let mut tables = Vec::new();

        // For each table, get detailed information
        for table_name in table_names {
            if let Ok(table) = self.describe_table(table_name).await {
                tables.push(table);
            }
        }

        Ok(tables)
    }

    pub async fn describe_table(&self, table_name: &str) -> anyhow::Result<DynamoDbTable> {
        let response = self.client
            .describe_table()
            .table_name(table_name)
            .send()
            .await
            .map_err(|e| format_dynamodb_error(e, "describe_table", table_name))?;

        let table = response.table().ok_or_else(|| anyhow::anyhow!("No table data returned"))?;

        // Parse key schema
        let mut partition_key: Option<KeyAttribute> = None;
        let mut sort_key: Option<KeyAttribute> = None;

        for key in table.key_schema() {
            let key_type = key.key_type().as_str().to_string();
            let attr_name = key.attribute_name().to_string();
            
            // Find attribute type from attribute definitions
            let attr_type = table.attribute_definitions()
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
        let billing_mode = table.billing_mode_summary()
            .and_then(|b| b.billing_mode())
            .map(|m| m.as_str().to_string());

        // Parse provisioned throughput
        let (read_capacity_units, write_capacity_units) = table.provisioned_throughput()
            .map(|pt| (pt.read_capacity_units(), pt.write_capacity_units()))
            .unwrap_or((None, None));

        // Parse global secondary indexes
        let global_secondary_indexes: Vec<GlobalSecondaryIndex> = table.global_secondary_indexes()
            .iter()
            .map(|gsi| {
                let key_schema: Vec<String> = gsi.key_schema()
                    .iter()
                    .map(|k| format!("{} ({})", k.attribute_name(), k.key_type().as_str()))
                    .collect();
                
                GlobalSecondaryIndex {
                    index_name: gsi.index_name().unwrap_or("-").to_string(),
                    key_schema: key_schema.join(", "),
                    projection_type: gsi.projection().and_then(|p| p.projection_type()).map(|t| t.as_str().to_string()),
                    index_status: gsi.index_status().map(|s| s.as_str().to_string()),
                    item_count: gsi.item_count(),
                }
            })
            .collect();

        // Parse local secondary indexes
        let local_secondary_indexes: Vec<LocalSecondaryIndex> = table.local_secondary_indexes()
            .iter()
            .map(|lsi| {
                let key_schema: Vec<String> = lsi.key_schema()
                    .iter()
                    .map(|k| format!("{} ({})", k.attribute_name(), k.key_type().as_str()))
                    .collect();
                
                LocalSecondaryIndex {
                    index_name: lsi.index_name().unwrap_or("-").to_string(),
                    key_schema: key_schema.join(", "),
                    projection_type: lsi.projection().and_then(|p| p.projection_type()).map(|t| t.as_str().to_string()),
                    item_count: lsi.item_count(),
                }
            })
            .collect();

        // Parse stream specification
        let (stream_enabled, stream_view_type) = table.stream_specification()
            .map(|s| (
                s.stream_enabled(),
                s.stream_view_type().map(|t| t.as_str().to_string())
            ))
            .unwrap_or((false, None));

        Ok(DynamoDbTable {
            table_name: table.table_name().unwrap_or_default().to_string(),
            table_status: table.table_status().map(|s| s.as_str().to_string()).unwrap_or_default(),
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
            table_class: table.table_class_summary().and_then(|c| c.table_class()).map(|c| c.as_str().to_string()),
        })
    }


}
