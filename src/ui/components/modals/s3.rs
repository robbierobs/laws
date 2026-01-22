//! S3 bucket creation modal

use super::helpers::centered_rect_fixed;
use crate::ui::theme::THEME;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render a modal for S3 bucket creation
pub fn render_bucket_creation(frame: &mut Frame, area: Rect, bucket_name: &str) {
    let block = Block::default()
        .title(" Create S3 Bucket ")
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary));

    let popup_width = area.width.clamp(40, 60);
    let popup_height = 11u16;

    let popup_area = centered_rect_fixed(popup_width, popup_height, area);
    let inner_area = block.inner(popup_area);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Input label
            Constraint::Length(1), // Input field
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Rules
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Controls
        ])
        .split(inner_area);

    // Title
    let title = Paragraph::new("Enter a name for the new bucket:")
        .style(Style::default().fg(THEME.fg))
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Input field with cursor
    let input_text = format!("▸ {}_", bucket_name);
    let input = Paragraph::new(input_text)
        .style(
            Style::default()
                .fg(THEME.selection_fg)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(input, chunks[3]);

    // Naming rules
    let rules = Paragraph::new("Lowercase letters, numbers, hyphens, periods only")
        .style(Style::default().fg(THEME.muted))
        .alignment(Alignment::Center);
    frame.render_widget(rules, chunks[5]);

    // Controls
    let controls = Paragraph::new(vec![Line::from(vec![
        Span::styled("Enter", Style::default().fg(THEME.success)),
        Span::styled(": Create   ", Style::default().fg(THEME.muted)),
        Span::styled("Esc", Style::default().fg(THEME.error)),
        Span::styled(": Cancel", Style::default().fg(THEME.muted)),
    ])])
    .alignment(Alignment::Center);
    frame.render_widget(controls, chunks[7]);
}
