use crate::app::App;
use crate::models::sqs::SqsQueue;
use crate::models::Filterable;
use crate::ui::components::detail_panel::render_detail_panel_with_selection;
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
    if let Some(area) = list_area {
        render_queue_list(frame, area, app);
    }

    if let Some(area) = detail_area {
        render_queue_details(frame, area, app);
    }
}

fn render_queue_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app
        .services
        .sqs
        .queues
        .iter()
        .filter(|q| {
            if filter.is_empty() {
                return true;
            }
            q.matches_filter(&filter)
        })
        .map(|queue| {
            let queue_type = if queue.is_fifo { "FIFO" } else { "Standard" };
            let dlq_indicator = if queue.has_dlq() { "✓" } else { "-" };
            let msg_color = if queue.approximate_number_of_messages > 0 {
                THEME.warning
            } else {
                THEME.muted
            };

            let cells = vec![
                Cell::from(queue.queue_name.clone()),
                Cell::from(queue_type),
                Cell::from(queue.approximate_number_of_messages.to_string())
                    .style(Style::default().fg(msg_color)),
                Cell::from(queue.approximate_number_of_messages_not_visible.to_string()),
                Cell::from(queue.approximate_number_of_messages_delayed.to_string()),
                Cell::from(dlq_indicator),
            ];

            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &[
            "Queue Name",
            "Type",
            "Messages",
            "In Flight",
            "Delayed",
            "DLQ",
        ],
        &[
            Constraint::Min(30),    // Queue Name
            Constraint::Length(10), // Type
            Constraint::Length(10), // Messages
            Constraint::Length(10), // In Flight
            Constraint::Length(10), // Delayed
            Constraint::Length(5),  // DLQ
        ],
        "SQS Queues",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.sqs.list_state,
    );
}

fn render_queue_details(frame: &mut Frame, area: Rect, app: &App) {
    render_detail_panel_with_selection(
        frame,
        area,
        app.services.sqs.selected_queue(),
        build_queue_detail_lines,
        "Queue Details",
        "Select a queue to view details (use j/k to navigate)",
        app.detail_panel_fullscreen,
        app.detail_scroll_offset,
    );
}

fn build_queue_detail_lines(queue: &SqsQueue) -> Vec<Line<'_>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Queue: ", Style::default().fg(THEME.primary)),
            Span::styled(
                queue.queue_name.clone(),
                Style::default()
                    .fg(THEME.selection_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Type: ", Style::default().fg(THEME.primary)),
            Span::styled(
                queue.queue_type(),
                if queue.is_fifo {
                    Style::default().fg(THEME.warning)
                } else {
                    Style::default()
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("URL: ", Style::default().fg(THEME.primary)),
            Span::raw(queue.queue_url.clone()),
        ]),
    ];

    if let Some(arn) = &queue.queue_arn {
        lines.push(Line::from(vec![
            Span::styled("ARN: ", Style::default().fg(THEME.primary)),
            Span::raw(arn.clone()),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "─── Message Counts ───",
        Style::default()
            .fg(THEME.secondary)
            .add_modifier(Modifier::BOLD),
    )]));

    let msg_color = if queue.approximate_number_of_messages > 0 {
        THEME.warning
    } else {
        THEME.success
    };

    lines.push(Line::from(vec![
        Span::styled("Available: ", Style::default().fg(THEME.primary)),
        Span::styled(
            queue.approximate_number_of_messages.to_string(),
            Style::default().fg(msg_color),
        ),
        Span::raw("   "),
        Span::styled("In Flight: ", Style::default().fg(THEME.primary)),
        Span::raw(queue.approximate_number_of_messages_not_visible.to_string()),
        Span::raw("   "),
        Span::styled("Delayed: ", Style::default().fg(THEME.primary)),
        Span::raw(queue.approximate_number_of_messages_delayed.to_string()),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Total: ", Style::default().fg(THEME.primary)),
        Span::styled(
            queue.total_messages().to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "─── Configuration ───",
        Style::default()
            .fg(THEME.secondary)
            .add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(vec![
        Span::styled("Visibility Timeout: ", Style::default().fg(THEME.primary)),
        Span::raw(format!("{} seconds", queue.visibility_timeout)),
        Span::raw("   "),
        Span::styled("Delay Seconds: ", Style::default().fg(THEME.primary)),
        Span::raw(queue.delay_seconds.to_string()),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Max Message Size: ", Style::default().fg(THEME.primary)),
        Span::raw(queue.format_max_size()),
        Span::raw("   "),
        Span::styled("Retention: ", Style::default().fg(THEME.primary)),
        Span::raw(queue.format_retention()),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Receive Wait Time: ", Style::default().fg(THEME.primary)),
        Span::raw(format!(
            "{} seconds",
            queue.receive_message_wait_time_seconds
        )),
    ]));

    if queue.is_fifo {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "─── FIFO Settings ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![
            Span::styled(
                "Content-Based Deduplication: ",
                Style::default().fg(THEME.primary),
            ),
            Span::styled(
                if queue.content_based_deduplication {
                    "Enabled"
                } else {
                    "Disabled"
                },
                if queue.content_based_deduplication {
                    Style::default().fg(THEME.success)
                } else {
                    Style::default().fg(THEME.muted)
                },
            ),
        ]));
    }

    if queue.has_dlq() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "─── Dead Letter Queue ───",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        )]));
        if let Some(dlq_arn) = &queue.dead_letter_target_arn {
            lines.push(Line::from(vec![
                Span::styled("Target ARN: ", Style::default().fg(THEME.primary)),
                Span::raw(dlq_arn.clone()),
            ]));
        }
    }

    if let Some(created) = &queue.created_timestamp {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Created: ", Style::default().fg(THEME.primary)),
            Span::raw(created.clone()),
        ]));
    }

    if let Some(modified) = &queue.last_modified_timestamp {
        lines.push(Line::from(vec![
            Span::styled("Last Modified: ", Style::default().fg(THEME.primary)),
            Span::raw(modified.clone()),
        ]));
    }

    lines
}
