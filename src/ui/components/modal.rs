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

    // Use fixed size that ensures content fits
    let popup_width = area.width.min(60).max(40);
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
            Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(THEME.muted)),
            Span::styled("y", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm or ", Style::default().fg(THEME.muted)),
            Span::styled("n/Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default().fg(THEME.muted)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, chunks[1]);
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

/// Render the profile switcher modal with scrollbar and filter support
pub fn render_profile_switcher_modal(
    frame: &mut Frame,
    area: Rect,
    profiles: &[String],
    selected_index: usize,
    current_profile: Option<&str>,
    read_only: bool,
    filter: &str,
    filter_active: bool,
) {
    let block = Block::default()
        .title(" Select AWS Profile ")
        .title_style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary));

    let popup_area = centered_rect(55, 70, area);
    let inner_area = block.inner(popup_area);
    
    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);
    
    // Calculate layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Help line
            Constraint::Length(2), // Filter input
            Constraint::Length(2), // Read-only toggle
            Constraint::Length(1), // Separator
            Constraint::Min(5),    // Profile list
            Constraint::Length(1), // Scroll indicator
        ])
        .split(inner_area);
    
    // Help line
    let help = Line::from(vec![
        Span::styled("j/k", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" nav  ", Style::default().fg(THEME.muted)),
        Span::styled("/", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" filter  ", Style::default().fg(THEME.muted)),
        Span::styled("R", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" read-only  ", Style::default().fg(THEME.muted)),
        Span::styled("Enter", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
        Span::styled(" select  ", Style::default().fg(THEME.muted)),
        Span::styled("Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
        Span::styled(" cancel", Style::default().fg(THEME.muted)),
    ]);
    frame.render_widget(Paragraph::new(help), chunks[0]);
    
    // Filter input
    let filter_style = if filter_active {
        Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.muted)
    };
    let filter_text = if filter.is_empty() && !filter_active {
        "/ to filter...".to_string()
    } else {
        format!("Filter: {}{}", filter, if filter_active { "▏" } else { "" })
    };
    let filter_line = Paragraph::new(filter_text).style(filter_style);
    frame.render_widget(filter_line, chunks[1]);
    
    // Read-only toggle
    let checkbox = if read_only { "[✓]" } else { "[ ]" };
    let ro_style = if read_only {
        Style::default().fg(THEME.warning).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.muted)
    };
    let readonly_line = Line::from(vec![
        Span::styled(checkbox, ro_style),
        Span::styled(" Read-only mode", ro_style),
    ]);
    frame.render_widget(Paragraph::new(readonly_line), chunks[2]);
    
    // Separator
    let separator = Line::from(Span::styled("─".repeat(chunks[3].width as usize), Style::default().fg(THEME.border)));
    frame.render_widget(Paragraph::new(separator), chunks[3]);
    
    // Calculate visible area for profiles
    let list_height = chunks[4].height as usize;
    let total_profiles = profiles.len();
    
    // Calculate scroll offset to keep selection visible
    let scroll_offset = if total_profiles <= list_height {
        0
    } else if selected_index < list_height / 2 {
        0
    } else if selected_index >= total_profiles.saturating_sub(list_height / 2) {
        total_profiles.saturating_sub(list_height)
    } else {
        selected_index.saturating_sub(list_height / 2)
    };
    
    // Profile list with scrolling
    let mut lines: Vec<Line> = Vec::new();
    for (i, profile) in profiles.iter().enumerate().skip(scroll_offset).take(list_height) {
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
        
        lines.push(Line::from(Span::styled(format!("{}{}{}", prefix, profile, suffix), style)));
    }
    
    let list = Paragraph::new(lines);
    frame.render_widget(list, chunks[4]);
    
    // Scroll indicator
    if total_profiles > list_height {
        let scroll_pos = if total_profiles <= list_height {
            0
        } else {
            (scroll_offset * 100) / (total_profiles - list_height)
        };
        let indicator = format!(
            " {} of {} profiles │ {}% ",
            selected_index + 1,
            total_profiles,
            scroll_pos
        );
        let scroll_line = Paragraph::new(indicator)
            .style(Style::default().fg(THEME.muted))
            .alignment(Alignment::Right);
        frame.render_widget(scroll_line, chunks[5]);
    } else if total_profiles == 0 {
        let no_results = Paragraph::new("No matching profiles")
            .style(Style::default().fg(THEME.muted))
            .alignment(Alignment::Center);
        frame.render_widget(no_results, chunks[5]);
    }
}

/// Render the region switcher modal with scrollbar and filter support
pub fn render_region_switcher_modal(
    frame: &mut Frame,
    area: Rect,
    regions: &[String],
    selected_index: usize,
    current_region: &str,
    selected_profile: Option<&str>,
    filter: &str,
    filter_active: bool,
) {
    let title = if let Some(profile) = selected_profile {
        format!(" Select Region (Profile: {}) ", profile)
    } else {
        " Select AWS Region ".to_string()
    };
    
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary));

    let popup_area = centered_rect(55, 70, area);
    let inner_area = block.inner(popup_area);
    
    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);
    
    // Calculate layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Help line
            Constraint::Length(2), // Filter input
            Constraint::Length(1), // Separator
            Constraint::Min(5),    // Region list
            Constraint::Length(1), // Scroll indicator
        ])
        .split(inner_area);
    
    // Help line
    let help = Line::from(vec![
        Span::styled("j/k", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" nav  ", Style::default().fg(THEME.muted)),
        Span::styled("/", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" filter  ", Style::default().fg(THEME.muted)),
        Span::styled("Enter", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
        Span::styled(" confirm  ", Style::default().fg(THEME.muted)),
        Span::styled("Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
        Span::styled(" cancel", Style::default().fg(THEME.muted)),
    ]);
    frame.render_widget(Paragraph::new(help), chunks[0]);
    
    // Filter input
    let filter_style = if filter_active {
        Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.muted)
    };
    let filter_text = if filter.is_empty() && !filter_active {
        "/ to filter...".to_string()
    } else {
        format!("Filter: {}{}", filter, if filter_active { "▏" } else { "" })
    };
    let filter_line = Paragraph::new(filter_text).style(filter_style);
    frame.render_widget(filter_line, chunks[1]);
    
    // Separator
    let separator = Line::from(Span::styled("─".repeat(chunks[2].width as usize), Style::default().fg(THEME.border)));
    frame.render_widget(Paragraph::new(separator), chunks[2]);
    
    // Calculate visible area for regions
    let list_height = chunks[3].height as usize;
    let total_regions = regions.len();
    
    // Calculate scroll offset to keep selection visible
    let scroll_offset = if total_regions <= list_height {
        0
    } else if selected_index < list_height / 2 {
        0
    } else if selected_index >= total_regions.saturating_sub(list_height / 2) {
        total_regions.saturating_sub(list_height)
    } else {
        selected_index.saturating_sub(list_height / 2)
    };
    
    // Region list with scrolling
    let mut lines: Vec<Line> = Vec::new();
    for (i, region) in regions.iter().enumerate().skip(scroll_offset).take(list_height) {
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
        
        lines.push(Line::from(Span::styled(format!("{}{}{}", prefix, region, suffix), style)));
    }
    
    let list = Paragraph::new(lines);
    frame.render_widget(list, chunks[3]);
    
    // Scroll indicator
    if total_regions > list_height {
        let scroll_pos = if total_regions <= list_height {
            0
        } else {
            (scroll_offset * 100) / (total_regions - list_height)
        };
        let indicator = format!(
            " {} of {} regions │ {}% ",
            selected_index + 1,
            total_regions,
            scroll_pos
        );
        let scroll_line = Paragraph::new(indicator)
            .style(Style::default().fg(THEME.muted))
            .alignment(Alignment::Right);
        frame.render_widget(scroll_line, chunks[4]);
    } else if total_regions == 0 {
        let no_results = Paragraph::new("No matching regions")
            .style(Style::default().fg(THEME.muted))
            .alignment(Alignment::Center);
        frame.render_widget(no_results, chunks[4]);
    }
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

/// Create a centered rect with fixed width and height (in characters/rows)
fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    // Ensure we don't exceed available space
    let actual_width = width.min(r.width);
    let actual_height = height.min(r.height);
    
    // Calculate centering offsets
    let x_offset = (r.width.saturating_sub(actual_width)) / 2;
    let y_offset = (r.height.saturating_sub(actual_height)) / 2;
    
    Rect {
        x: r.x + x_offset,
        y: r.y + y_offset,
        width: actual_width,
        height: actual_height,
    }
}
