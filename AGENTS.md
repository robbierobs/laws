# LazyAWS - Agent Guide

This document defines the personas, workflows, and standards for AI agents working on the **LazyAWS** project.

---

## 1. Project Context

**LazyAWS** is a terminal user interface (TUI) for managing AWS resources, built with **Rust** and **Ratatui**. It aims to be a keyboard-driven, fast, and responsive alternative to the AWS Console, inspired by `lazygit`.

### 1.1 Project Statistics
- **~25,000 lines** of Rust code
- **12 AWS services** supported
- **70+ source files** across 5 major modules
- **112 unit tests**

### 1.2 Technology Stack
| Component | Crate | Purpose |
|-----------|-------|---------|
| TUI Framework | `ratatui` 0.29 | Terminal UI rendering |
| Terminal Backend | `crossterm` 0.28 | Cross-platform terminal control |
| Async Runtime | `tokio` 1.x | Async execution |
| AWS SDK | `aws-sdk-*` 1.x | AWS service clients |
| CLI Parsing | `clap` 4.x | Command-line arguments |
| Error Handling | `anyhow`, `thiserror` | Error management |

---

## 2. Architecture Deep-Dive

### 2.1 Core Architecture Pattern: Hybrid TEA + Component

The application uses a **hybrid pattern** combining:
- **The Elm Architecture (TEA)**: Centralized state, message-based updates
- **Component Pattern**: UI split into reusable, focused components

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                     Event Loop                          ││
│  │  ┌──────────┐   ┌──────────┐   ┌──────────────────────┐ ││
│  │  │ Keyboard │-->│ App::    │-->│ App::update(Message) │ ││
│  │  │  Event   │   │ handle_  │   │  (State Mutation)    │ ││
│  │  └──────────┘   │ key()    │   └──────────────────────┘ ││
│  │                 └──────────┘            │               ││
│  │       │                                 │               ││
│  │       ▼                                 ▼               ││
│  │  ┌──────────┐                   ┌──────────────────┐    ││
│  │  │  Event:: │                   │ tokio::spawn()   │    ││
│  │  │  Aws()   │<------------------│ (AWS Operations) │    ││
│  │  └──────────┘                   └──────────────────┘    ││
│  │       │                                                 ││
│  │       ▼                                                 ││
│  │  ┌──────────────────┐     ┌───────────────────────────┐ ││
│  │  │ App::handle_     │     │     App::render()         │ ││
│  │  │ aws_event()      │     │  (Pure, no side effects)  │ ││
│  │  └──────────────────┘     └───────────────────────────┘ ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

1. **Input**: Keyboard event received via `crossterm`
2. **Handle**: `App::handle_key()` dispatches to `ServiceInputHandler` → returns `InputResult`
3. **Update**: `App::update(msg)` mutates state and/or spawns async tasks
4. **Async Result**: AWS operations complete, send `AwsEvent` via channel
5. **Event Handle**: `App::handle_aws_event()` updates state with results
6. **Render**: `App::render()` draws current state to terminal

### 2.3 Message Hierarchy

```rust
enum Message {
    Global(GlobalMessage),    // App-wide: Navigate, Quit, TogglePanel, etc.
    Service(ServiceAction),   // Per-service: Ec2Action, S3Action, etc.
}

enum GlobalMessage {
    Navigate(Service),
    Quit,
    RefreshData,
    ToggleDetailPanel,
    ToggleActionLog,
    OpenProfileSwitcher,
    SwitchProfileRegion { profile, region, read_only },
    ConfirmAction,
    CancelAction,
    // ...
}

enum ServiceAction {
    Ec2(Ec2Action),   // Start, Stop, Reboot
    S3(S3Action),     // LoadObjects, DeleteObject, DownloadObject, OpenObject
    Rds(RdsAction),   // Start, Stop, Reboot
    DynamoDb(DynamoDbAction), // DrillDownTable, LoadItems, DeleteItem
    Vpc(VpcAction),   // DrillDownSecurityGroup, ToggleDirection
    Iam(IamAction),   // DrillDownUser, DrillDownRole, DrillDownPolicy
    // ...
}
```

---

## 3. Trait Hierarchy (Critical for Development)

The codebase uses a well-defined trait hierarchy to standardize behavior across all 12 AWS services:

### 3.1 Core Traits Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SERVICE STATE TRAITS                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    ServiceInternal                               │    │
│  │  (src/app/states/traits.rs)                                      │    │
│  │  ───────────────────────────────────────────────────────────────│    │
│  │  Required methods:                                               │    │
│  │    - refresh(&mut self, tx, clients, tasks, config, report_err)  │    │
│  │    - clear(&mut self)                                            │    │
│  │    - auto_select_first(&mut self)                                │    │
│  │  Default methods:                                                 │    │
│  │    - can_cycle_view(&self) -> bool { false }                     │    │
│  │                                                                   │    │
│  │  Supertraits: AutoSelectable + Searchable + Send                 │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│          │                      │                                        │
│          ▼                      ▼                                        │
│  ┌──────────────────┐  ┌────────────────────────────────────┐           │
│  │  AutoSelectable  │  │           Searchable               │           │
│  │  (global_search) │  │         (global_search)            │           │
│  │  ────────────────│  │  ──────────────────────────────────│           │
│  │  - select_by_id  │  │  - get_search_results() -> Vec<>   │           │
│  │    (&mut, &str)  │  │                                    │           │
│  │    -> bool       │  │                                    │           │
│  └──────────────────┘  └────────────────────────────────────┘           │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                  ServiceInputHandler                             │    │
│  │  (src/app/mod.rs)                                                │    │
│  │  ───────────────────────────────────────────────────────────────│    │
│  │  Required methods:                                               │    │
│  │    - handle_input(&mut self, key: KeyEvent) -> InputResult       │    │
│  │    - reset_selection(&mut self)                                  │    │
│  │    - get_copiable_text(&self) -> Option<String>                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                              OTHER TRAITS                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        ViewMode                                  │    │
│  │  (src/app/view_mode.rs)                                          │    │
│  │  ───────────────────────────────────────────────────────────────│    │
│  │  For services with multiple tabs (VPC, IAM, Backup, etc.)       │    │
│  │  Required:                                                       │    │
│  │    - all() -> &'static [Self]                                    │    │
│  │    - index(&self) -> usize                                       │    │
│  │    - from_index(i: usize) -> Self                                │    │
│  │    - label(&self) -> &'static str                                │    │
│  │  Defaults:                                                       │    │
│  │    - main_tabs() -> all()                                        │    │
│  │    - next(&self), prev(&self) -- cyclic navigation               │    │
│  │    - is_main_tab(&self) -> bool { true }                         │    │
│  │    - is_drill_down(&self) -> bool { !is_main_tab() }             │    │
│  │    - supports_cycling(&self) -> bool { is_main_tab() }           │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  ┌─────────────────────────────────────────┐ ┌────────────────────────┐ │
│  │        AwsService<T>                    │ │   TableStateExt        │ │
│  │  (src/aws/traits.rs)                    │ │ (src/app/navigation.rs)│ │
│  │  ──────────────────────────────────────│ │  ─────────────────────│ │
│  │  For AWS SDK wrappers:                  │ │  Extension trait for   │ │
│  │    - list(&self) -> Future<Vec<T>>      │ │  TableState:           │ │
│  │                                         │ │  - nav_up(&mut, len)   │ │
│  │                                         │ │  - nav_down(&mut, len) │ │
│  │                                         │ │  - nav_first/last      │ │
│  │                                         │ │  - clamp_selection     │ │
│  └─────────────────────────────────────────┘ └────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Required Trait Implementations Per Service State

Every `XxxState` struct in `src/app/states/` MUST implement:

| Trait | Location | Purpose |
|-------|----------|---------|
| `ServiceInternal` | `states/traits.rs` | Refresh, clear, auto-select |
| `ServiceInputHandler` | `app/mod.rs` | Handle keyboard input |
| `Searchable` | `global_search.rs` | Provide search results |
| `AutoSelectable` | `global_search.rs` | Select by resource ID |

For services with multiple views (tabs):
| Trait | Location | Purpose |
|-------|----------|---------|
| `ViewMode` | `view_mode.rs` | Tab navigation (next/prev/label) |

### 3.3 Example: Complete Service State Implementation

```rust
// src/app/states/ec2.rs

use crate::app::{InputResult, Message, ServiceInputHandler, TableStateExt, Service};
use crate::app::global_search::{AutoSelectable, Searchable, SearchResult};
use crate::app::states::ServiceInternal;
use crate::app::EventSender;
use crate::aws::client::AwsClients;
use crate::app::task_manager::{TaskManager, task_keys};
use crate::app::update::refresh::spawn_list_task;
use crate::aws::traits::AwsService;
use crate::event::AwsEvent;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

/// State for EC2 service
#[derive(Default)]
pub struct Ec2State {
    pub instances: Vec<Ec2Instance>,
    pub list_state: TableState,
}

impl Ec2State {
    pub fn new() -> Self { Self::default() }

    /// Get the currently selected instance
    pub fn selected_instance(&self) -> Option<&Ec2Instance> {
        self.list_state.selected().and_then(|i| self.instances.get(i))
    }
}

// === TRAIT: Searchable ===
impl Searchable for Ec2State {
    fn get_search_results(&self) -> Vec<SearchResult> {
        self.instances.iter().map(|i| {
            let mut r = SearchResult::new(Service::EC2, "EC2 Instance", &i.instance_id);
            if let Some(name) = &i.name {
                r = r.with_secondary(name.clone());
            }
            r.with_tags(i.tags.clone())
        }).collect()
    }
}

// === TRAIT: AutoSelectable ===
impl AutoSelectable for Ec2State {
    fn select_by_id(&mut self, resource_id: &str) -> bool {
        if let Some(idx) = self.instances.iter().position(|i| i.instance_id == resource_id) {
            self.list_state.select(Some(idx));
            true
        } else {
            false
        }
    }
}

// === TRAIT: ServiceInputHandler ===
impl ServiceInputHandler for Ec2State {
    fn handle_input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.nav_down(self.instances.len()),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.nav_up(self.instances.len()),
            KeyCode::Char('s') => {
                if let Some(instance) = self.selected_instance() {
                    return InputResult::Action(Message::ec2_start(instance.instance_id.clone()));
                }
            }
            // ... more keybindings
            _ => {}
        }
        InputResult::None
    }

    fn reset_selection(&mut self) {
        self.list_state.select(Some(0));
    }

    fn get_copiable_text(&self) -> Option<String> {
        self.selected_instance().map(|i| i.instance_id.clone())
    }
}

// === TRAIT: ServiceInternal ===
impl ServiceInternal for Ec2State {
    fn refresh(
        &mut self,
        tx: EventSender,
        clients: &AwsClients,
        tasks: &mut TaskManager,
        _config: &crate::config::AppConfig,
        report_errors: bool,
    ) {
        let client = clients.ec2.clone();
        let handle = spawn_list_task(
            tx,
            move || async move { Ec2Service::new(client).list().await },
            AwsEvent::Ec2InstancesLoaded,
            report_errors,
        );
        tasks.spawn(task_keys::EC2_REFRESH, handle);
    }

    fn clear(&mut self) {
        self.instances.clear();
        self.list_state.select(Some(0));
    }

    fn auto_select_first(&mut self) {
        if self.list_state.selected().is_none() && !self.instances.is_empty() {
            self.list_state.select(Some(0));
        }
    }
    
    // Override if service has tabs (VPC, IAM, Backup, etc.)
    fn can_cycle_view(&self) -> bool {
        false  // EC2 has no view modes
    }
}
```

---

## 4. Module Structure

```
src/
├── main.rs                 # Entry point, event loop, terminal setup
├── config.rs               # CLI args (clap), Args struct, AppConfig
├── event.rs                # Event, AwsEvent, EventHandler (bounded channel)
├── error.rs                # Error types (thiserror)
│
├── app/                    # Application state machine (~15 files)
│   ├── mod.rs              # Re-exports, InputResult enum, ServiceInputHandler trait
│   ├── messages/           # Message, GlobalMessage, ServiceAction enums
│   │   ├── mod.rs          # Main message enums + constructors
│   │   ├── service.rs      # Service enum
│   │   ├── view_modes.rs   # ViewMode enums (VpcViewMode, IamViewMode, etc.)
│   │   └── actions/        # Per-service action enums (Ec2Action, S3Action, etc.)
│   ├── state.rs            # App struct, new(), render()
│   ├── states/             # Per-service state structs
│   │   ├── mod.rs          # ServiceStates container + get_mut(Service)
│   │   ├── traits.rs       # ServiceInternal trait
│   │   ├── ec2.rs          # Ec2State
│   │   ├── s3.rs           # S3State (with ViewerMode, BucketCreation)
│   │   └── ...             # Other service states
│   ├── update/             # Message handlers split by service (~17 files)
│   │   ├── mod.rs          # Main update() dispatcher
│   │   ├── global.rs       # GlobalMessage handlers
│   │   ├── refresh.rs      # spawn_list_task helper
│   │   ├── ec2.rs          # EC2 action handlers
│   │   └── ...             # Other service handlers
│   ├── input.rs            # handle_key() dispatcher
│   ├── input_handlers/     # Modal-specific input handlers
│   ├── events.rs           # handle_aws_event()
│   ├── task_manager.rs     # TaskManager for async task tracking
│   ├── navigation.rs       # TableStateExt, nav_up/down helpers
│   ├── view_mode.rs        # ViewMode trait
│   ├── filtered_list.rs    # FilteredList with caching
│   ├── global_search.rs    # Searchable, AutoSelectable, SearchResult
│   └── profile_switcher.rs # ProfileSwitcherState
│
├── aws/                    # AWS SDK wrappers (~15 files)
│   ├── client.rs           # AwsClients initialization
│   ├── traits.rs           # AwsService<T> trait, aws_service_struct! macro
│   ├── ec2.rs              # Ec2Service
│   └── ...                 # Other service clients
│
├── models/                 # Data structures (~14 files)
│   ├── ec2.rs              # Ec2Instance, InstanceState + state_color()
│   └── ...                 # Other models with from_aws() conversion
│
├── ui/                     # Rendering logic (~29 files)
│   ├── render.rs           # Main render dispatcher
│   ├── theme.rs            # THEME constant with colors
│   ├── components/         # Reusable widgets
│   │   ├── sidebar.rs      # Service navigation
│   │   ├── modal.rs        # All modal rendering functions
│   │   ├── action_bar.rs   # Bottom help bar with keybindings
│   │   └── ...
│   └── screens/            # Service-specific views
│       ├── ec2.rs          # render_ec2_screen()
│       └── ...
│
└── utils/                  # Utilities (~6 files)
    ├── aws_profiles.rs     # list_profiles(), is_sso_profile()
    └── ...
```

---

## 5. Agent Persona

When working on this project, adopt the following persona:

- **Role**: Senior Rust Systems Engineer & UI/UX Designer
- **Expertise**:
  - **Rust**: Deep knowledge of lifetimes, async/await (`tokio`), trait patterns
  - **TUI**: Expert in `ratatui` and `crossterm`
  - **AWS**: Familiar with AWS SDK patterns and service APIs
- **Style**:
  - Write idiomatic, safe, and performant Rust code
  - Prioritize user experience (responsiveness, clear feedback)
  - Follow the established architecture patterns strictly
  - Add comprehensive tests for new functionality

---

## 6. Coding Standards

### 6.1 Architecture Rules
- **State**: All mutable state lives in `App`. Components receive `&App` for rendering.
- **Updates**: State mutations only in `update/` modules via `Message` handling.
- **Input**: `input.rs` routes to `ServiceInputHandler`, never mutates state directly (except list navigation).
- **Async**: Use `spawn_aws_task` helper or `spawn_list_task`. Never block the UI thread.

### 6.2 Naming Conventions
- **Files**: `snake_case.rs`
- **Types**: `PascalCase` (e.g., `Ec2State`, `S3Action`)
- **Functions**: `snake_case` (e.g., `handle_refresh_data`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `ALL_REGIONS`)

### 6.3 Error Handling
- Use `anyhow::Result` for async operations
- Use `thiserror` for domain-specific error types
- Always send errors back via `AwsEvent::Error(String)` to display in UI

### 6.4 Testing
- Add unit tests in `#[cfg(test)] mod tests` at the bottom of files
- Test business logic (parsing, state transitions), not UI rendering
- Use descriptive test names: `test_parse_profile_endpoint_url_no_spaces`

---

## 7. Workflows

### 7.1 Adding a New AWS Service

1. **Update `Cargo.toml`**: Add `aws-sdk-<service> = "1.x"`
2. **Create Model** (`src/models/<service>.rs`): Define data structs with `from_aws()` conversion
3. **Create AWS Service** (`src/aws/<service>.rs`):
   - Use `aws_service_struct!(<Service>Service, Client)` macro
   - Implement methods for listing and actions
   - Implement `AwsService<Model>` trait
4. **Add to Event** (`src/event.rs`): Add `<Service>Loaded(Vec<Model>)` to `AwsEvent`
5. **Create State** (`src/app/states/<service>.rs`):
   - Define `<Service>State` struct with data fields + `TableState`
   - Implement `Searchable`, `AutoSelectable`, `ServiceInputHandler`, `ServiceInternal`
   - For multi-tab services: implement `ViewMode` trait on a `<Service>ViewMode` enum
6. **Add to ServiceStates** (`src/app/states/mod.rs`): Add field + update `get_mut()`, `get_all_mut()`
7. **Add Actions** (`src/app/messages/actions/<service>.rs`): Create `<Service>Action` enum
8. **Add to ServiceAction** (`src/app/messages/mod.rs`): Add variant
9. **Create Update Handler** (`src/app/update/<service>.rs`): Implement `handle_<service>_action()`
10. **Add to Dispatcher** (`src/app/update/mod.rs`): Add case in `handle_service_action()`
11. **Handle Events** (`src/app/events.rs`): Handle the new `AwsEvent` variants
12. **Add Task Keys** (`src/app/task_manager.rs`): Add refresh/action keys
13. **Create Screen** (`src/ui/screens/<service>.rs`): Implement `render_<service>_screen()`
14. **Update UI**:
    - `src/ui/render.rs`: Add dispatch case
    - `src/ui/components/sidebar.rs`: Add to service list
    - `src/ui/components/action_bar.rs`: Add keybinding hints

### 7.2 Implementing an Action

1. **Define Action** in `src/app/messages/actions/<service>.rs`:
   ```rust
   pub enum Ec2Action {
       Start(String),  // instance_id
       // ...
   }
   ```

2. **Add Keybinding** in service state's `handle_input()`:
   ```rust
   KeyCode::Char('s') => {
       if let Some(instance) = self.selected_instance() {
           return InputResult::Action(Message::ec2_start(instance.instance_id.clone()));
       }
   }
   ```

3. **Handle Message** in `src/app/update/<service>.rs`:
   ```rust
   pub(super) fn handle_ec2_action(&mut self, action: Ec2Action, event_tx: EventSender) {
       match action {
           Ec2Action::Start(id) => {
               self.spawn_aws_task(event_tx, task_keys::EC2_ACTION, move |clients, tx| async move {
                   // AWS call here
               });
           }
       }
   }
   ```

4. **Handle Result** in `src/app/events.rs`:
   ```rust
   AwsEvent::ActionCompleted(msg) => {
       self.action_log.push(format!("[SUCCESS] {}", msg));
       self.should_refresh = true;
   }
   ```

### 7.3 Implementing Hierarchical Navigation (Drill-Down)

1. **Add State Fields**:
   ```rust
   pub struct S3State {
       pub buckets: Vec<S3Bucket>,
       pub current_bucket: Option<String>,  // Indicates drill-down
       pub objects: Vec<S3Object>,
   }
   ```

2. **Add Helper Methods**:
   ```rust
   pub fn is_viewing_objects(&self) -> bool {
       self.current_bucket.is_some()
   }
   ```

3. **Handle Input**:
   ```rust
   KeyCode::Enter => {
       if !self.is_viewing_objects() {
           if let Some(bucket) = self.selected_bucket() {
               return InputResult::Message(Message::s3_load_objects(bucket.name.clone()));
           }
       }
   }
   KeyCode::Esc | KeyCode::Backspace => {
       if self.is_viewing_objects() {
           return InputResult::Message(Message::s3_leave_bucket());
       }
   }
   ```

---

## 8. Key Patterns

### 8.1 Standardized Async Task Pattern

**ALWAYS** use `spawn_aws_task` or `spawn_list_task`:

```rust
// For refresh operations:
let handle = spawn_list_task(
    tx,
    move || async move { Ec2Service::new(client).list().await },
    AwsEvent::Ec2InstancesLoaded,
    report_errors,
);
tasks.spawn(task_keys::EC2_REFRESH, handle);

// For action operations:
self.spawn_aws_task(event_tx, task_keys::EC2_ACTION, move |clients, tx| async move {
    let service = Ec2Service::new(clients.ec2.clone());
    match service.start_instance(&id).await {
        Ok(_) => tx.send(Event::Aws(Box::new(AwsEvent::ActionCompleted(...)))).await.ok(),
        Err(e) => tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string())))).await.ok(),
    }
});
```

### 8.2 Safe State Access

```rust
// Always use helper methods that return Option
if let Some(instance) = self.services.ec2.selected_instance() {
    // Safe to use instance
}
```

### 8.3 InputResult Types

```rust
pub enum InputResult {
    None,                        // No action
    Message(Message),            // Execute immediately
    Action(Message),             // Needs confirmation first
    OpenInputMode(InputMode),    // Switch to modal input mode
}
```

### 8.4 Dynamic Service Dispatch

```rust
// Get current service state dynamically:
self.services.get_mut(self.current_service).refresh(tx, clients, tasks, config, true);

// Iterate over all services:
for service in self.services.get_all_mut() {
    service.refresh(tx.clone(), clients, tasks, config, false);
}
```

---

## 9. Important Features

### 9.1 Read-Only Mode
- `App.read_only: bool` prevents destructive actions
- `--read-only` CLI flag or toggle via `Shift+R` in profile switcher
- `request_action()` in `input.rs` checks this flag

### 9.2 Confirmation Modals
- `App.pending_action: Option<Message>` stores action awaiting confirmation
- `App.show_confirmation: bool` triggers modal render
- Only `y/Y` confirms, `n/N/Esc` cancels (Enter does NOT confirm for safety)

### 9.3 Global Search
- `?` or `S` opens global search modal
- Searches across all services using `Searchable` trait
- `tag:` prefix for tag-only search
- Results show service, type, ID, and name

### 9.4 Task Manager
- All async tasks tracked via `TaskManager`
- Predefined keys in `task_keys::*` module
- `cancel_all()` on shutdown prevents dangling tasks

### 9.5 Profile/Region Switcher
- `Shift+P` opens modal
- Reads profiles from `~/.aws/config` and `~/.aws/credentials`
- Auto-detects SSO profiles and runs `aws sso login` if needed

---

## 10. Testing

Run tests with:
```bash
cargo test
```

Current test coverage areas:
- `app::navigation` - List navigation helpers
- `app::filtered_list` - Filter and selection logic
- `app::global_search` - Search matching and state
- `app::task_manager` - Task spawn/cancel/cleanup
- `app::view_mode` - Tab cycling behavior
- `event` - Channel backpressure handling
- `utils::aws_profiles` - Config parsing, SSO detection
- `models::*` - State display, color methods

---

*Last updated: 2025-12-18*
*Target: Rust 1.75+, Ratatui 0.29, AWS SDK for Rust 1.x*
