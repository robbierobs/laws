use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::App;
use crate::models::lambda::LambdaFunction;

pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
    render_function_list(frame, list_area, app);
    
    if let Some(area) = detail_area {
        render_function_details(frame, area, app);
    }
}

use crate::ui::theme::THEME;

fn render_function_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Function Name", "Runtime", "Memory", "Timeout", "Code Size", "State"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
    
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let filter = app.filter_input.to_lowercase();
    let rows = app.lambda_functions.iter()
        .filter(|f| {
            if filter.is_empty() { return true; }
            let name = f.function_name.to_lowercase();
            let runtime = f.runtime.as_deref().unwrap_or("").to_lowercase();
            name.contains(&filter) || runtime.contains(&filter)
        })
        .map(|func| {
        let state_color = func.state_color();
        let runtime = func.runtime.clone().unwrap_or_else(|| "-".to_string());
        let memory = func.memory_size.map(|m| format!("{} MB", m)).unwrap_or_else(|| "-".to_string());
        let timeout = func.timeout.map(|t| format!("{}s", t)).unwrap_or_else(|| "-".to_string());
        let state = func.state.clone().unwrap_or_else(|| "Unknown".to_string());
        
        let cells = vec![
            Cell::from(func.function_name.clone()),
            Cell::from(runtime),
            Cell::from(memory),
            Cell::from(timeout),
            Cell::from(func.format_code_size()),
            Cell::from(state).style(Style::default().fg(state_color)),
        ];
        
        Row::new(cells).height(1)
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Lambda Functions")
        .title_style(Style::default().fg(THEME.primary))
        .border_style(if matches!(app.focus, crate::app::Focus::Main) {
            Style::default().fg(THEME.secondary)
        } else {
            Style::default().fg(THEME.border)
        });

    let t = Table::new(
        rows,
        [
            Constraint::Length(35), // Function Name
            Constraint::Length(15), // Runtime
            Constraint::Length(10), // Memory
            Constraint::Length(10), // Timeout
            Constraint::Length(12), // Code Size
            Constraint::Min(10),    // State
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(t, area, &mut app.lambda_list_state);
}

fn render_function_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.lambda_list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(func) = app.lambda_functions.get(idx) {
            build_function_detail_lines(func)
        } else {
            vec![Line::from("No function selected")]
        }
    } else {
        vec![Line::from("Select a Lambda function to view details (use j/k to navigate)")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Function Details")
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)));
    
    frame.render_widget(paragraph, area);
}

fn build_function_detail_lines(func: &LambdaFunction) -> Vec<Line<'_>> {
        let state_color = func.state_color();
    
    let runtime_str = func.runtime.clone().unwrap_or_else(|| "-".to_string());
    let handler_str = func.handler.clone().unwrap_or_else(|| "-".to_string());
    let desc_str = func.description.clone().unwrap_or_else(|| "(no description)".to_string());
    let memory_str = func.memory_size.map(|m| format!("{} MB", m)).unwrap_or_else(|| "-".to_string());
    let timeout_str = func.timeout.map(|t| format!("{} seconds", t)).unwrap_or_else(|| "-".to_string());
    let ephemeral_str = func.ephemeral_storage_size.map(|s| format!("{} MB", s)).unwrap_or_else(|| "512 MB".to_string());
    let last_modified_str = func.last_modified.clone().unwrap_or_else(|| "-".to_string());
    let version_str = func.version.clone().unwrap_or_else(|| "$LATEST".to_string());
    let package_type_str = func.package_type.clone().unwrap_or_else(|| "Zip".to_string());
    let state_str = func.state.clone().unwrap_or_else(|| "Unknown".to_string());
    let arch_str = if func.architectures.is_empty() { "x86_64".to_string() } else { func.architectures.join(", ") };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Function: ", Style::default().fg(THEME.primary)),
            Span::styled(func.function_name.clone(), Style::default().fg(THEME.selection_fg).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled("State: ", Style::default().fg(THEME.primary)),
            Span::styled(state_str, Style::default().fg(state_color)),
        ]),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(THEME.primary)),
            Span::raw(desc_str),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Runtime ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Runtime: ", Style::default().fg(THEME.primary)),
            Span::raw(runtime_str),
            Span::raw("   "),
            Span::styled("Architecture: ", Style::default().fg(THEME.primary)),
            Span::raw(arch_str),
        ]),
        Line::from(vec![
            Span::styled("Handler: ", Style::default().fg(THEME.primary)),
            Span::raw(handler_str),
        ]),
        Line::from(vec![
            Span::styled("Package Type: ", Style::default().fg(THEME.primary)),
            Span::raw(package_type_str),
            Span::raw("   "),
            Span::styled("Version: ", Style::default().fg(THEME.primary)),
            Span::raw(version_str),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("─── Resources ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Memory: ", Style::default().fg(THEME.primary)),
            Span::raw(memory_str),
            Span::raw("   "),
            Span::styled("Timeout: ", Style::default().fg(THEME.primary)),
            Span::raw(timeout_str),
        ]),
        Line::from(vec![
            Span::styled("Code Size: ", Style::default().fg(THEME.primary)),
            Span::raw(func.format_code_size()),
            Span::raw("   "),
            Span::styled("Ephemeral Storage: ", Style::default().fg(THEME.primary)),
            Span::raw(ephemeral_str),
        ]),
    ];

    // VPC Configuration
    if func.vpc_id.is_some() || !func.subnet_ids.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("─── VPC Configuration ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]));
        if let Some(vpc) = &func.vpc_id {
            lines.push(Line::from(vec![
                Span::styled("VPC ID: ", Style::default().fg(THEME.primary)),
                Span::raw(vpc.clone()),
            ]));
        }
        if !func.subnet_ids.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Subnets: ", Style::default().fg(THEME.primary)),
                Span::raw(func.subnet_ids.join(", ")),
            ]));
        }
        if !func.security_group_ids.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Security Groups: ", Style::default().fg(THEME.primary)),
                Span::raw(func.security_group_ids.join(", ")),
            ]));
        }
    }

    // Environment Variables (show count only for security)
    if !func.environment_variables.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Environment Variables: ", Style::default().fg(THEME.primary)),
            Span::raw(format!("{} defined", func.environment_variables.len())),
        ]));
    }

    // Layers
    if !func.layers.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("─── Layers ───", Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD)),
        ]));
        for layer in &func.layers {
            // Extract layer name from ARN
            let layer_name = layer.split(':').nth(6).unwrap_or(layer);
            lines.push(Line::from(vec![
                Span::styled("• ", Style::default().fg(THEME.warning)),
                Span::raw(layer_name.to_string()),
            ]));
        }
    }

    // Last modified and Role
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Last Modified: ", Style::default().fg(THEME.primary)),
        Span::raw(last_modified_str),
    ]));

    if let Some(role) = &func.role {
        // Extract role name from ARN
        let role_name = role.split('/').last().unwrap_or(role);
        lines.push(Line::from(vec![
            Span::styled("IAM Role: ", Style::default().fg(THEME.primary)),
            Span::raw(role_name.to_string()),
        ]));
    }

    // State reason if there's an issue
    if let Some(reason) = &func.state_reason {
        if !reason.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("State Reason: ", Style::default().fg(THEME.warning)),
                Span::raw(reason.clone()),
            ]));
        }
    }

    lines
}
