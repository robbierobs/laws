use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row},
    Frame,
};
use crate::app::{App, IamViewMode};
use crate::models::iam::{IamRole, IamUser, IamPolicy};
use crate::ui::components::detail_panel::{render_detail_panel, DetailPanelConfig};
use crate::ui::components::table::render_table;

use crate::ui::theme::THEME;
use crate::app::ViewMode;

pub fn render(frame: &mut Frame, list_area: Option<Rect>, detail_area: Option<Rect>, app: &mut App) {
    use ratatui::layout::{Layout, Direction};
    
    // Render list area if provided (not in fullscreen detail mode)
    if let Some(area) = list_area {
        // Split list area for tabs
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // List
            ])
            .split(area);

        if !app.services.iam.view_mode.is_main_tab() {
            let title = match app.services.iam.view_mode {
                IamViewMode::UserAttachedPolicies => format!("Policies attached to User: {}", app.services.iam.selected_entity_name.as_deref().unwrap_or("Unknown")),
                IamViewMode::RoleAttachedPolicies => format!("Policies attached to Role: {}", app.services.iam.selected_entity_name.as_deref().unwrap_or("Unknown")),
                IamViewMode::PolicyDocument => format!("Policy Document: {}", app.services.iam.selected_entity_name.as_deref().unwrap_or("Unknown")),
                _ => "Details".to_string(),
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Style::default().fg(THEME.primary))
                .border_style(Style::default().fg(THEME.border));
            frame.render_widget(block, chunks[0]);
        } else {
            let tabs: Vec<&str> = crate::app::IamViewMode::iterator().map(|m| m.label()).collect();
            crate::ui::components::tabs::render_tabs(frame, chunks[0], &tabs, app.services.iam.view_mode.index());
        }

        match app.services.iam.view_mode {
            IamViewMode::Users => render_user_list(frame, chunks[1], app),
            IamViewMode::Roles => render_role_list(frame, chunks[1], app),
            IamViewMode::Policies => render_policy_list(frame, chunks[1], app),
            IamViewMode::UserAttachedPolicies | IamViewMode::RoleAttachedPolicies => render_attached_policies_list(frame, chunks[1], app),
            IamViewMode::PolicyDocument => render_policy_document(frame, chunks[1], app),
        }
    }
    
    if let Some(area) = detail_area {
        match app.services.iam.view_mode {
            IamViewMode::Users => render_user_details(frame, area, app),
            IamViewMode::Roles => render_role_details(frame, area, app),
            IamViewMode::Policies => render_policy_details(frame, area, app),
            IamViewMode::UserAttachedPolicies | IamViewMode::RoleAttachedPolicies => render_policy_details_from_list(frame, area, app),
            IamViewMode::PolicyDocument => {}, // No details pane for document view
        }
    }
}

use crate::models::Filterable;

fn render_user_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.iam.users.iter()
        .filter(|u| {
            if filter.is_empty() { return true; }
            u.matches_filter(&filter)
        })
        .map(|user| {
            let path = user.path.clone().unwrap_or_else(|| "/".to_string());
            let created = user.create_date.clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());
            let pwd_last_used = user.password_last_used.clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "Never".to_string());
            
            let cells = vec![
                Cell::from(user.user_name.clone()),
                Cell::from(user.user_id.clone()),
                Cell::from(path),
                Cell::from(created),
                Cell::from(pwd_last_used),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["User Name", "User ID", "Path", "Created", "Password Last Used"],
        &[
            Constraint::Length(25), // User Name
            Constraint::Length(22), // User ID
            Constraint::Length(15), // Path
            Constraint::Length(15), // Created
            Constraint::Min(15),    // Password Last Used
        ],
        "IAM Users (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.iam.list_state,
    );
}

fn render_user_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.iam.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(user) = app.services.iam.users.get(idx) {
            build_user_detail_lines(user)
        } else {
            vec![Line::from("No user selected")]
        }
    } else {
        vec![Line::from("Select an IAM user to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("IAM User Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_user_detail_lines(user: &IamUser) -> Vec<Line<'static>> {
    use crate::ui::components::detail_builder::DetailBuilder;
    
    // ARN on separate line with styled value
    let arn_line = Line::from(vec![
        Span::styled(user.arn.clone().unwrap_or_else(|| "-".into()), Style::default().fg(THEME.selection_fg)),
    ]);
    
    DetailBuilder::new()
        .header_field("User Name", user.user_name.clone())
        .field_owned("User ID", user.user_id.clone())
        .section("Details")
        .field_owned("Path", user.path.clone().unwrap_or_else(|| "/".into()))
        .field_owned("Created", user.create_date.clone().unwrap_or_else(|| "-".into()))
        .field_owned("Password Last Used", user.password_last_used.clone().unwrap_or_else(|| "Never".into()))
        .blank()
        .field("ARN", "")
        .raw_line(arn_line)
        .build()
}

fn render_role_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.iam.roles.iter()
        .filter(|r| {
            if filter.is_empty() { return true; }
            r.matches_filter(&filter)
        })
        .map(|role| {
            let path = role.path.clone().unwrap_or_else(|| "/".to_string());
            let created = role.create_date.clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());
            let max_session = role.max_session_duration
                .map(|d| format!("{}h", d / 3600))
                .unwrap_or_else(|| "-".to_string());
            
            let cells = vec![
                Cell::from(role.role_name.clone()),
                Cell::from(role.role_id.clone()),
                Cell::from(path),
                Cell::from(created),
                Cell::from(max_session),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["Role Name", "Role ID", "Path", "Created", "Max Session"],
        &[
            Constraint::Length(35), // Role Name
            Constraint::Length(22), // Role ID
            Constraint::Length(20), // Path
            Constraint::Length(12), // Created
            Constraint::Min(10),    // Max Session
        ],
        "IAM Roles (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.iam.list_state,
    );
}

fn render_role_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.iam.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(role) = app.services.iam.roles.get(idx) {
            build_role_detail_lines(role)
        } else {
            vec![Line::from("No role selected")]
        }
    } else {
        vec![Line::from("Select an IAM role to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("IAM Role Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_role_detail_lines(role: &IamRole) -> Vec<Line<'static>> {
    use crate::ui::components::detail_builder::DetailBuilder;
    
    let max_session = role.max_session_duration
        .map(|d| format!("{} seconds ({} hours)", d, d / 3600))
        .unwrap_or_else(|| "-".into());
    
    // ARN on separate line with styled value
    let arn_line = Line::from(vec![
        Span::styled(role.arn.clone().unwrap_or_else(|| "-".into()), Style::default().fg(THEME.selection_fg)),
    ]);
    
    DetailBuilder::new()
        .header_field("Role Name", role.role_name.clone())
        .field_owned("Role ID", role.role_id.clone())
        .field_owned("Description", role.description.clone().unwrap_or_else(|| "(no description)".into()))
        .section("Details")
        .field_owned("Path", role.path.clone().unwrap_or_else(|| "/".into()))
        .field_owned("Created", role.create_date.clone().unwrap_or_else(|| "-".into()))
        .field_owned("Max Session Duration", max_session)
        .blank()
        .field("ARN", "")
        .raw_line(arn_line)
        .build()
}

fn render_policy_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let filter = app.filter_input.to_lowercase();
    let rows = app.services.iam.policies.iter()
        .filter(|p| {
            if filter.is_empty() { return true; }
            p.matches_filter(&filter)
        })
        .map(|policy| {
            let created = policy.create_date.clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());
            let updated = policy.update_date.clone()
                .map(|d| d.split('T').next().unwrap_or(&d).to_string())
                .unwrap_or_else(|| "-".to_string());
            let attachments = policy.attachment_count.unwrap_or(0).to_string();
            
            let cells = vec![
                Cell::from(policy.policy_name.clone()),
                Cell::from(policy.policy_id.clone().unwrap_or_default()),
                Cell::from(attachments),
                Cell::from(created),
                Cell::from(updated),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["Policy Name", "Policy ID", "Attachments", "Created", "Updated"],
        &[
            Constraint::Length(40), // Policy Name
            Constraint::Length(22), // Policy ID
            Constraint::Length(12), // Attachments
            Constraint::Length(15), // Created
            Constraint::Min(15),    // Updated
        ],
        "IAM Policies (v/h/l to switch view)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.iam.list_state,
    );
}

fn render_policy_details(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.iam.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(policy) = app.services.iam.policies.get(idx) {
            build_policy_detail_lines(policy)
        } else {
            vec![Line::from("No policy selected")]
        }
    } else {
        vec![Line::from("Select an IAM policy to view details (use j/k to navigate)")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("IAM Policy Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}

fn build_policy_detail_lines(policy: &IamPolicy) -> Vec<Line<'static>> {
    use crate::ui::components::detail_builder::DetailBuilder;
    
    // ARN on separate line with styled value
    let arn_line = Line::from(vec![
        Span::styled(policy.arn.clone().unwrap_or_else(|| "-".into()), Style::default().fg(THEME.selection_fg)),
    ]);
    
    DetailBuilder::new()
        .header_field("Policy Name", policy.policy_name.clone())
        .field_owned("Policy ID", policy.policy_id.clone().unwrap_or_default())
        .section("Details")
        .field_owned("Attachment Count", policy.attachment_count.unwrap_or(0).to_string())
        .bool_field("Is Attachable", policy.is_attachable, "Yes", "No")
        .field_owned("Created", policy.create_date.clone().unwrap_or_else(|| "-".into()))
        .field_owned("Updated", policy.update_date.clone().unwrap_or_else(|| "-".into()))
        .blank()
        .field("ARN", "")
        .raw_line(arn_line)
        .build()
}

fn render_attached_policies_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = app.services.iam.current_policies.iter()
        .map(|policy| {
            let cells = vec![
                Cell::from(policy.policy_name.clone()),
                Cell::from(policy.arn.clone().unwrap_or_default()),
            ];
            
            Row::new(cells).height(1)
        });

    render_table(
        frame,
        area,
        rows,
        &["Policy Name", "ARN"],
        &[
            Constraint::Length(40), // Policy Name
            Constraint::Min(40),    // ARN
        ],
        "Attached Policies (Press Esc to back)",
        matches!(app.focus, crate::app::Focus::Main),
        &mut app.services.iam.list_state,
    );
}

fn render_policy_document(frame: &mut Frame, area: Rect, app: &App) {
    let text = app.services.iam.current_policy_document.clone();
    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Policy Document (Press Esc to back)")
            .title_style(Style::default().fg(THEME.primary))
            .border_style(Style::default().fg(THEME.border)))
        .wrap(ratatui::widgets::Wrap { trim: false });
    
    frame.render_widget(paragraph, area);
}

fn render_policy_details_from_list(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.services.iam.list_state.selected();
    
    let content: Vec<Line> = if let Some(idx) = selected {
        if let Some(policy) = app.services.iam.current_policies.get(idx) {
            build_policy_detail_lines(policy)
        } else {
            vec![Line::from("No policy selected")]
        }
    } else {
        vec![Line::from("Select a policy to view details")]
    };

    render_detail_panel(
        frame,
        area,
        content,
        DetailPanelConfig::new("Policy Details")
            .fullscreen(app.detail_panel_fullscreen)
            .scroll(app.detail_scroll_offset),
    );
}
