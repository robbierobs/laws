use crate::app::{App, InputMode, Message};
use crate::ui::components::Component;
use crate::ui::theme::THEME;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Body
            Constraint::Length(3), // Action Log / Action Bar
        ])
        .split(frame.area());

    // Build header text with status
    let is_loading = app.loading || app.tasks.active_count() > 0;
    let status = if is_loading {
        " [Loading...]"
    } else if app.detail_loading {
        " [Fetching details...]"
    } else {
        ""
    };

    // Build profile/region info (cached to avoid repeated allocations)
    let aws_info = app
        .render_cache
        .get_aws_info(app.profile.as_deref(), &app.region)
        .to_string();

    let filter_status = if !app.filter_input.is_empty() {
        format!(" [Filter: {}]", app.filter_input)
    } else {
        String::new()
    };

    let header_text = if let Some(ref err) = app.error_message {
        format!(
            "laws - {:?} {} | Error: {}",
            app.current_service, aws_info, err
        )
    } else {
        let read_only_status = if app.read_only { " [READ-ONLY]" } else { "" };
        format!(
            "laws - {:?} {}{}{}{}",
            app.current_service, aws_info, status, filter_status, read_only_status
        )
    };

    // If read-only, we want a more prominent warning.
    // Let's make the title background yellow if read-only, or just the text?
    // User asked for "yellow background so it stands out".
    let (header_style, block_style) = if app.read_only {
        (
            Style::default()
                .fg(Color::Black)
                .bg(THEME.warning)
                .add_modifier(ratatui::style::Modifier::BOLD),
            Style::default().fg(THEME.warning),
        )
    } else if app.error_message.is_some() {
        (
            Style::default().fg(THEME.error),
            Style::default().fg(THEME.error),
        )
    } else {
        (
            Style::default().fg(THEME.primary),
            Style::default().fg(THEME.border),
        )
    };

    let title = Paragraph::new(header_text).style(header_style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(block_style)
            .title("laws")
            .title_style(Style::default().fg(if app.read_only {
                Color::Black
            } else {
                THEME.secondary
            })),
    );
    frame.render_widget(title, chunks[0]);

    // Body split - sidebar on left, main content on right
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(app.config.sidebar_width),
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    // Render Sidebar
    app.sidebar.render(frame, body_chunks[0]);

    // Split main content area for list and details
    // In fullscreen mode, detail panel takes the entire area
    let (list_area, detail_area): (Option<Rect>, Option<Rect>) = if app.detail_panel_fullscreen {
        // Fullscreen: only show detail panel
        (None, Some(body_chunks[1]))
    } else if app.detail_panel_visible {
        // Normal: split between list and detail
        let detail_percent = app.config.detail_panel_percent;
        let list_percent = 100u16.saturating_sub(detail_percent);
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(list_percent),
                Constraint::Percentage(detail_percent),
            ])
            .split(body_chunks[1]);
        (Some(content_chunks[0]), Some(content_chunks[1]))
    } else {
        // No detail panel: list takes full area
        (Some(body_chunks[1]), None)
    };

    // Render the current service screen using polymorphic dispatch
    let screen = crate::ui::screens::get_screen(app.current_service);
    screen.render(frame, list_area, detail_area, app);

    // Footer / Action Bar with last action hint
    if app.input_mode == crate::app::InputMode::Filtering {
        let input = Paragraph::new(format!("/{}", app.filter_input))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Filter"));
        frame.render_widget(input, chunks[2]);
    } else {
        // Show action bar with last action embedded
        crate::ui::components::action_bar::render(frame, chunks[2], app);
    }

    // Render action log popup if expanded
    if app.action_log_expanded {
        crate::ui::components::action_log::render_popup(frame, frame.area(), app);
    }

    // Render all modals
    render_modals(frame, app);
}

fn render_modals(frame: &mut Frame, app: &mut App) {
    // Render S3 object viewer popup
    if app.services.s3.show_object_viewer {
        let object_key = app
            .services
            .s3
            .opened_object_key
            .as_deref()
            .unwrap_or("Unknown");
        let object_path = app.services.s3.opened_object_path.as_deref();
        let content = app.services.s3.opened_object_content.as_deref();
        let raw_bytes = app.services.s3.opened_object_bytes.as_deref();
        let scroll = app.services.s3.viewer_scroll_offset;
        let viewer_mode = app.services.s3.viewer_mode;

        crate::ui::components::modal::render_object_viewer_modal(
            frame,
            frame.area(),
            object_key,
            object_path,
            content,
            raw_bytes,
            scroll,
            viewer_mode,
        );
    }

    // Render S3 bucket creation modal
    if app.input_mode == InputMode::S3BucketCreation {
        crate::ui::components::modal::render_s3_bucket_creation_modal(
            frame,
            frame.area(),
            &app.services.s3.create_bucket_input,
        );
    }

    if app.show_confirmation {
        if let Some(action) = &app.pending_action {
            // Use ConfirmableAction trait for description instead of 110+ line match
            let description = match action {
                Message::Service(service_action) => {
                    use crate::app::messages::ConfirmableAction;
                    service_action.confirmation_description()
                }
                _ => "Unknown Action".to_string(),
            };
            crate::ui::components::modal::render_confirmation_modal(
                frame,
                frame.area(),
                &description,
            );
        }
    }

    // Render profile switcher modal
    if app.input_mode == InputMode::ProfileSwitcherProfile {
        let filtered: Vec<String> = app
            .profile_switcher
            .filtered_profiles()
            .into_iter()
            .cloned()
            .collect();
        crate::ui::components::modal::render_profile_switcher_modal(
            frame,
            frame.area(),
            &filtered,
            app.profile_switcher.profile_switcher_index,
            app.profile.as_deref(),
            app.profile_switcher.pending_read_only,
            &app.profile_switcher.profile_filter,
            app.profile_switcher.profile_filter_active,
        );
    }

    // Render region switcher modal
    if app.input_mode == InputMode::ProfileSwitcherRegion {
        let filtered: Vec<String> = app
            .profile_switcher
            .filtered_regions()
            .into_iter()
            .cloned()
            .collect();
        crate::ui::components::modal::render_region_switcher_modal(
            frame,
            frame.area(),
            &filtered,
            app.profile_switcher.region_switcher_index,
            &app.region,
            app.profile_switcher.pending_profile.as_deref(),
            &app.profile_switcher.region_filter,
            app.profile_switcher.region_filter_active,
        );
    }

    // Render ECS service editor modal
    if app.input_mode == InputMode::EcsServiceEditor {
        let service_name = app
            .services
            .ecs
            .service_editor
            .service_name
            .as_deref()
            .unwrap_or("Unknown");
        crate::ui::components::modal::render_ecs_service_editor_modal(
            frame,
            frame.area(),
            service_name,
            &app.services.ecs.service_editor.task_def,
            &app.services.ecs.service_editor.cpu,
            &app.services.ecs.service_editor.memory,
            app.services.ecs.service_editor.force_deploy,
            app.services.ecs.service_editor.active_field,
        );
    }

    // Render ECS task definition selector modal
    if app.input_mode == InputMode::EcsTaskDefSelector {
        let service_name = app
            .services
            .ecs
            .task_def_selector
            .service_name
            .as_deref()
            .unwrap_or("Unknown");
        crate::ui::components::modal::render_task_def_selector_modal(
            frame,
            frame.area(),
            service_name,
            &app.services.ecs.task_def_selector.list,
            app.services.ecs.task_def_selector.index,
            app.services.ecs.task_def_selector.force_deploy,
            app.services.ecs.task_def_selector.detail_scroll,
            app.services.ecs.task_def_selector.loading,
        );
    }

    // Render global search modal
    if app.input_mode == InputMode::GlobalSearch {
        crate::ui::components::modal::render_global_search_modal(
            frame,
            frame.area(),
            &app.global_search.query,
            app.global_search.results.as_slice(),
            app.global_search.selected_index,
            app.global_search.tag_search_mode,
            app.global_search.loading,
            app.global_search.spinner(),
        );
    }
}
