use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::backup::BackupVault;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    render_vault_list(frame, list_area, app);
    
    if let Some(area) = detail_area {
        render_vault_details(frame, area, app);
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

    let rows = app.backup_vaults.iter().map(|vault| {
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
    .block(Block::default().borders(Borders::ALL).title("Backup Vaults"))
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
