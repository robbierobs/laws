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
            Span::styled("y", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm or ", Style::default().fg(THEME.muted)),
            Span::styled("n/Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
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

/// Render a modal to display S3 object content
pub fn render_object_viewer_modal(
    frame: &mut Frame,
    area: Rect,
    object_key: &str,
    object_path: Option<&str>,
    content: Option<&str>,
    scroll_offset: u16,
) {
    let title = format!(" {} ", object_key);
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
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
        lines.push(Line::from(vec![
            Span::styled("─".repeat(60), Style::default().fg(THEME.border)),
        ]));
        lines.push(Line::from(""));
    }

    // Show content
    if let Some(text_content) = content {
        // Check if content is likely binary
        let is_binary = text_content.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t');
        
        if is_binary {
            lines.push(Line::from(vec![
                Span::styled("⚠️  Binary file - cannot display content", Style::default().fg(THEME.warning)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("File saved to path shown above.", Style::default().fg(THEME.muted)),
            ]));
        } else {
            // Add line numbers and content
            for (i, line) in text_content.lines().enumerate() {
                lines.push(Line::from(vec![
                    Span::styled(format!("{:4} │ ", i + 1), Style::default().fg(THEME.muted)),
                    Span::raw(line.to_string()),
                ]));
            }
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("📦 Binary file or large file - cannot display content inline", Style::default().fg(THEME.warning)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("File has been saved to the path shown above.", Style::default().fg(THEME.muted)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Use an external viewer to open it.", Style::default().fg(THEME.muted)),
        ]));
    }

    // Add footer with controls
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("─".repeat(60), Style::default().fg(THEME.border)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("j/k: Scroll   ", Style::default().fg(THEME.secondary)),
        Span::styled("Esc/q: Close", Style::default().fg(THEME.secondary)),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll_offset, 0));

    let popup_area = centered_rect(80, 80, area);
    
    frame.render_widget(Clear, popup_area); // Clear background
    frame.render_widget(paragraph, popup_area);
}

/// Render the profile switcher modal
pub fn render_profile_switcher_modal(
    frame: &mut Frame,
    area: Rect,
    profiles: &[String],
    selected_index: usize,
    current_profile: Option<&str>,
    read_only: bool,
) {
    let block = Block::default()
        .title(" Select AWS Profile (Shift+P) ")
        .title_style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary));

    let mut lines: Vec<Line> = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled("Use ", Style::default().fg(THEME.muted)),
        Span::styled("j/k", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" to navigate, ", Style::default().fg(THEME.muted)),
        Span::styled("Enter", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
        Span::styled(" to select, ", Style::default().fg(THEME.muted)),
        Span::styled("Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
        Span::styled(" to cancel", Style::default().fg(THEME.muted)),
    ]));
    lines.push(Line::from(""));
    
    // Read-only toggle checkbox
    let checkbox = if read_only { "[✓]" } else { "[ ]" };
    let ro_style = if read_only {
        Style::default().fg(THEME.warning).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.muted)
    };
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(checkbox, ro_style),
        Span::styled(" Read-only mode  ", ro_style),
        Span::styled("(", Style::default().fg(THEME.muted)),
        Span::styled("Shift+R", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" to toggle)", Style::default().fg(THEME.muted)),
    ]));
    lines.push(Line::from(""));
    
    lines.push(Line::from(vec![
        Span::styled("─".repeat(50), Style::default().fg(THEME.border)),
    ]));
    lines.push(Line::from(""));

    for (i, profile) in profiles.iter().enumerate() {
        let is_selected = i == selected_index;
        let is_current = current_profile.map_or(false, |cp| cp == profile);
        
        let prefix = if is_selected { "▶ " } else { "  " };
        let suffix = if is_current { " (current)" } else { "" };
        
        let style = if is_selected {
            Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default().fg(THEME.success)
        } else {
            Style::default().fg(THEME.fg)
        };
        
        lines.push(Line::from(vec![
            Span::styled(format!("{}{}{}", prefix, profile, suffix), style),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    let popup_area = centered_rect(50, 65, area);
    
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

/// Render the region switcher modal
pub fn render_region_switcher_modal(
    frame: &mut Frame,
    area: Rect,
    regions: &[String],
    selected_index: usize,
    current_region: &str,
    selected_profile: Option<&str>,
) {
    let title = if let Some(profile) = selected_profile {
        format!(" Select AWS Region (Profile: {}) ", profile)
    } else {
        " Select AWS Region ".to_string()
    };
    
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary));

    let mut lines: Vec<Line> = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled("Use ", Style::default().fg(THEME.muted)),
        Span::styled("j/k", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(THEME.muted)),
        Span::styled("↑/↓", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" to navigate, ", Style::default().fg(THEME.muted)),
        Span::styled("Enter", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
        Span::styled(" to confirm, ", Style::default().fg(THEME.muted)),
        Span::styled("Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
        Span::styled(" to cancel", Style::default().fg(THEME.muted)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("─".repeat(50), Style::default().fg(THEME.border)),
    ]));
    lines.push(Line::from(""));

    // Calculate scroll offset to keep selection visible
    let visible_rows = 15usize; // Approximate visible rows in modal
    let scroll_offset = if selected_index >= visible_rows {
        selected_index - visible_rows + 5
    } else {
        0
    };

    for (i, region) in regions.iter().enumerate().skip(scroll_offset).take(visible_rows + 5) {
        let is_selected = i == selected_index;
        let is_current = region == current_region;
        
        let prefix = if is_selected { "▶ " } else { "  " };
        let suffix = if is_current { " (current)" } else { "" };
        
        let style = if is_selected {
            Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default().fg(THEME.success)
        } else {
            Style::default().fg(THEME.fg)
        };
        
        lines.push(Line::from(vec![
            Span::styled(format!("{}{}{}", prefix, region, suffix), style),
        ]));
    }
    
    // Show scroll indicator if there are more items
    if regions.len() > visible_rows + 5 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(format!("  ... {} regions total", regions.len()), Style::default().fg(THEME.muted)),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    let popup_area = centered_rect(50, 70, area);
    
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
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
