use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::App;
use crate::models::secretsmanager::Secret;
use crate::ui::components::detail_panel::render_detail_panel_with_selection;
use crate::ui::components::table::render_table;
use crate::ui::theme::THEME;
use crate::models::Filterable;

pub fn render_secretsmanager_screen(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    if let Some(area) = list_area {
        render_secret_list(frame, area, app);
    }
    
    if let Some(area) = detail_area {
        render_secret_details(frame, area, app);
    }
}

fn render_secret_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.secretsmanager.secrets.iter()
        .filter(|s| {
            if filter.is_empty() { return true; }
            s.matches_filter(&filter)
        })
        .map(|secret| {
             let cells = vec![
                 Cell::from(secret.name.clone()),
                 Cell::from(secret.description.clone().unwrap_or_default()),
                 Cell::from(secret.created_date.clone().unwrap_or_default()),
                 Cell::from(secret.last_changed_date.clone().unwrap_or_default()),
             ];
             Row::new(cells).height(1)
        });
        
    render_table(
        frame,
        area,
        rows,
        &["Name", "Description", "Created", "Last Changed"],
        &[
            Constraint::Length(40),
            Constraint::Min(20),
            Constraint::Length(25),
            Constraint::Length(25),
        ],
        "Secrets",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.secretsmanager.list_state,
    );
}

fn render_secret_details(frame: &mut Frame, area: Rect, app: &App) {
    render_detail_panel_with_selection(
        frame,
        area,
        app.services.secretsmanager.selected_secret(),
        build_secret_detail_lines,
        "Secret Details",
        "Select a secret to view details",
        app.detail_panel_fullscreen,
        app.detail_scroll_offset,
    );
}

fn build_secret_detail_lines(secret: &Secret) -> Vec<Line<'_>> {
     let mut lines = vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(THEME.primary)),
            Span::raw(secret.name.clone()),
        ]),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
            Span::raw(secret.arn.clone().unwrap_or_default()),
        ]),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(THEME.primary)),
            Span::raw(secret.description.clone().unwrap_or_default()),
        ]),
        Line::from(vec![
             Span::styled("Created: ", Style::default().fg(THEME.primary)),
             Span::raw(secret.created_date.clone().unwrap_or_default()),
             Span::raw("   "),
             Span::styled("Last Changed: ", Style::default().fg(THEME.primary)),
             Span::raw(secret.last_changed_date.clone().unwrap_or_default()),
        ]),
        Line::from(vec![
             Span::styled("Last Accessed: ", Style::default().fg(THEME.primary)),
             Span::raw(secret.last_accessed_date.clone().unwrap_or_default()),
        ]),
     ];
     
     if let Some(deleted) = &secret.deleted_date {
         lines.push(Line::from(vec![
             Span::styled("Deleted Date: ", Style::default().fg(THEME.error)),
             Span::raw(deleted),
         ]));
     }
     
     lines
}
