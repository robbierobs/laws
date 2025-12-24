//! Generic filter modal rendering
//!
//! Provides a reusable filter modal UI component that works with FilterModalConfig.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::filter_modal::{FilterFieldConfig, FilterFieldType, FilterModalConfig, FilterModalState};
use crate::ui::theme::THEME;

/// Render the filter modal
pub fn render_filter_modal(
    frame: &mut Frame,
    area: Rect,
    config: &FilterModalConfig,
    state: &FilterModalState,
) {
    let popup_area = centered_rect(area, config.width_percent, config.height_percent);

    // Clear the area behind the modal
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" {} ", config.title))
        .title_style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Calculate layout - each field gets 2 rows, plus help at bottom
    let num_fields = config.fields.len();
    let mut constraints: Vec<Constraint> = config.fields.iter()
        .map(|_| Constraint::Length(3))
        .collect();
    constraints.push(Constraint::Length(2)); // Help line
    constraints.push(Constraint::Min(0));    // Remaining space

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    // Render each field
    for (i, field_config) in config.fields.iter().enumerate() {
        let is_selected = i == state.selected_field;
        let field_state = state.field_states.get(&field_config.id);
        
        render_field(frame, rows[i], field_config, field_state, is_selected);
    }

    // Render help line
    let help_row = rows[num_fields];
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD)),
        Span::raw("/"),
        Span::styled("↑↓", Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD)),
        Span::raw(": navigate  "),
        Span::styled("Enter", Style::default().fg(THEME.success).add_modifier(Modifier::BOLD)),
        Span::raw(": apply  "),
        Span::styled("Esc", Style::default().fg(THEME.warning).add_modifier(Modifier::BOLD)),
        Span::raw(": cancel"),
    ]))
    .style(Style::default().fg(THEME.muted));

    frame.render_widget(help, help_row);
}

fn render_field(
    frame: &mut Frame,
    area: Rect,
    config: &FilterFieldConfig,
    state: Option<&crate::app::filter_modal::FilterFieldState>,
    is_selected: bool,
) {
    let text_style = if is_selected {
        Style::default().fg(THEME.selection_fg).bg(THEME.selection_bg)
    } else {
        Style::default().fg(THEME.fg)
    };

    let border_style = if is_selected {
        Style::default().fg(THEME.primary)
    } else {
        Style::default().fg(THEME.muted)
    };

    let value = state.map(|s| s.display_value(&config.field_type)).unwrap_or_default();
    let value_empty = value.is_empty();
    
    let display_value = match &config.field_type {
        FilterFieldType::Text | FilterFieldType::Date | FilterFieldType::Time => {
            if value_empty {
                if is_selected {
                    "█".to_string() // Cursor
                } else {
                    config.placeholder.clone().unwrap_or_default()
                }
            } else if is_selected {
                format!("{}█", value) // Value with cursor
            } else {
                value
            }
        }
        FilterFieldType::Toggle => {
            let toggle_val = state.map(|s| s.toggle_value).unwrap_or(false);
            let indicator = if toggle_val { "[✓]" } else { "[ ]" };
            if is_selected {
                format!("{} (Space to toggle)", indicator)
            } else {
                indicator.to_string()
            }
        }
        FilterFieldType::Cycle(options) => {
            let idx = state.map(|s| s.cycle_index).unwrap_or(0);
            let current = options.get(idx).cloned().unwrap_or_default();
            if is_selected {
                format!("◀ {} ▶ (Space to cycle)", current)
            } else {
                current
            }
        }
    };

    let display_style = match &config.field_type {
        FilterFieldType::Text | FilterFieldType::Date | FilterFieldType::Time => {
            if value_empty && !is_selected {
                Style::default().fg(THEME.muted).add_modifier(Modifier::ITALIC)
            } else {
                text_style
            }
        }
        _ => text_style,
    };

    let block = Block::default()
        .title(format!(" {} ", config.label))
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(display_value)
        .style(display_style)
        .block(block);

    frame.render_widget(paragraph, area);
}

/// Create a centered rect with percentage dimensions
fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::filter_modal::{FilterFieldConfig, FilterModalConfig, FilterModalState};

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(area, 60, 70);
        
        // Should be roughly centered
        assert!(centered.x > 0);
        assert!(centered.y > 0);
        assert!(centered.width > 0);
        assert!(centered.height > 0);
    }

    #[test]
    fn test_filter_modal_state_creation() {
        let config = FilterModalConfig::new("Test")
            .field(FilterFieldConfig::text("q", "Query"))
            .field(FilterFieldConfig::date("date", "Date"));

        let state = FilterModalState::new(&config);
        assert_eq!(state.field_states.len(), 2);
        assert!(state.field_states.contains_key("q"));
        assert!(state.field_states.contains_key("date"));
    }
}
