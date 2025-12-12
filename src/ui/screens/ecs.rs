//! ECS screen rendering
//!
//! Renders clusters, services, tasks, and task definitions with drill-down navigation.

use crate::app::EcsViewMode;
use crate::app::{App, Focus};
use crate::models::ecs::{
    EcsCluster, EcsContainerDefinition, EcsService, EcsTask, EcsTaskDefinition,
};
use crate::ui::components::table::render_table;
use crate::ui::theme::THEME;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    match app.services.ecs.view_mode {
        EcsViewMode::Clusters => render_clusters_view(frame, list_area, detail_area, app),
        EcsViewMode::Services => render_services_view(frame, list_area, detail_area, app),
        EcsViewMode::Tasks => render_tasks_view(frame, list_area, detail_area, app),
        EcsViewMode::TaskDefinition => {
            render_task_definition_view(frame, list_area, detail_area, app)
        }
    }
}

// ============================================================================
// Clusters View
// ============================================================================

fn render_clusters_view(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    if let Some(area) = list_area {
        let rows = app.services.ecs.clusters.iter().map(|c| {
            let cells = vec![
                Cell::from(c.cluster_name.clone()),
                Cell::from(c.status.clone()).style(Style::default().fg(c.state_color())),
                Cell::from(c.active_services_count.to_string()),
                Cell::from(format!(
                    "{}/{}",
                    c.running_tasks_count, c.pending_tasks_count
                )),
                Cell::from(c.registered_container_instances_count.to_string()),
            ];
            Row::new(cells).height(1)
        });

        render_table(
            frame,
            area,
            rows,
            &["Cluster", "Status", "Services", "Tasks (R/P)", "Instances"],
            &[
                Constraint::Min(30),
                Constraint::Length(12),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Length(10),
            ],
            "ECS Clusters",
            matches!(app.focus, Focus::Main),
            &mut app.services.ecs.list_state.clone(),
        );
    }

    if let Some(area) = detail_area {
        render_cluster_details(frame, area, app.services.ecs.selected_cluster());
    }
}

fn render_cluster_details(frame: &mut Frame, area: Rect, cluster: Option<&EcsCluster>) {
    use crate::ui::components::detail_panel::render_detail_panel_with_selection;

    render_detail_panel_with_selection(
        frame,
        area,
        cluster,
        build_cluster_detail_lines,
        "Cluster Details",
        "Select a cluster (Enter to view services)",
        false,
        0,
    );
}

fn build_cluster_detail_lines(cluster: &EcsCluster) -> Vec<Line<'_>> {
    let state_color = cluster.state_color();
    let active_count = cluster.active_services_count.to_string();
    let running_count = cluster.running_tasks_count.to_string();
    let pending_count = cluster.pending_tasks_count.to_string();
    let instance_count = cluster.registered_container_instances_count.to_string();

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Cluster: ", Style::default().fg(THEME.primary)),
            Span::styled(
                cluster.cluster_name.as_str(),
                Style::default()
                    .fg(THEME.selection_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(cluster.status.as_str(), Style::default().fg(state_color)),
        ]),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
            Span::styled(
                cluster.cluster_arn.as_str(),
                Style::default().fg(THEME.muted),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Statistics ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Active Services: ", Style::default().fg(THEME.primary)),
            Span::raw(active_count),
        ]),
        Line::from(vec![
            Span::styled("Running Tasks: ", Style::default().fg(THEME.primary)),
            Span::raw(running_count),
        ]),
        Line::from(vec![
            Span::styled("Pending Tasks: ", Style::default().fg(THEME.primary)),
            Span::raw(pending_count),
        ]),
        Line::from(vec![
            Span::styled("Container Instances: ", Style::default().fg(THEME.primary)),
            Span::raw(instance_count),
        ]),
    ];

    if !cluster.capacity_providers.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "─── Capacity Providers ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]));
        for cp in &cluster.capacity_providers {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(cp.as_str(), Style::default().fg(THEME.fg)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[Enter] ", Style::default().fg(THEME.warning)),
        Span::raw("view services"),
    ]));

    lines
}

// ============================================================================
// Services View
// ============================================================================

fn render_services_view(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    if let Some(area) = list_area {
        let rows = app.services.ecs.services.iter().map(|s| {
            let cells = vec![
                Cell::from(s.service_name.clone()),
                Cell::from(s.status.clone()).style(Style::default().fg(s.state_color())),
                Cell::from(s.desired_count.to_string()),
                Cell::from(s.running_count.to_string()),
                Cell::from(s.pending_count.to_string()),
                Cell::from(s.launch_type.clone().unwrap_or_else(|| "-".to_string())),
            ];
            Row::new(cells).height(1)
        });

        let title = if let Some(arn) = &app.services.ecs.selected_cluster_arn {
            let name = arn.split('/').last().unwrap_or(arn);
            format!("Services in {}", name)
        } else {
            "Services".to_string()
        };

        render_table(
            frame,
            area,
            rows,
            &[
                "Service", "Status", "Desired", "Running", "Pending", "Launch",
            ],
            &[
                Constraint::Min(30),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(10),
            ],
            &title,
            matches!(app.focus, Focus::Main),
            &mut app.services.ecs.list_state.clone(),
        );
    }

    if let Some(area) = detail_area {
        render_service_details(frame, area, app.services.ecs.selected_service());
    }
}

fn render_service_details(frame: &mut Frame, area: Rect, service: Option<&EcsService>) {
    use crate::ui::components::detail_panel::render_detail_panel_with_selection;

    render_detail_panel_with_selection(
        frame,
        area,
        service,
        build_service_detail_lines,
        "Service Details",
        "Select a service (Enter to view tasks)",
        false,
        0,
    );
}

fn build_service_detail_lines(service: &EcsService) -> Vec<Line<'_>> {
    let state_color = service.state_color();
    let desired = service.desired_count;
    let running = service.running_count;
    let pending = service.pending_count;
    let td_short = service.task_definition_short();
    let desired_str = desired.to_string();
    let running_str = running.to_string();
    let pending_str = pending.to_string();

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Service: ", Style::default().fg(THEME.primary)),
            Span::styled(
                service.service_name.as_str(),
                Style::default()
                    .fg(THEME.selection_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(service.status.as_str(), Style::default().fg(state_color)),
        ]),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
            Span::styled(
                service.service_arn.as_str(),
                Style::default().fg(THEME.muted),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Configuration ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Task Definition: ", Style::default().fg(THEME.primary)),
            Span::raw(td_short),
        ]),
        Line::from(vec![
            Span::styled("Launch Type: ", Style::default().fg(THEME.primary)),
            Span::raw(service.launch_type.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled("Platform Version: ", Style::default().fg(THEME.primary)),
            Span::raw(service.platform_version.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Tasks ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Desired: ", Style::default().fg(THEME.primary)),
            Span::styled(desired_str, Style::default().fg(THEME.fg)),
            Span::raw("   "),
            Span::styled("Running: ", Style::default().fg(THEME.primary)),
            Span::styled(
                running_str,
                Style::default().fg(if running >= desired {
                    THEME.success
                } else {
                    THEME.warning
                }),
            ),
            Span::raw("   "),
            Span::styled("Pending: ", Style::default().fg(THEME.primary)),
            Span::styled(
                pending_str,
                Style::default().fg(if pending > 0 { THEME.warning } else { THEME.fg }),
            ),
        ]),
    ];

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[Enter] ", Style::default().fg(THEME.warning)),
        Span::raw("tasks  "),
        Span::styled("[t] ", Style::default().fg(THEME.warning)),
        Span::raw("view def  "),
        Span::styled("[T] ", Style::default().fg(THEME.warning)),
        Span::raw("change def  "),
        Span::styled("[e] ", Style::default().fg(THEME.warning)),
        Span::raw("edit"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("[d] ", Style::default().fg(THEME.warning)),
        Span::raw("deploy  "),
        Span::styled("[+/-] ", Style::default().fg(THEME.warning)),
        Span::raw("scale  "),
        Span::styled("[Esc] ", Style::default().fg(THEME.warning)),
        Span::raw("back"),
    ]));

    lines
}

// ============================================================================
// Tasks View
// ============================================================================

fn render_tasks_view(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    if let Some(area) = list_area {
        let rows = app.services.ecs.tasks.iter().map(|t| {
            let cells = vec![
                Cell::from(t.task_id()),
                Cell::from(t.last_status.clone()).style(Style::default().fg(t.state_color())),
                Cell::from(t.health_status.clone().unwrap_or_else(|| "-".to_string()))
                    .style(Style::default().fg(t.health_color())),
                Cell::from(t.task_definition_short()),
                Cell::from(t.cpu.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(t.memory.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(t.containers.len().to_string()),
            ];
            Row::new(cells).height(1)
        });

        let title = if let Some(name) = &app.services.ecs.selected_service_name {
            format!("Tasks in {}", name)
        } else {
            "Tasks".to_string()
        };

        render_table(
            frame,
            area,
            rows,
            &[
                "Task ID",
                "Status",
                "Health",
                "Task Def",
                "CPU",
                "Memory",
                "Containers",
            ],
            &[
                Constraint::Min(20),
                Constraint::Length(12),
                Constraint::Length(10),
                Constraint::Length(20),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(10),
            ],
            &title,
            matches!(app.focus, Focus::Main),
            &mut app.services.ecs.list_state.clone(),
        );
    }

    if let Some(area) = detail_area {
        render_task_details(frame, area, app.services.ecs.selected_task());
    }
}

fn render_task_details(frame: &mut Frame, area: Rect, task: Option<&EcsTask>) {
    use crate::ui::components::detail_panel::render_detail_panel_with_selection;

    render_detail_panel_with_selection(
        frame,
        area,
        task,
        build_task_detail_lines,
        "Task Details",
        "Select a task (Enter/t to view task definition)",
        false,
        0,
    );
}

fn build_task_detail_lines(task: &EcsTask) -> Vec<Line<'_>> {
    let task_id = task.task_id();
    let state_color = task.state_color();
    let health_color = task.health_color();
    let td_short = task.task_definition_short();
    let cpu_str = task.cpu.clone().unwrap_or_else(|| "N/A".to_string());
    let mem_str = task.memory.clone().unwrap_or_else(|| "N/A".to_string());
    let health_str = task
        .health_status
        .clone()
        .unwrap_or_else(|| "N/A".to_string());

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Task: ", Style::default().fg(THEME.primary)),
            Span::styled(
                task_id,
                Style::default()
                    .fg(THEME.selection_fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(task.last_status.as_str(), Style::default().fg(state_color)),
            Span::raw(" → "),
            Span::styled(
                task.desired_status.as_str(),
                Style::default().fg(THEME.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled("Health: ", Style::default().fg(THEME.primary)),
            Span::styled(health_str, Style::default().fg(health_color)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Configuration ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Task Definition: ", Style::default().fg(THEME.primary)),
            Span::raw(td_short),
        ]),
        Line::from(vec![
            Span::styled("Launch Type: ", Style::default().fg(THEME.primary)),
            Span::raw(task.launch_type.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled("Availability Zone: ", Style::default().fg(THEME.primary)),
            Span::raw(task.availability_zone.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Resources ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("CPU: ", Style::default().fg(THEME.primary)),
            Span::raw(cpu_str),
            Span::raw("   "),
            Span::styled("Memory: ", Style::default().fg(THEME.primary)),
            Span::raw(mem_str),
        ]),
    ];

    // Containers
    if !task.containers.is_empty() {
        lines.push(Line::from(""));
        let container_header = format!("─── Containers ({}) ───", task.containers.len());
        lines.push(Line::from(vec![Span::styled(
            container_header,
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]));
        for container in &task.containers {
            let status_color = match container.last_status.as_deref() {
                Some("RUNNING") => THEME.success,
                Some("PENDING") => THEME.warning,
                Some("STOPPED") => THEME.error,
                _ => THEME.fg,
            };
            let container_status = container
                .last_status
                .clone()
                .unwrap_or_else(|| "?".to_string());
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(container.name.as_str(), Style::default().fg(THEME.primary)),
                Span::raw(": "),
                Span::styled(container_status, Style::default().fg(status_color)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[Enter/t] ", Style::default().fg(THEME.warning)),
        Span::raw("task def  "),
        Span::styled("[S] ", Style::default().fg(THEME.warning)),
        Span::raw("stop task  "),
        Span::styled("[Esc] ", Style::default().fg(THEME.warning)),
        Span::raw("back"),
    ]));

    lines
}

// ============================================================================
// Task Definition View
// ============================================================================

fn render_task_definition_view(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    // In task definition view, we use the full area for detailed view
    let area = list_area.or(detail_area).unwrap_or(frame.area());

    if let Some(td) = &app.services.ecs.current_task_definition {
        let lines = build_task_definition_lines(td);
        let total_lines = lines.len();
        let scroll_offset = app.services.ecs.detail_scroll_offset;

        // Calculate visible lines based on area height (minus border)
        let visible_height = area.height.saturating_sub(2) as usize;
        let max_scroll = total_lines.saturating_sub(visible_height);
        let actual_offset = scroll_offset.min(max_scroll);

        // Update the scroll offset if it was clamped
        if actual_offset != scroll_offset {
            app.services.ecs.detail_scroll_offset = actual_offset;
        }

        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(actual_offset)
            .take(visible_height)
            .collect();

        let title = format!(
            "Task Definition: {} (scroll: {}/{})",
            td.short_name(),
            actual_offset + 1,
            total_lines
        );

        let paragraph = Paragraph::new(visible_lines)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(Style::default().fg(THEME.border))
                    .title(title)
                    .title_style(Style::default().fg(THEME.primary)),
            )
            .style(Style::default().fg(THEME.fg));

        frame.render_widget(paragraph, area);

        // Render scrollbar if content exceeds visible area
        if total_lines > visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));

            let mut scrollbar_state = ScrollbarState::new(total_lines).position(actual_offset);

            frame.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    } else {
        // Loading state
        let loading = Paragraph::new("Loading task definition...")
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(Style::default().fg(THEME.border))
                    .title("Task Definition")
                    .title_style(Style::default().fg(THEME.primary)),
            )
            .style(Style::default().fg(THEME.muted));
        frame.render_widget(loading, area);
    }
}

fn build_task_definition_lines(td: &EcsTaskDefinition) -> Vec<Line<'static>> {
    let short_name = td.short_name();
    let status = td.status.clone();
    let status_color = td.status_color();
    let arn = td.task_definition_arn.clone();
    let revision_str = td.revision.to_string();
    let compat_str = td.requires_compatibilities.join(", ");

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                short_name,
                Style::default()
                    .fg(THEME.selection_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(status, Style::default().fg(status_color)),
        ]),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
            Span::styled(arn, Style::default().fg(THEME.muted)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Overview ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Family: ", Style::default().fg(THEME.primary)),
            Span::raw(td.family.clone()),
        ]),
        Line::from(vec![
            Span::styled("Revision: ", Style::default().fg(THEME.primary)),
            Span::raw(revision_str),
        ]),
        Line::from(vec![
            Span::styled("Network Mode: ", Style::default().fg(THEME.primary)),
            Span::raw(td.network_mode.clone().unwrap_or_else(|| "N/A".to_string())),
        ]),
        Line::from(vec![
            Span::styled("CPU: ", Style::default().fg(THEME.primary)),
            Span::raw(td.cpu.clone().unwrap_or_else(|| "N/A".to_string())),
            Span::raw("   "),
            Span::styled("Memory: ", Style::default().fg(THEME.primary)),
            Span::raw(td.memory.clone().unwrap_or_else(|| "N/A".to_string())),
        ]),
    ];

    if !td.requires_compatibilities.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Compatibilities: ", Style::default().fg(THEME.primary)),
            Span::raw(compat_str),
        ]));
    }

    // Roles
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "─── IAM Roles ───",
        Style::default()
            .fg(THEME.secondary)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![
        Span::styled("Task Role: ", Style::default().fg(THEME.primary)),
        Span::raw(
            td.task_role_arn
                .clone()
                .unwrap_or_else(|| "None".to_string()),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Execution Role: ", Style::default().fg(THEME.primary)),
        Span::raw(
            td.execution_role_arn
                .clone()
                .unwrap_or_else(|| "None".to_string()),
        ),
    ]));

    // Container Definitions
    lines.push(Line::from(""));
    let container_header = format!(
        "─── Container Definitions ({}) ───",
        td.container_definitions.len()
    );
    lines.push(Line::from(vec![Span::styled(
        container_header,
        Style::default()
            .fg(THEME.secondary)
            .add_modifier(Modifier::BOLD),
    )]));

    for (i, cd) in td.container_definitions.iter().enumerate() {
        lines.extend(build_container_definition_lines(cd, i));
    }

    // Hints
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[j/k] ", Style::default().fg(THEME.warning)),
        Span::raw("scroll  "),
        Span::styled("[E] ", Style::default().fg(THEME.warning)),
        Span::raw("edit  "),
        Span::styled("[X] ", Style::default().fg(THEME.warning)),
        Span::raw("deregister  "),
        Span::styled("[Esc] ", Style::default().fg(THEME.warning)),
        Span::raw("back"),
    ]));

    lines
}

fn build_container_definition_lines(
    cd: &EcsContainerDefinition,
    index: usize,
) -> Vec<Line<'static>> {
    let name = cd.name.clone();
    let essential_span = if cd.essential {
        Span::styled(" (essential)", Style::default().fg(THEME.warning))
    } else {
        Span::raw("")
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {}. ", index + 1),
                Style::default().fg(THEME.muted),
            ),
            Span::styled(
                name,
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            essential_span,
        ]),
    ];

    if let Some(image) = &cd.image {
        lines.push(Line::from(vec![
            Span::raw("     Image: "),
            Span::styled(image.clone(), Style::default().fg(THEME.fg)),
        ]));
    }

    let memory_str = cd
        .memory
        .map(|m| m.to_string())
        .unwrap_or_else(|| "N/A".to_string());
    let mem_res_str = cd
        .memory_reservation
        .map(|m| m.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    lines.push(Line::from(vec![
        Span::raw("     CPU: "),
        Span::raw(cd.cpu.to_string()),
        Span::raw("  Memory: "),
        Span::raw(memory_str),
        Span::raw("  MemRes: "),
        Span::raw(mem_res_str),
    ]));

    // Port mappings
    if !cd.port_mappings.is_empty() {
        let ports: Vec<String> = cd
            .port_mappings
            .iter()
            .map(|pm| {
                format!(
                    "{}:{}",
                    pm.host_port.unwrap_or(0),
                    pm.container_port.unwrap_or(0)
                )
            })
            .collect();
        lines.push(Line::from(vec![
            Span::raw("     Ports: "),
            Span::raw(ports.join(", ")),
        ]));
    }

    // Environment variables (show count)
    if !cd.environment.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("     Env Vars: "),
            Span::raw(format!("{} defined", cd.environment.len())),
        ]));
    }

    // Secrets (show count)
    if !cd.secrets.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("     Secrets: "),
            Span::raw(format!("{} defined", cd.secrets.len())),
        ]));
    }

    // Log configuration
    if let Some(log_config) = &cd.log_configuration {
        lines.push(Line::from(vec![
            Span::raw("     Logging: "),
            Span::raw(log_config.log_driver.clone()),
        ]));
    }

    // Health check
    if let Some(hc) = &cd.health_check {
        let cmd = hc.command.join(" ");
        let cmd_display = if cmd.len() > 40 {
            format!("{}...", &cmd[..37])
        } else {
            cmd
        };
        lines.push(Line::from(vec![
            Span::raw("     Health: "),
            Span::raw(cmd_display),
        ]));
    }

    // Mount points
    if !cd.mount_points.is_empty() {
        let mounts: Vec<String> = cd
            .mount_points
            .iter()
            .filter_map(|mp| {
                Some(format!(
                    "{}:{}",
                    mp.source_volume.as_deref()?,
                    mp.container_path.as_deref()?
                ))
            })
            .collect();
        if !mounts.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("     Mounts: "),
                Span::raw(mounts.join(", ")),
            ]));
        }
    }

    lines
}
