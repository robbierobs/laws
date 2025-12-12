use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::ui::components::Component;
use crate::app::{App, InputMode, Message};
use crate::app::messages::{ServiceAction, Ec2Action, S3Action, RdsAction, DynamoDbAction, LambdaAction, VpcAction, IamAction, CloudTrailAction, SecretsManagerAction, EcsAction};
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(1),     // Body
            Constraint::Length(3),  // Action Log / Action Bar
        ])
        .split(frame.area());

    // Build header text with status
    let status = if app.loading {
        " [Loading...]"
    } else if app.detail_loading {
        " [Fetching details...]"
    } else {
        ""
    };
    
    // Build profile/region info (cached to avoid repeated allocations)
    let aws_info = app.render_cache.get_aws_info(app.profile.as_deref(), &app.region).to_string();
    
    let filter_status = if !app.filter_input.is_empty() {
        format!(" [Filter: {}]", app.filter_input)
    } else {
        String::new()
    };

    let header_text = if let Some(ref err) = app.error_message {
        format!("LazyAWS - {:?} {} | Error: {}", app.current_service, aws_info, err)
    } else {
        let read_only_status = if app.read_only { " [READ-ONLY]" } else { "" };
        format!("LazyAWS - {:?} {}{}{}{}", app.current_service, aws_info, status, filter_status, read_only_status)
    };
    

    // If read-only, we want a more prominent warning. 
    // Let's make the title background yellow if read-only, or just the text?
    // User asked for "yellow background so it stands out".
    let (header_style, block_style) = if app.read_only {
        (
            Style::default().fg(Color::Black).bg(THEME.warning).add_modifier(ratatui::style::Modifier::BOLD),
            Style::default().fg(THEME.warning)
        )
    } else if app.error_message.is_some() {
        (
            Style::default().fg(THEME.error),
            Style::default().fg(THEME.error)
        )
    } else {
        (
            Style::default().fg(THEME.primary),
            Style::default().fg(THEME.border)
        )
    };
    
    let title = Paragraph::new(header_text)
        .style(header_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(block_style)
            .title("LazyAWS")
            .title_style(Style::default().fg(if app.read_only { Color::Black } else { THEME.secondary })));
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
        let object_key = app.services.s3.opened_object_key.as_deref().unwrap_or("Unknown");
        let object_path = app.services.s3.opened_object_path.as_deref();
        let content = app.services.s3.opened_object_content.as_deref();
        let scroll = app.services.s3.viewer_scroll_offset;
        
        crate::ui::components::modal::render_object_viewer_modal(
            frame,
            frame.area(),
            object_key,
            object_path,
            content,
            scroll,
        );
    }
    
    if app.show_confirmation {
        if let Some(action) = &app.pending_action {
            let description = match action {
                Message::Service(ServiceAction::Ec2(Ec2Action::Start(id))) => format!("Start EC2 Instance {}", id),
                Message::Service(ServiceAction::Ec2(Ec2Action::Stop(id))) => format!("Stop EC2 Instance {}", id),
                Message::Service(ServiceAction::Ec2(Ec2Action::Reboot(id))) => format!("Reboot EC2 Instance {}", id),
                Message::Service(ServiceAction::Ec2(Ec2Action::Terminate(id))) => format!("Terminate EC2 Instance {}", id),

                Message::Service(ServiceAction::Rds(RdsAction::Start(id))) => format!("Start RDS Instance {}", id),
                Message::Service(ServiceAction::Rds(RdsAction::Stop(id))) => format!("Stop RDS Instance {}", id),
                Message::Service(ServiceAction::Rds(RdsAction::Reboot(id))) => format!("Reboot RDS Instance {}", id),
                Message::Service(ServiceAction::Rds(RdsAction::Delete(id))) => format!("Delete RDS Instance {}", id),

                Message::Service(ServiceAction::S3(S3Action::DeleteObject { bucket, key })) => format!("Delete S3 Object s3://{}/{}", bucket, key),
                Message::Service(ServiceAction::S3(S3Action::EditObject { bucket, key })) => format!("Edit S3 Object s3://{}/{}", bucket, key),

                Message::Service(ServiceAction::DynamoDb(DynamoDbAction::DeleteItem { table_name, .. })) => format!("Delete item from DynamoDB table {}", table_name),

                Message::Service(ServiceAction::Lambda(LambdaAction::InvokeFunction(name))) => format!("Invoke Lambda Function {}", name),
                Message::Service(ServiceAction::Lambda(LambdaAction::DeleteFunction(name))) => format!("Delete Lambda Function {}", name),

                Message::Service(ServiceAction::Vpc(VpcAction::DeleteSecurityGroup(id))) => format!("Delete Security Group {}", id),

                Message::Service(ServiceAction::Iam(IamAction::DeleteUser(name))) => format!("Delete IAM User {}", name),
                Message::Service(ServiceAction::Iam(IamAction::DeleteRole(name))) => format!("Delete IAM Role {}", name),
                Message::Service(ServiceAction::Iam(IamAction::DeletePolicy(arn))) => format!("Delete IAM Policy {}", arn),

                Message::Service(ServiceAction::CloudTrail(CloudTrailAction::DeleteTrail(name))) => format!("Delete CloudTrail Trail {}", name),

                Message::Service(ServiceAction::SecretsManager(SecretsManagerAction::DeleteSecret(arn))) => format!("Delete Secret {}", arn),

                Message::Service(ServiceAction::Ecs(EcsAction::StopTask { task_arn, .. })) => {
                    let short_arn = task_arn.split('/').next_back().unwrap_or(task_arn);
                    format!("Stop ECS Task {}", short_arn)
                },
                Message::Service(ServiceAction::Ecs(EcsAction::DeregisterTaskDefinition(arn))) => {
                    let short_arn = arn.split('/').next_back().unwrap_or(arn);
                    format!("Deregister Task Definition {}", short_arn)
                },
                Message::Service(ServiceAction::Ecs(EcsAction::EditTaskDefinition(arn))) => {
                    let short_arn = arn.split('/').next_back().unwrap_or(arn);
                    format!("Edit Task Definition {}", short_arn)
                },
                Message::Service(ServiceAction::Ecs(EcsAction::UpdateDesiredCount { service_name, desired_count, .. })) => {
                    format!("Update {} desired count to {}", service_name, desired_count)
                },
                Message::Service(ServiceAction::Ecs(EcsAction::ForceNewDeployment { service_name, .. })) => {
                    format!("Force new deployment for {}", service_name)
                },
                Message::Service(ServiceAction::Ecs(EcsAction::UpdateService { service_name, task_definition, .. })) => {
                    if let Some(td) = task_definition {
                        let short_td = td.split('/').next_back().unwrap_or(td);
                        format!("Update {} to use {}", service_name, short_td)
                    } else {
                        format!("Update service {}", service_name)
                    }
                },

                _ => "Unknown Action".to_string(),
            };
            crate::ui::components::modal::render_confirmation_modal(frame, frame.area(), &description);
        }
    }
    
    // Render profile switcher modal
    if app.input_mode == InputMode::ProfileSwitcherProfile {
        let filtered: Vec<String> = app.filtered_profiles().into_iter().cloned().collect();
        crate::ui::components::modal::render_profile_switcher_modal(
            frame,
            frame.area(),
            &filtered,
            app.profile_switcher_index,
            app.profile.as_deref(),
            app.pending_read_only,
            &app.profile_filter,
            app.profile_filter_active,
        );
    }
    
    // Render region switcher modal
    if app.input_mode == InputMode::ProfileSwitcherRegion {
        let filtered: Vec<String> = app.filtered_regions().into_iter().cloned().collect();
        crate::ui::components::modal::render_region_switcher_modal(
            frame,
            frame.area(),
            &filtered,
            app.region_switcher_index,
            &app.region,
            app.pending_profile.as_deref(),
            &app.region_filter,
            app.region_filter_active,
        );
    }
    
    // Render ECS service editor modal
    if app.input_mode == InputMode::EcsServiceEditor {
        let service_name = app.services.ecs.service_editor.service_name
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
        let service_name = app.services.ecs.task_def_selector.service_name
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
}
