use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::components::Component;

pub fn render(frame: &mut Frame, app: &mut App) {
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

    // Body split
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sidebar width
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    // Render Sidebar
    app.sidebar.render(frame, body_chunks[0]);

    match app.current_service {
        crate::app::Service::EC2 => {
            crate::ui::screens::ec2::render(frame, body_chunks[1], app);
        }
        crate::app::Service::S3 => {
            crate::ui::screens::s3::render(frame, body_chunks[1], app);
        }
        _ => {
            let content = Paragraph::new("Content goes here")
                .block(Block::default().borders(Borders::ALL).title("Body"));
            frame.render_widget(content, body_chunks[1]);
        }
    }

    // Footer / Action Bar
    crate::ui::components::action_bar::render(frame, chunks[2], app);
}
