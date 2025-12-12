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
        let is_current = current_profile.is_some_and(|cp| cp == profile);
        
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

/// Render the ECS service editor modal for modifying task definition, CPU, and memory
pub fn render_ecs_service_editor_modal(
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
        .title_style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
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
        Span::styled("Tab/↓↑", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" navigate  ", Style::default().fg(THEME.muted)),
        Span::styled("Enter", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
        Span::styled(" submit  ", Style::default().fg(THEME.muted)),
        Span::styled("Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
        Span::styled(" cancel", Style::default().fg(THEME.muted)),
    ]);
    frame.render_widget(Paragraph::new(help), chunks[0]);
    
    // Separator
    let sep = Line::from(Span::styled("─".repeat(chunks[1].width as usize), Style::default().fg(THEME.border)));
    frame.render_widget(Paragraph::new(sep.clone()), chunks[1]);
    
    // Task definition field
    let td_style = if active_field == 0 {
        Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)
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
        Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.fg)
    };
    let cursor = if active_field == 1 { "▏" } else { "" };
    let cpu_text = if cpu.is_empty() && active_field != 1 { "(unchanged)" } else { cpu };
    let cpu_line = Line::from(vec![
        Span::styled("CPU (vCPU units): ", Style::default().fg(THEME.secondary)),
        Span::styled(format!("{}{}", cpu_text, cursor), cpu_style),
    ]);
    frame.render_widget(Paragraph::new(cpu_line), chunks[3]);
    
    // Memory field
    let mem_style = if active_field == 2 {
        Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.fg)
    };
    let cursor = if active_field == 2 { "▏" } else { "" };
    let mem_text = if memory.is_empty() && active_field != 2 { "(unchanged)" } else { memory };
    let mem_line = Line::from(vec![
        Span::styled("Memory (MiB):     ", Style::default().fg(THEME.secondary)),
        Span::styled(format!("{}{}", mem_text, cursor), mem_style),
    ]);
    frame.render_widget(Paragraph::new(mem_line), chunks[4]);
    
    // Force deploy checkbox
    let fd_style = if active_field == 3 {
        Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)
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
        Span::styled("Changing CPU/Memory creates a new task definition revision", 
            Style::default().fg(THEME.muted).add_modifier(Modifier::ITALIC)),
    ]);
    frame.render_widget(Paragraph::new(hint), chunks[7]);
}

use crate::models::ecs::EcsTaskDefinition;

/// Render the ECS task definition selector modal with list and detail panes
pub fn render_task_def_selector_modal(
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
        .title_style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
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
            Constraint::Length(2),  // Help
            Constraint::Min(10),    // Main content
            Constraint::Length(2),  // Force deploy toggle
            Constraint::Length(1),  // Status line
        ])
        .split(inner_area);
    
    // Help line
    let help = Line::from(vec![
        Span::styled("j/k", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" nav  ", Style::default().fg(THEME.muted)),
        Span::styled("l/h", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" scroll detail  ", Style::default().fg(THEME.muted)),
        Span::styled("f", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" toggle force  ", Style::default().fg(THEME.muted)),
        Span::styled("Enter", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
        Span::styled(" select  ", Style::default().fg(THEME.muted)),
        Span::styled("Esc", Style::default().fg(THEME.error).add_modifier(Modifier::BOLD)),
        Span::styled(" cancel", Style::default().fg(THEME.muted)),
    ]);
    frame.render_widget(Paragraph::new(help), main_chunks[0]);
    
    // Split main content into list (left) and details (right)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),  // List
            Constraint::Percentage(65),  // Details
        ])
        .split(main_chunks[1]);
    
    // Render task definition list
    let list_block = Block::default()
        .title(" Task Definitions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.border));
    let list_inner = list_block.inner(content_chunks[0]);
    frame.render_widget(list_block, content_chunks[0]);
    
    if loading {
        let loading_text = Paragraph::new("Loading task definitions...")
            .style(Style::default().fg(THEME.muted))
            .alignment(Alignment::Center);
        frame.render_widget(loading_text, list_inner);
    } else if task_defs.is_empty() {
        let no_items = Paragraph::new("No task definitions found")
            .style(Style::default().fg(THEME.muted))
            .alignment(Alignment::Center);
        frame.render_widget(no_items, list_inner);
    } else {
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
        for (i, td) in task_defs.iter().enumerate().skip(scroll_offset).take(list_height) {
            let is_selected = i == selected_index;
            let prefix = if is_selected { "▶ " } else { "  " };
            let display = td.short_name();
            
            let style = if is_selected {
                Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)
            } else if td.status == "ACTIVE" {
                Style::default().fg(THEME.success)
            } else {
                Style::default().fg(THEME.muted)
            };
            
            lines.push(Line::from(Span::styled(format!("{}{}", prefix, display), style)));
        }
        frame.render_widget(Paragraph::new(lines), list_inner);
    }
    
    // Render details pane
    let detail_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.border));
    let detail_inner = detail_block.inner(content_chunks[1]);
    frame.render_widget(detail_block, content_chunks[1]);
    
    if let Some(td) = task_defs.get(selected_index) {
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
                    format!("...{}", &image[image.len()-47..])
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
                            container.memory.map(|m| m.to_string()).unwrap_or_else(|| "N/A".to_string())
                        ),
                        Style::default().fg(THEME.fg),
                    ),
                ]));
            }
            // Port mappings
            if !container.port_mappings.is_empty() {
                let ports: Vec<String> = container.port_mappings.iter()
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
        
        let paragraph = Paragraph::new(detail_lines)
            .scroll((detail_scroll as u16, 0));
        frame.render_widget(paragraph, detail_inner);
    }
    
    // Force deploy toggle
    let checkbox = if force_deploy { "[✓]" } else { "[ ]" };
    let fd_style = Style::default().fg(if force_deploy { THEME.warning } else { THEME.fg });
    let force_line = Line::from(vec![
        Span::styled(checkbox, fd_style.add_modifier(Modifier::BOLD)),
        Span::styled(" Force new deployment", fd_style),
        Span::styled("  (Press ", Style::default().fg(THEME.muted)),
        Span::styled("f", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
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
    frame.render_widget(Paragraph::new(status).alignment(Alignment::Right), main_chunks[3]);
}
