use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::App;
use crate::ui::components::detail_panel::{render_detail_panel, DetailPanelConfig};
use crate::ui::components::table::render_table;

pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    if let Some(bucket_name) = app.services.s3.current_bucket.clone() {
        // Viewing objects mode
        if let Some(area) = list_area {
            render_objects(frame, area, app, &bucket_name);
        }
        if let Some(area) = detail_area {
            render_object_details(frame, area, app);
        }
    } else {
        // Viewing buckets mode
        if let Some(area) = list_area {
            render_buckets(frame, area, app);
        }
        if let Some(area) = detail_area {
            render_bucket_details(frame, area, app);
        }
    }
}

use crate::ui::theme::THEME;

use crate::models::Filterable;

fn render_buckets(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.s3.buckets.iter()
        .filter(|b| {
            if filter.is_empty() { return true; }
            b.matches_filter(&filter)
        })
        .map(|bucket| {
            let cells = vec![
                Cell::from(bucket.name.clone()),
                Cell::from(bucket.creation_date.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(bucket.region.clone().unwrap_or_else(|| "-".to_string())),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["Name", "Creation Date", "Region"],
        &[
            Constraint::Length(40), // Name
            Constraint::Length(30), // Creation Date
            Constraint::Min(10),    // Region
        ],
        "S3 Buckets",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.s3.list_state,
    );
}

fn render_bucket_details(frame: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(bucket) = app.services.s3.selected_bucket() {
        build_bucket_detail_lines(bucket, app)
    } else {
        vec![Line::from("Select a bucket to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Bucket Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_bucket_detail_lines(bucket: &crate::models::s3::S3Bucket, app: &App) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Bucket Name: ", Style::default().fg(THEME.primary)),
            Span::raw(bucket.name.clone()),
        ]),
        Line::from(vec![
            Span::styled("Creation Date: ", Style::default().fg(THEME.primary)),
            Span::raw(bucket.creation_date.clone().unwrap_or_else(|| "-".to_string())),
        ]),
        Line::from(vec![
            Span::styled("Region: ", Style::default().fg(THEME.primary)),
            Span::raw(bucket.region.clone().unwrap_or_else(|| "-".to_string())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Configuration ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
    ];

    // Check if we have cached details for this bucket
    if let Some(details) = app.services.s3.bucket_details.get(&bucket.name) {
        if details.loading {
            lines.push(Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(THEME.warning)),
                Span::raw("Loading bucket details..."),
            ]));
        } else {
            // Versioning
            let versioning_text = match details.versioning_enabled {
                Some(true) => Span::styled("Enabled", Style::default().fg(THEME.success)),
                Some(false) => Span::styled("Disabled", Style::default().fg(THEME.error)),
                None => Span::styled("Unknown", Style::default().fg(THEME.muted)),
            };
            lines.push(Line::from(vec![
                Span::styled("Versioning: ", Style::default().fg(THEME.primary)),
                versioning_text,
            ]));

            // Encryption
            let encryption_text = details.encryption.clone()
                .unwrap_or_else(|| "None/Unknown".to_string());
            lines.push(Line::from(vec![
                Span::styled("Encryption: ", Style::default().fg(THEME.primary)),
                Span::raw(encryption_text),
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("─── Statistics ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
            ]));

            // Object count
            let object_count_text = details.object_count
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string());
            lines.push(Line::from(vec![
                Span::styled("Object Count: ", Style::default().fg(THEME.primary)),
                Span::raw(object_count_text),
            ]));

            // Total size
            let total_size_text = details.total_size
                .map(|s| format_size(s))
                .unwrap_or_else(|| "-".to_string());
            lines.push(Line::from(vec![
                Span::styled("Total Size: ", Style::default().fg(THEME.primary)),
                Span::raw(total_size_text),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(THEME.warning)),
            Span::raw("Loading bucket details..."),
        ]));
    }

    lines
}

fn render_objects(frame: &mut Frame, area: Rect, app: &mut App, bucket_name: &str) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.s3.objects.iter()
        .filter(|o| {
            if filter.is_empty() { return true; }
            o.matches_filter(&filter)
        })
        .map(|obj| {
            let cells = vec![
                Cell::from(obj.key.clone()),
                Cell::from(format_size(obj.size)),
                Cell::from(obj.last_modified.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(obj.storage_class.clone().unwrap_or_else(|| "STANDARD".to_string())),
            ];
            
            Row::new(cells).height(1)
        });

    let title = format!("Objects in {} (Esc: Back, D: Delete)", bucket_name);

    render_table(
        frame,
        area,
        rows,
        &["Key", "Size", "Last Modified", "Storage Class"],
        &[
            Constraint::Length(50), // Key
            Constraint::Length(15), // Size
            Constraint::Length(25), // Last Modified
            Constraint::Min(15),    // Storage Class
        ],
        &title,
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.s3.object_list_state,
    );
}

fn render_object_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.s3.object_list_state.selected();
    
    let content = if let Some(idx) = selected {
        if let Some(object) = app.services.s3.objects.get(idx) {
            build_object_detail_lines(object, app.services.s3.current_bucket.as_deref())
        } else {
            vec![Line::from("No object selected")]
        }
    } else {
        vec![Line::from("Select an object to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Object Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_object_detail_lines(object: &crate::models::s3::S3Object, bucket_name: Option<&str>) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Key: ", Style::default().fg(THEME.primary)),
            Span::raw(object.key.clone()),
        ]),
    ];

    if let Some(bucket) = bucket_name {
        lines.push(Line::from(vec![
            Span::styled("S3 URI: ", Style::default().fg(THEME.primary)),
            Span::styled(format!("s3://{}/{}", bucket, object.key), Style::default().fg(THEME.selection_fg)),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled("Size: ", Style::default().fg(THEME.primary)),
        Span::raw(format_size(object.size)),
        Span::raw(format!(" ({} bytes)", object.size)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Last Modified: ", Style::default().fg(THEME.primary)),
        Span::raw(object.last_modified.clone().unwrap_or_else(|| "-".to_string())),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Storage Class: ", Style::default().fg(THEME.primary)),
        Span::raw(object.storage_class.clone().unwrap_or_else(|| "STANDARD".to_string())),
    ]));
    
    if let Some(etag) = &object.etag {
        lines.push(Line::from(vec![
            Span::styled("ETag: ", Style::default().fg(THEME.primary)),
            Span::raw(etag.clone()),
        ]));
    }

    lines
}

fn format_size(size: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}
