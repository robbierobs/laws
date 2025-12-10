use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};
use crate::app::App;
use crate::ui::theme::THEME;

/// Render action log as a centered popup
pub fn render_popup(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(70, 60, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Action Log (Shift+A to close) ")
        .title_style(Style::default().fg(THEME.primary))
        .border_style(Style::default().fg(THEME.secondary));

    let items: Vec<ListItem> = app.action_log.iter().rev()
        .map(|msg| {
            let style = if msg.contains("[ERROR]") {
                Style::default().fg(THEME.error)
            } else if msg.contains("[SUCCESS]") {
                Style::default().fg(THEME.success)
            } else {
                Style::default().fg(THEME.fg)
            };
            ListItem::new(msg.clone()).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    
    frame.render_widget(Clear, popup_area);
    frame.render_widget(list, popup_area);
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
