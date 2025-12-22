//! ECS task definition selector modal
//!
//! Modal for browsing and selecting ECS task definitions with detail view.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::vec_init_then_push)] // Readable line-by-line building

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::models::ecs::EcsTaskDefinition;
use crate::ui::theme::THEME;
use super::helpers::centered_rect;

/// Render the ECS task definition selector modal with list and detail panes
pub fn render(
    frame: &mut Frame,
    area: Rect,
    service_name: &str,
    task_defs: &[EcsTaskDefinition],
    selected_index: usize,
    force_deploy: bool,
    detail_scroll: usize,
    loading: bool,
) {
    let title = format!(" Select Task Definition for: {} ", service_name);
    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary));

    // Use 90% of screen
    let popup_area = centered_rect(90, 85, area);
    let inner_area = block.inner(popup_area);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    // Layout: Help | Main content (list + detail) | Force deploy | Status
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Help
            Constraint::Min(10),   // Main content
            Constraint::Length(2), // Force deploy toggle
            Constraint::Length(1), // Status line
        ])
        .split(inner_area);

    // Help line
    let help = Line::from(vec![
        Span::styled(
            "j/k",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" nav  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "l/h",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll detail  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "f",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" toggle force  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(THEME.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" select  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(THEME.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" cancel", Style::default().fg(THEME.muted)),
    ]);
    frame.render_widget(Paragraph::new(help), main_chunks[0]);

    // Split main content into list (left) and details (right)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // List
            Constraint::Percentage(65), // Details
        ])
        .split(main_chunks[1]);

    // Render task definition list
    render_task_list(frame, content_chunks[0], task_defs, selected_index, loading);

    // Render details pane
    render_detail_pane(frame, content_chunks[1], task_defs, selected_index, detail_scroll);

    // Force deploy toggle
    let checkbox = if force_deploy { "[✓]" } else { "[ ]" };
    let fd_style = Style::default().fg(if force_deploy {
        THEME.warning
    } else {
        THEME.fg
    });
    let force_line = Line::from(vec![
        Span::styled(checkbox, fd_style.add_modifier(Modifier::BOLD)),
        Span::styled(" Force new deployment", fd_style),
        Span::styled("  (Press ", Style::default().fg(THEME.muted)),
        Span::styled(
            "f",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to toggle)", Style::default().fg(THEME.muted)),
    ]);
    frame.render_widget(Paragraph::new(force_line), main_chunks[2]);

    // Status line
    let status = if loading {
        Line::from(Span::styled("Loading...", Style::default().fg(THEME.muted)))
    } else {
        Line::from(vec![
            Span::styled(
                format!("{} task definitions", task_defs.len()),
                Style::default().fg(THEME.muted),
            ),
            if !task_defs.is_empty() {
                Span::styled(
                    format!(" | Selected: {}/{}", selected_index + 1, task_defs.len()),
                    Style::default().fg(THEME.muted),
                )
            } else {
                Span::raw("")
            },
        ])
    };
    frame.render_widget(
        Paragraph::new(status).alignment(Alignment::Right),
        main_chunks[3],
    );
}

fn render_task_list(
    frame: &mut Frame,
    area: Rect,
    task_defs: &[EcsTaskDefinition],
    selected_index: usize,
    loading: bool,
) {
    let list_block = Block::default()
        .title(" Task Definitions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.border));
    let list_inner = list_block.inner(area);
    frame.render_widget(list_block, area);

    if loading {
        let loading_text = Paragraph::new("Loading task definitions...")
            .style(Style::default().fg(THEME.muted))
            .alignment(Alignment::Center);
        frame.render_widget(loading_text, list_inner);
        return;
    }

    if task_defs.is_empty() {
        let no_items = Paragraph::new("No task definitions found")
            .style(Style::default().fg(THEME.muted))
            .alignment(Alignment::Center);
        frame.render_widget(no_items, list_inner);
        return;
    }

    let list_height = list_inner.height as usize;
    let total_items = task_defs.len();

    // Calculate scroll offset
    let scroll_offset = if total_items <= list_height {
        0
    } else if selected_index < list_height / 2 {
        0
    } else if selected_index >= total_items.saturating_sub(list_height / 2) {
        total_items.saturating_sub(list_height)
    } else {
        selected_index.saturating_sub(list_height / 2)
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, td) in task_defs
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(list_height)
    {
        let is_selected = i == selected_index;
        let prefix = if is_selected { "▶ " } else { "  " };
        let display = td.short_name();

        let style = if is_selected {
            Style::default()
                .fg(THEME.selection_fg)
                .add_modifier(Modifier::BOLD)
        } else if td.status == "ACTIVE" {
            Style::default().fg(THEME.success)
        } else {
            Style::default().fg(THEME.muted)
        };

        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, display),
            style,
        )));
    }
    frame.render_widget(Paragraph::new(lines), list_inner);
}

fn render_detail_pane(
    frame: &mut Frame,
    area: Rect,
    task_defs: &[EcsTaskDefinition],
    selected_index: usize,
    detail_scroll: usize,
) {
    let detail_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.border));
    let detail_inner = detail_block.inner(area);
    frame.render_widget(detail_block, area);

    if let Some(td) = task_defs.get(selected_index) {
        let detail_lines = build_task_def_detail_lines(td);
        let paragraph = Paragraph::new(detail_lines).scroll((detail_scroll as u16, 0));
        frame.render_widget(paragraph, detail_inner);
    }
}

/// Build detail lines for a task definition
fn build_task_def_detail_lines(td: &EcsTaskDefinition) -> Vec<Line<'_>> {
    let mut detail_lines: Vec<Line> = Vec::new();

    detail_lines.push(Line::from(vec![
        Span::styled("Family: ", Style::default().fg(THEME.secondary)),
        Span::styled(&td.family, Style::default().fg(THEME.fg)),
    ]));
    detail_lines.push(Line::from(vec![
        Span::styled("Revision: ", Style::default().fg(THEME.secondary)),
        Span::styled(td.revision.to_string(), Style::default().fg(THEME.fg)),
    ]));
    detail_lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(THEME.secondary)),
        Span::styled(&td.status, Style::default().fg(td.status_color())),
    ]));
    detail_lines.push(Line::from(""));

    // CPU and Memory
    detail_lines.push(Line::from(vec![
        Span::styled("CPU: ", Style::default().fg(THEME.secondary)),
        Span::styled(
            td.cpu.as_deref().unwrap_or("N/A"),
            Style::default().fg(THEME.fg),
        ),
        Span::styled("  Memory: ", Style::default().fg(THEME.secondary)),
        Span::styled(
            td.memory.as_deref().unwrap_or("N/A"),
            Style::default().fg(THEME.fg),
        ),
    ]));

    // Network mode
    if let Some(network_mode) = &td.network_mode {
        detail_lines.push(Line::from(vec![
            Span::styled("Network Mode: ", Style::default().fg(THEME.secondary)),
            Span::styled(network_mode, Style::default().fg(THEME.fg)),
        ]));
    }

    // Requires compatibilities
    if !td.requires_compatibilities.is_empty() {
        detail_lines.push(Line::from(vec![
            Span::styled("Compatibilities: ", Style::default().fg(THEME.secondary)),
            Span::styled(
                td.requires_compatibilities.join(", "),
                Style::default().fg(THEME.fg),
            ),
        ]));
    }

    detail_lines.push(Line::from(""));
    detail_lines.push(Line::from(Span::styled(
        "─ Containers ─",
        Style::default().fg(THEME.border),
    )));

    // Container definitions
    for container in &td.container_definitions {
        detail_lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(THEME.muted)),
            Span::styled(&container.name, Style::default().fg(THEME.primary)),
            if container.essential {
                Span::styled(" (essential)", Style::default().fg(THEME.warning))
            } else {
                Span::raw("")
            },
        ]));
        if let Some(image) = &container.image {
            // Truncate long image names
            let display_image = if image.len() > 50 {
                format!("...{}", &image[image.len() - 47..])
            } else {
                image.clone()
            };
            detail_lines.push(Line::from(vec![
                Span::styled("    Image: ", Style::default().fg(THEME.muted)),
                Span::styled(display_image, Style::default().fg(THEME.fg)),
            ]));
        }
        if container.cpu > 0 || container.memory.is_some() {
            detail_lines.push(Line::from(vec![
                Span::styled("    Resources: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    format!(
                        "CPU {} | Mem {}",
                        container.cpu,
                        container
                            .memory
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "N/A".to_string())
                    ),
                    Style::default().fg(THEME.fg),
                ),
            ]));
        }
        // Port mappings
        if !container.port_mappings.is_empty() {
            let ports: Vec<String> = container
                .port_mappings
                .iter()
                .filter_map(|pm| pm.container_port.map(|p| p.to_string()))
                .collect();
            if !ports.is_empty() {
                detail_lines.push(Line::from(vec![
                    Span::styled("    Ports: ", Style::default().fg(THEME.muted)),
                    Span::styled(ports.join(", "), Style::default().fg(THEME.fg)),
                ]));
            }
        }
    }

    // Roles
    if td.task_role_arn.is_some() || td.execution_role_arn.is_some() {
        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(Span::styled(
            "─ Roles ─",
            Style::default().fg(THEME.border),
        )));
        if let Some(role) = &td.task_role_arn {
            let short_role = role.split('/').next_back().unwrap_or(role);
            detail_lines.push(Line::from(vec![
                Span::styled("  Task Role: ", Style::default().fg(THEME.muted)),
                Span::styled(short_role, Style::default().fg(THEME.fg)),
            ]));
        }
        if let Some(role) = &td.execution_role_arn {
            let short_role = role.split('/').next_back().unwrap_or(role);
            detail_lines.push(Line::from(vec![
                Span::styled("  Exec Role: ", Style::default().fg(THEME.muted)),
                Span::styled(short_role, Style::default().fg(THEME.fg)),
            ]));
        }
    }

    detail_lines
}
