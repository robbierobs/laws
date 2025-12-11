# LazyAWS

A terminal user interface (TUI) for managing AWS resources, built with Rust and Ratatui. Inspired by [lazygit](https://github.com/jesseduffield/lazygit).

![LazyAWS Demo](docs/demo.gif) <!-- TODO: Add demo gif -->

## Features

### Supported AWS Services

| Service | Features |
|---------|----------|
| **EC2** | List instances, start/stop/reboot, view details |
| **S3** | Browse buckets and objects, download/view files, delete objects, auto-load bucket details |
| **RDS** | List instances, start/stop/reboot |
| **DynamoDB** | List tables, scan items, delete items |
| **Lambda** | List functions with runtime/memory details |
| **VPC** | View VPCs, subnets, security groups, drill into rules |
| **IAM** | View users/roles/policies, view attached policies and policy documents |
| **Backup** | View backup vaults, plans, and jobs |
| **CloudTrail** | View trails and recent events |

### Highlights

- 🎹 **Keyboard-driven** - Full Vim-style navigation (`j/k/h/l`)
- 🔒 **Read-only mode** - Safely browse production resources without accidental changes
- ✅ **Confirmation dialogs** - Extra safety for destructive actions
- 📋 **Action log** - Track all operations with success/error history
- 🔄 **Profile/Region switcher** - Switch AWS profiles and regions on the fly (`Shift+P`)
- 🏠 **LocalStack support** - Full compatibility with local AWS development
- ⚡ **Auto-loading** - S3 bucket details, IAM policies load automatically in background
- 🎨 **Status coloring** - Visual feedback with color-coded resource states

---

## Getting Started

### Prerequisites

- **Rust 1.75+** (install via [rustup](https://rustup.rs/))
- **AWS Credentials** configured (`~/.aws/credentials` or environment variables)

### Installation

```bash
# Clone and build
git clone https://github.com/youruser/lazy-aws.git
cd lazy-aws
cargo build --release

# Run
./target/release/lazy-aws
```

### Running

```bash
# Standard run - opens profile switcher if no profile set
cargo run

# With explicit profile and region
cargo run -- --profile my-profile --region us-west-2

# Connect to LocalStack (explicit endpoint)
cargo run -- --endpoint-url http://localhost:4566

# Read-only mode (safer for production)
cargo run -- --read-only

# Combine options
cargo run -- --profile prod --region eu-west-1 --read-only
```

### CLI Options

| Option | Description |
|--------|-------------|
| `--profile <NAME>` | AWS profile to use |
| `--region <REGION>` | AWS region (default: us-east-1) |
| `--endpoint-url <URL>` | Custom AWS endpoint (for LocalStack) |
| `--read-only` | Prevent all modifying actions |

### LocalStack / Custom Endpoints

The app automatically reads `endpoint_url` from your AWS profile config. For example:

```ini
# ~/.aws/config
[profile localstack]
region = us-east-1
endpoint_url = http://localhost.localstack.cloud:4566
```

Then simply run:
```bash
cargo run -- --profile localstack
```

**Priority for endpoint URL:** CLI `--endpoint-url` > profile config `endpoint_url` > `AWS_ENDPOINT_URL` env var

### SSO Profiles

SSO profiles are automatically detected. When you switch to an SSO profile, the app will run `aws sso login --profile <name>` for you.

---

## Key Bindings

### Global

| Key | Action |
|-----|--------|
| `1-9` | Quick switch to service (1=EC2, 2=S3, ...) |
| `Tab` | Toggle focus between Sidebar and Main View |
| `P` | Open profile/region switcher |
| `q` | Quit |
| `r` | Refresh current view |
| `d` | Toggle detail panel |
| `A` | Toggle action log popup |
| `/` | Filter items |
| `Ctrl+c` | Force quit |

### Profile/Region Switcher

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `/` | Start filtering (type to search) |
| `R` | Toggle read-only mode (profiles only) |
| `Enter` | Select profile → Select region → Confirm |
| `Esc` | Cancel filter / Cancel and close |

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `g` / `Home` | Go to first item |
| `G` / `End` | Go to last item |
| `Enter` | Select / Drill down |
| `Esc` | Back / Cancel / Close |

### Multi-View Services (VPC, IAM, Backup, CloudTrail)

| Key | Action |
|-----|--------|
| `v` | Cycle through views |
| `h` / `←` | Previous view |
| `l` / `→` | Next view |

### Actions (Requires Confirmation)

| Key | Action | Services |
|-----|--------|----------|
| `s` | Start | EC2, RDS |
| `S` | Stop | EC2, RDS |
| `R` | Reboot | EC2, RDS |
| `D` | Delete | S3 objects, DynamoDB items |
| `y` | Confirm action | All |
| `n` / `Esc` | Cancel action | All |

### S3 Object Actions

| Key | Action |
|-----|--------|
| `Enter` | Enter bucket / View object details |
| `o` | Open object in popup viewer |
| `w` | Download object to ~/Downloads |
| `D` | Delete object (requires confirmation) |
| `Esc` | Leave bucket / Close viewer |

### Object Viewer (Popup)

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `g` / `Home` | Go to top |
| `G` / `End` | Go to bottom |
| `PgDown` / `Ctrl+d` | Page down |
| `PgUp` / `Ctrl+u` | Page up |
| `Esc` / `q` | Close viewer |

---

## Architecture

LazyAWS follows a **hybrid Component + Elm Architecture** pattern:

```
┌────────────────────────────────────────────────────────────────┐
│                        Application Flow                         │
├────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────┐    ┌──────────────┐    ┌────────────────────────┐  │
│  │Keyboard │───>│ handle_key() │───>│ update(Message)        │  │
│  │  Event  │    │   -> Option  │    │   (State Mutation)     │  │
│  └─────────┘    │   <Message>  │    └───────────┬────────────┘  │
│                 └──────────────┘                │               │
│                                                 ▼               │
│  ┌─────────┐    ┌──────────────┐    ┌────────────────────────┐  │
│  │  AWS    │<───│handle_aws_  │<───│ tokio::spawn()         │  │
│  │ Event   │    │  event()    │    │   (Async AWS calls)    │  │
│  └────┬────┘    └──────────────┘    └────────────────────────┘  │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    render()                              │    │
│  │  (Pure function: State -> Terminal Output)               │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└────────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
src/
├── main.rs              # Entry point, event loop
├── config.rs            # CLI argument parsing
├── event.rs             # Event types and async event handler
├── error.rs             # Error types
│
├── app/                 # Application state machine
│   ├── messages.rs      # Message, Service, Action enums
│   ├── state.rs         # App struct and initialization
│   ├── update.rs        # Message handling (reducer)
│   ├── input.rs         # Keyboard input handling
│   ├── events.rs        # AWS event handling
│   ├── service_state.rs # Per-service state structs
│   ├── task_manager.rs  # Async task tracking
│   └── filtered_list.rs # Filter/search logic
│
├── aws/                 # AWS SDK wrappers
│   ├── client.rs        # AwsClients initialization
│   └── <service>.rs     # Per-service SDK wrappers
│
├── models/              # Data structures
│   └── <service>.rs     # Per-service models with from_aws()
│
├── ui/                  # Rendering
│   ├── render.rs        # Main render dispatcher
│   ├── theme.rs         # Color theme constants
│   ├── components/      # Reusable widgets (sidebar, modal, etc.)
│   └── screens/         # Service-specific views
│
└── utils/               # Utilities
    ├── aws_profiles.rs  # Profile/region/SSO detection
    └── formatting.rs    # Display helpers
```

---

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Contributing

See [AGENTS.md](AGENTS.md) for detailed development guidelines, architecture documentation, and coding standards.

---

## Troubleshooting

### "No credentials found"
Ensure you have AWS credentials configured:
```bash
aws configure
# Or set environment variables:
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

### "Failed to initialize AWS clients"
Check that your profile name and region are correct:
```bash
# List available profiles
cat ~/.aws/config | grep '\[profile'

# Test with AWS CLI
aws sts get-caller-identity --profile your-profile
```

### SSO login fails
Ensure you have the AWS CLI v2 installed and your SSO session is valid:
```bash
aws sso login --profile your-sso-profile
```

### LocalStack connection issues
Verify LocalStack is running and the endpoint is reachable:
```bash
curl http://localhost:4566/_localstack/health
```

---

## Roadmap

- [ ] EC2 terminate instance
- [ ] S3 bucket creation/deletion
- [ ] Lambda function invocation
- [ ] Configuration file support
- [ ] Keyboard shortcut customization
- [ ] Multi-account support
- [ ] Resource tagging/search

---

## License

MIT

---

## Acknowledgments

- [Ratatui](https://ratatui.rs/) - Rust TUI framework
- [lazygit](https://github.com/jesseduffield/lazygit) - Inspiration for the keyboard-driven interface
- [AWS SDK for Rust](https://aws.amazon.com/sdk-for-rust/) - AWS service clients
