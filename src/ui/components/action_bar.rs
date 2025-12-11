use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::{App, Service};

use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut actions = vec![
        ("q", "Quit"),
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
            ]);
        }
        Service::S3 => {
            if app.services.s3.current_bucket.is_some() {
                actions.extend_from_slice(&[
                    ("j/k", "Navigate"),
                    ("o", "Open"),
                    ("w", "Download"),
                    ("Esc", "Back"),
                ]);
            } else {
                actions.extend_from_slice(&[
                    ("j/k", "Navigate"),
                    ("Enter", "Browse"),
                ]);
            }
        }
        Service::RDS => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("s", "Start"),
                ("S", "Stop"),
                ("R", "Reboot"),
            ]);
        }
        Service::DynamoDB => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
            ]);
        }
        Service::Lambda => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
            ]);
        }
        Service::VPC => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("v", "View"),
                ("Enter", "Drill Down"),
                ("Esc", "Back"),
            ]);
        }
        Service::IAM => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("v", "View"),
                ("Enter", "Drill Down"),
                ("Esc", "Back"),
            ]);
        }
        Service::Backup => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("v", "View"),
            ]);
        }
        Service::CloudTrail => {
            actions.extend_from_slice(&[
                ("j/k", "Navigate"),
                ("v", "View"),
            ]);
        }
    }

    let spans: Vec<Span> = actions
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!(" {} ", key), Style::default().bg(THEME.selection_bg).fg(THEME.primary)),
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
    let title_style = if app.action_log.last().map(|s| s.contains("[ERROR]")).unwrap_or(false) {
        Style::default().fg(THEME.error)
    } else if app.action_log.last().map(|s| s.contains("[SUCCESS]")).unwrap_or(false) {
        Style::default().fg(THEME.success)
    } else {
        Style::default().fg(THEME.secondary)
    };

    let p = Paragraph::new(Line::from(spans))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border))
            .title(title)
            .title_style(title_style));

    frame.render_widget(p, area);
}
