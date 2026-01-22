use crate::app::{App, Service, ViewMode};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut actions = vec![
        ("q", "Quit"),
        ("?", "Search"),
        ("Tab", "Focus"),
        ("d", "Details"),
        ("1-9", "Service"),
    ];

    match app.current_service {
        Service::EC2 => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("s", "Start"),
                ("S", "Stop"),
                ("R", "Reboot"),
                ("X", "Terminate"),
            ]);
        }
        Service::S3 => {
            if app.services.s3.current_bucket.is_some() {
                actions.extend_from_slice(&[
                    ("j/k", "Navigate"),
                    ("o", "Open"),
                    ("w", "Download"),
                    ("X", "Delete"),
                    ("Esc", "Back"),
                ]);
            } else {
                actions.extend_from_slice(&[("j/k", "Navigate"), ("Enter", "Browse")]);
            }
        }
        Service::RDS => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("s", "Start"),
                ("S", "Stop"),
                ("R", "Reboot"),
                ("X", "Delete"),
            ]);
        }
        Service::DynamoDB => {
            actions.extend_from_slice(&[("j/k", "Navigate"), ("Enter", "Drill Down")]);
            if app.services.dynamodb.is_viewing_items() {
                actions.extend_from_slice(&[("X", "Delete"), ("Esc", "Back")]);
            }
        }
        Service::Lambda => {
            actions.extend_from_slice(&[("j/k", "Navigate"), ("I", "Invoke"), ("X", "Delete")]);
        }
        Service::VPC => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("v", "View"),
                ("Enter", "Drill Down"),
            ]);
            if app.services.vpc.view_mode == crate::app::VpcViewMode::SecurityGroups {
                actions.extend_from_slice(&[("X", "Delete")]);
            }
            actions.extend_from_slice(&[("Esc", "Back")]);
        }
        Service::IAM => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("v", "View"),
                ("Enter", "Drill Down"),
            ]);
            if app.services.iam.view_mode.is_main_tab() {
                actions.extend_from_slice(&[("X", "Delete")]);
            }
            actions.extend_from_slice(&[("Esc", "Back")]);
        }
        Service::Backup => {
            actions.extend_from_slice(&[("j/k", "Navigate"), ("v", "View")]);
        }
        Service::CloudTrail => {
            actions.extend_from_slice(&[("j/k", "Navigate"), ("v", "View")]);
        }
        Service::SecretsManager => {
            actions.extend_from_slice(&[("j/k", "Navigate"), ("s", "Get Value"), ("X", "Delete")]);
        }
        Service::ECS => {
            use crate::app::EcsViewMode;
            actions.extend_from_slice(&[("j/k", "Navigate")]);

            match app.services.ecs.view_mode {
                EcsViewMode::Clusters => {
                    actions.extend_from_slice(&[("Enter", "Services")]);
                }
                EcsViewMode::Services => {
                    actions.extend_from_slice(&[
                        ("Enter", "Tasks"),
                        ("t", "View Def"),
                        ("T", "Change Def"),
                        ("e", "Edit"),
                        ("d", "Deploy"),
                        ("+/-", "Scale"),
                        ("Esc", "Back"),
                    ]);
                }
                EcsViewMode::Tasks => {
                    actions.extend_from_slice(&[("t", "Task Def"), ("S", "Stop"), ("Esc", "Back")]);
                }
                EcsViewMode::TaskDefinition => {
                    actions.extend_from_slice(&[
                        ("j/k", "Scroll"),
                        ("E", "Edit"),
                        ("X", "Deregister"),
                        ("Esc", "Back"),
                    ]);
                }
            }
        }
        Service::ECR => {
            use crate::app::EcrViewMode;
            actions.extend_from_slice(&[("j/k", "Navigate")]);
            match app.services.ecr.view_mode {
                EcrViewMode::Repositories => {
                    actions.extend_from_slice(&[("Enter", "View Images")]);
                }
                EcrViewMode::Images => {
                    actions.extend_from_slice(&[("p", "Pull"), ("s", "Sort"), ("F", "Filter")]);
                    if app.services.ecr.images.has_more {
                        actions.push(("L", "More"));
                    }
                    actions.push(("Esc", "Back"));
                }
            }
        }
        Service::Budgets => {
            use crate::app::BudgetsViewMode;
            actions.extend_from_slice(&[("j/k", "Navigate")]);
            match app.services.budgets.view_mode {
                BudgetsViewMode::Budgets => {
                    actions.extend_from_slice(&[("Enter", "Alerts"), ("v", "View")]);
                }
                BudgetsViewMode::Notifications => {
                    actions.extend_from_slice(&[("Esc", "Back")]);
                }
                BudgetsViewMode::BillingViews => {
                    actions.extend_from_slice(&[("v", "View")]);
                }
            }
        }
    }

    let spans: Vec<Span> = actions
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default().bg(THEME.selection_bg).fg(THEME.primary),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(THEME.fg)),
            ]
        })
        .collect();

    // Build title with last action
    let title = if let Some(last_action) = app.action_log.last() {
        // Truncate long messages
        let truncated = if last_action.len() > 50 {
            format!("{}...", &last_action[..47])
        } else {
            last_action.clone()
        };
        format!("Actions | {} | A: Log", truncated)
    } else {
        "Actions | A: Log".to_string()
    };

    // Color the title based on last action type
    let title_style = if app
        .action_log
        .last()
        .map(|s| s.contains("[ERROR]"))
        .unwrap_or(false)
    {
        Style::default().fg(THEME.error)
    } else if app
        .action_log
        .last()
        .map(|s| s.contains("[SUCCESS]"))
        .unwrap_or(false)
    {
        Style::default().fg(THEME.success)
    } else {
        Style::default().fg(THEME.secondary)
    };

    let p = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border))
            .title(title)
            .title_style(title_style),
    );

    frame.render_widget(p, area);
}
