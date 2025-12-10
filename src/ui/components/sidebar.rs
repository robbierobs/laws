use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Style, Modifier},
    layout::Rect,
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::app::{Message, Service};
use crate::ui::components::Component;

pub struct Sidebar {
    pub items: Vec<Service>,
    pub state: ListState,
    pub is_focused: bool,
}

impl Sidebar {
    pub fn new() -> Self {
        let items: Vec<Service> = Service::iterator().collect();
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            items,
            state,
            is_focused: true, // Default focus for now
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn selected_service(&self) -> Option<Service> {
        self.state.selected().map(|i| self.items[i])
    }
}

use crate::ui::theme::THEME;

impl Component for Sidebar {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.items
            .iter()
            .map(|service| {
                ListItem::new(service.as_str())
            })
            .collect();

        // Highlight style
        let highlight_style = Style::default()
            .bg(THEME.selection_bg)
            .fg(THEME.selection_fg)
            .add_modifier(Modifier::BOLD);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Services")
            .title_style(Style::default().fg(THEME.primary));
            
        let block = if self.is_focused {
            block.border_style(Style::default().fg(THEME.secondary))
        } else {
            block.border_style(Style::default().fg(THEME.border))
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Message> {
        if !self.is_focused {
            return None;
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous();
                None
            }
            KeyCode::Enter => {
                if let Some(service) = self.selected_service() {
                    Some(Message::NavigateToService(service))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    

}
