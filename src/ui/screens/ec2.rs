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
    let state_color = instance.state_color();

    // Use references where possible, only allocate when necessary
    let state_str = instance.state.to_string();
    let name_str = instance.name.as_deref().unwrap_or("-");
    let arch_str = instance.architecture.as_deref().unwrap_or("-");
    let pub_ip_str = instance.public_ip.as_deref().unwrap_or("-");
    let priv_ip_str = instance.private_ip.as_deref().unwrap_or("-");
    let vpc_str = instance.vpc_id.as_deref().unwrap_or("-");
    let subnet_str = instance.subnet_id.as_deref().unwrap_or("-");
    let az_str = instance.availability_zone.as_deref().unwrap_or("-");
    let ami_str = instance.ami_id.as_deref().unwrap_or("-");
    let key_str = instance.key_name.as_deref().unwrap_or("-");
    let platform_str = instance.platform.as_deref().unwrap_or("Linux/UNIX");
    let monitoring_str = instance.monitoring_state.as_deref().unwrap_or("-");
    let launch_str = instance.launch_time.as_deref().unwrap_or("-");

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Instance ID: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.instance_id.clone()),
            Span::raw("   "),
            Span::styled("State: ", Style::default().fg(THEME.primary)),
            Span::styled(state_str, Style::default().fg(state_color)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(THEME.primary)),
            Span::raw(name_str),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(THEME.primary)),
            Span::raw(instance.instance_type.clone()),
            Span::raw("   "),
            Span::styled("Architecture: ", Style::default().fg(THEME.primary)),
            Span::raw(arch_str),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Network ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Public IP: ", Style::default().fg(THEME.primary)),
            Span::raw(pub_ip_str),
            Span::raw("   "),
            Span::styled("Private IP: ", Style::default().fg(THEME.primary)),
            Span::raw(priv_ip_str),
        ]),
        Line::from(vec![
            Span::styled("VPC: ", Style::default().fg(THEME.primary)),
            Span::raw(vpc_str),
            Span::raw("   "),
            Span::styled("Subnet: ", Style::default().fg(THEME.primary)),
            Span::raw(subnet_str),
        ]),
        Line::from(vec![
            Span::styled("Availability Zone: ", Style::default().fg(THEME.primary)),
            Span::raw(az_str),
        ]),
    ];

    // Security groups
    if !instance.security_groups.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Security Groups: ", Style::default().fg(THEME.primary)),
        ]));
        for sg in &instance.security_groups {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(sg.group_id.clone(), Style::default().fg(THEME.warning)),
                Span::raw(" ("),
                Span::raw(sg.group_name.clone()),
                Span::raw(")"),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("─── Instance Info ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("AMI ID: ", Style::default().fg(THEME.primary)),
        Span::raw(ami_str),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Key Name: ", Style::default().fg(THEME.primary)),
        Span::raw(key_str),
        Span::raw("   "),
        Span::styled("Platform: ", Style::default().fg(THEME.primary)),
        Span::raw(platform_str),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Monitoring: ", Style::default().fg(THEME.primary)),
        Span::raw(monitoring_str),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Launch Time: ", Style::default().fg(THEME.primary)),
        Span::raw(launch_str),
    ]));

    lines
}
