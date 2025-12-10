use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::iam::{IamRole, IamUser, IamPolicy};

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

    if app.iam_view_mode >= 3 {
        let title = match app.iam_view_mode {
            3 => format!("Policies attached to User: {}", app.selected_iam_entity_name.as_deref().unwrap_or("Unknown")),
            4 => format!("Policies attached to Role: {}", app.selected_iam_entity_name.as_deref().unwrap_or("Unknown")),
            5 => format!("Policy Document: {}", app.selected_iam_entity_name.as_deref().unwrap_or("Unknown")),
            _ => "Details".to_string(),
        };
        let block = Block::default().borders(Borders::ALL).title(title).style(Style::default().fg(Color::Magenta));
        frame.render_widget(block, chunks[0]);
    } else {
        let tabs = ["Users", "Roles", "Policies"];
        crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.iam_view_mode as usize);
    }

    match app.iam_view_mode {
        0 => render_user_list(frame, chunks[1], app),
        1 => render_role_list(frame, chunks[1], app),
        2 => render_policy_list(frame, chunks[1], app),
        3 | 4 => render_attached_policies_list(frame, chunks[1], app),
        5 => render_policy_document(frame, chunks[1], app),
        _ => render_user_list(frame, chunks[1], app),
    }
    
    if let Some(area) = detail_area {
        match app.iam_view_mode {
            0 => render_user_details(frame, area, app),
            1 => render_role_details(frame, area, app),
            2 => render_policy_details(frame, area, app),
            3 | 4 => render_policy_details_from_list(frame, area, app),
            5 => {}, // No details pane for document view, it takes full space? Or maybe just empty.
            _ => render_user_details(frame, area, app),
        }
    }
}

fn render_user_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["User Name", "User ID", "Path", "Created", "Password Last Used"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.iam_users.iter()
        .filter(|u| {
            if filter.is_empty() { return true; }
            let name = u.user_name.to_lowercase();
            let id = u.user_id.to_lowercase();
            name.contains(&filter) || id.contains(&filter)
        })
        .map(|user| {
        let path = user.path.clone().unwrap_or_else(|| "/".to_string());
        let created = user.create_date.clone()
            .map(|d| d.split('T').next().unwrap_or(&d).to_string())
            .unwrap_or_else(|| "-".to_string());
        let pwd_last_used = user.password_last_used.clone()
            .map(|d| d.split('T').next().unwrap_or(&d).to_string())
            .unwrap_or_else(|| "Never".to_string());
        
        let cells = vec![
            Cell::from(user.user_name.clone()),
            Cell::from(user.user_id.clone()),
            Cell::from(path),
            Cell::from(created),
            Cell::from(pwd_last_used),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("IAM Users (Tab to switch view)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(25), // User Name
            Constraint::Length(22), // User ID
            Constraint::Length(15), // Path
            Constraint::Length(15), // Created
            Constraint::Min(15),    // Password Last Used
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.iam_list_state);
}

fn render_user_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.iam_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(user) = app.iam_users.get(idx) {
            build_user_detail_lines(user)
        } else {
            vec![Line::from("No user selected")]
        }
    } else {
        vec![Line::from("Select an IAM user to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("IAM User Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_user_detail_lines(user: &IamUser) -> Vec<Line<'_>> {
    let path = user.path.clone().unwrap_or_else(|| "/".to_string());
    let created = user.create_date.clone().unwrap_or_else(|| "-".to_string());
    let pwd_last_used = user.password_last_used.clone().unwrap_or_else(|| "Never".to_string());
    let arn = user.arn.clone().unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("User Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(user.user_name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("User ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(user.user_id.clone()),
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
            Span::styled("Password Last Used: ", Style::default().fg(Color::Cyan)),
            Span::raw(pwd_last_used),
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
        .title("IAM Roles (Tab to switch view)")
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

fn render_policy_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Policy Name", "Policy ID", "Attachments", "Created", "Updated"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.iam_policies.iter()
        .filter(|p| {
            if filter.is_empty() { return true; }
            let name = p.policy_name.to_lowercase();
            let id = p.policy_id.as_deref().unwrap_or("").to_lowercase();
            name.contains(&filter) || id.contains(&filter)
        })
        .map(|policy| {
        let created = policy.create_date.clone()
            .map(|d| d.split('T').next().unwrap_or(&d).to_string())
            .unwrap_or_else(|| "-".to_string());
        let updated = policy.update_date.clone()
            .map(|d| d.split('T').next().unwrap_or(&d).to_string())
            .unwrap_or_else(|| "-".to_string());
        let attachments = policy.attachment_count.unwrap_or(0).to_string();
        
        let cells = vec![
            Cell::from(policy.policy_name.clone()),
            Cell::from(policy.policy_id.clone().unwrap_or_default()),
            Cell::from(attachments),
            Cell::from(created),
            Cell::from(updated),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("IAM Policies (Tab to switch view)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(40), // Policy Name
            Constraint::Length(22), // Policy ID
            Constraint::Length(12), // Attachments
            Constraint::Length(15), // Created
            Constraint::Min(15),    // Updated
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.iam_list_state);
}

fn render_policy_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.iam_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(policy) = app.iam_policies.get(idx) {
            build_policy_detail_lines(policy)
        } else {
            vec![Line::from("No policy selected")]
        }
    } else {
        vec![Line::from("Select an IAM policy to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("IAM Policy Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_policy_detail_lines(policy: &IamPolicy) -> Vec<Line<'_>> {
    let created = policy.create_date.clone().unwrap_or_else(|| "-".to_string());
    let updated = policy.update_date.clone().unwrap_or_else(|| "-".to_string());
    let arn = policy.arn.clone().unwrap_or_else(|| "-".to_string());
    let attachments = policy.attachment_count.unwrap_or(0).to_string();

    vec![
        Line::from(vec![
            Span::styled("Policy Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(policy.policy_name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Policy ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(policy.policy_id.clone().unwrap_or_default()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Details ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Attachment Count: ", Style::default().fg(Color::Cyan)),
            Span::raw(attachments),
        ]),
        Line::from(vec![
            Span::styled("Is Attachable: ", Style::default().fg(Color::Cyan)),
            if policy.is_attachable {
                Span::styled("Yes", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Cyan)),
            Span::raw(created),
        ]),
        Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(Color::Cyan)),
            Span::raw(updated),
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

fn render_attached_policies_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Policy Name", "ARN"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows = app.current_iam_policies.iter()
        .map(|policy| {
        let cells = vec![
            Cell::from(policy.policy_name.clone()),
            Cell::from(policy.arn.clone().unwrap_or_default()),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Attached Policies (Press Esc to back)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(40), // Policy Name
            Constraint::Min(40),    // ARN
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.iam_list_state);
}

fn render_policy_document(frame: &mut Frame, area: Rect, app: &App) {
    let text = app.current_policy_document.clone();
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Policy Document (Press Esc to back)"))
        .wrap(ratatui::widgets::Wrap { trim: false });
    
    frame.render_widget(paragraph, area);
}

fn render_policy_details_from_list(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.iam_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(policy) = app.current_iam_policies.get(idx) {
            build_policy_detail_lines(policy)
        } else {
            vec![Line::from("No policy selected")]
        }
    } else {
        vec![Line::from("Select a policy to view details")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Policy Details"));
    
    frame.render_widget(paragraph, area);
}
