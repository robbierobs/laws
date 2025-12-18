# GitHub Copilot Instructions for LazyAWS

## Project Overview

**LazyAWS** is a terminal user interface (TUI) for managing AWS resources, built with **Rust** and **Ratatui**. It's a keyboard-driven, fast, and responsive alternative to the AWS Console, inspired by `lazygit`.

- **~25,000 lines** of Rust code
- **12 AWS services** supported (EC2, S3, RDS, DynamoDB, Lambda, VPC, IAM, Backup, CloudTrail, SecretsManager, ECS, ECR)
- **70+ source files** across 5 major modules
- **112 unit tests**

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
2. **Updates**: State mutations only in `src/app/update/` modules via `Message` handling.
3. **Input Handling**: `src/app/input.rs` routes to `ServiceInputHandler`, returns `InputResult`.
4. **Async Operations**: Use `spawn_aws_task` or `spawn_list_task`. Never block the UI thread.

---

## Critical: Trait Hierarchy

Every service state MUST implement these 4 traits:

### 1. `ServiceInternal` (src/app/states/traits.rs)
```rust
pub trait ServiceInternal: AutoSelectable + Searchable + Send {
    fn refresh(&mut self, tx: EventSender, clients: &AwsClients, tasks: &mut TaskManager, config: &AppConfig, report_errors: bool);
    fn clear(&mut self);
    fn auto_select_first(&mut self);
    fn can_cycle_view(&self) -> bool { false }  // Override for tabbed services
}
```

### 2. `ServiceInputHandler` (src/app/mod.rs)
```rust
pub trait ServiceInputHandler {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult;
    fn reset_selection(&mut self);
    fn get_copiable_text(&self) -> Option<String>;
}
```

### 3. `Searchable` (src/app/global_search.rs)
```rust
pub trait Searchable {
    fn get_search_results(&self) -> Vec<SearchResult>;
}
```

### 4. `AutoSelectable` (src/app/global_search.rs)
```rust
pub trait AutoSelectable {
    fn select_by_id(&mut self, resource_id: &str) -> bool;
}
```

### 5. `ViewMode` (src/app/view_mode.rs) - For tabbed services
```rust
pub trait ViewMode: Clone + Copy + PartialEq + Sized + 'static {
    fn all() -> &'static [Self];
    fn main_tabs() -> &'static [Self] { Self::all() }  // Exclude drill-downs
    fn index(&self) -> usize;
    fn from_index(i: usize) -> Self;
    fn label(&self) -> &'static str;
    fn is_main_tab(&self) -> bool { true }
    fn next(&self) -> Self;  // Cyclic among main_tabs
    fn prev(&self) -> Self;  // Cyclic among main_tabs
}
```

---

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

---

## Module Structure

```
src/
├── main.rs                 # Entry point, event loop
├── config.rs               # CLI args, AppConfig
├── event.rs                # Event, AwsEvent, EventHandler
├── error.rs                # Error types
│
├── app/                    # Application state machine
│   ├── messages/           # Message, GlobalMessage, ServiceAction
│   │   ├── mod.rs          # Main enums + Message constructors
│   │   ├── actions/        # Per-service action enums
│   │   └── view_modes.rs   # ViewMode enums
│   ├── states/             # Per-service state structs
│   │   ├── mod.rs          # ServiceStates + get_mut(Service)
│   │   ├── traits.rs       # ServiceInternal trait
│   │   └── <service>.rs    # State implementing all 4 traits
│   ├── update/             # Message handlers
│   │   ├── mod.rs          # Main dispatcher
│   │   ├── global.rs       # GlobalMessage handlers
│   │   ├── refresh.rs      # spawn_list_task helper
│   │   └── <service>.rs    # Action handlers
│   ├── input.rs            # handle_key() dispatcher
│   ├── events.rs           # handle_aws_event()
│   ├── task_manager.rs     # TaskManager + task_keys
│   ├── navigation.rs       # TableStateExt trait
│   ├── view_mode.rs        # ViewMode trait
│   └── global_search.rs    # Searchable, AutoSelectable
│
├── aws/                    # AWS SDK wrappers
│   ├── traits.rs           # AwsService<T>, aws_service_struct!
│   └── <service>.rs        # Service client
│
├── models/                 # Data structures with from_aws()
├── ui/                     # Rendering (components/, screens/)
└── utils/                  # Helpers
```

---

## Common Workflows

### Adding a New AWS Service

1. `Cargo.toml`: Add `aws-sdk-<service> = "1.x"`
2. `models/<service>.rs`: Define model with `from_aws()`
3. `aws/<service>.rs`: Use `aws_service_struct!` macro, implement `AwsService<T>`
4. `event.rs`: Add `<Service>Loaded(Vec<Model>)` variant
5. `states/<service>.rs`: Implement ALL 4 traits
6. `states/mod.rs`: Add to `ServiceStates`, `get_mut()`, `get_all_mut()`
7. `messages/actions/<service>.rs`: Define action enum
8. `messages/mod.rs`: Add to `ServiceAction`
9. `update/<service>.rs`: Create action handler
10. `update/mod.rs`: Add to dispatcher
11. `events.rs`: Handle new events
12. `task_manager.rs`: Add task keys
13. `ui/screens/<service>.rs`: Create render function
14. Update sidebar and action_bar

### Implementing an Action

1. Add variant to action enum in `messages/actions/`
2. Add keybinding in service state's `handle_input()`:
   ```rust
   KeyCode::Char('s') => {
       if let Some(item) = self.selected_item() {
           return InputResult::Action(Message::service_action(item.id.clone()));
       }
   }
   ```
3. Handle in `update/<service>.rs`:
   ```rust
   self.spawn_aws_task(event_tx, task_keys::SERVICE_ACTION, move |clients, tx| async move {
       // AWS operation
   });
   ```

---

## Key Patterns

### Async Task Spawning

```rust
// For list/refresh operations:
let handle = spawn_list_task(
    tx,
    move || async move { Service::new(client).list().await },
    AwsEvent::DataLoaded,
    report_errors,
);
tasks.spawn(task_keys::SERVICE_REFRESH, handle);

// For actions:
self.spawn_aws_task(event_tx, task_keys::SERVICE_ACTION, move |clients, tx| async move {
    let result = clients.service.some_action(&id).await;
    // Send result via tx
});
```

### Navigation Helpers

```rust
// In handle_input():
KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(self.items.len()),
KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(self.items.len()),
```

### InputResult Types

```rust
InputResult::None             // No action
InputResult::Message(msg)     // Execute immediately
InputResult::Action(msg)      // Needs confirmation
InputResult::OpenInputMode(m) // Switch to modal
```

### Dynamic Service Dispatch

```rust
// Single service:
self.services.get_mut(self.current_service).refresh(...);

// All services:
for service in self.services.get_all_mut() {
    service.refresh(...);
}
```

---

## Important Features

- **Read-Only Mode**: `--read-only` flag prevents destructive actions
- **Confirmation Modals**: Only `y/Y` confirms (Enter does NOT confirm)
- **Global Search**: `?` or `S` searches all services
- **Profile Switcher**: `Shift+P` with SSO auto-login
- **Task Manager**: All async tasks tracked, cancelled on shutdown

---

## Build and Test

```bash
cargo build      # Build
cargo test       # Run tests
cargo clippy     # Lint
cargo run        # Run app
cargo run -- --profile my-profile --region us-east-1  # With profile
```

---

## Additional Resources

For detailed architecture documentation, trait diagrams, and complete workflows, see the comprehensive [AGENTS.md](../AGENTS.md) file in the repository root.

---

*For GitHub Copilot: When working on issues, implement ALL required traits for service states, use spawn_aws_task for async operations, follow the InputResult pattern for keybindings, and always add tests.*
