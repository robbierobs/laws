use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::{App, Service};

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
        _ => {
            actions.push(("j/k", "Navigate"));
        }
    }

    let spans: Vec<Span> = actions
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!(" {} ", key), Style::default().bg(Color::DarkGray).fg(Color::White)),
                Span::raw(format!(" {} ", desc)),
            ]
        })
        .collect();

    let p = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).title("Actions"));

    frame.render_widget(p, area);
}
