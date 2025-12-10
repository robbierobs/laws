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
            if app.current_bucket.is_some() {
                actions.extend_from_slice(&[
                    ("j/k", "Navigate"),
                    ("Esc", "Back"),
                ]);
            } else {
                actions.extend_from_slice(&[
                    ("j/k", "Navigate"),
                    ("Enter", "Browse"),
                    ("i", "Info"),
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

    let p = Paragraph::new(Line::from(spans))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border))
            .title("Actions")
            .title_style(Style::default().fg(THEME.secondary)));

    frame.render_widget(p, area);
}
