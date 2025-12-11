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
            Constraint::Length(3),  // Header
            Constraint::Min(1),     // Body
            Constraint::Length(3),  // Action Log / Action Bar
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
    
    // Build profile/region info (cached to avoid repeated allocations)
    let aws_info = app.render_cache.get_aws_info(app.profile.as_deref(), &app.region).to_string();
    
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
            Constraint::Length(app.config.sidebar_width),
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    // Render Sidebar
    app.sidebar.render(frame, body_chunks[0]);

    // Split main content area for list and details
    // In fullscreen mode, detail panel takes the entire area
    let (list_area, detail_area): (Option<Rect>, Option<Rect>) = if app.detail_panel_fullscreen {
        // Fullscreen: only show detail panel
        (None, Some(body_chunks[1]))
    } else if app.detail_panel_visible {
        // Normal: split between list and detail
        let detail_percent = app.config.detail_panel_percent;
        let list_percent = 100u16.saturating_sub(detail_percent);
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(list_percent),
                Constraint::Percentage(detail_percent),
            ])
            .split(body_chunks[1]);
        (Some(content_chunks[0]), Some(content_chunks[1]))
    } else {
        // No detail panel: list takes full area
        (Some(body_chunks[1]), None)
    };

    // Render the current service screen using polymorphic dispatch
    let screen = crate::ui::screens::get_screen(app.current_service);
    screen.render(frame, list_area, detail_area, app);

    // Footer / Action Bar with last action hint
    if app.input_mode == crate::app::InputMode::Filtering {
        let input = Paragraph::new(format!("/{}", app.filter_input))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Filter"));
        frame.render_widget(input, chunks[2]);
    } else {
        // Show action bar with last action embedded
        crate::ui::components::action_bar::render(frame, chunks[2], app);
    }

    // Render action log popup if expanded
    if app.action_log_expanded {
        crate::ui::components::action_log::render_popup(frame, frame.area(), app);
    }
}
