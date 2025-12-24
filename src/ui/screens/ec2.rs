use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::App;
use crate::models::ec2::Ec2Instance;
use crate::ui::components::detail_panel::render_detail_panel_with_selection;
use crate::ui::components::table::render_table;

pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    // Render the instance list (if not in fullscreen detail mode)
    if let Some(area) = list_area {
        render_instance_list(frame, area, app);
    }
    
    // Render detail panel if visible
    if let Some(area) = detail_area {
        render_instance_details(frame, area, app);
    }
}

use crate::ui::theme::THEME;

use crate::models::Filterable;

fn render_instance_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.ec2.instances.iter()
        .filter(|i| {
            if filter.is_empty() { return true; }
            i.matches_filter(&filter)
        })
        .map(|instance| {
            let state_style = Style::default().fg(instance.state_color());

            // Use as_deref() to avoid clones where possible
            let cells = vec![
                Cell::from(instance.instance_id.as_str()),
                Cell::from(instance.name.as_deref().unwrap_or("-")),
                Cell::from(instance.state.to_string()).style(state_style),
                Cell::from(instance.instance_type.as_str()),
                Cell::from(instance.public_ip.as_deref().unwrap_or("-")),
                Cell::from(instance.launch_time.as_deref().unwrap_or("-")),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["ID", "Name", "State", "Type", "Public IP", "Launch Time"],
        &[
            Constraint::Length(20), // ID
            Constraint::Length(20), // Name
            Constraint::Length(12), // State
            Constraint::Length(12), // Type
            Constraint::Length(16), // Public IP
            Constraint::Min(20),    // Launch Time
        ],
        "EC2 Instances",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.ec2.list_state,
    );
}

fn render_instance_details(frame: &mut Frame, area: Rect, app: &App) {
    render_detail_panel_with_selection(
        frame,
        area,
        app.services.ec2.selected_instance(),
        build_instance_detail_lines,
        "Instance Details",
        "Select an instance to view details (use j/k to navigate)",
        app.detail_panel_fullscreen,
        app.detail_scroll_offset,
    );
}

fn build_instance_detail_lines(instance: &Ec2Instance) -> Vec<Line<'_>> {
    use crate::ui::components::detail_builder::DetailBuilder;
    
    let state_color = instance.state_color();
    let state_str = instance.state.to_string();

    // Build base details
    let mut builder = DetailBuilder::new()
        // Instance ID + State on same conceptual "row"
        .raw_line(Line::from(vec![
            Span::styled("Instance ID: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.instance_id.as_str()),
            Span::raw("   "),
            Span::styled("State: ", Style::default().fg(THEME.primary)),
            Span::styled(state_str, Style::default().fg(state_color)),
        ]))
        .field("Name", instance.name.as_deref().unwrap_or("-"))
        // Type + Architecture
        .raw_line(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.instance_type.as_str()),
            Span::raw("   "),
            Span::styled("Architecture: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.architecture.as_deref().unwrap_or("-")),
        ]))
        .section("Network")
        // Public + Private IP
        .raw_line(Line::from(vec![
            Span::styled("Public IP: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.public_ip.as_deref().unwrap_or("-")),
            Span::raw("   "),
            Span::styled("Private IP: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.private_ip.as_deref().unwrap_or("-")),
        ]))
        // VPC + Subnet
        .raw_line(Line::from(vec![
            Span::styled("VPC: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.vpc_id.as_deref().unwrap_or("-")),
            Span::raw("   "),
            Span::styled("Subnet: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.subnet_id.as_deref().unwrap_or("-")),
        ]))
        .field("Availability Zone", instance.availability_zone.as_deref().unwrap_or("-"));

    // Security groups (dynamic list)
    if !instance.security_groups.is_empty() {
        let mut lines = builder.build();
        lines.push(Line::from(vec![
            Span::styled("Security Groups: ", Style::default().fg(THEME.primary)),
        ]));
        for sg in &instance.security_groups {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(sg.group_id.as_str(), Style::default().fg(THEME.warning)),
                Span::raw(" ("),
                Span::raw(sg.group_name.as_str()),
                Span::raw(")"),
            ]));
        }
        builder = DetailBuilder::new().lines(lines);
    }

    builder = builder
        .section("Instance Info")
        .field("AMI ID", instance.ami_id.as_deref().unwrap_or("-"))
        // Key Name + Platform
        .raw_line(Line::from(vec![
            Span::styled("Key Name: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.key_name.as_deref().unwrap_or("-")),
            Span::raw("   "),
            Span::styled("Platform: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.platform.as_deref().unwrap_or("Linux/UNIX")),
        ]))
        .field("Monitoring", instance.monitoring_state.as_deref().unwrap_or("-"))
        .field("Launch Time", instance.launch_time.as_deref().unwrap_or("-"));

    // Tags (dynamic list)
    if !instance.tags.is_empty() {
        let mut lines = builder.build();
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("─── Tags ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]));
        for (key, value) in &instance.tags {
            lines.push(Line::from(vec![
                Span::styled(format!("{}: ", key), Style::default().fg(THEME.primary)),
                Span::raw(value.as_str()),
            ]));
        }
        return lines;
    }

    builder.build()
}
