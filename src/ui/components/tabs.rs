use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme::THEME;

pub fn render_tabs(frame: &mut Frame, area: Rect, tabs: &[&str], selected_index: usize) {
    let mut spans = Vec::new();
    
    for (i, tab) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(THEME.muted)));
        }
        
        let style = if i == selected_index {
            Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(THEME.muted)
        };
        
        spans.push(Span::styled(*tab, style));
    }
    
    let p = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(THEME.border)));
    
    frame.render_widget(p, area);
}
