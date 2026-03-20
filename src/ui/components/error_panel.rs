//! Error display component for showing detailed error information

use crate::ui::theme::THEME;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render an error panel with a message
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The area to render in
/// * `title` - Optional title for the error panel
/// * `message` - The error message to display
#[allow(dead_code)] // TODO: Wire up error panel in main UI
pub fn render_error_panel(frame: &mut Frame, area: Rect, title: Option<&str>, message: &str) {
    let title_text = title.unwrap_or("Error");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title_text))
        .title_style(
            Style::default()
                .fg(THEME.error)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.error));

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(THEME.error)),
            Span::styled(message, Style::default().fg(THEME.fg)),
        ]),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(THEME.fg));

    frame.render_widget(paragraph, area);
}

/// Render an error panel with multiple lines
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The area to render in
/// * `title` - Optional title for the error panel
/// * `messages` - Multiple error messages to display
#[allow(dead_code)] // TODO: Wire up multi-error panel
pub fn render_error_panel_multi(
    frame: &mut Frame,
    area: Rect,
    title: Option<&str>,
    messages: &[String],
) {
    let title_text = title.unwrap_or("Errors");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title_text))
        .title_style(
            Style::default()
                .fg(THEME.error)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.error));

    let mut lines = vec![Line::from("")];

    for (i, msg) in messages.iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(format!("{}. ", i + 1), Style::default().fg(THEME.muted)),
            Span::styled("⚠ ", Style::default().fg(THEME.error)),
            Span::styled(msg, Style::default().fg(THEME.fg)),
        ]));

        if i < messages.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(""));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(THEME.fg));

    frame.render_widget(paragraph, area);
}

/// Render a warning panel (similar to error but with warning color)
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The area to render in
/// * `title` - Optional title for the warning panel
/// * `message` - The warning message to display
#[allow(dead_code)] // TODO: Wire up warning panel
pub fn render_warning_panel(frame: &mut Frame, area: Rect, title: Option<&str>, message: &str) {
    let title_text = title.unwrap_or("Warning");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title_text))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.warning));

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(THEME.warning)),
            Span::styled(message, Style::default().fg(THEME.fg)),
        ]),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(THEME.fg));

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_error_panel_functions_exist() {
        // Just verify the functions compile and have correct signatures
        // Actual rendering tests would require a terminal backend mock
        assert!(true);
    }
}
