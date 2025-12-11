//! Generic detail panel component
//!
//! Provides a reusable detail panel with scrolling support and consistent styling.

use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use crate::ui::theme::THEME;

/// Configuration for rendering a detail panel
pub struct DetailPanelConfig<'a> {
    /// Title of the panel (e.g., "Instance Details")
    pub title: &'a str,
    /// Whether the panel is in fullscreen mode
    pub is_fullscreen: bool,
    /// Current scroll offset
    pub scroll_offset: u16,
}

impl<'a> DetailPanelConfig<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            is_fullscreen: false,
            scroll_offset: 0,
        }
    }

    pub fn fullscreen(mut self, is_fullscreen: bool) -> Self {
        self.is_fullscreen = is_fullscreen;
        self
    }

    pub fn scroll(mut self, offset: u16) -> Self {
        self.scroll_offset = offset;
        self
    }
}

/// Render a detail panel with content, scrollbar, and consistent styling.
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The area to render in
/// * `content` - The lines of content to display
/// * `config` - Panel configuration (title, fullscreen, scroll)
///
/// # Example
/// ```ignore
/// let content = if let Some(item) = selected_item {
///     build_detail_lines(item)
/// } else {
///     vec![Line::from("Select an item to view details")]
/// };
/// render_detail_panel(frame, area, content, DetailPanelConfig::new("Item Details")
///     .fullscreen(app.detail_panel_fullscreen)
///     .scroll(app.detail_scroll_offset));
/// ```
pub fn render_detail_panel(
    frame: &mut Frame,
    area: Rect,
    content: Vec<Line<'_>>,
    config: DetailPanelConfig<'_>,
) {
    let total_lines = content.len();
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let scroll_offset = config.scroll_offset as usize;
    
    // Determine if we need to show scroll indicator
    let can_scroll = total_lines > visible_height;
    let scroll_info = if can_scroll {
        format!(" [{}/{}] ", 
            scroll_offset.min(total_lines.saturating_sub(visible_height)) + 1, 
            total_lines.saturating_sub(visible_height).max(1)
        )
    } else {
        String::new()
    };
    
    // Build title with scroll info and keyboard hints
    let title = if config.is_fullscreen {
        format!("{} (Fullscreen){} [D: exit, PgUp/PgDn: scroll]", config.title, scroll_info)
    } else if can_scroll {
        format!("{}{} [D: fullscreen, PgUp/PgDn: scroll]", config.title, scroll_info)
    } else {
        format!("{} [D: fullscreen]", config.title)
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)))
        .scroll((config.scroll_offset, 0));
    
    frame.render_widget(paragraph, area);
    
    // Render scrollbar if content overflows
    if can_scroll {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        let max_scroll = total_lines.saturating_sub(visible_height);
        let mut scrollbar_state = ScrollbarState::new(max_scroll)
            .position(scroll_offset.min(max_scroll));
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

/// Convenience function to render a detail panel with standard "no selection" handling.
///
/// This is the most common pattern where we have an optional selected item
/// and want to show either its details or a "select an item" message.
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The area to render in  
/// * `selected_item` - The currently selected item, if any
/// * `build_lines` - Function to build detail lines from the item
/// * `title` - Panel title
/// * `empty_message` - Message to show when no item is selected
/// * `is_fullscreen` - Whether panel is in fullscreen mode
/// * `scroll_offset` - Current scroll offset
pub fn render_detail_panel_with_selection<T, F>(
    frame: &mut Frame,
    area: Rect,
    selected_item: Option<&T>,
    build_lines: F,
    title: &str,
    empty_message: &str,
    is_fullscreen: bool,
    scroll_offset: u16,
) where
    F: FnOnce(&T) -> Vec<Line<'_>>,
{
    let content = if let Some(item) = selected_item {
        build_lines(item)
    } else {
        vec![Line::from(empty_message)]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new(title)
            .fullscreen(is_fullscreen)
            .scroll(scroll_offset),
    );
}
