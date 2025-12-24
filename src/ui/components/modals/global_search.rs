//! Global search modal for searching all resources

#![allow(clippy::too_many_arguments)]
#![allow(clippy::if_same_then_else)] // Scroll offset calculations intentionally follow same pattern

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::global_search::SearchResult;
use crate::app::Service;
use crate::ui::theme::THEME;
use super::helpers::centered_rect;

/// Render the global search modal for searching all resources
pub fn render(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    results: &[SearchResult],
    selected_index: usize,
    tag_mode: bool,
    loading: bool,
    spinner: &str,
) {
    let block = Block::default()
        .title(" 🔍 Global Search ")
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary));

    let popup_area = centered_rect(70, 75, area);
    let inner_area = block.inner(popup_area);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    // Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Help line
            Constraint::Length(2), // Search input
            Constraint::Length(1), // Mode indicator
            Constraint::Length(1), // Separator
            Constraint::Min(5),    // Results list
            Constraint::Length(1), // Status line
        ])
        .split(inner_area);

    // Help line
    let help = Line::from(vec![
        Span::styled(
            "↑/↓",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" nav  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(THEME.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" go to  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(THEME.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" close  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "tag:",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" tag search", Style::default().fg(THEME.muted)),
    ]);
    frame.render_widget(Paragraph::new(help), chunks[0]);

    // Search input
    let search_style = Style::default()
        .fg(THEME.primary)
        .add_modifier(Modifier::BOLD);
    let search_text = format!("Search: {}▏", query);
    let search_line = Paragraph::new(search_text).style(search_style);
    frame.render_widget(search_line, chunks[1]);

    // Mode indicator
    let mode_text = if tag_mode {
        Line::from(Span::styled(
            "🏷️  Tag search mode",
            Style::default().fg(THEME.warning),
        ))
    } else {
        Line::from(Span::styled(
            "Search by ID, name, or type",
            Style::default().fg(THEME.muted),
        ))
    };
    frame.render_widget(Paragraph::new(mode_text), chunks[2]);

    // Separator
    let separator = Line::from(Span::styled(
        "─".repeat(chunks[3].width as usize),
        Style::default().fg(THEME.border),
    ));
    frame.render_widget(Paragraph::new(separator), chunks[3]);

    // Results list
    let list_height = chunks[4].height as usize;
    let total_results = results.len();

    if total_results == 0 {
        let message = if loading {
            format!("{} Fetching resources...", spinner)
        } else if query.is_empty() {
            "Type to search all loaded resources...".to_string()
        } else {
            "No matching resources found".to_string()
        };
        let style = if loading {
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(THEME.muted)
        };
        let no_results = Paragraph::new(message)
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(no_results, chunks[4]);
    } else {
        // Calculate scroll offset to keep selection visible
        let scroll_offset = if total_results <= list_height {
            0
        } else if selected_index < list_height / 2 {
            0
        } else if selected_index >= total_results.saturating_sub(list_height / 2) {
            total_results.saturating_sub(list_height)
        } else {
            selected_index.saturating_sub(list_height / 2)
        };

        // Build result lines
        let mut lines: Vec<Line> = Vec::new();
        for (i, result) in results
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(list_height)
        {
            let is_selected = i == selected_index;
            let prefix = if is_selected { "▶ " } else { "  " };

            // Service icon/emoji
            let service_icon = get_service_icon(result.service);

            let style = if is_selected {
                Style::default()
                    .fg(THEME.selection_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.fg)
            };

            let type_style = if is_selected {
                Style::default().fg(THEME.selection_fg)
            } else {
                Style::default().fg(THEME.secondary)
            };

            let mut spans = vec![
                Span::styled(prefix, style),
                Span::styled(format!("{} ", service_icon), style),
                Span::styled(&result.primary_id, style),
            ];

            // Add resource type in brackets
            spans.push(Span::styled(
                format!(" [{}]", result.resource_type),
                type_style,
            ));

            // Add secondary info if present
            if let Some(ref info) = result.secondary_info {
                let info_style = Style::default().fg(THEME.muted);
                spans.push(Span::styled(format!(" - {}", info), info_style));
            }

            lines.push(Line::from(spans));
        }

        let list = Paragraph::new(lines);
        frame.render_widget(list, chunks[4]);
    }

    // Status line
    let status = if loading {
        Line::from(vec![
            Span::styled(
                format!("{} Fetching resources...", spinner),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            if total_results > 0 {
                Span::styled(
                    format!(" | {} results so far", total_results),
                    Style::default().fg(THEME.muted),
                )
            } else {
                Span::raw("")
            },
        ])
    } else if total_results > 0 {
        Line::from(vec![
            Span::styled(
                format!("{} results", total_results),
                Style::default().fg(THEME.muted),
            ),
            Span::styled(
                format!(" | {}/{}", selected_index + 1, total_results),
                Style::default().fg(THEME.muted),
            ),
        ])
    } else {
        Line::from("")
    };
    frame.render_widget(
        Paragraph::new(status).alignment(Alignment::Right),
        chunks[5],
    );
}

/// Get an emoji icon for a service
fn get_service_icon(service: Service) -> &'static str {
    match service {
        Service::EC2 => "💻",
        Service::S3 => "📦",
        Service::RDS => "🗄️",
        Service::DynamoDB => "📊",
        Service::Lambda => "λ ",
        Service::VPC => "🌐",
        Service::IAM => "👤",
        Service::Backup => "💾",
        Service::CloudTrail => "📜",
        Service::SecretsManager => "🔐",
        Service::ECS => "🐳",
        Service::ECR => "📷",
    }
}
