use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::iam::IamRole;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    render_role_list(frame, list_area, app);
    
    if let Some(area) = detail_area {
        render_role_details(frame, area, app);
    }
}

fn render_role_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Role Name", "Role ID", "Path", "Created", "Max Session"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.iam_roles.iter()
        .filter(|r| {
            if filter.is_empty() { return true; }
            let name = r.role_name.to_lowercase();
            let id = r.role_id.to_lowercase();
            name.contains(&filter) || id.contains(&filter)
        })
        .map(|role| {
        let path = role.path.clone().unwrap_or_else(|| "/".to_string());
        let created = role.create_date.clone()
            .map(|d| d.split('T').next().unwrap_or(&d).to_string())
            .unwrap_or_else(|| "-".to_string());
        let max_session = role.max_session_duration
            .map(|d| format!("{}h", d / 3600))
            .unwrap_or_else(|| "-".to_string());
        
        let cells = vec![
            Cell::from(role.role_name.clone()),
            Cell::from(role.role_id.clone()),
            Cell::from(path),
            Cell::from(created),
            Cell::from(max_session),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("IAM Roles")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(35), // Role Name
            Constraint::Length(22), // Role ID
            Constraint::Length(20), // Path
            Constraint::Length(12), // Created
            Constraint::Min(10),    // Max Session
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.iam_list_state);
}

fn render_role_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.iam_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(role) = app.iam_roles.get(idx) {
            build_role_detail_lines(role)
        } else {
            vec![Line::from("No role selected")]
        }
    } else {
        vec![Line::from("Select an IAM role to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("IAM Role Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_role_detail_lines(role: &IamRole) -> Vec<Line<'_>> {
    let path = role.path.clone().unwrap_or_else(|| "/".to_string());
    let created = role.create_date.clone().unwrap_or_else(|| "-".to_string());
    let desc = role.description.clone().unwrap_or_else(|| "(no description)".to_string());
    let max_session = role.max_session_duration
        .map(|d| format!("{} seconds ({} hours)", d, d / 3600))
        .unwrap_or_else(|| "-".to_string());
    let arn = role.arn.clone().unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("Role Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(role.role_name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Role ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(role.role_id.clone()),
        ]),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::Cyan)),
            Span::raw(desc),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Details ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Cyan)),
            Span::raw(path),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Cyan)),
            Span::raw(created),
        ]),
        Line::from(vec![
            Span::styled("Max Session Duration: ", Style::default().fg(Color::Cyan)),
            Span::raw(max_session),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled(arn, Style::default().fg(Color::Blue)),
        ]),
    ]
}
