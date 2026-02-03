use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};

use crate::app::{App, EcrViewMode, ViewMode};
use crate::models::ecr::EcrImage;
use crate::ui::components::detail_panel::{render_detail_panel, DetailPanelConfig};
use crate::ui::components::table::render_table;
use crate::ui::theme::THEME;

pub fn render(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    use ratatui::layout::{Direction, Layout};

    if let Some(area) = list_area {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let tabs: Vec<&str> = crate::app::EcrViewMode::iterator()
            .map(|m| m.label())
            .collect();
        crate::ui::components::tabs::render_tabs(
            frame,
            chunks[0],
            &tabs,
            app.services.ecr.view_mode.index(),
        );

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

    if app.services.ecr.filter_modal.visible {
        use crate::ui::components::filter_modal::render_filter_modal;
        render_filter_modal(
            frame,
            frame.area(),
            &app.services.ecr.filter_config,
            &app.services.ecr.filter_modal,
        );
    }
}

use crate::models::Filterable;

fn render_repository_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app
        .services
        .ecr
        .repositories
        .iter()
        .filter(|r| filter.is_empty() || r.matches_filter(&filter))
        .map(|repo| {
            let created = repo
                .created_at
                .clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(repo.repository_name.clone()),
                Cell::from(repo.repository_uri.clone().unwrap_or_default()),
                Cell::from(created),
                Cell::from(repo.image_tag_mutability.clone().unwrap_or_default()),
            ])
            .height(1)
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

/// Format scan status/findings for list display
fn format_scan_cell(img: &EcrImage) -> String {
    let status_str = img.scan_status.as_ref().and_then(|s| s.status.as_deref());

    match status_str {
        None => "-".to_string(),
        Some("COMPLETE") | Some("ACTIVE") => {
            if let Some(summary) = &img.scan_findings_summary {
                let c = summary
                    .finding_severity_counts
                    .get("CRITICAL")
                    .unwrap_or(&0);
                let h = summary.finding_severity_counts.get("HIGH").unwrap_or(&0);
                let m = summary.finding_severity_counts.get("MEDIUM").unwrap_or(&0);
                let l = summary.finding_severity_counts.get("LOW").unwrap_or(&0);
                format!("C:{} H:{} M:{} L:{}", c, h, m, l)
            } else {
                "✓".to_string()
            }
        }
        Some("IN_PROGRESS") | Some("PENDING") => "⏳".to_string(),
        Some("FAILED") => "✗".to_string(),
        Some("UNSUPPORTED_IMAGE") => "N/A".to_string(),
        Some(other) => other.chars().take(8).collect(),
    }
}

fn render_image_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let current_filters = &app.services.ecr.current_filters;

    let rows = app
        .services
        .ecr
        .images
        .items
        .iter()
        .filter(|img| {
            (filter.is_empty() || img.matches_filter(&filter)) && current_filters.matches(img)
        })
        .map(|img| {
            let pushed = img
                .image_pushed_at
                .clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());

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

            let scan = format_scan_cell(img);

            Row::new(vec![
                Cell::from(tags),
                Cell::from(img.image_digest.chars().take(12).collect::<String>()),
                Cell::from(size),
                Cell::from(scan),
                Cell::from(pushed),
            ])
            .height(1)
        });

    // Build dynamic title
    let image_count = app.services.ecr.images.len();
    let sort_field = app.services.ecr.sort_field.label();
    let sort_dir = app.services.ecr.sort_direction.label();
    let has_more = app.services.ecr.images.has_more;
    let loading_more = app.services.ecr.images.loading_more;

    let mut title_parts: Vec<String> = vec![];

    if let Some(repo) = &app.services.ecr.selected_repo_name {
        title_parts.push(format!("{} ({})", repo, image_count));
    } else {
        title_parts.push(format!("Images ({})", image_count));
    }

    let tag_status_labels = ["All", "Tagged", "Untagged"];
    let tag_status = tag_status_labels
        .get(current_filters.tag_status_index)
        .unwrap_or(&"All");
    let has_filters = !current_filters.is_empty();
    if current_filters.tag_status_index != 0 {
        title_parts.push(format!(" 🔍{}", tag_status));
    }
    if current_filters.has_local_filters() {
        title_parts.push(" 🔎".to_string());
    }

    if loading_more {
        title_parts.push(" ⏳".to_string());
    } else if has_more {
        title_parts.push(" 📥".to_string());
    }

    title_parts.push(format!(" ⇅{}{}", sort_field, sort_dir));
    title_parts.push(" [s:sort S:dir F:filter".to_string());
    if has_more && !loading_more {
        title_parts.push(" L:more".to_string());
    }
    if has_filters {
        title_parts.push(" c:clear".to_string());
    }
    title_parts.push("]".to_string());

    let title = title_parts.join("");

    render_table(
        frame,
        area,
        rows,
        &["Tags", "Digest", "Size", "Scan", "Pushed At"],
        &[
            Constraint::Min(25),
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Length(18),
            Constraint::Length(12),
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
                    Span::styled(
                        repo.repository_name.clone(),
                        Style::default().fg(THEME.selection_fg),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("URI: ", Style::default().fg(THEME.primary)),
                    Span::raw(repo.repository_uri.clone().unwrap_or_default()),
                ]),
                Line::from(vec![
                    Span::styled("ARN: ", Style::default().fg(THEME.primary)),
                    Span::raw(
                        repo.repository_arn
                            .clone()
                            .unwrap_or_else(|| "-".to_string()),
                    ),
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
        if let Some(img) = app.services.ecr.images.items.get(idx) {
            let size = if let Some(bytes) = img.image_size_in_bytes {
                if bytes > 1024 * 1024 * 1024 {
                    format!(
                        "{:.2} GB ({} bytes)",
                        bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                        bytes
                    )
                } else {
                    format!(
                        "{:.2} MB ({} bytes)",
                        bytes as f64 / (1024.0 * 1024.0),
                        bytes
                    )
                }
            } else {
                "-".to_string()
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Digest: ", Style::default().fg(THEME.primary)),
                    Span::styled(
                        img.image_digest.clone(),
                        Style::default().fg(THEME.selection_fg),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Pushed At: ", Style::default().fg(THEME.primary)),
                    Span::raw(
                        img.image_pushed_at
                            .clone()
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Size: ", Style::default().fg(THEME.primary)),
                    Span::raw(size),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Tags:",
                    Style::default().fg(THEME.secondary),
                )]),
            ];

            if img.image_tags.is_empty() {
                lines.push(Line::from("  <untagged>"));
            } else {
                for tag in &img.image_tags {
                    lines.push(Line::from(format!("  • {}", tag)));
                }
            }

            // Scan results section
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Scan Results:",
                Style::default().fg(THEME.secondary),
            )]));

            let status_str = img.scan_status.as_ref().and_then(|s| s.status.as_deref());

            match status_str {
                None => {
                    lines.push(Line::from("  No scan data available"));
                }
                Some(status) => {
                    lines.push(Line::from(vec![
                        Span::styled("  Status: ", Style::default().fg(THEME.primary)),
                        Span::raw(status.to_string()),
                    ]));

                    if let Some(summary) = &img.scan_findings_summary {
                        if let Some(completed) = &summary.scan_completed_at {
                            lines.push(Line::from(vec![
                                Span::styled("  Completed: ", Style::default().fg(THEME.primary)),
                                Span::raw(completed.clone()),
                            ]));
                        }

                        if !summary.finding_severity_counts.is_empty() {
                            lines.push(Line::from(""));
                            lines.push(Line::from(vec![Span::styled(
                                "  Vulnerabilities:",
                                Style::default().fg(THEME.primary),
                            )]));

                            let severities = [
                                "CRITICAL",
                                "HIGH",
                                "MEDIUM",
                                "LOW",
                                "INFORMATIONAL",
                                "UNDEFINED",
                            ];
                            let mut has_findings = false;

                            for severity in severities {
                                if let Some(&count) = summary.finding_severity_counts.get(severity)
                                {
                                    if count > 0 {
                                        has_findings = true;
                                        let color = match severity {
                                            "CRITICAL" => THEME.error,
                                            "HIGH" => THEME.warning,
                                            "MEDIUM" => THEME.warning,
                                            "LOW" => THEME.primary,
                                            _ => THEME.secondary,
                                        };
                                        lines.push(Line::from(vec![
                                            Span::raw("    "),
                                            Span::styled(
                                                format!("{}: ", severity),
                                                Style::default().fg(color),
                                            ),
                                            Span::raw(count.to_string()),
                                        ]));
                                    }
                                }
                            }

                            if !has_findings {
                                lines.push(Line::from(vec![Span::styled(
                                    "    No vulnerabilities found",
                                    Style::default().fg(THEME.success),
                                )]));
                            }
                        }
                    }
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
