use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::vpc::Vpc;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    render_vpc_list(frame, list_area, app);
    
    if let Some(area) = detail_area {
        render_vpc_details(frame, area, app);
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
        .title("VPCs")
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

fn render_vpc_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.vpc_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(vpc) = app.vpcs.get(idx) {
            build_vpc_detail_lines(vpc)
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

fn build_vpc_detail_lines(vpc: &Vpc) -> Vec<Line<'_>> {
    let state_color = vpc.state_color();
    let name = vpc.name.clone().unwrap_or_else(|| "(unnamed)".to_string());
    let cidr = vpc.cidr_block.clone().unwrap_or_else(|| "-".to_string());
    let owner = vpc.owner_id.clone().unwrap_or_else(|| "-".to_string());
    let tenancy = vpc.instance_tenancy.clone().unwrap_or_else(|| "default".to_string());
    let dhcp = vpc.dhcp_options_id.clone().unwrap_or_else(|| "-".to_string());

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
    ]
}
