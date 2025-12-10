# LazyAWS

A terminal user interface (TUI) for managing AWS resources, built with Rust and Ratatui.

## Features (Planned)
- **EC2**: List, start, stop, reboot instances.
- **S3**: Browse buckets and objects.
- **RDS**: Manage database instances.
- **DynamoDB**: View tables.
- **Lambda**: List functions.
- **VPC**: View networking resources.
- **IAM**: View users and roles.
- **CloudTrail**: View event logs.

## Getting Started

### Prerequisites
- Rust 1.75+
- AWS Credentials configured (`~/.aws/credentials` or environment variables)

### Running
```bash
# Standard run (uses default AWS profile)
cargo run

# With explicit profile and region
cargo run -- --profile my-profile --region us-west-2

# Connect to LocalStack
cargo run -- --endpoint-url http://localhost:4566

# Read-only mode (safer for production browsing)
cargo run -- --read-only
```

### Key Bindings

**Global**
- `1-9`: Quick switch to service
- `Tab`: Toggle focus between Sidebar and Main View
- `q`: Quit
- `?`: Show Help (Coming Soon)

**Navigation**
- `j` / `Down`: Move selection down
- `k` / `Up`: Move selection up
- `Enter`: Select / Drill down
- `Esc`: Back / Cancel / Close Modal

**Multi-View Services (VPC, IAM, Backup, CloudTrail)**
- `v`: Cycle views
- `h` / `Left`: Previous view
- `l` / `Right`: Next view

**Actions**
- `s`: Start Instance (EC2, RDS)
- `S`: Stop Instance (EC2, RDS)
- `R`: Reboot Instance (EC2, RDS)

## Architecture
See [DESIGN.md](DESIGN.md) for detailed architecture and [AGENTS.md](AGENTS.md) for development guidelines.
