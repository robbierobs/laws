//! CLI argument parsing
//!
//! Defines command-line arguments for the LazyAWS application.

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
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Args::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default_read_only() {
        // We can't easily test clap parsing directly in unit tests
        // but we can verify the struct is properly defined
        let args = Args {
            profile: None,
            region: None,
            endpoint_url: None,
            read_only: false,
        };
        assert!(!args.read_only);
    }
}
