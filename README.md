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
cargo run
```

## Architecture
See [DESIGN.md](DESIGN.md) for detailed architecture and [AGENTS.md](AGENTS.md) for development guidelines.
