use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::components::Component;

use crate::ui::theme::THEME;

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
    } else if app.detail_loading {
        " [Fetching details...]"
    } else {
        ""
    };
    
    // Build profile/region info
    let profile_str = app.profile.as_deref().unwrap_or("default");
    let aws_info = format!("[{}@{}]", profile_str, app.region);
    
    let filter_status = if !app.filter_input.is_empty() {
        format!(" [Filter: {}]", app.filter_input)
    } else {
        String::new()
    };

    let header_text = if let Some(ref err) = app.error_message {
        format!("LazyAWS - {:?} {} | Error: {}", app.current_service, aws_info, err)
    } else {
        let read_only_status = if app.read_only { " [READ-ONLY]" } else { "" };
        format!("LazyAWS - {:?} {}{}{}{}", app.current_service, aws_info, status, filter_status, read_only_status)
    };
    


    // If read-only, we want a more prominent warning. 
    // Let's make the title background yellow if read-only, or just the text?
    // User asked for "yellow background so it stands out".
    let (header_style, block_style) = if app.read_only {
        (
            Style::default().fg(Color::Black).bg(THEME.warning).add_modifier(ratatui::style::Modifier::BOLD),
            Style::default().fg(THEME.warning)
        )
    } else if app.error_message.is_some() {
        (
            Style::default().fg(THEME.error),
            Style::default().fg(THEME.error)
        )
    } else {
        (
            Style::default().fg(THEME.primary),
            Style::default().fg(THEME.border)
        )
    };
    
    let title = Paragraph::new(header_text)
        .style(header_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(block_style)
            .title("LazyAWS")
            .title_style(Style::default().fg(if app.read_only { Color::Black } else { THEME.secondary })));
    frame.render_widget(title, chunks[0]);

    // Body split - sidebar on left, main content on right
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sidebar width
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    // Render Sidebar
    app.sidebar.render(frame, body_chunks[0]);

    // Split main content area for list and details if panel is visible
    let (list_area, detail_area): (Rect, Option<Rect>) = if app.detail_panel_visible {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),  // List takes 60%
                Constraint::Percentage(40),  // Detail panel takes 40%
            ])
            .split(body_chunks[1]);
        (content_chunks[0], Some(content_chunks[1]))
    } else {
        (body_chunks[1], None)
    };

    // Render the current service screen
    match app.current_service {
        crate::app::Service::EC2 => {
            crate::ui::screens::ec2::render(frame, list_area, detail_area, app);
        }
        crate::app::Service::S3 => {
            crate::ui::screens::s3::render(frame, list_area, detail_area, app);
        }
        crate::app::Service::RDS => {
            crate::ui::screens::rds::render(frame, list_area, detail_area, app);
        }
        crate::app::Service::DynamoDB => {
            crate::ui::screens::dynamodb::render(frame, list_area, detail_area, app);
        }
        crate::app::Service::Lambda => {
            crate::ui::screens::lambda::render(frame, list_area, detail_area, app);
        }
        crate::app::Service::VPC => {
            crate::ui::screens::vpc::render(frame, list_area, detail_area, app);
        }
        crate::app::Service::IAM => {
            crate::ui::screens::iam::render(frame, list_area, detail_area, app);
        }
        crate::app::Service::Backup => {
            crate::ui::screens::backup::render(frame, list_area, detail_area, app);
        }
        crate::app::Service::CloudTrail => {
            crate::ui::screens::cloudtrail::render(frame, list_area, detail_area, app);
        }
    }

    // Footer / Action Bar / Filter Input
    if app.input_mode == crate::app::InputMode::Filtering {
        let input = Paragraph::new(format!("/{}", app.filter_input))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Filter"));
        frame.render_widget(input, chunks[2]);
    } else {
        crate::ui::components::action_bar::render(frame, chunks[2], app);
    }
}

