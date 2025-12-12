//! App struct definition and constructors

use crate::aws::client::AwsClients;
use crate::ui::components::sidebar::Sidebar;

use super::service_state::ServiceStates;
use super::task_manager::TaskManager;
use super::{Focus, InputMode, Message, Service};
use std::sync::{Arc, atomic::AtomicBool};

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
    
    // Shared state for input handling
    pub input_paused: Arc<AtomicBool>,
    
    // Loading states
    pub loading: bool,
    pub should_refresh: bool,
    pub error_message: Option<String>,
    pub detail_panel_visible: bool,
    pub detail_panel_fullscreen: bool,
    pub detail_scroll_offset: u16,
    pub detail_loading: bool,
    
    // Terminal state
    /// Set to true when terminal needs a full redraw (e.g., after external editor)
    pub needs_redraw: bool,
    
    // Config and State
    pub read_only: bool,
    pub pending_action: Option<Message>,
    pub show_confirmation: bool,
    
    // Action Log
    pub action_log: Vec<String>,
    pub action_log_expanded: bool,
    pub action_log_selected_index: usize,
    pub action_log_detail_scroll: u16,
    
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
    pub fn new(aws_clients: Option<AwsClients>, profile: Option<String>, region: String, read_only: bool, input_paused: Arc<AtomicBool>) -> Self {
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
            input_paused,
            loading: false,
            should_refresh: false,
            error_message: None,
            detail_panel_visible: true,
            detail_panel_fullscreen: false,
            detail_scroll_offset: 0,
            detail_loading: false,
            needs_redraw: false,
            read_only,
            pending_action: None,
            show_confirmation: false,
            action_log: Vec::new(),
            action_log_expanded: false,
            action_log_selected_index: 0,
            action_log_detail_scroll: 0,
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
    }
    
    /// Shutdown the app - cancel all async tasks
    pub fn shutdown(&mut self) {
        self.tasks.cancel_all();
    }

    /// Spawn a standardized AWS async task
    ///
    /// This helper reduces boilerplate for spawning async AWS operations.
    /// It handles:
    /// 1. Checking if AWS clients are available
    /// 2. Setting the global loading state to true
    /// 3. Cloning clients and the event sender
    /// 4. Spawning the tokio task
    /// 5. Registering the task with the TaskManager
    pub fn spawn_aws_task<F, Fut>(
        &mut self,
        event_tx: super::EventSender,
        key: impl Into<String>,
        action: F,
    ) where
        F: FnOnce(AwsClients, super::EventSender) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        self.loading = true;
        let clients = clients.clone();
        let tx = event_tx;

        let handle = tokio::spawn(async move {
            action(clients, tx).await;
        });

        self.tasks.spawn(key, handle);
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
