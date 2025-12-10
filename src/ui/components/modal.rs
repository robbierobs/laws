use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::ui::theme::THEME;

pub fn render_confirmation_modal(frame: &mut Frame, area: Rect, action_description: &str) {
    let block = Block::default()
        .title(" Confirm Action ")
        .title_style(Style::default().fg(THEME.warning).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.warning));

    let text = vec![
        Line::from(vec![
            Span::styled("Are you sure you want to perform this action?", Style::default().fg(THEME.fg)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(action_description, Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(THEME.muted)),
            Span::styled("Enter", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm or ", Style::default().fg(THEME.muted)),
            Span::styled("Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel.", Style::default().fg(THEME.muted)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    let area = centered_rect(60, 20, area);
    
    frame.render_widget(Clear, area); // Clear background
    frame.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
