//! Confirmation modal for destructive actions

use super::helpers::centered_rect_fixed;
use crate::ui::theme::THEME;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render a confirmation modal for destructive actions
pub fn render(frame: &mut Frame, area: Rect, action_description: &str) {
    let block = Block::default()
        .title(" Confirm Action ")
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.warning));

    // Use fixed size that ensures content fits
    let popup_width = area.width.clamp(40, 60);
    let popup_height = 9u16; // Fixed height for 5 lines + border + padding

    let popup_area = centered_rect_fixed(popup_width, popup_height, area);
    let inner_area = block.inner(popup_area);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    // Calculate vertical centering - our content is 5 lines
    let content_height = 5u16;
    let vertical_padding = inner_area.height.saturating_sub(content_height) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Min(content_height),
            Constraint::Length(vertical_padding),
        ])
        .split(inner_area);

    let text = vec![
        Line::from(Span::styled(
            "Are you sure you want to perform this action?",
            Style::default().fg(THEME.fg),
        )),
        Line::from(""),
        Line::from(Span::styled(
            action_description,
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(THEME.muted)),
            Span::styled(
                "y",
                Style::default()
                    .fg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to confirm or ", Style::default().fg(THEME.muted)),
            Span::styled(
                "n/Esc",
                Style::default()
                    .fg(THEME.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to cancel", Style::default().fg(THEME.muted)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, chunks[1]);
}
