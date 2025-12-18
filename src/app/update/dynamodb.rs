//! DynamoDB update handlers
//!
//! Handles DynamoDB-specific state mutations and async operations.

use super::super::task_manager::task_keys;
use super::super::{App, DynamoDbViewMode};
use crate::event::{AwsEvent, Event};
use std::collections::HashMap;

impl App {
    pub(super) fn handle_drill_down_dynamodb_table(
        &mut self,
        event_tx: crate::app::EventSender,
    ) {
        let Some(idx) = self.services.dynamodb.list_state.selected() else {
            return;
        };

        let Some(table) = self.services.dynamodb.tables.get(idx) else {
            return;
        };

        let table_name = table.table_name.clone();
        self.services.dynamodb.current_table = Some(table_name.clone());
        self.services.dynamodb.view_mode = DynamoDbViewMode::Items;
        self.services.dynamodb.items.clear();
        self.services.dynamodb.item_list_state.select(None);

        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.dynamodb.clone();
        let tx = event_tx;
        let limit = self.config.max_dynamodb_items as i32;

        let handle = tokio::spawn(async move {
            let service = crate::aws::dynamodb::DynamoDbService::new(client);
            match service.scan_items(&table_name, limit).await {
                Ok(items) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::DynamoDbItemsLoaded(items))))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::DYNAMODB_ITEMS, handle);
    }

    pub(super) fn handle_dynamodb_exit_drilldown(&mut self) {
        self.services.dynamodb.view_mode = DynamoDbViewMode::Tables;
        self.services.dynamodb.current_table = None;
        self.services.dynamodb.items.clear();
        self.services.dynamodb.list_state.select(Some(0));
    }

    pub(super) fn handle_load_dynamodb_items(
        &mut self,
        table_name: String,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let client = clients.dynamodb.clone();
        let tx = event_tx;
        let limit = self.config.max_dynamodb_items as i32;

        let handle = tokio::spawn(async move {
            let service = crate::aws::dynamodb::DynamoDbService::new(client);
            match service.scan_items(&table_name, limit).await {
                Ok(items) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::DynamoDbItemsLoaded(items))))
                        .await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });

        self.tasks.spawn(task_keys::DYNAMODB_ITEMS, handle);
    }

    pub(super) fn handle_delete_dynamodb_item(
        &mut self,
        table_name: String,
        key_attrs: HashMap<String, String>,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        let Some(table) = self
            .services
            .dynamodb
            .tables
            .iter()
            .find(|t| t.table_name == table_name)
        else {
            return;
        };

        let pk_name = table.partition_key.as_ref().map(|k| k.name.clone());
        let sk_name = table.sort_key.as_ref().map(|k| k.name.clone());
        let pk_type = table
            .partition_key
            .as_ref()
            .map(|k| k.attribute_type.clone());
        let sk_type = table.sort_key.as_ref().map(|k| k.attribute_type.clone());

        self.loading = true;
        let client = clients.dynamodb.clone();
        let tx = event_tx;
        let tbl = table_name.clone();

        tokio::spawn(async move {
            use aws_sdk_dynamodb::types::AttributeValue;

            let mut key = HashMap::new();

            if let Some(pk) = pk_name {
                if let Some(val) = key_attrs.get(&pk) {
                    let av = match pk_type.as_deref() {
                        Some("N") => AttributeValue::N(val.clone()),
                        _ => AttributeValue::S(val.clone()),
                    };
                    key.insert(pk, av);
                }
            }

            if let Some(sk) = sk_name {
                if let Some(val) = key_attrs.get(&sk) {
                    let av = match sk_type.as_deref() {
                        Some("N") => AttributeValue::N(val.clone()),
                        _ => AttributeValue::S(val.clone()),
                    };
                    key.insert(sk, av);
                }
            }

            let service = crate::aws::dynamodb::DynamoDbService::new(client);
            match service.delete_item(&tbl, key).await {
                Ok(_) => {
                    let msg = format!("Deleted item from {}", tbl);
                    tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(msg)))).await.ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok();
                }
            }
        });
    }
}
