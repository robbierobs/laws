# LazyAWS 🦥☁️

A keyboard-driven terminal UI for browsing and managing AWS resources. Think [lazygit](https://github.com/jesseduffield/lazygit), but for AWS.

Built with Rust 🦀 + [Ratatui](https://ratatui.rs/).

> [!CAUTION]
> This is a work in progress and has largely been vibed out with Antigravity, Kiro, etc. While I do have very good confidence in the instructions, specs, and context which I have been injecting into the agents, there is a chance that I, or the agentic coding, have made some mistakes. Please use with caution and report any issues to me.

## Why?

The AWS Console is powerful but slow. The CLI is fast but you need to remember a million flags. LazyAWS gives you the best of both worlds - browse your infrastructure visually, take actions with simple keypresses, and never leave the terminal.

Perfect for:
- **Quick checks** - "Is that EC2 instance running?"
- **Exploring** - "What's in this S3 bucket again?"
- **Operations** - Start/stop instances, invoke lambdas, all with a keypress
- **Learning** - See your resources laid out, drill into details

---

## What Can It Do?

| Service | Browse | Actions |
|---------|--------|---------|
| **EC2** | Instances with state, type, IPs | Start, Stop, Reboot, Terminate |
| **S3** | Buckets & objects, versioning, encryption | Download, Open, Edit, Delete, Create bucket |
| **RDS** | Instances with engine, status, endpoint | Start, Stop, Reboot, Delete |
| **DynamoDB** | Tables with key schema, indexes | Scan items, Delete items |
| **Lambda** | Functions with runtime, memory, timeout | Invoke, View details |
| **VPC** | VPCs, Subnets, Security Groups | View rules, Delete SGs |
| **IAM** | Users, Roles, Policies | View attached policies, policy documents |
| **ECS** | Clusters, Services, Tasks, Task Defs | Scale services, Force deploy, Stop tasks |
| **ECR** | Repositories and images | Browse image tags |
| **Backup** | Vaults, Plans, Jobs, Recovery Points | Browse recovery points |
| **CloudTrail** | Trails and recent events | View event details |
| **Secrets Manager** | Secrets with metadata | View secret values |

### The Good Stuff

- 🎹 **Vim keys** - `j/k` to navigate, `Enter` to drill in, `Esc` to go back
- 🔍 **Global search** - Press `/` to find any resource across all services
- 🔒 **Read-only mode** - Browse prod safely with `--read-only`
- 🏠 **LocalStack support** - Test locally with `--endpoint-url`
- ⚡ **Background loading** - Details load async so the UI stays snappy
- 📋 **Copy to clipboard** - Press `y` to copy IDs, ARNs, whatever
- 🔄 **Profile switching** - Press `P` to switch AWS profiles/regions on the fly

---

## Getting Started

### Install

```bash
# Clone it
git clone https://github.com/youruser/lazy-aws.git
cd lazy-aws

# Build it (requires Rust 1.75+)
cargo build --release

# Run it
./target/release/lazy-aws
```

Or just `cargo run` during development.

### Basic Usage

```bash
# Just run it - opens profile switcher if no profile is set
lazy-aws

# Specify a profile
lazy-aws --profile my-profile

# Different region
lazy-aws --profile my-profile --region eu-west-1

# Read-only mode (can't break anything!)
lazy-aws --profile prod --read-only

# LocalStack
lazy-aws --endpoint-url http://localhost:4566
```

### LocalStack Setup

The easiest way is to add the endpoint to your AWS config:

```ini
# ~/.aws/config
[profile localstack]
region = us-east-1
endpoint_url = http://localhost:4566
```

Then just: `lazy-aws --profile localstack`

---

## How To Use It

### The Basics

| Key | What it does |
|-----|--------------|
| `j` / `k` | Move down/up |
| `Enter` | Select / drill into |
| `Esc` | Go back |
| `Tab` | Switch between sidebar and main view |
| `/` | Filter current list OR open global search |
| `r` | Refresh |
| `q` | Quit |

### Quick Navigation

| Key | What it does |
|-----|--------------|
| `1-9` | Jump to service (1=EC2, 2=S3, 3=RDS...) |
| `g` | Go to top of list |
| `G` | Go to bottom |
| `P` | Open profile/region switcher |

### Actions

| Key | What it does |
|-----|--------------|
| `s` | Start (EC2/RDS) |
| `S` | Stop (EC2/RDS) |
| `R` | Reboot (EC2/RDS) |
| `x` | Terminate/Delete (with confirmation!) |
| `y` | Copy selected item to clipboard |

### Viewing Details

| Key | What it does |
|-----|--------------|
| `d` | Toggle detail panel |
| `D` | Toggle fullscreen detail |
| `PgUp/PgDn` | Scroll details |
| `A` | Show action log |

### Services with Multiple Views (VPC, IAM, ECS...)

| Key | What it does |
|-----|--------------|
| `v` | Cycle through views |
| `h` / `l` | Previous/next view |

---

## Development

### Building

```bash
cargo build           # Debug
cargo build --release # Release
```

### Testing

```bash
cargo test            # Run all tests
cargo test -- --nocapture  # With output
```

### Local Development with LocalStack

There's a seed script to populate LocalStack with sample data:

```bash
# Start LocalStack
docker-compose up -d

# Seed some resources
./scripts/seed_localstack.sh

# Run against it
cargo run -- --profile localstack
```

### Project Structure

```
src/
├── app/           # Application state, messages, input handling
│   ├── messages/  # Service enums, action types, view modes
│   ├── states/    # Per-service state structs
│   └── update/    # Message handlers (like Redux reducers)
├── aws/           # AWS SDK wrappers (one file per service)
├── models/        # Data structures with from_aws() conversions
├── ui/            # All the rendering
│   ├── components/  # Reusable widgets (sidebar, modal, etc)
│   └── screens/     # Per-service screens
└── utils/         # Helpers (profiles, formatting, errors)
```

See [AGENTS.md](AGENTS.md) for detailed architecture docs and coding guidelines.

---

## Troubleshooting

**"No credentials found"**
```bash
aws configure  # Set up credentials
# Or use environment variables
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

**SSO login issues**
```bash
aws sso login --profile your-sso-profile
```

**LocalStack not connecting**
```bash
curl http://localhost:4566/_localstack/health  # Check if it's running
```
OR
```bash
curl http://localhost.localstack.cloud:4566/_localstack/health
```

---

## Roadmap

Some ideas for the future:

- [ ] Configuration file for custom keybindings
- [ ] More ECS actions (run task, update task def)
- [ ] CloudWatch logs viewer
- [ ] Cost explorer integration
- [ ] Resource tagging

---

## License

MIT

---

## Thanks

- [Ratatui](https://ratatui.rs/) - Amazing Rust TUI framework
- [lazygit](https://github.com/jesseduffield/lazygit) - The inspiration
- [AWS SDK for Rust](https://aws.amazon.com/sdk-for-rust/) - The foundation

---

Built with ☕ and too many late nights. PRs welcome!
