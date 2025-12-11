use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row},
    Frame,
};
use crate::app::{App, VpcViewMode};
use crate::models::vpc::{Vpc, Subnet, SecurityGroup, SecurityGroupRule};
use crate::ui::components::detail_panel::{render_detail_panel, DetailPanelConfig};
use crate::ui::components::table::render_table;

use crate::ui::theme::THEME;

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

        if app.services.vpc.view_mode == VpcViewMode::SecurityGroupRules {
            let title = format!("Rules for {}", app.services.vpc.selected_sg_id.as_deref().unwrap_or("Unknown"));
            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Style::default().fg(THEME.primary))
                .border_style(Style::default().fg(THEME.border));
            frame.render_widget(block, chunks[0]);
        } else {
            let tabs = ["VPCs", "Subnets", "Security Groups"];
            crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.services.vpc.view_mode.to_index());
        }

        match app.services.vpc.view_mode {
            VpcViewMode::Vpcs => render_vpc_list(frame, chunks[1], app),
            VpcViewMode::Subnets => render_subnet_list(frame, chunks[1], app),
            VpcViewMode::SecurityGroups => render_security_group_list(frame, chunks[1], app),
            VpcViewMode::SecurityGroupRules => render_sg_rules_list(frame, chunks[1], app),
        }
    }
    
    if let Some(area) = detail_area {
        match app.services.vpc.view_mode {
            VpcViewMode::Vpcs => render_vpc_details(frame, area, app),
            VpcViewMode::Subnets => render_subnet_details(frame, area, app),
            VpcViewMode::SecurityGroups => render_security_group_details(frame, area, app),
            VpcViewMode::SecurityGroupRules => render_sg_rule_details(frame, area, app),
        }
    }
}

fn render_vpc_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.vpc.vpcs.iter()
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

    render_table(
        frame,
        area,
        rows,
        &["VPC ID", "Name", "CIDR Block", "State", "Default", "Tenancy"],
        &[
            Constraint::Length(25), // VPC ID
            Constraint::Length(20), // Name
            Constraint::Length(18), // CIDR Block
            Constraint::Length(12), // State
            Constraint::Length(8),  // Default
            Constraint::Min(10),    // Tenancy
        ],
        "VPCs (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.vpc.list_state,
    );
}

fn render_subnet_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.vpc.subnets.iter()
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

    render_table(
        frame,
        area,
        rows,
        &["Subnet ID", "Name", "VPC ID", "CIDR Block", "AZ", "Available IPs"],
        &[
            Constraint::Length(25), // Subnet ID
            Constraint::Length(20), // Name
            Constraint::Length(25), // VPC ID
            Constraint::Length(18), // CIDR Block
            Constraint::Length(15), // AZ
            Constraint::Min(10),    // Available IPs
        ],
        "Subnets (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.vpc.list_state,
    );
}

fn render_security_group_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.vpc.security_groups.iter()
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

    render_table(
        frame,
        area,
        rows,
        &["Group ID", "Name", "VPC ID", "Inbound Rules", "Outbound Rules"],
        &[
            Constraint::Length(25), // Group ID
            Constraint::Length(30), // Name
            Constraint::Length(25), // VPC ID
            Constraint::Length(15), // Inbound
            Constraint::Min(15),    // Outbound
        ],
        "Security Groups (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.vpc.list_state,
    );
}

fn render_sg_rules_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = app.services.vpc.current_sg_rules.iter()
        .map(|rule| {
            let cells = vec![
                Cell::from(rule.protocol.clone()),
                Cell::from(rule.port_range.clone()),
                Cell::from(rule.source.clone()),
                Cell::from(rule.description.clone().unwrap_or_default()),
            ];
            
            Row::new(cells).height(1)
        });

    let direction = if app.services.vpc.sg_rules_inbound { "Inbound" } else { "Outbound" };
    let title = format!("{} Rules (t: toggle direction, Esc: back)", direction);

    render_table(
        frame,
        area,
        rows,
        &["Protocol", "Port Range", "Source", "Description"],
        &[
            Constraint::Length(10), // Protocol
            Constraint::Length(15), // Port Range
            Constraint::Length(20), // Source
            Constraint::Min(20),    // Description
        ],
        &title,
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.vpc.list_state,
    );
}

fn render_vpc_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.vpc.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(vpc) = app.services.vpc.vpcs.get(idx) {
            build_vpc_detail_lines(vpc, app)
        } else {
            vec![Line::from("No VPC selected")]
        }
    } else {
        vec![Line::from("Select a VPC to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("VPC Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn render_subnet_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.vpc.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(subnet) = app.services.vpc.subnets.get(idx) {
            build_subnet_detail_lines(subnet)
        } else {
            vec![Line::from("No Subnet selected")]
        }
    } else {
        vec![Line::from("Select a Subnet to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Subnet Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn render_security_group_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.vpc.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(sg) = app.services.vpc.security_groups.get(idx) {
            build_security_group_detail_lines(sg)
        } else {
            vec![Line::from("No Security Group selected")]
        }
    } else {
        vec![Line::from("Select a Security Group to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Security Group Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn render_sg_rule_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.vpc.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(rule) = app.services.vpc.current_sg_rules.get(idx) {
            build_sg_rule_detail_lines(rule)
        } else {
            vec![Line::from("No Rule selected")]
        }
    } else {
        vec![Line::from("Select a Rule to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Rule Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_vpc_detail_lines<'a>(vpc: &Vpc, app: &App) -> Vec<Line<'a>> {
    let state_color = vpc.state_color();
    let name = vpc.name.clone().unwrap_or_else(|| "(unnamed)".to_string());
    let cidr = vpc.cidr_block.clone().unwrap_or_else(|| "-".to_string());
    let owner = vpc.owner_id.clone().unwrap_or_else(|| "-".to_string());
    let tenancy = vpc.instance_tenancy.clone().unwrap_or_else(|| "default".to_string());
    let dhcp = vpc.dhcp_options_id.clone().unwrap_or_else(|| "-".to_string());

    // Count related resources
    let subnet_count = app.services.vpc.subnets.iter().filter(|s| s.vpc_id.as_deref() == Some(&vpc.vpc_id)).count();
    let sg_count = app.services.vpc.security_groups.iter().filter(|sg| sg.vpc_id.as_deref() == Some(&vpc.vpc_id)).count();

    vec![
        Line::from(vec![
            Span::styled("VPC ID: ", Style::default().fg(THEME.primary)),
            Span::styled(vpc.vpc_id.clone(), Style::default().fg(THEME.selection_fg)),
            Span::raw("   "),
            Span::styled("State: ", Style::default().fg(THEME.primary)),
            Span::styled(vpc.state.clone(), Style::default().fg(state_color)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(THEME.primary)),
            Span::raw(name),
        ]),
        Line::from(vec![
            Span::styled("CIDR Block: ", Style::default().fg(THEME.primary)),
            Span::raw(cidr),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Configuration ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Default VPC: ", Style::default().fg(THEME.primary)),
            if vpc.is_default {
                Span::styled("Yes", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
        Line::from(vec![
            Span::styled("Instance Tenancy: ", Style::default().fg(THEME.primary)),
            Span::raw(tenancy),
        ]),
        Line::from(vec![
            Span::styled("DHCP Options Set: ", Style::default().fg(THEME.primary)),
            Span::raw(dhcp),
        ]),
        Line::from(vec![
            Span::styled("Owner ID: ", Style::default().fg(THEME.primary)),
            Span::raw(owner),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Related Resources ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Subnets: ", Style::default().fg(THEME.primary)),
            Span::raw(subnet_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Security Groups: ", Style::default().fg(THEME.primary)),
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
            Span::styled("Subnet ID: ", Style::default().fg(THEME.primary)),
            Span::styled(subnet.subnet_id.clone(), Style::default().fg(THEME.selection_fg)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(THEME.primary)),
            Span::raw(name),
        ]),
        Line::from(vec![
            Span::styled("VPC ID: ", Style::default().fg(THEME.primary)),
            Span::raw(vpc_id),
        ]),
        Line::from(vec![
            Span::styled("CIDR Block: ", Style::default().fg(THEME.primary)),
            Span::raw(cidr),
        ]),
        Line::from(vec![
            Span::styled("Availability Zone: ", Style::default().fg(THEME.primary)),
            Span::raw(az),
        ]),
        Line::from(vec![
            Span::styled("Available IPs: ", Style::default().fg(THEME.primary)),
            Span::raw(ips),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Configuration ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Default for AZ: ", Style::default().fg(THEME.primary)),
            if subnet.is_default {
                Span::styled("Yes", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
        Line::from(vec![
            Span::styled("Auto-assign Public IP: ", Style::default().fg(THEME.primary)),
            if subnet.map_public_ip {
                Span::styled("Yes", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
    ]
}

fn build_security_group_detail_lines(sg: &SecurityGroup) -> Vec<Line<'_>> {
    let vpc_id = sg.vpc_id.clone().unwrap_or_else(|| "-".to_string());
    let description = sg.description.clone().unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("Group ID: ", Style::default().fg(THEME.primary)),
            Span::styled(sg.group_id.clone(), Style::default().fg(THEME.selection_fg)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(THEME.primary)),
            Span::raw(sg.group_name.clone()),
        ]),
        Line::from(vec![
            Span::styled("VPC ID: ", Style::default().fg(THEME.primary)),
            Span::raw(vpc_id),
        ]),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(THEME.primary)),
            Span::raw(description),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Rules ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Inbound Rules: ", Style::default().fg(THEME.primary)),
            Span::raw(sg.inbound_rules_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Outbound Rules: ", Style::default().fg(THEME.primary)),
            Span::raw(sg.outbound_rules_count.to_string()),
        ]),
    ]
}

fn build_sg_rule_detail_lines(rule: &SecurityGroupRule) -> Vec<Line<'_>> {
    vec![
        Line::from(vec![
            Span::styled("Protocol: ", Style::default().fg(THEME.primary)),
            Span::styled(rule.protocol.clone(), Style::default().fg(THEME.selection_fg)),
        ]),
        Line::from(vec![
            Span::styled("Port Range: ", Style::default().fg(THEME.primary)),
            Span::raw(rule.port_range.clone()),
        ]),
        Line::from(vec![
            Span::styled("Source: ", Style::default().fg(THEME.primary)),
            Span::raw(rule.source.clone()),
        ]),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(THEME.primary)),
            Span::raw(rule.description.clone().unwrap_or_else(|| "-".to_string())),
        ]),
    ]
}
