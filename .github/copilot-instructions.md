# GitHub Copilot Instructions for LazyAWS

## Project Overview

**LazyAWS** is a terminal user interface (TUI) for managing AWS resources, built with **Rust** and **Ratatui**. It's a keyboard-driven, fast, and responsive alternative to the AWS Console, inspired by `lazygit`.

- **~11,000 lines** of Rust code
- **10 AWS services** supported (EC2, S3, RDS, DynamoDB, Lambda, VPC, IAM, Backup, CloudTrail)
- **65 source files** across 5 major modules

## Technology Stack

| Component | Crate | Version |
|-----------|-------|---------|
| TUI Framework | `ratatui` | 0.29 |
| Terminal Backend | `crossterm` | 0.28 |
| Async Runtime | `tokio` | 1.x |
| AWS SDK | `aws-sdk-*` | 1.x |
| CLI Parsing | `clap` | 4.x |
| Error Handling | `anyhow`, `thiserror` | - |

## Architecture

The application uses a **hybrid pattern** combining:
- **The Elm Architecture (TEA)**: Centralized state, message-based updates
- **Component Pattern**: UI split into reusable, focused components

### Key Architecture Rules

1. **State Management**: All mutable state lives in `App`. Components receive `&App` for rendering.
2. **Updates**: State mutations only in `src/app/update.rs` via `Message` handling.
3. **Input Handling**: `src/app/input.rs` returns `Option<Message>`, never mutates state directly (except list navigation).
4. **Async Operations**: Use `tokio::spawn` for AWS operations. Never block the UI thread.

## Coding Standards

### Naming Conventions
- **Files**: `snake_case.rs`
- **Types**: `PascalCase` (e.g., `Ec2State`, `S3Action`)
- **Functions**: `snake_case` (e.g., `handle_refresh_data`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `ALL_REGIONS`)

### Error Handling
- Use `anyhow::Result` for async operations
- Use `thiserror` for domain-specific error types
- Always send errors back via `AwsEvent::Error(String)` to display in UI

### Testing
- Add unit tests in `#[cfg(test)] mod tests` at the bottom of files
- Test business logic (parsing, state transitions), not UI rendering
- Use descriptive test names: `test_parse_profile_endpoint_url_no_spaces`
- Run tests with: `cargo test`

### Code Style
- Write idiomatic, safe, and performant Rust code
- Prioritize user experience (responsiveness, clear feedback)
- Follow the established architecture patterns strictly
- Add comprehensive tests for new functionality

## Module Structure

```
src/
├── main.rs                 # Entry point, event loop, terminal setup
├── config.rs               # CLI args (clap), Args struct
├── event.rs                # Event, AwsEvent, EventHandler
├── error.rs                # Error types (thiserror)
├── app/                    # Application state machine (9 files)
│   ├── messages.rs         # Message, GlobalMessage, ServiceAction enums
│   ├── state.rs            # App struct, new(), render()
│   ├── update.rs           # Message handler (reducer)
│   ├── input.rs            # Keyboard handling
│   ├── events.rs           # AWS event handling
│   └── service_state.rs    # Per-service state structs
├── aws/                    # AWS SDK wrappers (12 files)
├── models/                 # Data structures (11 files)
├── ui/                     # Rendering logic (25 files)
│   ├── components/         # Reusable widgets
│   └── screens/            # Service-specific views
└── utils/                  # Utilities (5 files)
```

## Common Workflows

### Adding a New AWS Service

1. Update `Cargo.toml`: Add `aws-sdk-<service> = "1.x"`
2. Create Model in `src/models/<service>.rs`
3. Create AWS Service in `src/aws/<service>.rs`
4. Add to State in `src/app/service_state.rs`
5. Add Actions in `src/app/messages.rs`
6. Add Events in `src/event.rs`
7. Create Screen in `src/ui/screens/<service>.rs`
8. Update Handlers in `src/app/update.rs`, `input.rs`, `events.rs`
9. Update Navigation in `src/ui/components/sidebar.rs` and `src/ui/render.rs`

### Implementing an Action

1. Define Action in `src/app/messages.rs`
2. Add Keybinding in `src/app/input.rs`
3. Handle Message in `src/app/update.rs`
4. Implement AWS Call (async with `tokio::spawn`)
5. Handle result in `src/app/events.rs`

## Key Patterns

### Safe State Access
```rust
// Always use helper methods that return Option
if let Some(instance) = self.services.ec2.selected_instance() {
    // Safe to use instance
}
```

### Async Results
```rust
// Spawn task, send result via channel
let tx = event_tx.clone();
tokio::spawn(async move {
    match service.some_operation().await {
        Ok(data) => tx.send(Event::Aws(AwsEvent::DataLoaded(data))).ok(),
        Err(e) => tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(),
    }
});
```

## Important Features

- **Read-Only Mode**: `--read-only` flag prevents destructive actions
- **Confirmation Modals**: Safety mechanism for destructive operations (only `y/Y` confirms)
- **Profile/Region Switcher**: `Shift+P` with auto-detection of SSO profiles
- **Action Log**: `Shift+A` to view history of operations
- **Task Manager**: Tracks spawned async tasks, prevents dangling operations

## Build and Test

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run

# Run with specific profile
cargo run -- --profile my-profile --region us-east-1
```

## Additional Resources

For detailed architecture documentation, workflows, and advanced patterns, see the comprehensive [AGENTS.md](../AGENTS.md) file in the repository root.

---

*For GitHub Copilot coding agent: When working on issues, follow the architecture patterns strictly, make minimal surgical changes, and always add tests for new functionality.*
