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
    pub detail_panel_fullscreen: bool,
    pub detail_scroll_offset: u16,
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
    pub pending_read_only: bool,
    pub profile_filter: String,
    pub region_filter: String,
    pub profile_filter_active: bool,
    pub region_filter_active: bool,
    
    // Configuration
    pub config: crate::config::AppConfig,
    
    // Render cache for optimized string formatting
    pub render_cache: RenderCache,
}

/// Cache for render-time string formatting to avoid repeated allocations
#[derive(Default)]
pub struct RenderCache {
    /// Cached AWS info string "[profile@region]"
    pub aws_info: String,
    /// Last profile used for cache
    cached_profile: Option<String>,
    /// Last region used for cache  
    cached_region: String,
}

impl RenderCache {
    /// Get or update the cached AWS info string
    pub fn get_aws_info(&mut self, profile: Option<&str>, region: &str) -> &str {
        let profile_changed = self.cached_profile.as_deref() != profile;
        let region_changed = self.cached_region != region;
        
        if profile_changed || region_changed {
            let profile_str = profile.unwrap_or("default");
            self.aws_info = format!("[{}@{}]", profile_str, region);
            self.cached_profile = profile.map(String::from);
            self.cached_region = region.to_string();
        }
        
        &self.aws_info
    }
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
            detail_panel_fullscreen: false,
            detail_scroll_offset: 0,
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
            pending_read_only: read_only,
            profile_filter: String::new(),
            region_filter: String::new(),
            profile_filter_active: false,
            region_filter_active: false,
            config: crate::config::AppConfig::default(),
            render_cache: RenderCache::default(),
        }
    }
    
    /// Get filtered profile list based on current filter
    pub fn filtered_profiles(&self) -> Vec<&String> {
        if self.profile_filter.is_empty() {
            self.available_profiles.iter().collect()
        } else {
            let filter_lower = self.profile_filter.to_lowercase();
            self.available_profiles
                .iter()
                .filter(|p| p.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }
    
    /// Get filtered region list based on current filter
    pub fn filtered_regions(&self) -> Vec<&String> {
        if self.region_filter.is_empty() {
            self.available_regions.iter().collect()
        } else {
            let filter_lower = self.region_filter.to_lowercase();
            self.available_regions
                .iter()
                .filter(|r| r.to_lowercase().contains(&filter_lower))
                .collect()
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
                use super::messages::{ServiceAction, Ec2Action, S3Action, RdsAction, DynamoDbAction, LambdaAction, VpcAction, IamAction, CloudTrailAction, SecretsManagerAction, EcsAction};
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
                        let short_arn = task_arn.split('/').last().unwrap_or(&task_arn);
                        format!("Stop ECS Task {}", short_arn)
                    },
                    Message::Service(ServiceAction::Ecs(EcsAction::DeregisterTaskDefinition(arn))) => {
                        let short_arn = arn.split('/').last().unwrap_or(&arn);
                        format!("Deregister Task Definition {}", short_arn)
                    },
                    Message::Service(ServiceAction::Ecs(EcsAction::EditTaskDefinition(arn))) => {
                        let short_arn = arn.split('/').last().unwrap_or(&arn);
                        format!("Edit Task Definition {}", short_arn)
                    },
                    Message::Service(ServiceAction::Ecs(EcsAction::UpdateDesiredCount { service_name, desired_count, .. })) => {
                        format!("Update {} desired count to {}", service_name, desired_count)
                    },
                    Message::Service(ServiceAction::Ecs(EcsAction::ForceNewDeployment { service_name, .. })) => {
                        format!("Force new deployment for {}", service_name)
                    },

                    _ => "Unknown Action".to_string(),
                };
                crate::ui::components::modal::render_confirmation_modal(frame, frame.area(), &description);
            }
        }
        
        // Render profile switcher modal
        if self.input_mode == InputMode::ProfileSwitcherProfile {
            let filtered: Vec<String> = self.filtered_profiles().into_iter().cloned().collect();
            crate::ui::components::modal::render_profile_switcher_modal(
                frame,
                frame.area(),
                &filtered,
                self.profile_switcher_index,
                self.profile.as_deref(),
                self.pending_read_only,
                &self.profile_filter,
                self.profile_filter_active,
            );
        }
        
        // Render region switcher modal
        if self.input_mode == InputMode::ProfileSwitcherRegion {
            let filtered: Vec<String> = self.filtered_regions().into_iter().cloned().collect();
            crate::ui::components::modal::render_region_switcher_modal(
                frame,
                frame.area(),
                &filtered,
                self.region_switcher_index,
                &self.region,
                self.pending_profile.as_deref(),
                &self.region_filter,
                self.region_filter_active,
            );
        }
    }
    
    /// Shutdown the app - cancel all async tasks
    pub fn shutdown(&mut self) {
        self.tasks.cancel_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_cache_default() {
        let cache = RenderCache::default();
        assert!(cache.aws_info.is_empty());
    }

    #[test]
    fn test_render_cache_get_aws_info() {
        let mut cache = RenderCache::default();
        let result = cache.get_aws_info(Some("my-profile"), "us-east-1");
        assert_eq!(result, "[my-profile@us-east-1]");
    }

    #[test]
    fn test_render_cache_default_profile() {
        let mut cache = RenderCache::default();
        let result = cache.get_aws_info(None, "eu-west-1");
        assert_eq!(result, "[default@eu-west-1]");
    }

    #[test]
    fn test_render_cache_caching() {
        let mut cache = RenderCache::default();
        
        // First call should create the string
        let result1 = cache.get_aws_info(Some("profile"), "us-east-1");
        assert_eq!(result1, "[profile@us-east-1]");
        
        // Second call with same values should return cached version
        let result2 = cache.get_aws_info(Some("profile"), "us-east-1");
        assert_eq!(result2, "[profile@us-east-1]");
    }

    #[test]
    fn test_render_cache_invalidation_on_profile_change() {
        let mut cache = RenderCache::default();
        
        cache.get_aws_info(Some("profile1"), "us-east-1");
        let result = cache.get_aws_info(Some("profile2"), "us-east-1");
        assert_eq!(result, "[profile2@us-east-1]");
    }

    #[test]
    fn test_render_cache_invalidation_on_region_change() {
        let mut cache = RenderCache::default();
        
        cache.get_aws_info(Some("profile"), "us-east-1");
        let result = cache.get_aws_info(Some("profile"), "eu-west-1");
        assert_eq!(result, "[profile@eu-west-1]");
    }

    #[test]
    fn test_render_cache_profile_none_to_some() {
        let mut cache = RenderCache::default();
        
        cache.get_aws_info(None, "us-east-1");
        let result = cache.get_aws_info(Some("new-profile"), "us-east-1");
        assert_eq!(result, "[new-profile@us-east-1]");
    }
}
