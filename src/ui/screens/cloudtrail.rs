use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::{App, CloudTrailViewMode};
use crate::models::cloudtrail::{Trail, CloudTrailEvent};
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    use ratatui::layout::{Layout, Direction};
    
    // Split list area for tabs
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // List
        ])
        .split(list_area);

    let tabs = ["Trails", "Events"];
    crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.cloudtrail_view_mode.to_index());

    match app.cloudtrail_view_mode {
        CloudTrailViewMode::Trails => {
            render_trail_list(frame, chunks[1], app);
            if let Some(area) = detail_area {
                render_trail_details(frame, area, app);
            }
        }
        CloudTrailViewMode::Events => {
            render_event_list(frame, chunks[1], app);
            if let Some(area) = detail_area {
                render_event_details(frame, area, app);
            }
        }
    }
}

fn render_trail_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Trail Name", "S3 Bucket", "Multi-Region", "Organization", "Region"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.cloudtrail_trails.iter()
        .filter(|t| {
            if filter.is_empty() { return true; }
            let name = t.name.to_lowercase();
            let bucket = t.s3_bucket_name.as_deref().unwrap_or("").to_lowercase();
            name.contains(&filter) || bucket.contains(&filter)
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

    let block = Block::default()
        .borders(Borders::ALL)
        .title("CloudTrail Trails (v/h/l to switch view)")
        .title_style(Style::default().fg(THEME.primary))
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(THEME.secondary)
        } else {
            Style::default().fg(THEME.border)
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(30), // Trail Name
            Constraint::Length(30), // S3 Bucket
            Constraint::Length(12), // Multi-Region
            Constraint::Length(12), // Organization
            Constraint::Min(15),    // Region
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(t, area, &mut app.cloudtrail_list_state);
}

fn render_trail_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.cloudtrail_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(trail) = app.cloudtrail_trails.get(idx) {
            build_trail_detail_lines(trail)
        } else {
            vec![Line::from("No trail selected")]
        }
    } else {
        vec![Line::from("Select a trail to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Trail Details")
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)));
    
    frame.render_widget(paragraph, area);
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
    let header_cells = ["Event Name", "Time", "Source", "Username", "Read Only"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.cloudtrail_events.iter()
        .filter(|e| {
            if filter.is_empty() { return true; }
            let name = e.event_name.as_deref().unwrap_or("").to_lowercase();
            let source = e.event_source.as_deref().unwrap_or("").to_lowercase();
            let user = e.username.as_deref().unwrap_or("").to_lowercase();
            name.contains(&filter) || source.contains(&filter) || user.contains(&filter)
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

    let block = Block::default()
        .borders(Borders::ALL)
        .title("CloudTrail Events (v/h/l to switch view)")
        .title_style(Style::default().fg(THEME.primary))
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(THEME.secondary)
        } else {
            Style::default().fg(THEME.border)
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(30), // Event Name
            Constraint::Length(12), // Time
            Constraint::Length(25), // Source
            Constraint::Length(20), // Username
            Constraint::Min(10),    // Read Only
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(t, area, &mut app.cloudtrail_list_state);
}

fn render_event_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.cloudtrail_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(event) = app.cloudtrail_events.get(idx) {
            build_event_detail_lines(event)
        } else {
            vec![Line::from("No event selected")]
        }
    } else {
        vec![Line::from("Select an event to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Event Details")
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)));
    
    frame.render_widget(paragraph, area);
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
