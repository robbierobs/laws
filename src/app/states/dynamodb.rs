use ratatui::widgets::TableState;
use crate::models::dynamodb::{DynamoDbItem, DynamoDbTable};
use crate::app::{DynamoDbViewMode, InputResult, Message, ServiceInputHandler, TableStateExt, EventSender};
use crate::app::states::ServiceInternal;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::event::AwsEvent;
use crossterm::event::{KeyCode, KeyEvent};

/// State for DynamoDB service
#[derive(Default)]
pub struct DynamoDbState {
    pub tables: Vec<DynamoDbTable>,
    pub list_state: TableState,
    pub view_mode: DynamoDbViewMode,
    pub current_table: Option<String>,
    pub items: Vec<DynamoDbItem>,
    pub item_list_state: TableState,
}

impl DynamoDbState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're currently viewing items inside a table
    pub fn is_viewing_items(&self) -> bool {
        self.current_table.is_some()
    }

    /// Get the currently selected table, if any
    pub fn selected_table(&self) -> Option<&DynamoDbTable> {
        self.list_state.selected().and_then(|i| self.tables.get(i))
    }

    /// Get the currently selected item, if any
    #[allow(dead_code)] // May be used for detail panel
    pub fn selected_item(&self) -> Option<&DynamoDbItem> {
        self.item_list_state
            .selected()
            .and_then(|i| self.items.get(i))
    }
}

impl crate::app::global_search::Searchable for DynamoDbState {
    fn get_search_results(&self) -> Vec<crate::app::global_search::SearchResult> {
        use crate::app::Service;
        use crate::app::global_search::SearchResult;

        let mut results = Vec::new();
        for table in &self.tables {
            results.push(SearchResult::new(
                Service::DynamoDB,
                "DynamoDB Table",
                &table.table_name,
            ));
        }
        results
    }
}

impl crate::app::global_search::AutoSelectable for DynamoDbState {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        if let Some(idx) = self.tables.iter().position(|t| t.table_name == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

impl ServiceInputHandler for DynamoDbState {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        if self.view_mode == DynamoDbViewMode::Items {
            // In items view
            match key.code {
                KeyCode::Esc | KeyCode::Backspace => {
                    return InputResult::Message(Message::dynamodb_exit_drill_down())
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.item_list_state.nav_down(self.items.len());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.item_list_state.nav_up(self.items.len());
                }
                KeyCode::Char('X') | KeyCode::Delete => {
                    // Delete selected item
                    if let Some(idx) = self.item_list_state.selected() {
                        if let Some(item) = self.items.get(idx) {
                            if let Some(table_name) = &self.current_table {
                                // Build key attributes from item
                                let key_attrs: std::collections::HashMap<String, String> =
                                    item.attributes.clone();
                                return InputResult::Action(Message::dynamodb_delete_item(
                                    table_name.clone(),
                                    key_attrs,
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char('r') => {
                    // Refresh items
                    if let Some(table_name) = &self.current_table {
                        return InputResult::Message(Message::dynamodb_load_items(
                            table_name.clone(),
                        ));
                    }
                }
                _ => {}
            }
        } else {
            // In tables list view
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.list_state.nav_down(self.tables.len());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.list_state.nav_up(self.tables.len());
                }
                KeyCode::Enter => return InputResult::Message(Message::dynamodb_drill_down()),
                _ => {}
            }
        }
        InputResult::None
    }

    fn reset_selection(&mut self) {
        if self.view_mode == DynamoDbViewMode::Items {
            self.item_list_state.select(Some(0));
        } else {
            self.list_state.select(Some(0));
        }
    }

    fn get_copiable_text(&self) -> Option<String> {
        self.selected_table().map(|t| t.table_name.clone())
    }
}

impl ServiceInternal for DynamoDbState {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.dynamodb.clone();
        let handle = spawn_list_task(
            tx,
            move || async move {
                crate::aws::dynamodb::DynamoDbService::new(client)
                    .list_tables()
                    .await
            },
            AwsEvent::DynamoDbTablesLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::DYNAMODB_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.tables.clear();
        self.items.clear();
        self.current_table = None;
        self.view_mode = DynamoDbViewMode::Tables;
        self.list_state.select(Some(0));
        self.item_list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() && !self.tables.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}
