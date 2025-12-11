use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::rds::RdsInstance;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    render_instance_list(frame, list_area, app);
    
    if let Some(area) = detail_area {
        render_instance_details(frame, area, app);
    }
}

use crate::ui::theme::THEME;

fn render_instance_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Identifier", "Engine", "Class", "Status", "Endpoint", "Multi-AZ"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.services.rds.instances.iter()
        .filter(|i| {
            if filter.is_empty() { return true; }
            let id = i.db_instance_identifier.to_lowercase();
            let engine = i.engine.to_lowercase();
            id.contains(&filter) || engine.contains(&filter)
        })
        .map(|instance| {
        let status_color = instance.status_color();
        
        let cells = vec![
            Cell::from(instance.db_instance_identifier.clone()),
            Cell::from(format!("{} {}", 
                instance.engine.clone(),
                instance.engine_version.clone().unwrap_or_default()
            )),
            Cell::from(instance.db_instance_class.clone()),
            Cell::from(instance.status.clone()).style(Style::default().fg(status_color)),
            Cell::from(instance.endpoint.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(if instance.multi_az { "Yes" } else { "No" }),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("RDS Instances")
        .title_style(Style::default().fg(THEME.primary))
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(THEME.secondary)
        } else {
            Style::default().fg(THEME.border)
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(25), // Identifier
            Constraint::Length(20), // Engine
            Constraint::Length(15), // Class
            Constraint::Length(12), // Status
            Constraint::Length(40), // Endpoint
            Constraint::Min(8),     // Multi-AZ
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(t, area, &mut app.services.rds.list_state);
}

fn render_instance_details(frame: &mut Frame, area: Rect, app: &App) {
    let content: Vec<Line> = if let Some(instance) = app.services.rds.selected_instance() {
        build_instance_detail_lines(instance)
    } else {
        vec![Line::from("Select an RDS instance to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("RDS Instance Details")
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)));
    
    frame.render_widget(paragraph, area);
}

fn build_instance_detail_lines(instance: &RdsInstance) -> Vec<Line<'_>> {
    let status_color = instance.status_color();
    
    let engine_str = format!("{} {}", 
        instance.engine.clone(),
        instance.engine_version.clone().unwrap_or_default()
    );
    let endpoint_str = instance.endpoint.clone().unwrap_or_else(|| "-".to_string());
    let master_str = instance.master_username.clone().unwrap_or_else(|| "-".to_string());
    let storage_str = instance.allocated_storage
        .map(|s| format!("{} GB", s))
        .unwrap_or_else(|| "-".to_string());
    let storage_type_str = instance.storage_type.clone().unwrap_or_else(|| "-".to_string());
    let iops_str = instance.iops
        .map(|i| i.to_string())
        .unwrap_or_else(|| "-".to_string());
    let az_str = instance.availability_zone.clone().unwrap_or_else(|| "-".to_string());
    let vpc_str = instance.vpc_id.clone().unwrap_or_else(|| "-".to_string());
    let subnet_str = instance.db_subnet_group.clone().unwrap_or_else(|| "-".to_string());
    let backup_window_str = instance.preferred_backup_window.clone().unwrap_or_else(|| "-".to_string());
    let backup_retention_str = instance.backup_retention_period
        .map(|d| format!("{} days", d))
        .unwrap_or_else(|| "-".to_string());
    let maint_window_str = instance.preferred_maintenance_window.clone().unwrap_or_else(|| "-".to_string());
    let created_str = instance.created_time.clone().unwrap_or_else(|| "-".to_string());
    let license_str = instance.license_model.clone().unwrap_or_else(|| "-".to_string());

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Identifier: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.db_instance_identifier.clone()),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(instance.status.clone(), Style::default().fg(status_color)),
        ]),
        Line::from(vec![
            Span::styled("Engine: ", Style::default().fg(THEME.primary)),
            Span::raw(engine_str),
            Span::raw("   "),
            Span::styled("Class: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.db_instance_class.clone()),
        ]),
        Line::from(vec![
            Span::styled("Endpoint: ", Style::default().fg(THEME.primary)),
            Span::styled(endpoint_str, Style::default().fg(THEME.selection_fg)),
        ]),
        Line::from(vec![
            Span::styled("Master User: ", Style::default().fg(THEME.primary)),
            Span::raw(master_str),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Storage ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Allocated Storage: ", Style::default().fg(THEME.primary)),
            Span::raw(storage_str),
            Span::raw("   "),
            Span::styled("Type: ", Style::default().fg(THEME.primary)),
            Span::raw(storage_type_str),
            Span::raw("   "),
            Span::styled("IOPS: ", Style::default().fg(THEME.primary)),
            Span::raw(iops_str),
        ]),
        Line::from(vec![
            Span::styled("Encrypted: ", Style::default().fg(THEME.primary)),
            if instance.storage_encrypted {
                Span::styled("Yes ✓", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.error))
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Network ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Availability Zone: ", Style::default().fg(THEME.primary)),
            Span::raw(az_str),
            Span::raw("   "),
            Span::styled("Multi-AZ: ", Style::default().fg(THEME.primary)),
            if instance.multi_az {
                Span::styled("Yes ✓", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.warning))
            },
        ]),
        Line::from(vec![
            Span::styled("VPC: ", Style::default().fg(THEME.primary)),
            Span::raw(vpc_str),
            Span::raw("   "),
            Span::styled("Subnet Group: ", Style::default().fg(THEME.primary)),
            Span::raw(subnet_str),
        ]),
        Line::from(vec![
            Span::styled("Publicly Accessible: ", Style::default().fg(THEME.primary)),
            if instance.publicly_accessible {
                Span::styled("Yes", Style::default().fg(THEME.warning))
            } else {
                Span::styled("No ✓", Style::default().fg(THEME.success))
            },
        ]),
    ];

    // Security groups
    if !instance.security_groups.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Security Groups: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.security_groups.join(", ")),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("─── Backup & Maintenance ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Backup Window: ", Style::default().fg(THEME.primary)),
        Span::raw(backup_window_str),
        Span::raw("   "),
        Span::styled("Retention: ", Style::default().fg(THEME.primary)),
        Span::raw(backup_retention_str),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Maintenance Window: ", Style::default().fg(THEME.primary)),
        Span::raw(maint_window_str),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Auto Minor Version Upgrade: ", Style::default().fg(THEME.primary)),
        if instance.auto_minor_version_upgrade {
            Span::styled("Yes", Style::default().fg(THEME.success))
        } else {
            Span::styled("No", Style::default().fg(THEME.muted))
        },
    ]));
    lines.push(Line::from(vec![
        Span::styled("Deletion Protection: ", Style::default().fg(THEME.primary)),
        if instance.deletion_protection {
            Span::styled("Enabled ✓", Style::default().fg(THEME.success))
        } else {
            Span::styled("Disabled", Style::default().fg(THEME.warning))
        },
    ]));
    lines.push(Line::from(vec![
        Span::styled("License: ", Style::default().fg(THEME.primary)),
        Span::raw(license_str),
        Span::raw("   "),
        Span::styled("Created: ", Style::default().fg(THEME.primary)),
        Span::raw(created_str),
    ]));

    lines
}
