use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use crate::app::App;
use crate::models::ec2::{Ec2Instance, InstanceState};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["ID", "Name", "State", "Type", "Public IP", "Launch Time"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows = app.ec2_instances.iter().map(|instance| {
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
    .block(Block::default().borders(Borders::ALL).title("EC2 Instances"))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    // Use app state for selection
    frame.render_stateful_widget(t, area, &mut app.ec2_list_state);
}
