use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::cloudtrail::Trail;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    render_trail_list(frame, list_area, app);
    
    if let Some(area) = detail_area {
        render_trail_details(frame, area, app);
    }
}

fn render_trail_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Trail Name", "S3 Bucket", "Multi-Region", "Organization", "Region"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows = app.cloudtrail_trails.iter().map(|trail| {
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
    .block(Block::default().borders(Borders::ALL).title("CloudTrail Trails"))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

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
        .block(Block::default().borders(Borders::ALL).title("Trail Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_trail_detail_lines(trail: &Trail) -> Vec<Line<'_>> {
    let bucket = trail.s3_bucket_name.clone().unwrap_or_else(|| "-".to_string());
    let prefix = trail.s3_key_prefix.clone().unwrap_or_else(|| "(none)".to_string());
    let region = trail.home_region.clone().unwrap_or_else(|| "-".to_string());
    let arn = trail.trail_arn.clone().unwrap_or_else(|| "-".to_string());
    let kms = trail.kms_key_id.clone().unwrap_or_else(|| "(none)".to_string());
    let cw_logs = trail.cloudwatch_logs_log_group_arn.clone().unwrap_or_else(|| "(not configured)".to_string());

    vec![
        Line::from(vec![
            Span::styled("Trail Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(trail.name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Home Region: ", Style::default().fg(Color::Cyan)),
            Span::raw(region),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Storage ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("S3 Bucket: ", Style::default().fg(Color::Cyan)),
            Span::raw(bucket),
        ]),
        Line::from(vec![
            Span::styled("S3 Key Prefix: ", Style::default().fg(Color::Cyan)),
            Span::raw(prefix),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Configuration ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Multi-Region Trail: ", Style::default().fg(Color::Cyan)),
            if trail.is_multi_region_trail {
                Span::styled("Yes ✓", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
        Line::from(vec![
            Span::styled("Organization Trail: ", Style::default().fg(Color::Cyan)),
            if trail.is_organization_trail {
                Span::styled("Yes ✓", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
        Line::from(vec![
            Span::styled("Global Service Events: ", Style::default().fg(Color::Cyan)),
            if trail.include_global_service_events {
                Span::styled("Yes", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
        Line::from(vec![
            Span::styled("Log File Validation: ", Style::default().fg(Color::Cyan)),
            if trail.log_file_validation_enabled {
                Span::styled("Enabled ✓", Style::default().fg(Color::Green))
            } else {
                Span::styled("Disabled", Style::default().fg(Color::Yellow))
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Integrations ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("KMS Key: ", Style::default().fg(Color::Cyan)),
            Span::raw(kms),
        ]),
        Line::from(vec![
            Span::styled("CloudWatch Logs: ", Style::default().fg(Color::Cyan)),
            Span::raw(cw_logs),
        ]),
        Line::from(vec![
            Span::styled("Custom Event Selectors: ", Style::default().fg(Color::Cyan)),
            if trail.has_custom_event_selectors {
                Span::styled("Yes", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
        Line::from(vec![
            Span::styled("Insight Selectors: ", Style::default().fg(Color::Cyan)),
            if trail.has_insight_selectors {
                Span::styled("Yes", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
    ]
}
