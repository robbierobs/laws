use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
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

    // Build header text with status
    let status = if app.loading {
        " [Loading...]"
    } else {
        ""
    };
    
    // Build profile/region info
    let profile_str = app.profile.as_deref().unwrap_or("default");
    let aws_info = format!("[{}@{}]", profile_str, app.region);
    
    let header_text = if let Some(ref err) = app.error_message {
        format!("LazyAWS - {:?} {} | Error: {}", app.current_service, aws_info, err)
    } else {
        format!("LazyAWS - {:?} {}{}", app.current_service, aws_info, status)
    };
    
    let header_style = if app.error_message.is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };
    
    let title = Paragraph::new(header_text)
        .style(header_style)
        .block(Block::default().borders(Borders::ALL).title("LazyAWS"));
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
