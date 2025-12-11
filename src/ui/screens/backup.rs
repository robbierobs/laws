use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::{App, BackupViewMode};
use crate::models::backup::{BackupVault, BackupPlan, BackupJob};
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

        let tabs = ["Vaults", "Plans", "Jobs"];
        crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.services.backup.view_mode.to_index());

        match app.services.backup.view_mode {
            BackupViewMode::Vaults => render_vault_list(frame, chunks[1], app),
            BackupViewMode::Plans => render_plan_list(frame, chunks[1], app),
            BackupViewMode::Jobs => render_job_list(frame, chunks[1], app),
        }
    }
    
    if let Some(area) = detail_area {
        match app.services.backup.view_mode {
            BackupViewMode::Vaults => render_vault_details(frame, area, app),
            BackupViewMode::Plans => render_plan_details(frame, area, app),
            BackupViewMode::Jobs => render_job_details(frame, area, app),
        }
    }
}

use crate::models::Filterable;

fn render_vault_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.backup.vaults.iter()
        .filter(|v| {
            if filter.is_empty() { return true; }
            v.matches_filter(&filter)
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

    render_table(
        frame,
        area,
        rows,
        &["Vault Name", "Recovery Points", "Locked", "Created"],
        &[
            Constraint::Length(35), // Vault Name
            Constraint::Length(18), // Recovery Points
            Constraint::Length(10), // Locked
            Constraint::Min(15),    // Created
        ],
        "Backup Vaults (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.backup.list_state,
    );
}

fn render_vault_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.backup.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(vault) = app.services.backup.vaults.get(idx) {
            build_vault_detail_lines(vault)
        } else {
            vec![Line::from("No vault selected")]
        }
    } else {
        vec![Line::from("Select a backup vault to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Backup Vault Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_vault_detail_lines(vault: &BackupVault) -> Vec<Line<'_>> {
    let created = vault.creation_date.clone().unwrap_or_else(|| "-".to_string());
    let arn = vault.backup_vault_arn.clone().unwrap_or_else(|| "-".to_string());
    let encryption_key = vault.encryption_key_arn.clone().unwrap_or_else(|| "(default)".to_string());

    vec![
        Line::from(vec![
            Span::styled("Vault Name: ", Style::default().fg(THEME.primary)),
            Span::styled(vault.backup_vault_name.clone(), Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Statistics ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Recovery Points: ", Style::default().fg(THEME.primary)),
            Span::raw(vault.number_of_recovery_points.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Locked: ", Style::default().fg(THEME.primary)),
            if vault.locked {
                Span::styled("Yes ✓", Style::default().fg(THEME.success))
            } else {
                Span::styled("No", Style::default().fg(THEME.muted))
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Details ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(THEME.primary)),
            Span::raw(created),
        ]),
        Line::from(vec![
            Span::styled("Encryption Key: ", Style::default().fg(THEME.primary)),
            Span::raw(encryption_key),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
        ]),
        Line::from(vec![
            Span::styled(arn, Style::default().fg(THEME.selection_fg)),
        ]),
    ]
}

fn render_plan_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.backup.plans.iter()
        .filter(|p| {
            if filter.is_empty() { return true; }
            p.matches_filter(&filter)
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

    render_table(
        frame,
        area,
        rows,
        &["Plan Name", "Plan ID", "Version", "Last Execution"],
        &[
            Constraint::Length(30), // Plan Name
            Constraint::Length(36), // Plan ID
            Constraint::Length(36), // Version
            Constraint::Min(15),    // Last Execution
        ],
        "Backup Plans (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.backup.list_state,
    );
}

fn render_plan_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.backup.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(plan) = app.services.backup.plans.get(idx) {
            build_plan_detail_lines(plan)
        } else {
            vec![Line::from("No plan selected")]
        }
    } else {
        vec![Line::from("Select a backup plan to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Backup Plan Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_plan_detail_lines(plan: &BackupPlan) -> Vec<Line<'_>> {
    let created = plan.creation_date.clone().unwrap_or_else(|| "-".to_string());
    let last_exec = plan.last_execution_date.clone().unwrap_or_else(|| "-".to_string());
    let arn = plan.backup_plan_arn.clone().unwrap_or_else(|| "-".to_string());
    let version = plan.version_id.clone().unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("Plan Name: ", Style::default().fg(THEME.primary)),
            Span::styled(plan.backup_plan_name.clone(), Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Plan ID: ", Style::default().fg(THEME.primary)),
            Span::raw(plan.backup_plan_id.clone()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Details ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Version ID: ", Style::default().fg(THEME.primary)),
            Span::raw(version),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(THEME.primary)),
            Span::raw(created),
        ]),
        Line::from(vec![
            Span::styled("Last Execution: ", Style::default().fg(THEME.primary)),
            Span::raw(last_exec),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
        ]),
        Line::from(vec![
            Span::styled(arn, Style::default().fg(THEME.selection_fg)),
        ]),
    ]
}

fn render_job_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.backup.jobs.iter()
        .filter(|j| {
            if filter.is_empty() { return true; }
            j.matches_filter(&filter)
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

    render_table(
        frame,
        area,
        rows,
        &["Job ID", "State", "Resource Type", "Percent Done", "Created"],
        &[
            Constraint::Length(36), // Job ID
            Constraint::Length(12), // State
            Constraint::Length(15), // Resource Type
            Constraint::Length(12), // Percent Done
            Constraint::Min(15),    // Created
        ],
        "Backup Jobs (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.backup.list_state,
    );
}

fn render_job_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.backup.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(job) = app.services.backup.jobs.get(idx) {
            build_job_detail_lines(job)
        } else {
            vec![Line::from("No job selected")]
        }
    } else {
        vec![Line::from("Select a backup job to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Backup Job Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_job_detail_lines(job: &BackupJob) -> Vec<Line<'_>> {
    let state_color = job.state_color();
    let created = job.creation_date.clone().unwrap_or_else(|| "-".to_string());
    let completed = job.completion_date.clone().unwrap_or_else(|| "-".to_string());
    let resource_arn = job.resource_arn.clone().unwrap_or_else(|| "-".to_string());
    let vault = job.backup_vault_name.clone().unwrap_or_else(|| "-".to_string());

    vec![
        Line::from(vec![
            Span::styled("Job ID: ", Style::default().fg(THEME.primary)),
            Span::styled(job.backup_job_id.clone(), Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("State: ", Style::default().fg(THEME.primary)),
            Span::styled(job.state.clone(), Style::default().fg(state_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Resource ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(THEME.primary)),
            Span::raw(job.resource_type.clone().unwrap_or_default()),
        ]),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
            Span::raw(resource_arn),
        ]),
        Line::from(vec![
            Span::styled("Vault: ", Style::default().fg(THEME.primary)),
            Span::raw(vault),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Progress ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Percent Done: ", Style::default().fg(THEME.primary)),
            Span::raw(job.percent_done.clone().unwrap_or_else(|| "-".to_string())),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(THEME.primary)),
            Span::raw(created),
        ]),
        Line::from(vec![
            Span::styled("Completed: ", Style::default().fg(THEME.primary)),
            Span::raw(completed),
        ]),
    ]
}
