use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Style, Modifier},
    layout::{Rect, Layout, Direction, Constraint},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::app::{Message, Service};
use crate::ui::components::Component;

pub struct Sidebar {
    /// All available services
    pub items: Vec<Service>,
    /// Indices of services that match the current filter
    filtered_indices: Vec<usize>,
    /// Selection state for the filtered list
    pub state: ListState,
    pub is_focused: bool,
    /// Current filter text
    pub filter: String,
    /// Whether the filter input is active
    pub filter_active: bool,
}

impl Sidebar {
    pub fn new() -> Self {
        let items: Vec<Service> = Service::iterator().collect();
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            items,
            filtered_indices,
            state,
            is_focused: true,
            filter: String::new(),
            filter_active: false,
        }
    }

    /// Rebuild the filtered indices based on current filter text
    fn rebuild_filter(&mut self) {
        self.filtered_indices.clear();
        
        if self.filter.is_empty() {
            // No filter - include all indices
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            let filter_lower = self.filter.to_lowercase();
            for (idx, service) in self.items.iter().enumerate() {
                if service.as_str().to_lowercase().contains(&filter_lower) {
                    self.filtered_indices.push(idx);
                }
            }
        }
        
        // Adjust selection if it's now out of bounds
        if let Some(selected) = self.state.selected() {
            if selected >= self.filtered_indices.len() {
                self.state.select(if self.filtered_indices.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
        } else if !self.filtered_indices.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Get the number of filtered services
    #[allow(dead_code)]
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_indices.len() - 1 {
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
        if self.filtered_indices.is_empty() {
            return;
        }
        
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_indices.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    /// Get the currently selected service from the filtered list
    pub fn selected_service(&self) -> Option<Service> {
        self.state
            .selected()
            .and_then(|sel_idx| self.filtered_indices.get(sel_idx))
            .and_then(|&item_idx| self.items.get(item_idx))
            .copied()
    }

    /// Select a specific service (finds it in the filtered list)
    pub fn select_service(&mut self, service: Service) {
        // Find the service in the filtered indices
        for (filtered_idx, &item_idx) in self.filtered_indices.iter().enumerate() {
            if self.items[item_idx] == service {
                self.state.select(Some(filtered_idx));
                return;
            }
        }
        // If not found in filtered list, clear filter and select
        if let Some(index) = self.items.iter().position(|&s| s == service) {
            self.filter.clear();
            self.rebuild_filter();
            self.state.select(Some(index));
        }
    }

    /// Get filtered services for rendering
    pub fn filtered_services(&self) -> Vec<Service> {
        self.filtered_indices
            .iter()
            .filter_map(|&idx| self.items.get(idx))
            .copied()
            .collect()
    }

    /// Clear the filter and reset to showing all services
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.filter_active = false;
        self.rebuild_filter();
    }
}

use crate::ui::theme::THEME;

impl Component for Sidebar {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Split area for filter input at bottom when filter is active or has text
        let show_filter = self.filter_active || !self.filter.is_empty();
        
        let chunks = if show_filter {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),    // List area
                    Constraint::Length(3), // Filter input
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3)])
                .split(area)
        };
        
        let list_area = chunks[0];

        // Build list items from filtered services
        let filtered_services = self.filtered_services();
        let items: Vec<ListItem> = filtered_services
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

        let title = if self.filter.is_empty() {
            "Services".to_string()
        } else {
            format!("Services ({}/{})", self.filtered_indices.len(), self.items.len())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(THEME.primary));
            
        let block = if self.is_focused && !self.filter_active {
            block.border_style(Style::default().fg(THEME.secondary))
        } else {
            block.border_style(Style::default().fg(THEME.border))
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, list_area, &mut self.state);

        // Render filter input if active
        if show_filter {
            let filter_area = chunks[1];
            let filter_text = format!("/{}", self.filter);
            
            let filter_block = Block::default()
                .borders(Borders::ALL)
                .title("Filter")
                .title_style(Style::default().fg(THEME.primary));
            
            let filter_block = if self.filter_active {
                filter_block.border_style(Style::default().fg(THEME.secondary))
            } else {
                filter_block.border_style(Style::default().fg(THEME.border))
            };
            
            let filter_paragraph = Paragraph::new(filter_text)
                .style(Style::default().fg(if self.filter_active { THEME.fg } else { THEME.muted }))
                .block(filter_block);
            
            frame.render_widget(filter_paragraph, filter_area);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Message> {
        if !self.is_focused {
            return None;
        }

        // Handle filter input mode
        if self.filter_active {
            match key.code {
                KeyCode::Esc => {
                    // Cancel filter and clear
                    self.clear_filter();
                }
                KeyCode::Enter => {
                    // Confirm filter, exit filter mode
                    self.filter_active = false;
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.rebuild_filter();
                }
                KeyCode::Char(c) => {
                    self.filter.push(c);
                    self.rebuild_filter();
                }
                _ => {}
            }
            return None;
        }

        // Normal navigation mode
        match key.code {
            KeyCode::Char('/') => {
                // Enter filter mode
                self.filter_active = true;
                None
            }
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
                    Some(Message::navigate(service))
                } else {
                    None
                }
            }
            KeyCode::Esc => {
                // Clear filter if it exists
                if !self.filter.is_empty() {
                    self.clear_filter();
                    None
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
