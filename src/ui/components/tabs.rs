use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_tabs(frame: &mut Frame, area: Rect, tabs: &[&str], selected_index: usize) {
    let mut spans = Vec::new();
    
    for (i, tab) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" | "));
        }
        
        let style = if i == selected_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        spans.push(Span::styled(*tab, style));
    }
    
    let p = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::BOTTOM));
    
    frame.render_widget(p, area);
}
