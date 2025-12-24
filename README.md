# laws

TUI for AWS. [lazygit](https://github.com/jesseduffield/lazygit) meets AWS Console.

Rust + Ratatui. Vim keys. Async.

> [!CAUTION]
> Built this for myself while testing out AI coding agents (Antigravity, Kiro, etc). It works for me. Use at your own risk.

> [!NOTE]
> PRs welcome but I'll get to them when I get to them. This isn't my job.

## Install

```bash
cargo build --release && ./target/release/laws
```

Rust 1.75+

## Usage

```bash
laws                              # Profile switcher on first run
laws -p prod                      # Specify profile
laws -p prod -r eu-west-1         # Region
laws -p prod --read-only          # Safe mode
laws --endpoint-url http://localhost:4566  # LocalStack
```

## Services

EC2, S3, RDS, DynamoDB, Lambda, VPC, IAM, ECS, ECR, Backup, CloudTrail, Secrets Manager

## Keys

| Navigation | |
|---|---|
| `j/k` | Up/down |
| `g/G` | Top/bottom |
| `Enter` | Drill in |
| `Esc` | Back |
| `Tab` | Sidebar ↔ Main |
| `1-9` | Jump to service |
| `/` | Filter / Global search |
| `?` | Tag search |

| Actions | |
|---|---|
| `s/S` | Start/Stop |
| `R` | Reboot |
| `x` | Delete (confirms first) |
| `y` | Copy to clipboard |
| `e` | Edit (S3 objects, task defs) |
| `i` | Invoke (Lambda) |

| Views | |
|---|---|
| `d/D` | Detail panel / Fullscreen |
| `v` / `h/l` | Cycle views |
| `P` | Profile/region switcher |
| `A` | Action log |
| `r` | Refresh |
| `q` | Quit |

## Architecture

TEA + Component pattern. ~25k LOC.

```
src/
├── app/     # State, messages, input, update handlers
├── aws/     # SDK wrappers
├── models/  # Data structs
├── ui/      # Ratatui rendering
└── utils/   # Helpers
```

See [AGENTS.md](AGENTS.md) for the deep dive.

## Config

`~/.config/laws/config.toml`

```toml
theme = "dark"  # dark, light, monokai, nord
tick_rate_ms = 250
api_timeout_secs = 30
```

## License

MIT
