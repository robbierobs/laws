//! ECS service editor modal
//!
//! Modal for modifying ECS service task definition, CPU, memory, and force deployment.

#![allow(clippy::too_many_arguments)]

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::ui::theme::THEME;
use super::helpers::centered_rect_fixed;

/// Render the ECS service editor modal for modifying task definition, CPU, and memory
pub fn render(
    frame: &mut Frame,
    area: Rect,
    service_name: &str,
    task_def: &str,
    cpu: &str,
    memory: &str,
    force_deploy: bool,
    active_field: usize,
) {
    let title = format!(" Edit Service: {} ", service_name);
    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary));

    let popup_area = centered_rect_fixed(70, 16, area);
    let inner_area = block.inner(popup_area);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    // Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Help line
            Constraint::Length(1), // Separator
            Constraint::Length(2), // Task definition field
            Constraint::Length(2), // CPU field
            Constraint::Length(2), // Memory field
            Constraint::Length(2), // Force deploy checkbox
            Constraint::Length(1), // Separator
            Constraint::Length(2), // Submit hint
        ])
        .split(inner_area);

    // Help line
    let help = Line::from(vec![
        Span::styled(
            "Tab/↓↑",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" navigate  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(THEME.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" submit  ", Style::default().fg(THEME.muted)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(THEME.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" cancel", Style::default().fg(THEME.muted)),
    ]);
    frame.render_widget(Paragraph::new(help), chunks[0]);

    // Separator
    let sep = Line::from(Span::styled(
        "─".repeat(chunks[1].width as usize),
        Style::default().fg(THEME.border),
    ));
    frame.render_widget(Paragraph::new(sep.clone()), chunks[1]);

    // Task definition field
    let td_style = if active_field == 0 {
        Style::default()
            .fg(THEME.selection_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.fg)
    };
    let cursor = if active_field == 0 { "▏" } else { "" };
    let td_line = Line::from(vec![
        Span::styled("Task Definition: ", Style::default().fg(THEME.secondary)),
        Span::styled(format!("{}{}", task_def, cursor), td_style),
    ]);
    frame.render_widget(Paragraph::new(td_line), chunks[2]);

    // CPU field
    let cpu_style = if active_field == 1 {
        Style::default()
            .fg(THEME.selection_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.fg)
    };
    let cursor = if active_field == 1 { "▏" } else { "" };
    let cpu_text = if cpu.is_empty() && active_field != 1 {
        "(unchanged)"
    } else {
        cpu
    };
    let cpu_line = Line::from(vec![
        Span::styled("CPU (vCPU units): ", Style::default().fg(THEME.secondary)),
        Span::styled(format!("{}{}", cpu_text, cursor), cpu_style),
    ]);
    frame.render_widget(Paragraph::new(cpu_line), chunks[3]);

    // Memory field
    let mem_style = if active_field == 2 {
        Style::default()
            .fg(THEME.selection_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.fg)
    };
    let cursor = if active_field == 2 { "▏" } else { "" };
    let mem_text = if memory.is_empty() && active_field != 2 {
        "(unchanged)"
    } else {
        memory
    };
    let mem_line = Line::from(vec![
        Span::styled("Memory (MiB):     ", Style::default().fg(THEME.secondary)),
        Span::styled(format!("{}{}", mem_text, cursor), mem_style),
    ]);
    frame.render_widget(Paragraph::new(mem_line), chunks[4]);

    // Force deploy checkbox
    let fd_style = if active_field == 3 {
        Style::default()
            .fg(THEME.selection_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.fg)
    };
    let checkbox = if force_deploy { "[✓]" } else { "[ ]" };
    let fd_line = Line::from(vec![
        Span::styled(checkbox, fd_style),
        Span::styled(" Force new deployment", fd_style),
        if active_field == 3 {
            Span::styled("  (Space to toggle)", Style::default().fg(THEME.muted))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(fd_line), chunks[5]);

    // Separator
    frame.render_widget(Paragraph::new(sep), chunks[6]);

    // Submit hint
    let hint = Line::from(vec![
        Span::styled("💡 ", Style::default()),
        Span::styled(
            "Changing CPU/Memory creates a new task definition revision",
            Style::default()
                .fg(THEME.muted)
                .add_modifier(Modifier::ITALIC),
        ),
    ]);
    frame.render_widget(Paragraph::new(hint), chunks[7]);
}
