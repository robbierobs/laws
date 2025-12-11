# LazyAWS

A terminal user interface (TUI) for managing AWS resources, built with Rust and Ratatui.

![LazyAWS Demo](docs/demo.gif) <!-- TODO: Add demo gif -->

## Features

### Supported Services
- **EC2**: List, start, stop, reboot instances
- **S3**: Browse buckets and objects, download/open objects, delete objects, view bucket details
- **RDS**: Manage database instances (start, stop, reboot)
- **DynamoDB**: View tables
- **Lambda**: List functions
- **VPC**: View VPCs, subnets, security groups, and drill into rules
- **IAM**: View users, roles, policies, and attached policy documents
- **Backup**: View backup vaults, plans, and jobs
- **CloudTrail**: View trails and recent events

### Highlights
- **Keyboard-driven**: Vim-style navigation (`j/k/h/l`)
- **Read-only mode**: Safely browse production resources
- **Confirmation dialogs**: Prevent accidental destructive actions
- **Action log**: Track all operations with success/error history
- **LocalStack support**: Develop locally with `--endpoint-url`
- **Auto-loading**: S3 bucket details load automatically in the background

## Getting Started

### Prerequisites
- Rust 1.75+
- AWS Credentials configured (`~/.aws/credentials` or environment variables)

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
# Standard run (uses default AWS profile)
cargo run

# With explicit profile and region
cargo run -- --profile my-profile --region us-west-2

# Connect to LocalStack (explicit endpoint)
cargo run -- --endpoint-url http://localhost:4566

# Read-only mode (safer for production browsing)
cargo run -- --read-only
```

### LocalStack / Custom Endpoints

The app automatically reads `endpoint_url` from your AWS profile config. For example, if your `~/.aws/config` contains:

```ini
[profile localstack]
region = us-east-1
endpoint_url = http://localhost.localstack.cloud:4566
```

Simply run with that profile:
```bash
cargo run -- --profile localstack
```

Priority for endpoint URL: CLI `--endpoint-url` > profile config `endpoint_url` > `AWS_ENDPOINT_URL` env var


## Key Bindings

### Global
| Key | Action |
|-----|--------|
| `1-9` | Quick switch to service |
| `Tab` | Toggle focus between Sidebar and Main View |
| `P` (Shift+P) | Open profile/region switcher |
| `q` | Quit |
| `d` | Toggle detail panel |
| `A` | Toggle action log popup |
| `/` | Filter items |

### Profile/Region Switcher (when open)
| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `R` (Shift+R) | Toggle read-only mode |
| `Enter` | Select profile → region → confirm |
| `Esc` | Cancel and close |

### Navigation
| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Select / Drill down |
| `Esc` | Back / Cancel |

### Multi-View Services (VPC, IAM, Backup, CloudTrail)
| Key | Action |
|-----|--------|
| `v` | Cycle views |
| `h` / `←` | Previous view |
| `l` / `→` | Next view |

### Actions (requires confirmation)
| Key | Action |
|-----|--------|
| `s` | Start instance (EC2, RDS) |
| `S` | Stop instance (EC2, RDS) |
| `R` | Reboot instance (EC2, RDS) |
| `D` | Delete object (S3, DynamoDB) |
| `y` | Confirm action |
| `n` / `Esc` | Cancel action |

### S3 Object Actions
| Key | Action |
|-----|--------|
| `o` | Open object (view in popup) |
| `w` | Download object to ~/Downloads |
| `D` | Delete object (requires confirmation) |

### Object Viewer (when popup is open)
| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `g` / `Home` | Go to top |
| `G` / `End` | Go to bottom |
| `PgDown` | Page down |
| `PgUp` | Page up |
| `Esc` / `q` | Close viewer |

## Architecture

The application follows a hybrid Component + Elm Architecture pattern:

```
src/
├── main.rs              # Entry point, event loop
├── app/                 # Application state (modular)
│   ├── messages.rs      # Enums: Service, Message, Focus, InputMode
│   ├── state.rs         # App struct and initialization
│   ├── update.rs        # Message handling (reducer)
│   ├── input.rs         # Keyboard input handling
│   └── events.rs        # AWS event handling
├── aws/                 # AWS SDK wrappers
├── models/              # Data structures
├── ui/                  # Rendering
│   ├── components/      # Reusable widgets
│   └── screens/         # Service-specific views
└── utils/               # Helpers
```

See [AGENTS.md](AGENTS.md) for development guidelines.

## License

MIT
