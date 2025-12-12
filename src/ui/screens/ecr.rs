use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::{App, EcrViewMode};

use crate::ui::components::detail_panel::{render_detail_panel, DetailPanelConfig};
use crate::ui::components::table::render_table;
use crate::ui::theme::THEME;
use crate::app::ViewMode;

pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    use ratatui::layout::{Layout, Direction};
    
    // Render list area if provided (not in fullscreen detail mode)
    if let Some(area) = list_area {
        // Split list area for tabs if needed, though usually just list
        // If we want tabs at top like Backup
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // List
            ])
            .split(area);

        let tabs: Vec<&str> = crate::app::EcrViewMode::iterator().map(|m| m.label()).collect();
        // Since ECR is drill-down (Repo -> Images), we might just show relevant title/tabs or just list.
        // But let's follow standard pattern.
        crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.services.ecr.view_mode.index());

        match app.services.ecr.view_mode {
            EcrViewMode::Repositories => render_repository_list(frame, chunks[1], app),
            EcrViewMode::Images => render_image_list(frame, chunks[1], app),
        }
    }
    
    if let Some(area) = detail_area {
        match app.services.ecr.view_mode {
            EcrViewMode::Repositories => render_repository_details(frame, area, app),
            EcrViewMode::Images => render_image_details(frame, area, app),
        }
    }
}

use crate::models::Filterable;

fn render_repository_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.ecr.repositories.iter()
        .filter(|r| {
            if filter.is_empty() { return true; }
            r.matches_filter(&filter)
        })
        .map(|repo| {
            let created = repo.created_at.clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());
            
            let cells = vec![
                Cell::from(repo.repository_name.clone()),
                Cell::from(repo.repository_uri.clone().unwrap_or_default()),
                Cell::from(created),
                Cell::from(repo.image_tag_mutability.clone().unwrap_or_default()),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["Name", "URI", "Created", "Immutability"],
        &[
            Constraint::Length(30),
            Constraint::Min(40),
            Constraint::Length(15),
            Constraint::Length(15),
        ],
        "Repositories (Enter to view images)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.ecr.list_state,
    );
}

fn render_image_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.ecr.images.iter()
        .filter(|img| {
            if filter.is_empty() { return true; }
            img.matches_filter(&filter)
        })
        .map(|img| {
            let pushed = img.image_pushed_at.clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());
            
             // Format size
            let size = if let Some(bytes) = img.image_size_in_bytes {
                if bytes > 1024 * 1024 * 1024 {
                    format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
                } else {
                    format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
                }
            } else {
                "-".to_string()
            };

            let tags = if img.image_tags.is_empty() {
                "<untagged>".to_string()
            } else {
                img.image_tags.join(", ")
            };
            
            let cells = vec![
                Cell::from(tags),
                Cell::from(img.image_digest.chars().take(12).collect::<String>()),
                Cell::from(size),
                Cell::from(pushed),
            ];
            
            Row::new(cells).height(1)
        });

    let title = if let Some(repo) = &app.services.ecr.selected_repo_name {
        format!("Images for {} (Esc to back)", repo)
    } else {
        "Images (Esc to back)".to_string()
    };

    render_table(
        frame,
        area,
        rows,
        &["Tags", "Digest", "Size", "Pushed At"],
        &[
            Constraint::Min(30),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(20),
        ],
        &title,
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.ecr.list_state,
    );
}

fn render_repository_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.ecr.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(repo) = app.services.ecr.repositories.get(idx) {
            vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(THEME.primary)),
                    Span::styled(repo.repository_name.clone(), Style::default().fg(THEME.selection_fg)),
                ]),
                Line::from(vec![
                    Span::styled("URI: ", Style::default().fg(THEME.primary)),
                    Span::raw(repo.repository_uri.clone().unwrap_or_default()),
                ]),
                Line::from(vec![
                    Span::styled("ARN: ", Style::default().fg(THEME.primary)),
                    Span::raw(repo.repository_arn.clone().unwrap_or_else(|| "-".to_string())),
                ]),
                Line::from(vec![
                    Span::styled("Created: ", Style::default().fg(THEME.primary)),
                    Span::raw(repo.created_at.clone().unwrap_or_else(|| "-".to_string())),
                ]),
                Line::from(vec![
                    Span::styled("Tag Immutability: ", Style::default().fg(THEME.primary)),
                    Span::raw(repo.image_tag_mutability.clone().unwrap_or_default()),
                ]),
            ]
        } else {
            vec![Line::from("No repository selected")]
        }
    } else {
        vec![Line::from("Select a repository to view details")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Repository Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn render_image_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.ecr.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(img) = app.services.ecr.images.get(idx) {
             let size = if let Some(bytes) = img.image_size_in_bytes {
                if bytes > 1024 * 1024 * 1024 {
                    format!("{:.2} GB ({} bytes)", bytes as f64 / (1024.0 * 1024.0 * 1024.0), bytes)
                } else {
                    format!("{:.2} MB ({} bytes)", bytes as f64 / (1024.0 * 1024.0), bytes)
                }
            } else {
                "-".to_string()
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Digest: ", Style::default().fg(THEME.primary)),
                    Span::styled(img.image_digest.clone(), Style::default().fg(THEME.selection_fg)),
                ]),
                Line::from(vec![
                    Span::styled("Pushed At: ", Style::default().fg(THEME.primary)),
                    Span::raw(img.image_pushed_at.clone().unwrap_or_else(|| "-".to_string())),
                ]),
                Line::from(vec![
                    Span::styled("Size: ", Style::default().fg(THEME.primary)),
                    Span::raw(size),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Tags: ", Style::default().fg(THEME.secondary)),
                ]),
            ];
            
            if img.image_tags.is_empty() {
                lines.push(Line::from("  <untagged>"));
            } else {
                for tag in &img.image_tags {
                    lines.push(Line::from(format!("  • {}", tag)));
                }
            }
            
            lines
        } else {
            vec![Line::from("No image selected")]
        }
    } else {
        vec![Line::from("Select an image to view details")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Image Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}
