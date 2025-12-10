use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::ec2::{Ec2Instance, InstanceState};

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    // Render the instance list
    render_instance_list(frame, list_area, app);
    
    // Render detail panel if visible
    if let Some(area) = detail_area {
        render_instance_details(frame, area, app);
    }
}

fn render_instance_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["ID", "Name", "State", "Type", "Public IP", "Launch Time"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.ec2_instances.iter()
        .filter(|i| {
            if filter.is_empty() { return true; }
            let name = i.name.as_deref().unwrap_or("").to_lowercase();
            let id = i.instance_id.to_lowercase();
            let ip = i.public_ip.as_deref().unwrap_or("").to_lowercase();
            name.contains(&filter) || id.contains(&filter) || ip.contains(&filter)
        })
        .map(|instance| {
        let state_style = match instance.state {
            InstanceState::Running => Style::default().fg(Color::Green),
            InstanceState::Stopped => Style::default().fg(Color::Red),
            InstanceState::Pending | InstanceState::Stopping => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::Gray),
        };

        let cells = vec![
            Cell::from(instance.instance_id.clone()),
            Cell::from(instance.name.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(format!("{:?}", instance.state)).style(state_style),
            Cell::from(instance.instance_type.clone()),
            Cell::from(instance.public_ip.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(instance.launch_time.clone().unwrap_or_else(|| "-".to_string())),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("EC2 Instances")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(20), // ID
            Constraint::Length(20), // Name
            Constraint::Length(12), // State
            Constraint::Length(12), // Type
            Constraint::Length(16), // Public IP
            Constraint::Min(20),    // Launch Time
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    // Use app state for selection
    frame.render_stateful_widget(t, area, &mut app.ec2_list_state);
}

fn render_instance_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.ec2_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(instance) = app.ec2_instances.get(idx) {
            build_instance_detail_lines(instance)
        } else {
            vec![Line::from("No instance selected")]
        }
    } else {
        vec![Line::from("Select an instance to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Instance Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_instance_detail_lines(instance: &Ec2Instance) -> Vec<Line<'_>> {
    let state_color = match instance.state {
        InstanceState::Running => Color::Green,
        InstanceState::Stopped => Color::Red,
        InstanceState::Pending | InstanceState::Stopping => Color::Yellow,
        _ => Color::Gray,
    };

    let state_str = format!("{:?}", instance.state);
    let name_str = instance.name.clone().unwrap_or_else(|| "-".to_string());
    let arch_str = instance.architecture.clone().unwrap_or_else(|| "-".to_string());
    let pub_ip_str = instance.public_ip.clone().unwrap_or_else(|| "-".to_string());
    let priv_ip_str = instance.private_ip.clone().unwrap_or_else(|| "-".to_string());
    let vpc_str = instance.vpc_id.clone().unwrap_or_else(|| "-".to_string());
    let subnet_str = instance.subnet_id.clone().unwrap_or_else(|| "-".to_string());
    let az_str = instance.availability_zone.clone().unwrap_or_else(|| "-".to_string());
    let ami_str = instance.ami_id.clone().unwrap_or_else(|| "-".to_string());
    let key_str = instance.key_name.clone().unwrap_or_else(|| "-".to_string());
    let platform_str = instance.platform.clone().unwrap_or_else(|| "Linux/UNIX".to_string());
    let monitoring_str = instance.monitoring_state.clone().unwrap_or_else(|| "-".to_string());
    let launch_str = instance.launch_time.clone().unwrap_or_else(|| "-".to_string());

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Instance ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(instance.instance_id.clone()),
            Span::raw("   "),
            Span::styled("State: ", Style::default().fg(Color::Cyan)),
            Span::styled(state_str, Style::default().fg(state_color)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::raw(name_str),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Cyan)),
            Span::raw(instance.instance_type.clone()),
            Span::raw("   "),
            Span::styled("Architecture: ", Style::default().fg(Color::Cyan)),
            Span::raw(arch_str),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Network ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Public IP: ", Style::default().fg(Color::Cyan)),
            Span::raw(pub_ip_str),
            Span::raw("   "),
            Span::styled("Private IP: ", Style::default().fg(Color::Cyan)),
            Span::raw(priv_ip_str),
        ]),
        Line::from(vec![
            Span::styled("VPC: ", Style::default().fg(Color::Cyan)),
            Span::raw(vpc_str),
            Span::raw("   "),
            Span::styled("Subnet: ", Style::default().fg(Color::Cyan)),
            Span::raw(subnet_str),
        ]),
        Line::from(vec![
            Span::styled("Availability Zone: ", Style::default().fg(Color::Cyan)),
            Span::raw(az_str),
        ]),
    ];

    // Security groups
    if !instance.security_groups.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Security Groups: ", Style::default().fg(Color::Cyan)),
        ]));
        for sg in &instance.security_groups {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(sg.group_id.clone(), Style::default().fg(Color::Yellow)),
                Span::raw(" ("),
                Span::raw(sg.group_name.clone()),
                Span::raw(")"),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("─── Instance Info ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("AMI ID: ", Style::default().fg(Color::Cyan)),
        Span::raw(ami_str),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Key Name: ", Style::default().fg(Color::Cyan)),
        Span::raw(key_str),
        Span::raw("   "),
        Span::styled("Platform: ", Style::default().fg(Color::Cyan)),
        Span::raw(platform_str),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Monitoring: ", Style::default().fg(Color::Cyan)),
        Span::raw(monitoring_str),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Launch Time: ", Style::default().fg(Color::Cyan)),
        Span::raw(launch_str),
    ]));

    lines
}
