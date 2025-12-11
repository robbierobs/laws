use crate::app::EcsViewMode;
use crate::app::{App, Focus};
use crate::models::ecs::{EcsCluster, EcsService};
use crate::ui::components::table::render_table;
use crate::ui::theme::THEME;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    let mode = app.services.ecs.view_mode;

    match mode {
        EcsViewMode::Clusters => {
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
                    ];
                    Row::new(cells).height(1)
                });

                render_table(
                    frame,
                    area,
                    rows,
                    &["Cluster Name", "Status", "Services", "Tasks (Run/Pend)"],
                    &[
                        Constraint::Length(40),
                        Constraint::Length(15),
                        Constraint::Length(10),
                        Constraint::Length(20),
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
        EcsViewMode::Services => {
            if let Some(area) = list_area {
                let rows = app.services.ecs.services.iter().map(|s| {
                    let cells = vec![
                        Cell::from(s.service_name.clone()),
                        Cell::from(s.status.clone()).style(Style::default().fg(s.state_color())),
                        Cell::from(s.desired_count.to_string()),
                        Cell::from(s.running_count.to_string()),
                        Cell::from(s.pending_count.to_string()),
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
                    &["Service Name", "Status", "Desired", "Running", "Pending"],
                    &[
                        Constraint::Length(40),
                        Constraint::Length(15),
                        Constraint::Length(10),
                        Constraint::Length(10),
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
        "Select a cluster to view details (Enter to view services)",
        false,
        0,
    );
}

fn render_service_details(frame: &mut Frame, area: Rect, service: Option<&EcsService>) {
    use crate::ui::components::detail_panel::render_detail_panel_with_selection;

    render_detail_panel_with_selection(
        frame,
        area,
        service,
        build_service_detail_lines,
        "Service Details",
        "Select a service to view details",
        false,
        0,
    );
}

fn build_cluster_detail_lines(cluster: &EcsCluster) -> Vec<Line<'_>> {
    vec![
        Line::from(vec![
            Span::styled("Cluster: ", Style::default().fg(THEME.primary)),
            Span::styled(
                &cluster.cluster_name,
                Style::default()
                    .fg(THEME.selection_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(&cluster.status, Style::default().fg(cluster.state_color())),
        ]),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
            Span::raw(&cluster.cluster_arn),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "─── Stats ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Active Services: ", Style::default().fg(THEME.primary)),
            Span::raw(cluster.active_services_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Running Tasks: ", Style::default().fg(THEME.primary)),
            Span::raw(cluster.running_tasks_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Pending Tasks: ", Style::default().fg(THEME.primary)),
            Span::raw(cluster.pending_tasks_count.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Hint: ", Style::default().fg(THEME.warning)),
            Span::raw("Press Enter to view services within this cluster"),
        ]),
    ]
}

fn build_service_detail_lines(service: &EcsService) -> Vec<Line<'_>> {
    vec![
        Line::from(vec![
            Span::styled("Service: ", Style::default().fg(THEME.primary)),
            Span::styled(
                &service.service_name,
                Style::default()
                    .fg(THEME.selection_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(THEME.primary)),
            Span::styled(&service.status, Style::default().fg(service.state_color())),
        ]),
        Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
            Span::raw(&service.service_arn),
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
            Span::raw(
                service
                    .task_definition
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Launch Type: ", Style::default().fg(THEME.primary)),
            Span::raw(
                service
                    .launch_type
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string()),
            ),
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
            Span::raw(service.desired_count.to_string()),
            Span::raw("   "),
            Span::styled("Running: ", Style::default().fg(THEME.primary)),
            Span::raw(service.running_count.to_string()),
            Span::raw("   "),
            Span::styled("Pending: ", Style::default().fg(THEME.primary)),
            Span::raw(service.pending_count.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Hint: ", Style::default().fg(THEME.warning)),
            Span::raw("Press Esc to go back to clusters list"),
        ]),
    ]
}
