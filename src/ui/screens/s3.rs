use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
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
