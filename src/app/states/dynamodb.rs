use ratatui::widgets::TableState;
use crate::models::dynamodb::{DynamoDbItem, DynamoDbTable};
use crate::app::{DynamoDbViewMode, InputResult, Message, ServiceInputHandler, TableStateExt};
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
    pub fn selected_item(&self) -> Option<&DynamoDbItem> {
        self.item_list_state
            .selected()
            .and_then(|i| self.items.get(i))
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
        if self.view_mode == DynamoDbViewMode::Items {
            // No easy way to copy items yet as per existing code
            // "For items, we could format as JSON, but for now let's stick to IDs/names if possible"
            // "Detailed item copy is better handled in a specific view"
            // "self.services.dynamodb.selected_table().map(|t| t.table_name.clone())"
            
            // Wait, existing code just returns table name even in items view?
            // "self.services.dynamodb.selected_table().map(|t| t.table_name.clone())"
            // Yes, because `selected_table` depends on `list_state`, which might still be selected.
            // But let's check `src/app/input.rs` again.
            // It has:
            // Service::DynamoDB => {
            //     self.services.dynamodb.selected_table().map(|t| t.table_name.clone())
            // }
            // So it actually always copies the table name regardless of view mode.
            self.selected_table().map(|t| t.table_name.clone())
        } else {
             self.selected_table().map(|t| t.table_name.clone())
        }
    }
}
