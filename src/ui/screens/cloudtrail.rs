use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::{App, CloudTrailViewMode};
use crate::models::cloudtrail::{Trail, CloudTrailEvent};
use crate::ui::components::detail_panel::{render_detail_panel, DetailPanelConfig};
use crate::ui::components::table::render_table;
use crate::ui::theme::THEME;
use crate::app::ViewMode;

pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    use ratatui::layout::{Layout, Direction};
    
    // Render list area if provided (not in fullscreen detail mode)
    if let Some(area) = list_area {
        // Split list area for tabs
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // List
            ])
            .split(area);

        let tabs: Vec<&str> = crate::app::CloudTrailViewMode::iterator().map(|m| m.label()).collect();
        crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.services.cloudtrail.view_mode.index());

        match app.services.cloudtrail.view_mode {
            CloudTrailViewMode::Trails => render_trail_list(frame, chunks[1], app),
            CloudTrailViewMode::Events => render_event_list(frame, chunks[1], app),
        }
    }
    
    if let Some(area) = detail_area {
        match app.services.cloudtrail.view_mode {
            CloudTrailViewMode::Trails => render_trail_details(frame, area, app),
            CloudTrailViewMode::Events => render_event_details(frame, area, app),
        }
    }

    // Render filter modal if open
    if app.services.cloudtrail.filter_modal.visible {
        use crate::ui::components::filter_modal::render_filter_modal;
        render_filter_modal(
            frame,
            frame.area(),
            &app.services.cloudtrail.filter_config,
            &app.services.cloudtrail.filter_modal,
        );
    }

    if app.services.cloudtrail.show_detail_modal {
        if let Some(json) = &app.services.cloudtrail.selected_event_detail {
            let title = app.services.cloudtrail.selected_event()
                .and_then(|e| e.event_name.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("Event Details");
            render_event_detail_modal(frame, frame.area(), title, json);
        }
    }
}

fn render_event_detail_modal(frame: &mut Frame, area: Rect, title: &str, content: &str) {
    use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

    
    let popup_area = centered_rect(area, 80, 80); 
    
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary));
        
    let paragraph = Paragraph::new(content)
        .block(block) // No wrap for JSON to keep formatting? Or wrap? JSON is usually wide. 
                      // Wrap is better than cutting off.
        .wrap(Wrap { trim: false }) 
        .style(Style::default().fg(THEME.fg));
        
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    use ratatui::layout::{Layout, Direction, Constraint};
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

use crate::models::Filterable;

fn render_trail_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.cloudtrail.trails.iter()
        .filter(|t| {
            if filter.is_empty() { return true; }
            t.matches_filter(&filter)
        })
        .map(|trail| {
            let bucket = trail.s3_bucket_name.clone().unwrap_or_else(|| "-".to_string());
            let region = trail.home_region.clone().unwrap_or_else(|| "-".to_string());
            
            let cells = vec![
                Cell::from(trail.name.clone()),
                Cell::from(bucket),
                Cell::from(if trail.is_multi_region_trail { "Yes" } else { "No" }),
                Cell::from(if trail.is_organization_trail { "Yes" } else { "No" }),
                Cell::from(region),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["Trail Name", "S3 Bucket", "Multi-Region", "Organization", "Region"],
        &[
            Constraint::Length(30), // Trail Name
            Constraint::Length(30), // S3 Bucket
            Constraint::Length(12), // Multi-Region
            Constraint::Length(12), // Organization
            Constraint::Min(15),    // Region
        ],
        "CloudTrail Trails (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.cloudtrail.list_state,
    );
}

fn render_trail_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.cloudtrail.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(trail) = app.services.cloudtrail.trails.get(idx) {
            build_trail_detail_lines(trail)
        } else {
            vec![Line::from("No trail selected")]
        }
    } else {
        vec![Line::from("Select a trail to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Trail Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_trail_detail_lines(trail: &Trail) -> Vec<Line<'_>> {
    let bucket = trail.s3_bucket_name.clone().unwrap_or_else(|| "-".to_string());
    let prefix = trail.s3_key_prefix.clone().unwrap_or_else(|| "(none)".to_string());
    let region = trail.home_region.clone().unwrap_or_else(|| "-".to_string());
    let kms = trail.kms_key_id.clone().unwrap_or_else(|| "(none)".to_string());
    let cw_logs = trail.cloudwatch_logs_log_group_arn.clone().unwrap_or_else(|| "(not configured)".to_string());

    vec![
        Line::from(vec![
            Span::styled("Trail Name: ", Style::default().fg(THEME.primary)),
            Span::styled(trail.name.clone(), Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Home Region: ", Style::default().fg(THEME.primary)),
            Span::raw(region),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Storage ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("S3 Bucket: ", Style::default().fg(THEME.primary)),
            Span::raw(bucket),
        ]),
        Line::from(vec![
            Span::styled("S3 Key Prefix: ", Style::default().fg(THEME.primary)),
            Span::raw(prefix),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Configuration ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Multi-Region Trail: ", Style::default().fg(THEME.primary)),
            if trail.is_multi_region_trail {
                Span::styled("Yes ✓", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
        Line::from(vec![
            Span::styled("Organization Trail: ", Style::default().fg(THEME.primary)),
            if trail.is_organization_trail {
                Span::styled("Yes ✓", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
        Line::from(vec![
            Span::styled("Global Service Events: ", Style::default().fg(THEME.primary)),
            if trail.include_global_service_events {
                Span::styled("Yes", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
        Line::from(vec![
            Span::styled("Log File Validation: ", Style::default().fg(THEME.primary)),
            if trail.log_file_validation_enabled {
                Span::styled("Enabled ✓", Style::default().fg(THEME.success))
            } else {
                Span::styled("Disabled", Style::default().fg(THEME.warning))
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Integrations ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("KMS Key: ", Style::default().fg(THEME.primary)),
            Span::raw(kms),
        ]),
        Line::from(vec![
            Span::styled("CloudWatch Logs: ", Style::default().fg(THEME.primary)),
            Span::raw(cw_logs),
        ]),
        Line::from(vec![
            Span::styled("Custom Event Selectors: ", Style::default().fg(THEME.primary)),
            if trail.has_custom_event_selectors {
                Span::styled("Yes", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
        Line::from(vec![
            Span::styled("Insight Selectors: ", Style::default().fg(THEME.primary)),
            if trail.has_insight_selectors {
                Span::styled("Yes", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
    ]
}

fn render_event_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.cloudtrail.events.items.iter()
        .filter(|e| {
            if filter.is_empty() { return true; }
            e.matches_filter(&filter)
        })
        .map(|event| {
            let time = event.event_time.clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());
            
            let cells = vec![
                Cell::from(event.event_name.clone().unwrap_or_default()),
                Cell::from(time),
                Cell::from(event.event_source.clone().unwrap_or_default()),
                Cell::from(event.username.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(event.read_only.clone().unwrap_or_else(|| "-".to_string())),
            ];
            
            Row::new(cells).height(1)
        });

    // Build dynamic title with status info
    let event_count = app.services.cloudtrail.events.len();
    let has_filters = !app.services.cloudtrail.current_filters.is_empty();
    let has_more = app.services.cloudtrail.events.has_more;
    let loading_more = app.services.cloudtrail.events.loading_more;
    let sort_field = app.services.cloudtrail.sort_field.label();
    let sort_dir = app.services.cloudtrail.sort_direction.label();
    
    let mut title_parts = vec![format!("CloudTrail Events ({}", event_count)];
    if has_filters {
        title_parts.push(" 🔍".to_string());
    }
    if loading_more {
        title_parts.push(" ⏳".to_string());
    } else if has_more {
        title_parts.push(" 📥L".to_string());
    }
    // Show sort info
    title_parts.push(format!(") ⇅{}{}  s:sort S:dir F:filter", sort_field, sort_dir));
    let title = title_parts.join("");

    render_table(
        frame,
        area,
        rows,
        &["Event Name", "Time", "Source", "Username", "Read Only"],
        &[
            Constraint::Length(30), // Event Name
            Constraint::Length(12), // Time
            Constraint::Length(25), // Source
            Constraint::Length(20), // Username
            Constraint::Min(10),    // Read Only
        ],
        &title,
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.cloudtrail.list_state,
    );
}

fn render_event_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.cloudtrail.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(event) = app.services.cloudtrail.events.items.get(idx) {
            build_event_detail_lines(event)
        } else {
            vec![Line::from("No event selected")]
        }
    } else {
        vec![Line::from("Select an event to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Event Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_event_detail_lines(event: &CloudTrailEvent) -> Vec<Line<'_>> {
    let event_id = event.event_id.clone().unwrap_or_else(|| "-".to_string());
    let event_time = event.event_time.clone().unwrap_or_else(|| "-".to_string());
    let username = event.username.clone().unwrap_or_else(|| "-".to_string());
    let source = event.event_source.clone().unwrap_or_else(|| "-".to_string());
    let name = event.event_name.clone().unwrap_or_else(|| "-".to_string());
    
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Event Name: ", Style::default().fg(THEME.primary)),
            Span::styled(name, Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Event ID: ", Style::default().fg(THEME.primary)),
            Span::raw(event_id),
        ]),
        Line::from(vec![
            Span::styled("Time: ", Style::default().fg(THEME.primary)),
            Span::raw(event_time),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Source ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Source: ", Style::default().fg(THEME.primary)),
            Span::raw(source),
        ]),
        Line::from(vec![
            Span::styled("Username: ", Style::default().fg(THEME.primary)),
            Span::raw(username),
        ]),
    ];

    if !event.resources.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("─── Resources ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]));
        for resource in &event.resources {
            lines.push(Line::from(vec![
                Span::styled("• ", Style::default().fg(THEME.warning)),
                Span::raw(resource.clone()),
            ]));
        }
    }

    lines
}
