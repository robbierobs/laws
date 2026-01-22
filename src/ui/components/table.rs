use crate::ui::theme::THEME;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

/// Generic helper to render a consistent table across all services.
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The area to render the table in
/// * `rows` - An iterator of Rows to display
/// * `header_labels` - A list of strings for the column headers
/// * `constraints` - A list of constraints for column widths
/// * `title` - The title of the table (will be shown in the border)
/// * `is_focused` - Whether this table currently has focus (affects border color)
/// * `state` - The mutable TableState from the app for scrolling
#[allow(clippy::too_many_arguments)] // Central rendering utility, arguments are all required
pub fn render_table<'a, I>(
    frame: &mut Frame,
    area: Rect,
    rows: I,
    header_labels: &[&str],
    constraints: &[Constraint],
    title: &str,
    is_focused: bool,
    state: &mut TableState,
) where
    I: IntoIterator<Item = Row<'a>>,
{
    // Create styled header cells
    let header_cells = header_labels
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));

    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    // Create block with dynamic border color based on focus
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(THEME.primary))
        .border_style(if is_focused {
            Style::default().fg(THEME.secondary)
        } else {
            Style::default().fg(THEME.border)
        });

    // Create the table widget
    let table = Table::new(rows, constraints)
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::default()
                .bg(THEME.selection_bg)
                .fg(THEME.selection_fg)
                .add_modifier(Modifier::BOLD),
        );

    // Render the table with state
    frame.render_stateful_widget(table, area, state);
}
