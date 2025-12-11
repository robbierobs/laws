//! Application configuration
//!
//! Provides CLI argument parsing and config file support.
//! Configuration is loaded from `~/.config/lazy-aws/config.toml`.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ui::theme::ThemePreset;

/// LazyAWS - A TUI for managing AWS resources
#[derive(Parser, Debug)]
#[command(name = "lazy-aws")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// AWS profile to use (overrides AWS_PROFILE env var)
    #[arg(short, long)]
    pub profile: Option<String>,

    /// AWS region to use (overrides AWS_REGION env var)
    #[arg(short, long)]
    pub region: Option<String>,

    /// Custom endpoint URL (for LocalStack, MinIO, etc.)
    /// Overrides AWS_ENDPOINT_URL env var
    #[arg(short, long)]
    pub endpoint_url: Option<String>,

    /// Read-only mode: prevents any actions that modify state
    #[arg(long, default_value = "false")]
    pub read_only: bool,
    
    /// Theme to use (dark, light, monokai, nord)
    #[arg(long)]
    pub theme: Option<String>,
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}

/// Configuration that can be saved to/loaded from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigFile {
    /// Theme preset to use
    pub theme: ThemePreset,
    
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
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            theme: ThemePreset::Dark,
            tick_rate_ms: 250,
            api_timeout_secs: 30,
            max_history_items: 100,
            max_s3_objects: 1000,
            max_dynamodb_items: 100,
            max_cloudtrail_events: 50,
        }
    }
}

impl ConfigFile {
    /// Get the config file path
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("lazy-aws").join("config.toml"))
    }
    
    /// Load config from file, returning defaults if file doesn't exist
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        
        if !path.exists() {
            return Self::default();
        }
        
        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                match toml::from_str(&contents) {
                    Ok(config) => config,
                    Err(e) => {
                        tracing::warn!("Failed to parse config file: {}", e);
                        Self::default()
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read config file: {}", e);
                Self::default()
            }
        }
    }
    
    /// Save config to file (creates directory if needed)
    #[allow(dead_code)]
    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = Self::config_path() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            ));
        };
        
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        std::fs::write(&path, contents)
    }
    
    /// Generate example config content
    pub fn example_config() -> String {
        r#"# LazyAWS Configuration
# Place this file at ~/.config/lazy-aws/config.toml

# Theme: dark, light, monokai, nord
theme = "dark"

# Event loop tick rate (milliseconds)
tick_rate_ms = 250

# AWS API timeout (seconds)
api_timeout_secs = 30

# Maximum items to keep in action history
max_history_items = 100

# Maximum S3 objects to load per bucket
max_s3_objects = 1000

# Maximum DynamoDB items to scan
max_dynamodb_items = 100

# Maximum CloudTrail events to lookup
max_cloudtrail_events = 50
"#.to_string()
    }
}

/// Centralized application configuration
/// 
/// Contains all magic numbers and configurable values in one place.
/// This struct combines CLI args with config file settings.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    /// Theme preset to use
    pub theme: ThemePreset,
    
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
            theme: ThemePreset::Dark,
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
    /// Create a new AppConfig from config file with CLI overrides
    pub fn from_args(args: &Args) -> Self {
        // Load config file
        let file_config = ConfigFile::load();
        
        // Determine theme: CLI arg > config file > default
        let theme = if let Some(theme_name) = &args.theme {
            match theme_name.to_lowercase().as_str() {
                "light" => ThemePreset::Light,
                "monokai" => ThemePreset::Monokai,
                "nord" => ThemePreset::Nord,
                _ => ThemePreset::Dark,
            }
        } else {
            file_config.theme
        };
        
        Self {
            theme,
            tick_rate_ms: file_config.tick_rate_ms,
            api_timeout_secs: file_config.api_timeout_secs,
            max_history_items: file_config.max_history_items,
            max_s3_objects: file_config.max_s3_objects,
            max_dynamodb_items: file_config.max_dynamodb_items,
            max_cloudtrail_events: file_config.max_cloudtrail_events,
            s3_detail_concurrency: 3,
            s3_detail_delay_ms: 100,
            sidebar_width: 20,
            detail_panel_percent: 40,
        }
    }
    
    /// Create a new AppConfig with default values (for backwards compatibility)
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_file_default() {
        let config = ConfigFile::default();
        assert_eq!(config.theme, ThemePreset::Dark);
        assert_eq!(config.tick_rate_ms, 250);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.theme, ThemePreset::Dark);
        assert_eq!(config.sidebar_width, 20);
    }

    #[test]
    fn test_config_file_path() {
        let path = ConfigFile::config_path();
        // Should return Some on most systems
        if let Some(p) = path {
            assert!(p.ends_with("config.toml"));
        }
    }

    #[test]
    fn test_example_config_is_valid_toml() {
        let example = ConfigFile::example_config();
        let parsed: Result<ConfigFile, _> = toml::from_str(&example);
        assert!(parsed.is_ok(), "Example config should be valid TOML");
    }
}

