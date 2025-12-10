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
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}
