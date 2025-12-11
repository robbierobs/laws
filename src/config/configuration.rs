//! Centralized application configuration
//!
//! This module provides a centralized configuration structure that replaces
//! scattered constants and magic numbers throughout the codebase. It enables
//! easy customization and runtime configuration of the application.

/// Centralized application configuration
/// 
/// Contains all magic numbers and configurable values in one place.
/// This can be extended in the future to support loading from a config file.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Tick rate in milliseconds for the main event loop
    pub tick_rate_ms: u64,
    
    /// API timeout in seconds for AWS SDK calls
    pub api_timeout_secs: u64,
    
    /// Maximum number of items in action history log
    pub max_history_items: usize,
    
    /// Maximum number of S3 objects to load per bucket
    pub max_s3_objects: usize,
    
    /// Maximum number of DynamoDB items to scan
    pub max_dynamodb_items: usize,
    
    /// Maximum number of CloudTrail events to lookup
    pub max_cloudtrail_events: usize,
    
    /// Number of buckets to load details for concurrently
    pub s3_detail_concurrency: usize,
    
    /// Delay between S3 bucket detail requests (ms)
    pub s3_detail_delay_ms: u64,
    
    /// Sidebar width in characters
    pub sidebar_width: u16,
    
    /// Detail panel height percentage
    pub detail_panel_percent: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 250,
            api_timeout_secs: 30,
            max_history_items: 100,
            max_s3_objects: 1000,
            max_dynamodb_items: 100,
            max_cloudtrail_events: 50,
            s3_detail_concurrency: 3,
            s3_detail_delay_ms: 100,
            sidebar_width: 20,
            detail_panel_percent: 40,
        }
    }
}

impl AppConfig {
    /// Create a new AppConfig with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load configuration from environment variables (if present)
    /// 
    /// This method can be extended to support loading from config files.
    /// Environment variables take precedence over defaults.
    #[allow(dead_code)]
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Example: LAZY_AWS_TICK_RATE_MS=100
        if let Ok(val) = std::env::var("LAZY_AWS_TICK_RATE_MS") {
            if let Ok(tick_rate) = val.parse() {
                config.tick_rate_ms = tick_rate;
            }
        }
        
        if let Ok(val) = std::env::var("LAZY_AWS_API_TIMEOUT_SECS") {
            if let Ok(timeout) = val.parse() {
                config.api_timeout_secs = timeout;
            }
        }
        
        if let Ok(val) = std::env::var("LAZY_AWS_MAX_S3_OBJECTS") {
            if let Ok(max) = val.parse() {
                config.max_s3_objects = max;
            }
        }
        
        config
    }
    
    /// Validate configuration values
    /// 
    /// Returns an error if any configuration value is invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.tick_rate_ms == 0 {
            return Err("tick_rate_ms must be greater than 0".to_string());
        }
        
        if self.api_timeout_secs == 0 {
            return Err("api_timeout_secs must be greater than 0".to_string());
        }
        
        if self.sidebar_width < 10 {
            return Err("sidebar_width must be at least 10".to_string());
        }
        
        if self.detail_panel_percent > 90 {
            return Err("detail_panel_percent must be at most 90".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.tick_rate_ms, 250);
        assert_eq!(config.api_timeout_secs, 30);
        assert_eq!(config.max_history_items, 100);
    }
    
    #[test]
    fn test_new_config() {
        let config = AppConfig::new();
        assert_eq!(config.tick_rate_ms, 250);
    }
    
    #[test]
    fn test_validate_success() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_validate_tick_rate_zero() {
        let mut config = AppConfig::default();
        config.tick_rate_ms = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_validate_sidebar_too_narrow() {
        let mut config = AppConfig::default();
        config.sidebar_width = 5;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_validate_detail_panel_too_large() {
        let mut config = AppConfig::default();
        config.detail_panel_percent = 95;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_from_env_defaults() {
        // Without env vars, should match default
        let config = AppConfig::from_env();
        let default = AppConfig::default();
        assert_eq!(config.tick_rate_ms, default.tick_rate_ms);
    }
}
