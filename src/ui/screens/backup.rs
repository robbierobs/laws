use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::backup::{BackupVault, BackupPlan, BackupJob};

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

    let tabs = ["Vaults", "Plans", "Jobs"];
    crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.backup_view_mode as usize);

    match app.backup_view_mode {
        0 => {
            render_vault_list(frame, chunks[1], app);
            if let Some(area) = detail_area {
                render_vault_details(frame, area, app);
            }
        }
        1 => {
            render_plan_list(frame, chunks[1], app);
            if let Some(area) = detail_area {
                render_plan_details(frame, area, app);
            }
        }
        2 => {
            render_job_list(frame, chunks[1], app);
            if let Some(area) = detail_area {
                render_job_details(frame, area, app);
            }
        }
        _ => {}
    }
}

fn render_vault_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Vault Name", "Recovery Points", "Locked", "Created"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.backup_vaults.iter()
        .filter(|v| {
            if filter.is_empty() { return true; }
            v.backup_vault_name.to_lowercase().contains(&filter)
        })
        .map(|vault| {
        let created = vault.creation_date.clone()
            .map(|d| d.split('T').next().unwrap_or(&d).to_string())
            .unwrap_or_else(|| "-".to_string());
        
        let cells = vec![
            Cell::from(vault.backup_vault_name.clone()),
            Cell::from(vault.number_of_recovery_points.to_string()),
            Cell::from(if vault.locked { "Yes" } else { "No" }),
            Cell::from(created),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Backup Vaults (Tab to switch view)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(35), // Vault Name
            Constraint::Length(18), // Recovery Points
            Constraint::Length(10), // Locked
            Constraint::Min(15),    // Created
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.backup_list_state);
}

fn render_vault_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.backup_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(vault) = app.backup_vaults.get(idx) {
            build_vault_detail_lines(vault)
        } else {
            vec![Line::from("No vault selected")]
        }
    } else {
        vec![Line::from("Select a backup vault to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Backup Vault Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_vault_detail_lines(vault: &BackupVault) -> Vec<Line<'_>> {
    let created = vault.creation_date.clone().unwrap_or_else(|| "-".to_string());
    let arn = vault.backup_vault_arn.clone().unwrap_or_else(|| "-".to_string());
    let encryption_key = vault.encryption_key_arn.clone().unwrap_or_else(|| "(default)".to_string());

    vec![
        Line::from(vec![
            Span::styled("Vault Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(vault.backup_vault_name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Statistics ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Recovery Points: ", Style::default().fg(Color::Cyan)),
            Span::raw(vault.number_of_recovery_points.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Locked: ", Style::default().fg(Color::Cyan)),
            if vault.locked {
                Span::styled("Yes ✓", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::Gray))
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Details ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Cyan)),
            Span::raw(created),
        ]),
        Line::from(vec![
            Span::styled("Encryption Key: ", Style::default().fg(Color::Cyan)),
            Span::raw(encryption_key),
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

fn render_plan_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Plan Name", "Plan ID", "Version", "Last Execution"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.backup_plans.iter()
        .filter(|p| {
            if filter.is_empty() { return true; }
            let name = p.backup_plan_name.to_lowercase();
            let id = p.backup_plan_id.to_lowercase();
            name.contains(&filter) || id.contains(&filter)
        })
        .map(|plan| {
        let last_exec = plan.last_execution_date.clone()
            .map(|d| d.split('T').next().unwrap_or(&d).to_string())
            .unwrap_or_else(|| "-".to_string());
        
        let cells = vec![
            Cell::from(plan.backup_plan_name.clone()),
            Cell::from(plan.backup_plan_id.clone()),
            Cell::from(plan.version_id.clone().unwrap_or_default()),
            Cell::from(last_exec),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Backup Plans (Tab to switch view)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(30), // Plan Name
            Constraint::Length(36), // Plan ID
            Constraint::Length(36), // Version
            Constraint::Min(15),    // Last Execution
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.backup_list_state);
}

fn render_plan_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.backup_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(plan) = app.backup_plans.get(idx) {
            build_plan_detail_lines(plan)
        } else {
            vec![Line::from("No plan selected")]
        }
    } else {
        vec![Line::from("Select a backup plan to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Backup Plan Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_plan_detail_lines(plan: &BackupPlan) -> Vec<Line<'_>> {
    let created = plan.creation_date.clone().unwrap_or_else(|| "-".to_string());
    let last_exec = plan.last_execution_date.clone().unwrap_or_else(|| "-".to_string());
    let arn = plan.backup_plan_arn.clone().unwrap_or_else(|| "-".to_string());
    let version = plan.version_id.clone().unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("Plan Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(plan.backup_plan_name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Plan ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(plan.backup_plan_id.clone()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Details ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Version ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(version),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Cyan)),
            Span::raw(created),
        ]),
        Line::from(vec![
            Span::styled("Last Execution: ", Style::default().fg(Color::Cyan)),
            Span::raw(last_exec),
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

fn render_job_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Job ID", "State", "Resource Type", "Percent Done", "Created"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.backup_jobs.iter()
        .filter(|j| {
            if filter.is_empty() { return true; }
            let id = j.backup_job_id.to_lowercase();
            let resource = j.resource_type.as_deref().unwrap_or("").to_lowercase();
            id.contains(&filter) || resource.contains(&filter)
        })
        .map(|job| {
        let state_color = job.state_color();
        let created = job.creation_date.clone()
            .map(|d| d.split('T').next().unwrap_or(&d).to_string())
            .unwrap_or_else(|| "-".to_string());
        
        let cells = vec![
            Cell::from(job.backup_job_id.clone()),
            Cell::from(job.state.clone()).style(Style::default().fg(state_color)),
            Cell::from(job.resource_type.clone().unwrap_or_default()),
            Cell::from(job.percent_done.clone().unwrap_or_else(|| "-".to_string())),
            Cell::from(created),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Backup Jobs (Tab to switch view)")
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(36), // Job ID
            Constraint::Length(12), // State
            Constraint::Length(15), // Resource Type
            Constraint::Length(12), // Percent Done
            Constraint::Min(15),    // Created
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(t, area, &mut app.backup_list_state);
}

fn render_job_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.backup_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(job) = app.backup_jobs.get(idx) {
            build_job_detail_lines(job)
        } else {
            vec![Line::from("No job selected")]
        }
    } else {
        vec![Line::from("Select a backup job to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Backup Job Details"));
    
    frame.render_widget(paragraph, area);
}

fn build_job_detail_lines(job: &BackupJob) -> Vec<Line<'_>> {
    let state_color = job.state_color();
    let created = job.creation_date.clone().unwrap_or_else(|| "-".to_string());
    let completed = job.completion_date.clone().unwrap_or_else(|| "-".to_string());
    let resource_arn = job.resource_arn.clone().unwrap_or_else(|| "-".to_string());
    let vault = job.backup_vault_name.clone().unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("Job ID: ", Style::default().fg(Color::Cyan)),
            Span::styled(job.backup_job_id.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("State: ", Style::default().fg(Color::Cyan)),
            Span::styled(job.state.clone(), Style::default().fg(state_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Resource ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Cyan)),
            Span::raw(job.resource_type.clone().unwrap_or_default()),
        ]),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(Color::Cyan)),
            Span::raw(resource_arn),
        ]),
        Line::from(vec![
            Span::styled("Vault: ", Style::default().fg(Color::Cyan)),
            Span::raw(vault),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Progress ───", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Percent Done: ", Style::default().fg(Color::Cyan)),
            Span::raw(job.percent_done.clone().unwrap_or_else(|| "-".to_string())),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Cyan)),
            Span::raw(created),
        ]),
        Line::from(vec![
            Span::styled("Completed: ", Style::default().fg(Color::Cyan)),
            Span::raw(completed),
        ]),
    ]
}
