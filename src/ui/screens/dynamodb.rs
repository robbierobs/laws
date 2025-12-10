use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::dynamodb::DynamoDbTable;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    render_table_list(frame, list_area, app);
    
    if let Some(area) = detail_area {
        render_table_details(frame, area, app);
    }
}

use crate::ui::theme::THEME;

fn render_table_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Table Name", "Status", "Items", "Size", "Partition Key", "Billing"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.dynamodb_tables.iter()
        .filter(|t| {
            if filter.is_empty() { return true; }
            t.table_name.to_lowercase().contains(&filter)
        })
        .map(|table| {
        let status_color = match table.table_status.as_str() {
            "ACTIVE" => THEME.success,
            "CREATING" | "UPDATING" | "DELETING" => THEME.warning,
            _ => THEME.muted,
        };
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

    let block = Block::default()
        .borders(Borders::ALL)
        .title("DynamoDB Tables")
        .title_style(Style::default().fg(THEME.primary))
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(THEME.secondary)
        } else {
            Style::default().fg(THEME.border)
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(30), // Table Name
            Constraint::Length(12), // Status
            Constraint::Length(10), // Items
            Constraint::Length(12), // Size
            Constraint::Length(20), // Partition Key
            Constraint::Min(15),    // Billing
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(t, area, &mut app.dynamodb_list_state);
}

fn render_table_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.dynamodb_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(table) = app.dynamodb_tables.get(idx) {
            build_table_detail_lines(table)
        } else {
            vec![Line::from("No table selected")]
        }
    } else {
        vec![Line::from("Select a DynamoDB table to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Table Details")
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)));
    
    frame.render_widget(paragraph, area);
}

fn build_table_detail_lines(table: &DynamoDbTable) -> Vec<Line<'_>> {
    let status_color = match table.table_status.as_str() {
        "ACTIVE" => THEME.success,
        "CREATING" | "UPDATING" | "DELETING" => THEME.warning,
        _ => THEME.muted,
    };
    
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
