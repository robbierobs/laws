use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};
use crate::app::App;
use crate::models::lambda::LambdaFunction;
use crate::ui::components::detail_panel::render_detail_panel_with_selection;
use crate::ui::components::table::render_table;

pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    if let Some(area) = list_area {
        render_function_list(frame, area, app);
    }
    
    if let Some(area) = detail_area {
        render_function_details(frame, area, app);
    }
}

use crate::ui::theme::THEME;

fn render_function_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.lambda.functions.iter()
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

    render_table(
        frame,
        area,
        rows,
        &["Function Name", "Runtime", "Memory", "Timeout", "Code Size", "State"],
        &[
            Constraint::Length(35), // Function Name
            Constraint::Length(15), // Runtime
            Constraint::Length(10), // Memory
            Constraint::Length(10), // Timeout
            Constraint::Length(12), // Code Size
            Constraint::Min(10),    // State
        ],
        "Lambda Functions",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.lambda.list_state,
    );
}

fn render_function_details(frame: &mut Frame, area: Rect, app: &App) {
    render_detail_panel_with_selection(
        frame,
        area,
        app.services.lambda.selected_function(),
        build_function_detail_lines,
        "Function Details",
        "Select a Lambda function to view details (use j/k to navigate)",
        app.detail_panel_fullscreen,
        app.detail_scroll_offset,
    );
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
