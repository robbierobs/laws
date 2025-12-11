use clap::Parser;

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
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}

/// Centralized application configuration
/// 
/// Contains all magic numbers and configurable values in one place.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
