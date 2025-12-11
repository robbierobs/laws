//! App struct definition and constructors

use crate::aws::client::AwsClients;
use crate::ui::components::sidebar::Sidebar;

use super::service_state::ServiceStates;
use super::task_manager::TaskManager;
use super::{Focus, InputMode, Message, Service};

/// Main application state
/// 
/// This struct uses `ServiceStates` which consolidates all service-specific 
/// fields into organized per-service structs (Ec2State, S3State, etc.).
pub struct App {
    pub should_quit: bool,
    pub current_service: Service,
    pub sidebar: Sidebar,
    pub focus: Focus,
    pub input_mode: InputMode,
    pub filter_input: String,
    pub aws_clients: Option<AwsClients>,
    pub profile: Option<String>,
    pub region: String,
    
    // Loading states
    pub loading: bool,
    pub should_refresh: bool,
    pub error_message: Option<String>,
    pub detail_panel_visible: bool,
    pub detail_loading: bool,
    
    // Config and State
    pub read_only: bool,
    pub pending_action: Option<Message>,
    pub show_confirmation: bool,
    
    // Action Log
    pub action_log: Vec<String>,
    pub action_log_expanded: bool,
    
    // Service-specific states consolidated into one struct
    pub services: ServiceStates,
    
    // Task manager for async task tracking
    pub tasks: TaskManager,
    
    // Profile/Region Switcher State
    pub available_profiles: Vec<String>,
    pub available_regions: Vec<String>,
    pub profile_switcher_index: usize,
    pub region_switcher_index: usize,
    pub pending_profile: Option<String>,
}

impl App {
    pub fn new(aws_clients: Option<AwsClients>, profile: Option<String>, region: String, read_only: bool) -> Self {
        // Load available profiles from AWS config
        let available_profiles = crate::utils::aws_profiles::list_profiles();
        let available_regions: Vec<String> = crate::utils::aws_profiles::ALL_REGIONS
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        // Find current profile/region index for pre-selection
        let profile_switcher_index = profile
            .as_ref()
            .and_then(|p| available_profiles.iter().position(|x| x == p))
            .unwrap_or(0);
        let region_switcher_index = available_regions
            .iter()
            .position(|r| r == &region)
            .unwrap_or(0);
        
        Self {
            should_quit: false,
            current_service: Service::EC2,
            sidebar: Sidebar::new(),
            focus: Focus::Sidebar,
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            aws_clients,
            profile,
            region,
            loading: false,
            should_refresh: false,
            error_message: None,
            detail_panel_visible: true,
            detail_loading: false,
            read_only,
            pending_action: None,
            show_confirmation: false,
            action_log: Vec::new(),
            action_log_expanded: false,
            services: ServiceStates::new(),
            tasks: TaskManager::new(),
            available_profiles,
            available_regions,
            profile_switcher_index,
            region_switcher_index,
            pending_profile: None,
        }
    }
    
    pub fn on_tick(&mut self) {
        // Handle tick events if needed
    }
    
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        crate::ui::render::render(frame, self);
        
        // Render S3 object viewer popup
        if self.services.s3.show_object_viewer {
            let object_key = self.services.s3.opened_object_key.as_deref().unwrap_or("Unknown");
            let object_path = self.services.s3.opened_object_path.as_deref();
            let content = self.services.s3.opened_object_content.as_deref();
            let scroll = self.services.s3.viewer_scroll_offset;
            
            crate::ui::components::modal::render_object_viewer_modal(
                frame,
                frame.area(),
                object_key,
                object_path,
                content,
                scroll,
            );
        }
        
        if self.show_confirmation {
            if let Some(action) = &self.pending_action {
                use super::messages::{ServiceAction, Ec2Action, S3Action, RdsAction, DynamoDbAction};
                let description = match action {
                    Message::Service(ServiceAction::Ec2(Ec2Action::Start(id))) => format!("Start EC2 Instance {}", id),
                    Message::Service(ServiceAction::Ec2(Ec2Action::Stop(id))) => format!("Stop EC2 Instance {}", id),
                    Message::Service(ServiceAction::Ec2(Ec2Action::Reboot(id))) => format!("Reboot EC2 Instance {}", id),
                    Message::Service(ServiceAction::Rds(RdsAction::Start(id))) => format!("Start RDS Instance {}", id),
                    Message::Service(ServiceAction::Rds(RdsAction::Stop(id))) => format!("Stop RDS Instance {}", id),
                    Message::Service(ServiceAction::Rds(RdsAction::Reboot(id))) => format!("Reboot RDS Instance {}", id),
                    Message::Service(ServiceAction::S3(S3Action::DeleteObject { bucket, key })) => format!("Delete S3 Object s3://{}/{}", bucket, key),
                    Message::Service(ServiceAction::DynamoDb(DynamoDbAction::DeleteItem { table_name, .. })) => format!("Delete item from DynamoDB table {}", table_name),
                    _ => "Unknown Action".to_string(),
                };
                crate::ui::components::modal::render_confirmation_modal(frame, frame.area(), &description);
            }
        }
        
        // Render profile switcher modal
        if self.input_mode == InputMode::ProfileSwitcherProfile {
            crate::ui::components::modal::render_profile_switcher_modal(
                frame,
                frame.area(),
                &self.available_profiles,
                self.profile_switcher_index,
                self.profile.as_deref(),
            );
        }
        
        // Render region switcher modal
        if self.input_mode == InputMode::ProfileSwitcherRegion {
            crate::ui::components::modal::render_region_switcher_modal(
                frame,
                frame.area(),
                &self.available_regions,
                self.region_switcher_index,
                &self.region,
                self.pending_profile.as_deref(),
            );
        }
    }
    
    /// Shutdown the app - cancel all async tasks
    pub fn shutdown(&mut self) {
        self.tasks.cancel_all();
    }
}
