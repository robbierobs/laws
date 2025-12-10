use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::vpc::{Vpc, Subnet, SecurityGroup, SecurityGroupRule};

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

    if app.vpc_view_mode == 3 {
        let title = format!("Rules for {}", app.selected_sg_id.as_deref().unwrap_or("Unknown"));
        let block = Block::default().borders(Borders::ALL).title(title).style(Style::default().fg(Color::Magenta));
        frame.render_widget(block, chunks[0]);
    } else {
        let tabs = ["VPCs", "Subnets", "Security Groups"];
        crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.vpc_view_mode as usize);
    }

    match app.vpc_view_mode {
        0 => render_vpc_list(frame, chunks[1], app),
        1 => render_subnet_list(frame, chunks[1], app),
        2 => render_security_group_list(frame, chunks[1], app),
        3 => render_sg_rules_list(frame, chunks[1], app),
        _ => render_vpc_list(frame, chunks[1], app),
    }
    
    if let Some(area) = detail_area {
        match app.vpc_view_mode {
            0 => render_vpc_details(frame, area, app),
            1 => render_subnet_details(frame, area, app),
            2 => render_security_group_details(frame, area, app),
            3 => render_sg_rule_details(frame, area, app),
            _ => render_vpc_details(frame, area, app),
        }
    }
}

fn render_vpc_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["VPC ID", "Name", "CIDR Block", "State", "Default", "Tenancy"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.vpcs.iter()
        .filter(|v| {
            if filter.is_empty() { return true; }
            let id = v.vpc_id.to_lowercase();
            let name = v.name.as_deref().unwrap_or("").to_lowercase();
            let cidr = v.cidr_block.as_deref().unwrap_or("").to_lowercase();
            id.contains(&filter) || name.contains(&filter) || cidr.contains(&filter)
        })
        .map(|vpc| {
        let state_color = vpc.state_color();
        let name = vpc.name.clone().unwrap_or_else(|| "-".to_string());
        let cidr = vpc.cidr_block.clone().unwrap_or_else(|| "-".to_string());
        let tenancy = vpc.instance_tenancy.clone().unwrap_or_else(|| "default".to_string());
        
        let cells = vec![
            Cell::from(vpc.vpc_id.clone()),
            Cell::from(name),
            Cell::from(cidr),
            Cell::from(vpc.state.clone()).style(Style::default().fg(state_color)),
            Cell::from(if vpc.is_default { "Yes" } else { "No" }),
            Cell::from(tenancy),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("VPCs (Press 'v' to switch view)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(25), // VPC ID
            Constraint::Length(20), // Name
            Constraint::Length(18), // CIDR Block
            Constraint::Length(12), // State
            Constraint::Length(8),  // Default
            Constraint::Min(10),    // Tenancy
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.vpc_list_state);
}

fn render_subnet_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Subnet ID", "Name", "VPC ID", "CIDR Block", "AZ", "Available IPs"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.subnets.iter()
        .filter(|s| {
            if filter.is_empty() { return true; }
            let id = s.subnet_id.to_lowercase();
            let name = s.name.as_deref().unwrap_or("").to_lowercase();
            let vpc_id = s.vpc_id.as_deref().unwrap_or("").to_lowercase();
            id.contains(&filter) || name.contains(&filter) || vpc_id.contains(&filter)
        })
        .map(|subnet| {
        let name = subnet.name.clone().unwrap_or_else(|| "-".to_string());
        let vpc_id = subnet.vpc_id.clone().unwrap_or_else(|| "-".to_string());
        let cidr = subnet.cidr_block.clone().unwrap_or_else(|| "-".to_string());
        let az = subnet.availability_zone.clone().unwrap_or_else(|| "-".to_string());
        let ips = subnet.available_ip_count.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
        
        let cells = vec![
            Cell::from(subnet.subnet_id.clone()),
            Cell::from(name),
            Cell::from(vpc_id),
            Cell::from(cidr),
            Cell::from(az),
            Cell::from(ips),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Subnets (Press 'v' to switch view)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(25), // Subnet ID
            Constraint::Length(20), // Name
            Constraint::Length(25), // VPC ID
            Constraint::Length(18), // CIDR Block
            Constraint::Length(15), // AZ
            Constraint::Min(10),    // Available IPs
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.vpc_list_state);
}

fn render_security_group_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Group ID", "Name", "VPC ID", "Inbound Rules", "Outbound Rules"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.security_groups.iter()
        .filter(|sg| {
            if filter.is_empty() { return true; }
            let id = sg.group_id.to_lowercase();
            let name = sg.group_name.to_lowercase();
            let vpc_id = sg.vpc_id.as_deref().unwrap_or("").to_lowercase();
            id.contains(&filter) || name.contains(&filter) || vpc_id.contains(&filter)
        })
        .map(|sg| {
        let vpc_id = sg.vpc_id.clone().unwrap_or_else(|| "-".to_string());
        
        let cells = vec![
            Cell::from(sg.group_id.clone()),
            Cell::from(sg.group_name.clone()),
            Cell::from(vpc_id),
            Cell::from(sg.inbound_rules_count.to_string()),
            Cell::from(sg.outbound_rules_count.to_string()),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Security Groups (Press 'v' to switch view)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(25), // Group ID
            Constraint::Length(30), // Name
            Constraint::Length(25), // VPC ID
            Constraint::Length(15), // Inbound
            Constraint::Min(15),    // Outbound
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.vpc_list_state);
}

fn render_sg_rules_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Protocol", "Port Range", "Source", "Description"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows = app.current_sg_rules.iter()
        .map(|rule| {
        let cells = vec![
            Cell::from(rule.protocol.clone()),
            Cell::from(rule.port_range.clone()),
            Cell::from(rule.source.clone()),
            Cell::from(rule.description.clone().unwrap_or_default()),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Inbound Rules (Press Esc to back)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(10), // Protocol
            Constraint::Length(15), // Port Range
            Constraint::Length(20), // Source
            Constraint::Min(20),    // Description
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.vpc_list_state);
}

fn render_vpc_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.vpc_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(vpc) = app.vpcs.get(idx) {
            build_vpc_detail_lines(vpc, app)
        } else {
            vec![Line::from("No VPC selected")]
        }
    } else {
        vec![Line::from("Select a VPC to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("VPC Details"));
    
    frame.render_widget(paragraph, area);
}

fn render_subnet_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.vpc_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(subnet) = app.subnets.get(idx) {
            build_subnet_detail_lines(subnet)
        } else {
            vec![Line::from("No Subnet selected")]
        }
    } else {
        vec![Line::from("Select a Subnet to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Subnet Details"));
    
    frame.render_widget(paragraph, area);
}

fn render_security_group_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.vpc_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(sg) = app.security_groups.get(idx) {
            build_security_group_detail_lines(sg)
        } else {
            vec![Line::from("No Security Group selected")]
        }
    } else {
        vec![Line::from("Select a Security Group to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Security Group Details"));
    
    frame.render_widget(paragraph, area);
}

fn render_sg_rule_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.vpc_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(rule) = app.current_sg_rules.get(idx) {
            build_sg_rule_detail_lines(rule)
        } else {
            vec![Line::from("No Rule selected")]
        }
    } else {
        vec![Line::from("Select a Rule to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Rule Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_vpc_detail_lines<'a>(vpc: &Vpc, app: &App) -> Vec<Line<'a>> {
    let state_color = vpc.state_color();
    let name = vpc.name.clone().unwrap_or_else(|| "(unnamed)".to_string());
    let cidr = vpc.cidr_block.clone().unwrap_or_else(|| "-".to_string());
    let owner = vpc.owner_id.clone().unwrap_or_else(|| "-".to_string());
    let tenancy = vpc.instance_tenancy.clone().unwrap_or_else(|| "default".to_string());
    let dhcp = vpc.dhcp_options_id.clone().unwrap_or_else(|| "-".to_string());

    // Count related resources
    let subnet_count = app.subnets.iter().filter(|s| s.vpc_id.as_deref() == Some(&vpc.vpc_id)).count();
    let sg_count = app.security_groups.iter().filter(|sg| sg.vpc_id.as_deref() == Some(&vpc.vpc_id)).count();

    vec![
        Line::from(vec![
            Span::styled("VPC ID: ", Style::default().fg(Color::Cyan)),
            Span::styled(vpc.vpc_id.clone(), Style::default().fg(Color::Yellow)),
            Span::raw("   "),
            Span::styled("State: ", Style::default().fg(Color::Cyan)),
            Span::styled(vpc.state.clone(), Style::default().fg(state_color)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::raw(name),
        ]),
        Line::from(vec![
            Span::styled("CIDR Block: ", Style::default().fg(Color::Cyan)),
            Span::raw(cidr),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Configuration ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Default VPC: ", Style::default().fg(Color::Cyan)),
            if vpc.is_default {
                Span::styled("Yes", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
        Line::from(vec![
            Span::styled("Instance Tenancy: ", Style::default().fg(Color::Cyan)),
            Span::raw(tenancy),
        ]),
        Line::from(vec![
            Span::styled("DHCP Options Set: ", Style::default().fg(Color::Cyan)),
            Span::raw(dhcp),
        ]),
        Line::from(vec![
            Span::styled("Owner ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(owner),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Related Resources ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Subnets: ", Style::default().fg(Color::Cyan)),
            Span::raw(subnet_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Security Groups: ", Style::default().fg(Color::Cyan)),
            Span::raw(sg_count.to_string()),
        ]),
    ]
}

fn build_subnet_detail_lines(subnet: &Subnet) -> Vec<Line<'_>> {
    let name = subnet.name.clone().unwrap_or_else(|| "(unnamed)".to_string());
    let vpc_id = subnet.vpc_id.clone().unwrap_or_else(|| "-".to_string());
    let cidr = subnet.cidr_block.clone().unwrap_or_else(|| "-".to_string());
    let az = subnet.availability_zone.clone().unwrap_or_else(|| "-".to_string());
    let ips = subnet.available_ip_count.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("Subnet ID: ", Style::default().fg(Color::Cyan)),
            Span::styled(subnet.subnet_id.clone(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::raw(name),
        ]),
        Line::from(vec![
            Span::styled("VPC ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(vpc_id),
        ]),
        Line::from(vec![
            Span::styled("CIDR Block: ", Style::default().fg(Color::Cyan)),
            Span::raw(cidr),
        ]),
        Line::from(vec![
            Span::styled("Availability Zone: ", Style::default().fg(Color::Cyan)),
            Span::raw(az),
        ]),
        Line::from(vec![
            Span::styled("Available IPs: ", Style::default().fg(Color::Cyan)),
            Span::raw(ips),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Configuration ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Default for AZ: ", Style::default().fg(Color::Cyan)),
            if subnet.is_default {
                Span::styled("Yes", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
        Line::from(vec![
            Span::styled("Auto-assign Public IP: ", Style::default().fg(Color::Cyan)),
            if subnet.map_public_ip {
                Span::styled("Yes", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
    ]
}

fn build_security_group_detail_lines(sg: &SecurityGroup) -> Vec<Line<'_>> {
    let vpc_id = sg.vpc_id.clone().unwrap_or_else(|| "-".to_string());
    let description = sg.description.clone().unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("Group ID: ", Style::default().fg(Color::Cyan)),
            Span::styled(sg.group_id.clone(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::raw(sg.group_name.clone()),
        ]),
        Line::from(vec![
            Span::styled("VPC ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(vpc_id),
        ]),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::Cyan)),
            Span::raw(description),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Rules ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Inbound Rules: ", Style::default().fg(Color::Cyan)),
            Span::raw(sg.inbound_rules_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Outbound Rules: ", Style::default().fg(Color::Cyan)),
            Span::raw(sg.outbound_rules_count.to_string()),
        ]),
    ]
}

fn build_sg_rule_detail_lines(rule: &SecurityGroupRule) -> Vec<Line<'_>> {
    vec![
        Line::from(vec![
            Span::styled("Protocol: ", Style::default().fg(Color::Cyan)),
            Span::styled(rule.protocol.clone(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Port Range: ", Style::default().fg(Color::Cyan)),
            Span::raw(rule.port_range.clone()),
        ]),
        Line::from(vec![
            Span::styled("Source: ", Style::default().fg(Color::Cyan)),
            Span::raw(rule.source.clone()),
        ]),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::Cyan)),
            Span::raw(rule.description.clone().unwrap_or_else(|| "-".to_string())),
        ]),
    ]
}
