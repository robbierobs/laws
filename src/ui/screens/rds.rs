use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::App;
use crate::models::rds::RdsInstance;
use crate::ui::components::detail_panel::render_detail_panel_with_selection;
use crate::ui::components::table::render_table;

pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    if let Some(area) = list_area {
        render_instance_list(frame, area, app);
    }
    
    if let Some(area) = detail_area {
        render_instance_details(frame, area, app);
    }
}

use crate::ui::theme::THEME;

use crate::models::Filterable;

fn render_instance_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.rds.instances.iter()
        .filter(|i| {
            if filter.is_empty() { return true; }
            i.matches_filter(&filter)
        })
        .map(|instance| {
            let status_color = instance.state_color();
            
            // Use as_deref() to avoid clones where possible
            let cells = vec![
                Cell::from(instance.db_instance_identifier.as_str()),
                Cell::from(format!("{} {}", 
                    &instance.engine,
                    instance.engine_version.as_deref().unwrap_or("")
                )),
                Cell::from(instance.db_instance_class.as_str()),
                Cell::from(instance.status.as_str()).style(Style::default().fg(status_color)),
                Cell::from(instance.endpoint.as_deref().unwrap_or("-")),
                Cell::from(if instance.multi_az { "Yes" } else { "No" }),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["Identifier", "Engine", "Class", "Status", "Endpoint", "Multi-AZ"],
        &[
            Constraint::Length(25), // Identifier
            Constraint::Length(20), // Engine
            Constraint::Length(15), // Class
            Constraint::Length(12), // Status
            Constraint::Length(40), // Endpoint
            Constraint::Min(8),     // Multi-AZ
        ],
        "RDS Instances",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.rds.list_state,
    );
}

fn render_instance_details(frame: &mut Frame, area: Rect, app: &App) {
    render_detail_panel_with_selection(
        frame,
        area,
        app.services.rds.selected_instance(),
        build_instance_detail_lines,
        "RDS Instance Details",
        "Select an RDS instance to view details (use j/k to navigate)",
        app.detail_panel_fullscreen,
        app.detail_scroll_offset,
    );
}

fn build_instance_detail_lines(instance: &RdsInstance) -> Vec<Line<'_>> {
    use crate::ui::components::detail_builder::DetailBuilder;
    
    let status_color = instance.state_color();
    let engine_str = format!("{} {}", 
        &instance.engine,
        instance.engine_version.as_deref().unwrap_or("")
    );
    let storage_str = instance.allocated_storage
        .map(|s| format!("{} GB", s))
        .unwrap_or_else(|| "-".to_string());
    let iops_str = instance.iops
        .map(|i| i.to_string())
        .unwrap_or_else(|| "-".to_string());
    let backup_retention_str = instance.backup_retention_period
        .map(|d| format!("{} days", d))
        .unwrap_or_else(|| "-".to_string());

    let mut builder = DetailBuilder::new()
        // Identifier + Status
        .raw_line(Line::from(vec![
            Span::styled("Identifier: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.db_instance_identifier.as_str()),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(instance.status.as_str(), Style::default().fg(status_color)),
        ]))
        // Engine + Class
        .raw_line(Line::from(vec![
            Span::styled("Engine: ", Style::default().fg(THEME.primary)),
            Span::raw(engine_str),
            Span::raw("   "),
            Span::styled("Class: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.db_instance_class.as_str()),
        ]))
        .raw_line(Line::from(vec![
            Span::styled("Endpoint: ", Style::default().fg(THEME.primary)),
            Span::styled(instance.endpoint.as_deref().unwrap_or("-"), Style::default().fg(THEME.selection_fg)),
        ]))
        .field("Master User", instance.master_username.as_deref().unwrap_or("-"))
        .section("Storage")
        // Storage + Type + IOPS
        .raw_line(Line::from(vec![
            Span::styled("Allocated Storage: ", Style::default().fg(THEME.primary)),
            Span::raw(storage_str),
            Span::raw("   "),
            Span::styled("Type: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.storage_type.as_deref().unwrap_or("-")),
            Span::raw("   "),
            Span::styled("IOPS: ", Style::default().fg(THEME.primary)),
            Span::raw(iops_str),
        ]))
        .bool_field("Encrypted", instance.storage_encrypted, "Yes ✓", "No")
        .section("Network")
        // AZ + Multi-AZ
        .raw_line(Line::from(vec![
            Span::styled("Availability Zone: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.availability_zone.as_deref().unwrap_or("-")),
            Span::raw("   "),
            Span::styled("Multi-AZ: ", Style::default().fg(THEME.primary)),
            if instance.multi_az {
                Span::styled("Yes ✓", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.warning))
            },
        ]))
        // VPC + Subnet
        .raw_line(Line::from(vec![
            Span::styled("VPC: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.vpc_id.as_deref().unwrap_or("-")),
            Span::raw("   "),
            Span::styled("Subnet Group: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.db_subnet_group.as_deref().unwrap_or("-")),
        ]))
        .raw_line(Line::from(vec![
            Span::styled("Publicly Accessible: ", Style::default().fg(THEME.primary)),
            if instance.publicly_accessible {
                Span::styled("Yes", Style::default().fg(THEME.warning))
            } else {
                Span::styled("No ✓", Style::default().fg(THEME.success))
            },
        ]));

    // Security groups
    if !instance.security_groups.is_empty() {
        builder = builder.raw_line(Line::from(vec![
            Span::styled("Security Groups: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.security_groups.join(", ")),
        ]));
    }

    builder = builder
        .section("Backup & Maintenance")
        // Backup window + retention
        .raw_line(Line::from(vec![
            Span::styled("Backup Window: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.preferred_backup_window.as_deref().unwrap_or("-")),
            Span::raw("   "),
            Span::styled("Retention: ", Style::default().fg(THEME.primary)),
            Span::raw(backup_retention_str),
        ]))
        .field("Maintenance Window", instance.preferred_maintenance_window.as_deref().unwrap_or("-"))
        .bool_field("Auto Minor Version Upgrade", instance.auto_minor_version_upgrade, "Yes", "No")
        .raw_line(Line::from(vec![
            Span::styled("Deletion Protection: ", Style::default().fg(THEME.primary)),
            if instance.deletion_protection {
                Span::styled("Enabled ✓", Style::default().fg(THEME.success))
            } else {
                Span::styled("Disabled", Style::default().fg(THEME.warning))
            },
        ]))
        // License + Created
        .raw_line(Line::from(vec![
            Span::styled("License: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.license_model.as_deref().unwrap_or("-")),
            Span::raw("   "),
            Span::styled("Created: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.created_time.as_deref().unwrap_or("-")),
        ]));

    // Tags
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
