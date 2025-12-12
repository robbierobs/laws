use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use crate::app::App;
use crate::ui::theme::THEME;

/// Render action log as a centered popup with list and details pane
pub fn render_popup(frame: &mut Frame, area: Rect, app: &mut App) {
    let popup_area = centered_rect(85, 75, area);

    // Clear the popup area
    frame.render_widget(Clear, popup_area);

    // Split into list (left) and details (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(popup_area);

    let list_area = chunks[0];
    let detail_area = chunks[1];

    // Render the log list
    render_log_list(frame, list_area, app);

    // Render the selected log details
    render_log_details(frame, detail_area, app);
}

fn render_log_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Action Log ")
        .title_style(Style::default().fg(THEME.primary))
        .border_style(Style::default().fg(THEME.secondary));

    // Show logs in reverse order (newest first)
    let logs: Vec<&String> = app.action_log.iter().rev().collect();
    
    let items: Vec<ListItem> = logs.iter().enumerate()
        .map(|(i, msg)| {
            let is_selected = i == app.action_log_selected_index;
            let base_style = if msg.contains("[ERROR]") {
                Style::default().fg(THEME.error)
            } else if msg.contains("[SUCCESS]") {
                Style::default().fg(THEME.success)
            } else {
                Style::default().fg(THEME.fg)
            };
            
            let style = if is_selected {
                base_style.add_modifier(Modifier::BOLD).bg(THEME.selection_bg)
            } else {
                base_style
            };
            
            // Truncate message for list display
            let display = if msg.len() > 50 {
                format!("{}...", &msg[..47])
            } else {
                msg.to_string()
            };
            
            ListItem::new(display).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.action_log_selected_index));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(THEME.selection_bg));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_log_details(frame: &mut Frame, area: Rect, app: &mut App) {
    let logs: Vec<&String> = app.action_log.iter().rev().collect();
    
    let (title, content, style) = if logs.is_empty() {
        ("Details".to_string(), "No log entries".to_string(), Style::default().fg(THEME.muted))
    } else if let Some(msg) = logs.get(app.action_log_selected_index) {
        let entry_style = if msg.contains("[ERROR]") {
            Style::default().fg(THEME.error)
        } else if msg.contains("[SUCCESS]") {
            Style::default().fg(THEME.success)
        } else {
            Style::default().fg(THEME.fg)
        };
        (
            format!(" Details ({}/{}) ", app.action_log_selected_index + 1, logs.len()),
            msg.to_string(),
            entry_style,
        )
    } else {
        ("Details".to_string(), "No entry selected".to_string(), Style::default().fg(THEME.muted))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(THEME.primary))
        .border_style(Style::default().fg(THEME.border));

    // Calculate visible area
    let inner_area = block.inner(area);
    let visible_height = inner_area.height as usize;
    
    // Split content into lines for scrolling
    let content_lines: Vec<Line> = content
        .lines()
        .flat_map(|line| {
            // Word wrap long lines
            if line.len() > inner_area.width as usize {
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut wrapped_lines = Vec::new();
                let mut current_line = String::new();
                
                for word in words {
                    if current_line.is_empty() {
                        current_line = word.to_string();
                    } else if current_line.len() + 1 + word.len() <= inner_area.width as usize {
                        current_line.push(' ');
                        current_line.push_str(word);
                    } else {
                        wrapped_lines.push(Line::from(Span::styled(current_line.clone(), style)));
                        current_line = word.to_string();
                    }
                }
                if !current_line.is_empty() {
                    wrapped_lines.push(Line::from(Span::styled(current_line, style)));
                }
                wrapped_lines
            } else {
                vec![Line::from(Span::styled(line.to_string(), style))]
            }
        })
        .collect();

    let total_lines = content_lines.len();
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll_offset = (app.action_log_detail_scroll as usize).min(max_scroll);
    
    // Update scroll offset if it was clamped
    if scroll_offset != app.action_log_detail_scroll as usize {
        app.action_log_detail_scroll = scroll_offset as u16;
    }

    let visible_lines: Vec<Line> = content_lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_height)
        .collect();

    // Add hints at the bottom
    let mut all_lines = visible_lines;
    if all_lines.len() < visible_height {
        // Add padding
        for _ in all_lines.len()..(visible_height.saturating_sub(2)) {
            all_lines.push(Line::from(""));
        }
        // Add hints
        all_lines.push(Line::from(""));
        all_lines.push(Line::from(vec![
            Span::styled("[j/k] ", Style::default().fg(THEME.warning)),
            Span::raw("select  "),
            Span::styled("[PgUp/PgDn] ", Style::default().fg(THEME.warning)),
            Span::raw("scroll  "),
            Span::styled("[Shift+A] ", Style::default().fg(THEME.warning)),
            Span::raw("close"),
        ]));
    }

    let paragraph = Paragraph::new(all_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // Render scrollbar if content exceeds visible area
    if total_lines > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        let mut scrollbar_state = ScrollbarState::new(total_lines).position(scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
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
