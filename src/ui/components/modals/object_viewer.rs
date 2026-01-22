//! S3 object viewer modal with text/hex view toggle

#![allow(clippy::too_many_arguments)]

use super::helpers::{centered_rect, format_hex_dump, get_syntax_style};
use crate::app::states::s3::ViewerMode;
use crate::ui::theme::THEME;
use ratatui::style::Modifier;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render a modal to display S3 object content with text/hex view toggle
pub fn render(
    frame: &mut Frame,
    area: Rect,
    object_key: &str,
    object_path: Option<&str>,
    content: Option<&str>,
    raw_bytes: Option<&[u8]>,
    scroll_offset: u16,
    viewer_mode: ViewerMode,
) {
    let mode_label = viewer_mode.label();
    let title = format!(" {} [{}] ", object_key, mode_label);
    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary));

    let mut lines: Vec<Line> = Vec::new();

    // Show file path
    if let Some(path) = object_path {
        lines.push(Line::from(vec![
            Span::styled("📁 Path: ", Style::default().fg(THEME.secondary)),
            Span::styled(path, Style::default().fg(THEME.muted)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "─".repeat(60),
            Style::default().fg(THEME.border),
        )]));
        lines.push(Line::from(""));
    }

    match viewer_mode {
        ViewerMode::Text => {
            // Show content as text
            if let Some(text_content) = content {
                // Check if content is likely binary
                let is_binary = text_content
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t');

                if is_binary {
                    lines.push(Line::from(vec![Span::styled(
                        "⚠️  Binary file - press Tab to switch to Hex view",
                        Style::default().fg(THEME.warning),
                    )]));
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![Span::styled(
                        "File saved to path shown above.",
                        Style::default().fg(THEME.muted),
                    )]));
                } else {
                    // Syntax highlight based on extension
                    let ext = object_key.rsplit('.').next().unwrap_or("");
                    let highlight_style = get_syntax_style(ext);

                    for (i, line) in text_content.lines().enumerate() {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("{:4} │ ", i + 1),
                                Style::default().fg(THEME.muted),
                            ),
                            Span::styled(line.to_string(), highlight_style),
                        ]));
                    }
                }
            } else {
                lines.push(Line::from(vec![Span::styled(
                    "📦 Binary file or large file - press Tab for Hex view",
                    Style::default().fg(THEME.warning),
                )]));
            }
        }
        ViewerMode::Hex => {
            // Show hex dump
            if let Some(bytes) = raw_bytes {
                let hex_lines = format_hex_dump(bytes, 16);
                for line in hex_lines {
                    lines.push(Line::from(vec![Span::styled(
                        line,
                        Style::default().fg(THEME.fg),
                    )]));
                }
            } else if let Some(text_content) = content {
                let bytes = text_content.as_bytes();
                let hex_lines = format_hex_dump(bytes, 16);
                for line in hex_lines {
                    lines.push(Line::from(vec![Span::styled(
                        line,
                        Style::default().fg(THEME.fg),
                    )]));
                }
            } else {
                lines.push(Line::from(vec![Span::styled(
                    "No content available for hex view",
                    Style::default().fg(THEME.warning),
                )]));
            }
        }
    }

    // Add footer with controls
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "─".repeat(60),
        Style::default().fg(THEME.border),
    )]));
    lines.push(Line::from(vec![
        Span::styled("j/k: Scroll   ", Style::default().fg(THEME.secondary)),
        Span::styled("Tab: Toggle View   ", Style::default().fg(THEME.secondary)),
        Span::styled("Esc/q: Close", Style::default().fg(THEME.secondary)),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll_offset, 0));

    let popup_area = centered_rect(80, 80, area);

    frame.render_widget(Clear, popup_area); // Clear background
    frame.render_widget(paragraph, popup_area);
}
