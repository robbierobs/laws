use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    if let Some(bucket_name) = app.current_bucket.clone() {
        render_objects(frame, list_area, app, &bucket_name);
        if let Some(area) = detail_area {
            render_object_details(frame, area, app);
        }
    } else {
        render_buckets(frame, list_area, app);
        if let Some(area) = detail_area {
            render_bucket_details(frame, area, app);
        }
    }
}

use crate::ui::theme::THEME;

fn render_buckets(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Name", "Creation Date", "Region"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.s3_buckets.iter()
        .filter(|b| {
            if filter.is_empty() { return true; }
            b.name.to_lowercase().contains(&filter)
        })
        .map(|bucket| {
        let cells = vec![
            Cell::from(bucket.name.clone()),
            Cell::from(bucket.creation_date.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(bucket.region.clone().unwrap_or_else(|| "-".to_string())),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("S3 Buckets")
        .title_style(Style::default().fg(THEME.primary))
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(THEME.secondary)
        } else {
            Style::default().fg(THEME.border)
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(40), // Name
            Constraint::Length(30), // Creation Date
            Constraint::Min(10),    // Region
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(t, area, &mut app.s3_list_state);
}

fn render_bucket_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.s3_list_state.selected();
    
    let content = if let Some(idx) = selected {
        if let Some(bucket) = app.s3_buckets.get(idx) {
            build_bucket_detail_lines(bucket, app)
        } else {
            vec![Line::from("No bucket selected")]
        }
    } else {
        vec![Line::from("Select a bucket to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Bucket Details")
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)));
    
    frame.render_widget(paragraph, area);
}

fn build_bucket_detail_lines(bucket: &crate::models::s3::S3Bucket, app: &App) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Bucket Name: ", Style::default().fg(THEME.primary)),
            Span::raw(bucket.name.clone()),
        ]),
        Line::from(vec![
            Span::styled("Creation Date: ", Style::default().fg(THEME.primary)),
            Span::raw(bucket.creation_date.clone().unwrap_or_else(|| "-".to_string())),
        ]),
        Line::from(vec![
            Span::styled("Region: ", Style::default().fg(THEME.primary)),
            Span::raw(bucket.region.clone().unwrap_or_else(|| "-".to_string())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Configuration ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
    ];

    // Check if we have cached details for this bucket
    if let Some(details) = app.s3_bucket_details.get(&bucket.name) {
        if details.loading {
            lines.push(Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(THEME.warning)),
                Span::raw("Loading bucket details..."),
            ]));
        } else {
            // Versioning
            let versioning_text = match details.versioning_enabled {
                Some(true) => Span::styled("Enabled", Style::default().fg(THEME.success)),
                Some(false) => Span::styled("Disabled", Style::default().fg(THEME.error)),
                None => Span::styled("Unknown", Style::default().fg(THEME.muted)),
            };
            lines.push(Line::from(vec![
                Span::styled("Versioning: ", Style::default().fg(THEME.primary)),
                versioning_text,
            ]));

            // Encryption
            let encryption_text = details.encryption.clone()
                .unwrap_or_else(|| "None/Unknown".to_string());
            lines.push(Line::from(vec![
                Span::styled("Encryption: ", Style::default().fg(THEME.primary)),
                Span::raw(encryption_text),
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("─── Statistics ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
            ]));

            // Object count
            let object_count_text = details.object_count
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string());
            lines.push(Line::from(vec![
                Span::styled("Object Count: ", Style::default().fg(THEME.primary)),
                Span::raw(object_count_text),
            ]));

            // Total size
            let total_size_text = details.total_size
                .map(|s| format_size(s))
                .unwrap_or_else(|| "-".to_string());
            lines.push(Line::from(vec![
                Span::styled("Total Size: ", Style::default().fg(THEME.primary)),
                Span::raw(total_size_text),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("ℹ ", Style::default().fg(THEME.primary)),
            Span::raw("Press 'i' to load detailed bucket info"),
        ]));
    }

    lines
}

fn render_objects(frame: &mut Frame, area: Rect, app: &mut App, bucket_name: &str) {
    let header_cells = ["Key", "Size", "Last Modified", "Storage Class"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.s3_objects.iter()
        .filter(|o| {
            if filter.is_empty() { return true; }
            o.key.to_lowercase().contains(&filter)
        })
        .map(|obj| {
        let cells = vec![
            Cell::from(obj.key.clone()),
            Cell::from(format_size(obj.size)),
            Cell::from(obj.last_modified.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(obj.storage_class.clone().unwrap_or_else(|| "STANDARD".to_string())),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Objects in {}", bucket_name))
        .title_style(Style::default().fg(THEME.primary))
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(THEME.secondary)
        } else {
            Style::default().fg(THEME.border)
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(50), // Key
            Constraint::Length(15), // Size
            Constraint::Length(25), // Last Modified
            Constraint::Min(15),    // Storage Class
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(t, area, &mut app.s3_object_list_state);
}

fn render_object_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.s3_object_list_state.selected();
    
    let content = if let Some(idx) = selected {
        if let Some(object) = app.s3_objects.get(idx) {
            build_object_detail_lines(object, app.current_bucket.as_deref())
        } else {
            vec![Line::from("No object selected")]
        }
    } else {
        vec![Line::from("Select an object to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Object Details")
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)));
    
    frame.render_widget(paragraph, area);
}

fn build_object_detail_lines(object: &crate::models::s3::S3Object, bucket_name: Option<&str>) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Key: ", Style::default().fg(THEME.primary)),
            Span::raw(object.key.clone()),
        ]),
    ];

    if let Some(bucket) = bucket_name {
        lines.push(Line::from(vec![
            Span::styled("S3 URI: ", Style::default().fg(THEME.primary)),
            Span::styled(format!("s3://{}/{}", bucket, object.key), Style::default().fg(THEME.selection_fg)),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled("Size: ", Style::default().fg(THEME.primary)),
        Span::raw(format_size(object.size)),
        Span::raw(format!(" ({} bytes)", object.size)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Last Modified: ", Style::default().fg(THEME.primary)),
        Span::raw(object.last_modified.clone().unwrap_or_else(|| "-".to_string())),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Storage Class: ", Style::default().fg(THEME.primary)),
        Span::raw(object.storage_class.clone().unwrap_or_else(|| "STANDARD".to_string())),
    ]));
    
    if let Some(etag) = &object.etag {
        lines.push(Line::from(vec![
            Span::styled("ETag: ", Style::default().fg(THEME.primary)),
            Span::raw(etag.clone()),
        ]));
    }

    lines
}

fn format_size(size: i64) -> String {
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
}
