# LazyAWS - Agent Guide

This document defines the personas, workflows, and standards for AI agents working on the **LazyAWS** project.

---

## 1. Project Context

**LazyAWS** is a terminal user interface (TUI) for managing AWS resources, built with **Rust** and **Ratatui**. It aims to be a keyboard-driven, fast, and responsive alternative to the AWS Console, inspired by `lazygit`.

### 1.1 Project Statistics
- **~12,000 lines** of Rust code
- **11 AWS services** supported
- **70+ source files** across 5 major modules
- **88 unit tests**

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
2. **Handle**: `App::handle_key()` returns `Option<Message>`
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

### 2.4 State Organization

State is organized hierarchically to avoid a monolithic `App` struct:

```rust
struct App {
    // Core state
    current_service: Service,
    input_mode: InputMode,
    focus: Focus,
    
    // AWS
    aws_clients: Option<AwsClients>,
    profile: Option<String>,
    region: String,
    
    // Per-service state (organized in ServiceStates)
    services: ServiceStates,
    
    // UI modals and action tracking
    pending_action: Option<Message>,
    show_confirmation: bool,
    action_log: Vec<String>,
    
    // Async task management
    tasks: TaskManager,
    
    // Profile switcher state
    available_profiles: Vec<String>,
    available_regions: Vec<String>,
    
    // Configuration
    config: AppConfig,
}

struct ServiceStates {
    ec2: Ec2State,
    s3: S3State,
    rds: RdsState,
    dynamodb: DynamoDbState,
    lambda: LambdaState,
    vpc: VpcState,
    iam: IamState,
    backup: BackupState,
    cloudtrail: CloudTrailState,
    secretsmanager: SecretsManagerState,
    ecs: EcsState,
}
```

---

## 3. Module Structure

```
src/
├── main.rs                 # Entry point, event loop, terminal setup
├── config.rs               # CLI args (clap), Args struct
├── event.rs                # Event, AwsEvent, EventHandler
├── error.rs                # Error types (thiserror)
│
├── app/                    # Application state machine (12 files, ~170KB)
│   ├── mod.rs              # Re-exports: App, Message, Service, Focus, InputMode
│   ├── messages.rs         # Message, GlobalMessage, ServiceAction, ViewMode enums
│   ├── state.rs            # App struct, new(), render()
│   ├── update/             # Message handlers split by service (14 files)
│   ├── input.rs            # Keyboard handling, handle_key() (~40KB)
│   ├── events.rs           # AWS event handling
│   ├── service_state.rs    # Per-service state structs (~50KB)
│   ├── task_manager.rs     # Async task tracking with TaskManager
│   ├── filtered_list.rs    # Generic filter logic for resource lists
│   ├── navigation.rs       # List navigation helpers (TableStateExt trait)
│   ├── ecs_modals.rs       # ECS modal state components
│   └── view_mode.rs        # Generic ViewMode trait for multi-tab services
│
├── aws/                    # AWS SDK wrappers (12 files)
│   ├── client.rs           # AwsClients initialization, endpoint_url handling
│   ├── traits.rs           # AwsService trait definition
│   ├── ec2.rs              # Ec2Service: list, start, stop, reboot
│   ├── s3.rs               # S3Service: list buckets/objects, get/delete object
│   ├── rds.rs              # RdsService: list, start, stop, reboot
│   ├── dynamodb.rs         # DynamoDbService: list tables, scan items, delete
│   ├── lambda.rs           # LambdaService: list functions
│   ├── vpc.rs              # VpcService: list VPCs, subnets, security groups
│   ├── iam.rs              # IamService: list users/roles/policies, get policy doc
│   ├── backup.rs           # BackupService: list vaults/plans/jobs
│   ├── cloudtrail.rs       # CloudTrailService: list trails, lookup events
│   ├── secretsmanager.rs   # SecretsManagerService: list secrets, get values
│   └── ecs.rs              # EcsClient: clusters, services, tasks, task definitions
│
├── models/                 # Data structures (13 files)
│   ├── ec2.rs              # Ec2Instance, InstanceState + state_color()
│   ├── s3.rs               # S3Bucket, S3Object, S3BucketDetails
│   ├── rds.rs              # RdsInstance + status_color()
│   ├── dynamodb.rs         # DynamoDbTable, DynamoDbItem, KeySchema
│   ├── lambda.rs           # LambdaFunction + runtime_color()
│   ├── vpc.rs              # Vpc, Subnet, SecurityGroup, SgRule
│   ├── iam.rs              # IamUser, IamRole, IamPolicy
│   ├── backup.rs           # BackupVault, BackupPlan, BackupJob + status_color()
│   ├── cloudtrail.rs       # Trail, CloudTrailEvent
│   ├── secretsmanager.rs   # Secret
│   ├── ecs.rs              # EcsCluster, EcsService, EcsTask, EcsTaskDefinition
│   └── ids.rs              # Type-safe ID wrappers (Ec2InstanceId, etc.)
│
├── ui/                     # Rendering logic (25 files)
│   ├── render.rs           # Main render dispatcher
│   ├── theme.rs            # THEME constant with colors
│   ├── components/         # Reusable widgets (11 files)
│   │   ├── sidebar.rs      # Service navigation
│   │   ├── modal.rs        # Confirmation, object viewer, profile switcher modals
│   │   ├── action_bar.rs   # Bottom help bar with keybindings
│   │   ├── action_log.rs   # Action history popup
│   │   └── tabs.rs         # Tab navigation component
│   └── screens/            # Service-specific views (10 files)
│       ├── ec2.rs          # render_ec2_screen, render_ec2_row, render_ec2_details
│       ├── s3.rs           # render_s3_screen (buckets + objects views)
│       ├── vpc.rs          # render_vpc_screen (VPCs, Subnets, SGs, Rules)
│       ├── iam.rs          # render_iam_screen (Users, Roles, Policies, Docs)
│       └── ...
│
└── utils/                  # Utilities (5 files)
    ├── aws_profiles.rs     # list_profiles(), is_sso_profile(), get_profile_endpoint_url()
    ├── error.rs            # Error helpers
    ├── formatting.rs       # Display formatting
    └── pagination.rs       # AWS pagination helpers
```

---

## 4. Agent Persona

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

## 5. Coding Standards

### 5.1 Architecture Rules
- **State**: All mutable state lives in `App`. Components receive `&App` for rendering.
- **Updates**: State mutations only in `update.rs` via `Message` handling.
- **Input**: `input.rs` returns `Option<Message>`, never mutates state directly (except list navigation).
- **Async**: Use `tokio::spawn` for AWS operations. Never block the UI thread.

### 5.2 Naming Conventions
- **Files**: `snake_case.rs`
- **Types**: `PascalCase` (e.g., `Ec2State`, `S3Action`)
- **Functions**: `snake_case` (e.g., `handle_refresh_data`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `ALL_REGIONS`)

### 5.3 Error Handling
- Use `anyhow::Result` for async operations
- Use `thiserror` for domain-specific error types
- Always send errors back via `AwsEvent::Error(String)` to display in UI

### 5.4 Testing
- Add unit tests in `#[cfg(test)] mod tests` at the bottom of files
- Test business logic (parsing, state transitions), not UI rendering
- Use descriptive test names: `test_parse_profile_endpoint_url_no_spaces`

---

## 6. Workflows

### 6.1 Adding a New AWS Service

1. **Update `Cargo.toml`**: Add `aws-sdk-<service> = "1.x"`
2. **Create Model** (`src/models/<service>.rs`): Define data structs with `from_aws()` conversion
3. **Create AWS Service** (`src/aws/<service>.rs`):
   - Implement `<Service>Service` struct with implicit methods
   - Implement `AwsService<Model>` trait for standard listing
4. **Add to State** (`src/app/service_state.rs`): Create `<Service>State` struct
5. **Add to Messages** (`src/app/messages.rs`):
   - Add variant to `Service` enum
   - Create `<Service>Action` enum
   - Add to `ServiceAction` enum
6. **Add to AwsEvent** (`src/event.rs`): Add loaded event variants
7. **Create Screen** (`src/ui/screens/<service>.rs`): Implement `render_<service>_screen()`
8. **Update Handlers**:
   - `src/app/update.rs`: Handle service actions
   - `src/app/input.rs`: Add keybindings
   - `src/app/events.rs`: Handle AWS events
9. **Update Navigation**:
   - `src/ui/components/sidebar.rs`: Add to service list
   - `src/ui/screens/mod.rs`: Add screen function
   - `src/ui/render.rs`: Add dispatch case

### 6.2 Implementing an Action (e.g., Start Instance)

1. **Define Action** in `messages.rs`:
   ```rust
   pub enum Ec2Action {
       Start(String),  // instance_id
       // ...
   }
   ```

2. **Add Keybinding** in `input.rs`:
   ```rust
   KeyCode::Char('s') => {
       if let Some(id) = self.services.ec2.selected_instance_id() {
           return InputResult::Action(Message::ec2_start(id));
       }
   }
   ```

3. **Request Confirmation** (for destructive actions):
   ```rust
   // In request_action() helper, it checks read_only mode and sets pending_action
   ```

4. **Handle Message** in `update.rs`:
   ```rust
   ServiceAction::Ec2(Ec2Action::Start(id)) => {
       self.handle_ec2_action("start", id, event_tx);
   }
   ```

5. **Implement AWS Call**:
   ```rust
   fn handle_ec2_action(&mut self, action: &str, id: String, event_tx: ...) {
       let client = clients.ec2.clone();
       tokio::spawn(async move {
           let service = Ec2Service::new(client);
           match service.start_instance(&id).await {
               Ok(_) => tx.send(Event::Aws(AwsEvent::ActionCompleted(...))).ok(),
               Err(e) => tx.send(Event::Aws(AwsEvent::Error(e.to_string()))).ok(),
           }
       });
   }
   ```

### 6.3 Implementing Hierarchical Navigation (e.g., S3 Bucket → Objects)

1. **Add State Fields** in `service_state.rs`:
   ```rust
   pub struct S3State {
       pub buckets: Vec<S3Bucket>,
       pub current_bucket: Option<String>,  // Indicates drill-down
       pub objects: Vec<S3Object>,
       // ...
   }
   ```

2. **Add Helper Methods**:
   ```rust
   pub fn is_viewing_objects(&self) -> bool {
       self.current_bucket.is_some()
   }
   ```

3. **Add Actions** in `messages.rs`:
   ```rust
   pub enum S3Action {
       LoadObjects(String),  // bucket_name
       LeaveBucket,
       // ...
   }
   ```

4. **Handle in Input**:
   ```rust
   // Enter drills down
   KeyCode::Enter => {
       if self.services.s3.is_viewing_objects() {
           // Handle object selection
       } else {
           // Drill into bucket
           return Some(Message::s3_load_objects(bucket_name));
       }
   }
   
   // Esc goes back
   KeyCode::Esc => {
       if self.services.s3.is_viewing_objects() {
           return Some(Message::s3_leave_bucket());
       }
   }
   ```

5. **Conditional Rendering** in screen:
   ```rust
   if app.services.s3.is_viewing_objects() {
       render_objects_table(frame, area, app);
   } else {
       render_buckets_table(frame, area, app);
   }
   ```

---

## 7. Key Features & Patterns

### 7.1 Read-Only Mode
- `App.read_only: bool` prevents destructive actions
- `--read-only` CLI flag or toggle via `Shift+R` in profile switcher
- `request_action()` helper in `input.rs` checks this flag

### 7.2 Confirmation Modals
- `App.pending_action: Option<Message>` stores action awaiting confirmation
- `App.show_confirmation: bool` triggers modal render
- Only `y/Y` confirms, `n/N/Esc` cancels (Enter does NOT confirm for safety)

### 7.3 Profile/Region Switcher
- `Shift+P` opens modal
- Reads profiles from `~/.aws/config` and `~/.aws/credentials`
- Auto-detects SSO profiles and runs `aws sso login` if needed
- Reads `endpoint_url` from profile config for LocalStack support

### 7.4 Task Manager
- `TaskManager` tracks spawned async tasks
- `spawn(key, handle)` replaces existing task with same key
- `cancel_all()` on shutdown prevents dangling tasks
- Predefined keys: `task_keys::EC2_REFRESH`, `EC2_ACTION`, etc.

### 7.5 Action Log
- `App.action_log: Vec<String>` stores history
- Success/error events logged in `events.rs`
- `Shift+A` opens full-screen popup
- Last action shown in action bar (color-coded)

### 7.6 Multi-View Navigation
- Services like VPC, IAM, Backup have multiple views
- Implements `ViewMode` trait (`next()`, `prev()`, `label()`, `iterator()`)
- Keys: `v` cycles, `h/l` navigate, tabs shown at top

### 7.7 Status Coloring
- Models implement `state_color()` or `status_color()` methods
- Return `ratatui::style::Color` using `THEME` constants
- Consistent visual feedback: Green=running, Yellow=pending, Red=stopped

### 7.8 Auto-Loading Details
- S3 bucket details fetched automatically after listing
- Rate limiting controlled by `AppConfig` (concurrency, delay)
- UI shows loading indicator until complete

### 7.9 Centralized Configuration
- `AppConfig` struct in `src/config.rs`
- Defines magic numbers: tick rate, API limits, UI layout percentages
- Initialized in `App::new()` and accessed via `self.config`

### 7.10 Navigation Helpers (`TableStateExt`)
- Extension trait for `ratatui::widgets::TableState` in `src/app/navigation.rs`
- Methods: `nav_up(len)`, `nav_down(len)`, `nav_first()`, `nav_last()`, `clamp_selection(len)`
- All service states use these for consistent list navigation with wrap-around
- Standalone functions also available: `nav_up()`, `nav_down()` for custom index management

### 7.11 Modal State Components
- Complex modals extracted into dedicated state structs in `src/app/ecs_modals.rs`
- Pattern: Encapsulate modal fields + behavior methods (init, reset, navigation)
- Example: `ServiceEditorState`, `TaskDefSelectorState` for ECS
- Benefits: Reduced parent struct size, improved testability, reusable patterns

### 7.12 Standardized Async Task Pattern
All action handlers should follow this pattern:
```rust
// CORRECT: Sync function that spawns a tracked task
pub(super) fn handle_some_action(&mut self, ..., event_tx: EventSender) {
    let Some(clients) = &self.aws_clients else { return; };
    self.loading = true;
    
    let client = clients.service.clone();
    let handle = tokio::spawn(async move {
        // ... async operation ...
        tx.send(Event::Aws(AwsEvent::...)).await.ok();
    });
    
    self.tasks.spawn(task_keys::SERVICE_ACTION, handle);  // TRACKED!
}
```

Key requirements:
1. Function is sync (not async)
2. Task is spawned with `tokio::spawn`
3. Task handle is tracked via `self.tasks.spawn(key, handle)`
4. Use predefined keys from `task_keys::*` module

---

## 8. Testing

Run tests with:
```bash
cargo test
```

Current test coverage:
- `app::filtered_list` - Filter and selection logic
- `app::service_state` - State struct initialization and helpers
- `app::task_manager` - Task spawn/cancel behavior
- `app::navigation` - List navigation helper functions
- `app::ecs_modals` - Modal state initialization, navigation, reset
- `utils::aws_profiles` - Config parsing, SSO detection, endpoint_url
- `models::ids` - Type-safe ID wrappers
- `event` - Channel backpressure handling
- `error` - Error type construction

---

## 9. Common Patterns

### Pattern: Safe State Access
```rust
// Always use helper methods that return Option
if let Some(instance) = self.services.ec2.selected_instance() {
    // Safe to use instance
}
```

### Pattern: Async Results
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

### Pattern: Conditional Action
```rust
// Check preconditions before creating action
if !self.services.s3.is_viewing_objects() {
    if let Some(bucket) = self.services.s3.selected_bucket() {
        return Some(Message::s3_load_objects(bucket.name.clone()));
    }
}
```

---

*Last updated: 2025-12-12*
*Target: Rust 1.75+, Ratatui 0.29, AWS SDK for Rust 1.x*
