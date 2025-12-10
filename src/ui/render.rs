use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new(format!("LazyAWS - {:?}", app.current_service))
        .block(Block::default().borders(Borders::ALL).title("Header"));
    frame.render_widget(title, chunks[0]);

    let body = Paragraph::new("Content goes here")
        .block(Block::default().borders(Borders::ALL).title("Body"));
    frame.render_widget(body, chunks[1]);

    let footer = Paragraph::new("Press 'q' to quit")
        .block(Block::default().borders(Borders::ALL).title("Footer"));
    frame.render_widget(footer, chunks[2]);
}
