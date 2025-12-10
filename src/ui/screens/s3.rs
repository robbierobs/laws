use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    if let Some(bucket_name) = app.current_bucket.clone() {
        render_objects(frame, area, app, &bucket_name);
    } else {
        render_buckets(frame, area, app);
    }
}

fn render_buckets(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Name", "Creation Date", "Region"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows = app.s3_buckets.iter().map(|bucket| {
        let cells = vec![
            Cell::from(bucket.name.clone()),
            Cell::from(bucket.creation_date.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(bucket.region.clone().unwrap_or_else(|| "-".to_string())),
        ];
        
        Row::new(cells).height(1)
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
    .block(Block::default().borders(Borders::ALL).title("S3 Buckets"))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.s3_list_state);
}

fn render_objects(frame: &mut Frame, area: Rect, app: &mut App, bucket_name: &str) {
    let header_cells = ["Key", "Size", "Last Modified"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows = app.s3_objects.iter().map(|obj| {
        let cells = vec![
            Cell::from(obj.key.clone()),
            Cell::from(format_size(obj.size)),
            Cell::from(obj.last_modified.clone().unwrap_or_else(|| "-".to_string())),
        ];
        
        Row::new(cells).height(1)
    });

    let t = Table::new(
        rows,
        [
            Constraint::Length(50), // Key
            Constraint::Length(15), // Size
            Constraint::Min(20),    // Last Modified
        ]
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!("Objects in {}", bucket_name)))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.s3_object_list_state);
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
