//! Budgets service screen

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::app::{App, BudgetsViewMode};
use crate::ui::theme::THEME;

pub fn render(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    match app.services.budgets.view_mode {
        BudgetsViewMode::Budgets => render_budgets_list(frame, list_area, detail_area, app),
        BudgetsViewMode::Notifications => render_notifications(frame, list_area, detail_area, app),
        BudgetsViewMode::BillingViews => render_billing_views(frame, list_area, detail_area, app),
    }
}

fn render_budgets_list(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    if let Some(area) = list_area {
        let header_cells = ["Name", "Type", "Limit", "Actual", "Usage %"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1);

        let rows = app.services.budgets.budgets.iter().map(|b| {
            let usage_pct = b.usage_percentage().map(|p| format!("{:.1}%", p)).unwrap_or_default();
            let usage_color = b.status_color();
            
            Row::new(vec![
                Cell::from(b.budget_name.as_str()),
                Cell::from(b.budget_type.as_str()),
                Cell::from(b.budget_limit.map(|l| format!("${:.2}", l)).unwrap_or_default()),
                Cell::from(b.actual_spend.map(|s| format!("${:.2}", s)).unwrap_or_default()),
                Cell::from(usage_pct).style(Style::default().fg(usage_color)),
            ])
        });

        let table = Table::new(
            rows,
            [
                ratatui::layout::Constraint::Percentage(30),
                ratatui::layout::Constraint::Percentage(15),
                ratatui::layout::Constraint::Percentage(18),
                ratatui::layout::Constraint::Percentage(18),
                ratatui::layout::Constraint::Percentage(19),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
                .title(format!(" Budgets ({}) ", app.services.budgets.budgets.len())),
        )
        .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg));

        frame.render_stateful_widget(table, area, &mut app.services.budgets.list_state);
    }

    if let Some(area) = detail_area {
        let detail_text = if let Some(budget) = app.services.budgets.selected_budget() {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(THEME.secondary)),
                    Span::raw(&budget.budget_name),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(THEME.secondary)),
                    Span::raw(&budget.budget_type),
                ]),
                Line::from(vec![
                    Span::styled("Time Unit: ", Style::default().fg(THEME.secondary)),
                    Span::raw(&budget.time_unit),
                ]),
                Line::from(""),
            ];

            if let Some(limit) = budget.budget_limit {
                lines.push(Line::from(vec![
                    Span::styled("Limit: ", Style::default().fg(THEME.secondary)),
                    Span::raw(format!("${:.2}", limit)),
                ]));
            }
            if let Some(actual) = budget.actual_spend {
                lines.push(Line::from(vec![
                    Span::styled("Actual Spend: ", Style::default().fg(THEME.secondary)),
                    Span::styled(format!("${:.2}", actual), Style::default().fg(budget.status_color())),
                ]));
            }
            if let Some(forecasted) = budget.forecasted_spend {
                lines.push(Line::from(vec![
                    Span::styled("Forecasted: ", Style::default().fg(THEME.secondary)),
                    Span::raw(format!("${:.2}", forecasted)),
                ]));
            }
            if let Some(pct) = budget.usage_percentage() {
                lines.push(Line::from(vec![
                    Span::styled("Usage: ", Style::default().fg(THEME.secondary)),
                    Span::styled(format!("{:.1}%", pct), Style::default().fg(budget.status_color())),
                ]));
            }

            lines
        } else {
            vec![Line::from("Select a budget to view details")]
        };

        let paragraph = Paragraph::new(detail_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
                .title(" Budget Details "),
        );
        frame.render_widget(paragraph, area);
    }
}

fn render_notifications(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    let budget_name = app.services.budgets.selected_budget.as_deref().unwrap_or("Unknown");

    if let Some(area) = list_area {
        let header_cells = ["Type", "Operator", "Threshold", "State"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1);

        let rows = app.services.budgets.notifications.iter().map(|n| {
            let state_color = n.state_color();
            Row::new(vec![
                Cell::from(n.notification_type.as_str()),
                Cell::from(n.comparison_operator.as_str()),
                Cell::from(n.threshold_display()),
                Cell::from(n.notification_state.as_deref().unwrap_or("Unknown"))
                    .style(Style::default().fg(state_color)),
            ])
        });

        let table = Table::new(
            rows,
            [
                ratatui::layout::Constraint::Percentage(25),
                ratatui::layout::Constraint::Percentage(25),
                ratatui::layout::Constraint::Percentage(25),
                ratatui::layout::Constraint::Percentage(25),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
                .title(format!(" Notifications for '{}' ({}) ", budget_name, app.services.budgets.notifications.len())),
        )
        .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg));

        frame.render_stateful_widget(table, area, &mut app.services.budgets.list_state);
    }

    if let Some(area) = detail_area {
        let detail_text = if let Some(notif) = app.services.budgets.selected_notification() {
            vec![
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(THEME.secondary)),
                    Span::raw(&notif.notification_type),
                ]),
                Line::from(vec![
                    Span::styled("Operator: ", Style::default().fg(THEME.secondary)),
                    Span::raw(&notif.comparison_operator),
                ]),
                Line::from(vec![
                    Span::styled("Threshold: ", Style::default().fg(THEME.secondary)),
                    Span::raw(notif.threshold_display()),
                ]),
                Line::from(vec![
                    Span::styled("State: ", Style::default().fg(THEME.secondary)),
                    Span::styled(
                        notif.notification_state.as_deref().unwrap_or("Unknown"),
                        Style::default().fg(notif.state_color()),
                    ),
                ]),
            ]
        } else {
            vec![Line::from("Select a notification to view details")]
        };

        let paragraph = Paragraph::new(detail_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
                .title(" Notification Details "),
        );
        frame.render_widget(paragraph, area);
    }
}

fn render_billing_views(
    frame: &mut Frame,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    app: &mut App,
) {
    if let Some(area) = list_area {
        let header_cells = ["Name", "Owner Account", "ARN"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(THEME.primary)));
        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1);

        let rows = app.services.budgets.billing_views.iter().map(|v| {
            Row::new(vec![
                Cell::from(v.name.as_str()),
                Cell::from(v.owner_account_id.as_str()),
                Cell::from(v.arn.as_str()),
            ])
        });

        let table = Table::new(
            rows,
            [
                ratatui::layout::Constraint::Percentage(25),
                ratatui::layout::Constraint::Percentage(25),
                ratatui::layout::Constraint::Percentage(50),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
                .title(format!(" Billing Views ({}) ", app.services.budgets.billing_views.len())),
        )
        .row_highlight_style(Style::default().bg(THEME.selection_bg).fg(THEME.selection_fg));

        frame.render_stateful_widget(table, area, &mut app.services.budgets.list_state);
    }

    if let Some(area) = detail_area {
        let detail_text = if let Some(view) = app.services.budgets.selected_billing_view() {
            vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(THEME.secondary)),
                    Span::raw(&view.name),
                ]),
                Line::from(vec![
                    Span::styled("Owner Account: ", Style::default().fg(THEME.secondary)),
                    Span::raw(&view.owner_account_id),
                ]),
                Line::from(vec![
                    Span::styled("ARN: ", Style::default().fg(THEME.secondary)),
                    Span::raw(&view.arn),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(THEME.secondary)),
                    Span::raw(view.description.as_deref().unwrap_or("N/A")),
                ]),
            ]
        } else {
            vec![Line::from("Select a billing view to view details")]
        };

        let paragraph = Paragraph::new(detail_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
                .title(" Billing View Details "),
        );
        frame.render_widget(paragraph, area);
    }
}
