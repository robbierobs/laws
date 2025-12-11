use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::{App, DynamoDbViewMode};
use crate::models::dynamodb::DynamoDbTable;
use crate::ui::components::detail_panel::{render_detail_panel, render_detail_panel_with_selection, DetailPanelConfig};
use crate::ui::components::table::render_table;

use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    match app.services.dynamodb.view_mode {
        DynamoDbViewMode::Tables => {
            // Tables list view
            if let Some(area) = list_area {
                render_table_list(frame, area, app);
            }
            if let Some(area) = detail_area {
                render_table_details(frame, area, app);
            }
        }
        DynamoDbViewMode::Items => {
            // Table detail/drill-down view
            render_table_drilldown(frame, list_area, detail_area, app);
        }
    }
}

use crate::models::Filterable;

fn render_table_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.dynamodb.tables.iter()
        .filter(|t| {
            if filter.is_empty() { return true; }
            t.matches_filter(&filter)
        })
        .map(|table| {
            let status_color = table.state_color();
            let pk_str = table.partition_key.as_ref()
                .map(|pk| format!("{} ({})", pk.name, pk.attribute_type))
                .unwrap_or_else(|| "-".to_string());
            let billing = table.billing_mode.clone()
                .unwrap_or_else(|| "PROVISIONED".to_string());
            
            let cells = vec![
                Cell::from(table.table_name.clone()),
                Cell::from(table.table_status.clone()).style(Style::default().fg(status_color)),
                Cell::from(table.item_count.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string())),
                Cell::from(table.format_size()),
                Cell::from(pk_str),
                Cell::from(billing),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["Table Name", "Status", "Items", "Size", "Partition Key", "Billing"],
        &[
            Constraint::Length(30), // Table Name
            Constraint::Length(12), // Status
            Constraint::Length(10), // Items
            Constraint::Length(12), // Size
            Constraint::Length(20), // Partition Key
            Constraint::Min(15),    // Billing
        ],
        "DynamoDB Tables (Enter: view details)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.dynamodb.list_state,
    );
}

fn render_table_details(frame: &mut Frame, area: Rect, app: &App) {
    render_detail_panel_with_selection(
        frame,
        area,
        app.services.dynamodb.selected_table(),
        build_table_detail_lines,
        "Table Details",
        "Select a DynamoDB table to view details (use j/k to navigate)",
        app.detail_panel_fullscreen,
        app.detail_scroll_offset,
    );
}

fn build_table_detail_lines(table: &DynamoDbTable) -> Vec<Line<'_>> {
    let status_color = table.state_color();
    
    let item_count_str = table.item_count.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
    let created_str = table.creation_date_time.clone().unwrap_or_else(|| "-".to_string());
    let table_class_str = table.table_class.clone().unwrap_or_else(|| "STANDARD".to_string());

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Table Name: ", Style::default().fg(THEME.primary)),
            Span::raw(table.table_name.clone()),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(table.table_status.clone(), Style::default().fg(status_color)),
        ]),
        Line::from(vec![
            Span::styled("Items: ", Style::default().fg(THEME.primary)),
            Span::raw(item_count_str),
            Span::raw("   "),
            Span::styled("Size: ", Style::default().fg(THEME.primary)),
            Span::raw(table.format_size()),
            Span::raw("   "),
            Span::styled("Class: ", Style::default().fg(THEME.primary)),
            Span::raw(table_class_str),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(THEME.primary)),
            Span::raw(created_str),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Key Schema ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
    ];

    // Partition Key
    if let Some(pk) = &table.partition_key {
        lines.push(Line::from(vec![
            Span::styled("Partition Key: ", Style::default().fg(THEME.primary)),
            Span::styled(&pk.name, Style::default().fg(THEME.warning)),
            Span::raw(format!(" ({}) ", pk.attribute_type)),
        ]));
    }

    // Sort Key
    if let Some(sk) = &table.sort_key {
        lines.push(Line::from(vec![
            Span::styled("Sort Key: ", Style::default().fg(THEME.primary)),
            Span::styled(&sk.name, Style::default().fg(THEME.warning)),
            Span::raw(format!(" ({}) ", sk.attribute_type)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("─── Billing ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
    ]));

    let billing_mode = table.billing_mode.clone().unwrap_or_else(|| "PROVISIONED".to_string());
    lines.push(Line::from(vec![
        Span::styled("Billing Mode: ", Style::default().fg(THEME.primary)),
        Span::raw(billing_mode.clone()),
    ]));

    if billing_mode == "PROVISIONED" {
        let rcu = table.read_capacity_units.map(|r| r.to_string()).unwrap_or_else(|| "-".to_string());
        let wcu = table.write_capacity_units.map(|w| w.to_string()).unwrap_or_else(|| "-".to_string());
        lines.push(Line::from(vec![
            Span::styled("Read Capacity: ", Style::default().fg(THEME.primary)),
            Span::raw(rcu),
            Span::raw(" RCU   "),
            Span::styled("Write Capacity: ", Style::default().fg(THEME.primary)),
            Span::raw(wcu),
            Span::raw(" WCU"),
        ]));
    }

    // Global Secondary Indexes
    if !table.global_secondary_indexes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("─── Global Secondary Indexes ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]));
        for gsi in &table.global_secondary_indexes {
            let status = gsi.index_status.clone().unwrap_or_else(|| "?".to_string());
            let items = gsi.item_count.map(|c| c.to_string()).unwrap_or_else(|| "?".to_string());
            lines.push(Line::from(vec![
                Span::styled("• ", Style::default().fg(THEME.warning)),
                Span::styled(&gsi.index_name, Style::default().fg(THEME.warning)),
                Span::raw(format!(" [{}] - {} - {} items", status, gsi.key_schema, items)),
            ]));
        }
    }

    // Local Secondary Indexes
    if !table.local_secondary_indexes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("─── Local Secondary Indexes ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]));
        for lsi in &table.local_secondary_indexes {
            let items = lsi.item_count.map(|c| c.to_string()).unwrap_or_else(|| "?".to_string());
            lines.push(Line::from(vec![
                Span::styled("• ", Style::default().fg(THEME.warning)),
                Span::styled(&lsi.index_name, Style::default().fg(THEME.warning)),
                Span::raw(format!(" - {} - {} items", lsi.key_schema, items)),
            ]));
        }
    }

    // Stream
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("─── Stream ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Stream Enabled: ", Style::default().fg(THEME.primary)),
        if table.stream_enabled {
            Span::styled("Yes ✓", Style::default().fg(THEME.success))
        } else {
            Span::styled("No", Style::default().fg(THEME.muted))
        },
    ]));
    if table.stream_enabled {
        if let Some(view_type) = &table.stream_view_type {
            lines.push(Line::from(vec![
                Span::styled("View Type: ", Style::default().fg(THEME.primary)),
                Span::raw(view_type.clone()),
            ]));
        }
    }

    // Deletion Protection
    lines.push(Line::from(vec![
        Span::styled("Deletion Protection: ", Style::default().fg(THEME.primary)),
        if table.deletion_protection {
            Span::styled("Enabled ✓", Style::default().fg(THEME.success))
        } else {
            Span::styled("Disabled", Style::default().fg(THEME.warning))
        },
    ]));

    lines
}

fn render_table_drilldown(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    let table_name = app.services.dynamodb.current_table.as_deref().unwrap_or("Unknown");
    let table = app.services.dynamodb.tables.iter().find(|t| t.table_name == table_name);
    
    // Get key attribute names for display priority
    let pk_name = table.and_then(|t| t.partition_key.as_ref().map(|k| k.name.clone()));
    let sk_name = table.and_then(|t| t.sort_key.as_ref().map(|k| k.name.clone()));
    
    // Build column headers - start with key columns, then others
    let mut column_names: Vec<String> = Vec::new();
    if let Some(pk) = &pk_name {
        column_names.push(pk.clone());
    }
    if let Some(sk) = &sk_name {
        column_names.push(sk.clone());
    }
    
    // Discover other columns from items (up to 4 additional columns)
    for item in app.services.dynamodb.items.iter().take(10) {
        for key in item.attributes.keys() {
            if !column_names.contains(key) && column_names.len() < 6 {
                column_names.push(key.clone());
            }
        }
    }
    
    // Ensure we have at least some columns
    if column_names.is_empty() {
        column_names.push("(no data)".to_string());
    }
    
    // Build rows
    let filter = app.filter_input.to_lowercase();
    let rows: Vec<Row> = app.services.dynamodb.items.iter()
        .filter(|item| {
            if filter.is_empty() { return true; }
            item.matches_filter(&filter)
        })
        .map(|item| {
            let cells: Vec<Cell> = column_names.iter()
                .map(|col| {
                    let val = item.get(col).map(|s| {
                        if s.len() > 40 { format!("{}...", &s[..37]) } else { s.clone() }
                    }).unwrap_or_else(|| "-".to_string());
                    Cell::from(val)
                })
                .collect();
            Row::new(cells).height(1)
        })
        .collect();

    let item_count = app.services.dynamodb.items.len();
    let title = format!(
        "{} - {} items (D: delete, r: refresh, Esc: back)",
        table_name,
        item_count
    );
    
    // Calculate column widths
    let col_count = column_names.len();
    let constraints: Vec<Constraint> = if col_count <= 1 {
        vec![Constraint::Min(20)]
    } else {
        let base_width = 100 / col_count as u16;
        column_names.iter().enumerate().map(|(i, _)| {
            if i < 2 {
                Constraint::Length(25) // Key columns get fixed width
            } else if i == col_count - 1 {
                Constraint::Min(15) // Last column fills remaining
            } else {
                Constraint::Length(base_width.max(15))
            }
        }).collect()
    };

    if let Some(area) = list_area {
        let headers: Vec<&str> = column_names.iter().map(|s| s.as_str()).collect();
        render_table(
            frame,
            area,
            rows,
            &headers,
            &constraints,
            &title,
            matches!(app.focus, crate::app::Focus::Main),
            &mut app.services.dynamodb.item_list_state,
        );
    }
    
    // Render item details in detail area if available
    if let Some(area) = detail_area {
        render_item_details(frame, area, app, &column_names);
    }
}

fn render_item_details(frame: &mut Frame, area: Rect, app: &App, _column_names: &[String]) {
    let selected = app.services.dynamodb.item_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(item) = app.services.dynamodb.items.get(idx) {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("─── Item Attributes ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
            ];
            
            // Sort keys to put partition/sort keys first
            let mut attrs: Vec<(&String, &String)> = item.attributes.iter().collect();
            attrs.sort_by(|a, b| a.0.cmp(b.0));
            
            for (key, value) in attrs {
                let display_value = if value.len() > 60 {
                    format!("{}...", &value[..57])
                } else {
                    value.clone()
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}: ", key), Style::default().fg(THEME.primary)),
                    Span::raw(display_value),
                ]));
            }
            lines
        } else {
            vec![Line::from("No item selected")]
        }
    } else {
        if app.loading {
            vec![Line::from("⏳ Loading items...")]
        } else if app.services.dynamodb.items.is_empty() {
            vec![Line::from("No items in table (or table is empty)")]
        } else {
            vec![Line::from("Select an item to view details (j/k to navigate)")]
        }
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Item Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}
