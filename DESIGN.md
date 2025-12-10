
# AWS TUI - Rust/Ratatui Design Plan
## Project: LazyAWS (working title)

---

## 1. Project Overview

### 1.1 Goals
- Recreate Python AWS management TUI in Rust
- Keyboard-driven interface inspired by lazygit
- Support for EC2, S3, RDS, DynamoDB, Lambda, VPC, IAM, Backup, CloudTrail
- Async AWS operations with responsive UI
- Profile-based AWS credential management

### 1.2 Technology Stack
| Component | Rust Crate | Purpose |
|-----------|------------|----------|
| TUI Framework | `ratatui` (0.29) | Terminal UI rendering |
| Terminal Backend | `crossterm` (0.28) | Cross-platform terminal control |
| Async Runtime | `tokio` (1.x) | Async execution |
| AWS SDK | `aws-sdk-*` | AWS service clients |
| AWS Config | `aws-config` | Credential/region loading |
| Serialization | `serde`, `serde_json` | Data serialization |
| Error Handling | `anyhow`, `thiserror` | Error management |
| Logging | `tracing`, `tracing-subscriber` | Structured logging |
| CLI Args | `clap` (4.x) | Command-line argument parsing |
| Time | `chrono` | Date/time handling |

---

## 2. Project Structure

```
lazy-aws/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point, terminal setup
│   ├── app.rs                  # Main App struct and state machine
│   ├── config.rs               # Configuration and CLI args
│   ├── event.rs                # Event handling (keyboard, async)
│   ├── ui/
│   │   ├── mod.rs              # UI module exports
│   │   ├── render.rs           # Main render dispatcher
│   │   ├── theme.rs            # Colors, styles, theming
│   │   ├── components/
│   │   │   ├── mod.rs
│   │   │   ├── header.rs       # Top status bar
│   │   │   ├── sidebar.rs      # Service navigation
│   │   │   ├── resource_list.rs # Generic resource table
│   │   │   ├── detail_panel.rs # Resource details view
│   │   │   ├── action_bar.rs   # Bottom keybindings help
│   │   │   ├── modal.rs        # Confirmation dialogs
│   │   │   ├── input.rs        # Text input widget
│   │   │   └── loading.rs      # Loading spinner
│   │   └── screens/
│   │       ├── mod.rs
│   │       ├── ec2.rs          # EC2 instances view
│   │       ├── s3.rs           # S3 buckets/objects view
│   │       ├── rds.rs          # RDS instances view
│   │       ├── dynamodb.rs     # DynamoDB tables view
│   │       ├── lambda.rs       # Lambda functions view
│   │       ├── vpc.rs          # VPC resources view
│   │       ├── iam.rs          # IAM resources view
│   │       ├── backup.rs       # AWS Backup view
│   │       └── cloudtrail.rs   # CloudTrail view
│   ├── aws/
│   │   ├── mod.rs              # AWS module exports
│   │   ├── client.rs           # Shared AWS client config
│   │   ├── ec2.rs              # EC2 operations
│   │   ├── s3.rs               # S3 operations
│   │   ├── rds.rs              # RDS operations
│   │   ├── dynamodb.rs         # DynamoDB operations
│   │   ├── lambda.rs           # Lambda operations
│   │   ├── vpc.rs              # VPC operations
│   │   ├── iam.rs              # IAM operations
│   │   ├── backup.rs           # AWS Backup operations
│   │   └── cloudtrail.rs       # CloudTrail operations
│   ├── models/
│   │   ├── mod.rs              # Model exports
│   │   ├── ec2.rs              # EC2 data models
│   │   ├── s3.rs               # S3 data models
│   │   ├── rds.rs              # RDS data models
│   │   ├── dynamodb.rs         # DynamoDB data models
│   │   ├── lambda.rs           # Lambda data models
│   │   ├── vpc.rs              # VPC data models
│   │   ├── iam.rs              # IAM data models
│   │   ├── backup.rs           # Backup data models
│   │   └── cloudtrail.rs       # CloudTrail data models
│   ├── actions/
│   │   ├── mod.rs              # Action definitions
│   │   └── handlers.rs         # Action execution logic
│   └── utils/
│       ├── mod.rs
│       ├── formatting.rs       # Display formatting helpers
│       └── pagination.rs       # AWS pagination helpers
└── tests/
    └── ...
```

---

## 3. Architecture Pattern: Component + Message Passing

### 3.1 Core Architecture

We'll use a hybrid of **Component Pattern** and **Elm Architecture (TEA)**:

```rust
// Core message types
pub enum Message {
    // Navigation
    NavigateToService(Service),
    NavigateBack,

    // Selection
    SelectNext,
    SelectPrevious,
    SelectItem(usize),

    // Actions
    RefreshData,
    ExecuteAction(Action),
    ConfirmAction,
    CancelAction,

    // Async results
    DataLoaded(ServiceData),
    ActionCompleted(ActionResult),
    Error(String),

    // UI
    ToggleDetailPanel,
    ShowHelp,
    Quit,
}

pub enum Service {
    EC2,
    S3,
    RDS,
    DynamoDB,
    Lambda,
    VPC,
    IAM,
    Backup,
    CloudTrail,
}
```

### 3.2 State Machine

```rust
pub struct App {
    // AWS Configuration
    pub aws_config: AwsConfig,
    pub profile: Option<String>,
    pub region: String,

    // Navigation State
    pub current_service: Service,
    pub screen_stack: Vec<Screen>,

    // UI State
    pub sidebar_state: SidebarState,
    pub list_state: TableState,
    pub detail_visible: bool,
    pub modal: Option<Modal>,
    pub input_mode: InputMode,

    // Data State (per service)
    pub ec2_state: Ec2State,
    pub s3_state: S3State,
    pub rds_state: RdsState,
    // ... other services

    // Async State
    pub loading: bool,
    pub last_error: Option<String>,

    // Control
    pub should_quit: bool,
}

pub enum InputMode {
    Normal,
    Input,
    Confirm,
}

pub enum Screen {
    ServiceList,
    ResourceList(Service),
    ResourceDetail(Service, String), // Service + Resource ID
    S3Objects(String),               // Bucket name
    IamPolicies(String),             // Role/User ARN
}
```

---

## 4. Cargo.toml Dependencies

```toml
[package]
name = "lazy-aws"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[dependencies]
# TUI
ratatui = { version = "0.29", features = ["crossterm", "all-widgets"] }
crossterm = "0.28"

# Async
tokio = { version = "1", features = ["full"] }

# AWS SDK (add services as needed)
aws-config = "1.5"
aws-sdk-ec2 = "1.82"
aws-sdk-s3 = "1.65"
aws-sdk-rds = "1.99"
aws-sdk-dynamodb = "1.54"
aws-sdk-lambda = "1.59"
aws-sdk-iam = "1.51"
aws-sdk-backup = "1.56"
aws-sdk-cloudtrail = "1.55"
aws-smithy-types = "1.2"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# CLI
clap = { version = "4", features = ["derive"] }

# Utilities
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
unicode-width = "0.2"
```

---

## 5. Key Module Designs

### 5.1 Event System (src/event.rs)

```rust
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum Event {
    Key(KeyEvent),
    Tick,
    Aws(AwsEvent),
}

pub enum AwsEvent {
    Ec2InstancesLoaded(Vec<Ec2Instance>),
    S3BucketsLoaded(Vec<S3Bucket>),
    ActionCompleted(ActionResult),
    Error(String),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        // Spawn input handling task
        tokio::spawn(async move {
            let tick_rate = Duration::from_millis(tick_rate);
            loop {
                if event::poll(tick_rate).unwrap() {
                    if let CrosstermEvent::Key(key) = event::read().unwrap() {
                        event_tx.send(Event::Key(key)).unwrap();
                    }
                }
                event_tx.send(Event::Tick).ok();
            }
        });

        Self { rx, _tx: tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self._tx.clone()
    }
}
```

### 5.2 AWS Client Layer (src/aws/client.rs)

```rust
use aws_config::{BehaviorVersion, Region};
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_s3::Client as S3Client;
// ... other clients

pub struct AwsClients {
    pub ec2: Ec2Client,
    pub s3: S3Client,
    pub rds: RdsClient,
    // ... other clients
}

impl AwsClients {
    pub async fn new(profile: Option<&str>, region: &str) -> anyhow::Result<Self> {
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(region.to_string()));

        if let Some(profile_name) = profile {
            config_loader = config_loader.profile_name(profile_name);
        }

        let config = config_loader.load().await;

        Ok(Self {
            ec2: Ec2Client::new(&config),
            s3: S3Client::new(&config),
            rds: RdsClient::new(&config),
            // ...
        })
    }
}
```

### 5.3 Example Service Module (src/aws/ec2.rs)

```rust
use crate::models::ec2::{Ec2Instance, InstanceState};
use aws_sdk_ec2::Client;

pub struct Ec2Service {
    client: Client,
}

impl Ec2Service {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_instances(&self) -> anyhow::Result<Vec<Ec2Instance>> {
        let response = self.client
            .describe_instances()
            .send()
            .await?;

        let instances = response
            .reservations()
            .iter()
            .flat_map(|r| r.instances())
            .map(|i| Ec2Instance::from_aws(i))
            .collect();

        Ok(instances)
    }

    pub async fn start_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .start_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }

    pub async fn stop_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .stop_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }

    pub async fn reboot_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .reboot_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }

    pub async fn terminate_instance(&self, instance_id: &str) -> anyhow::Result<()> {
        self.client
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }
}
```

### 5.4 Component Trait (src/ui/components/mod.rs)

```rust
use ratatui::{Frame, layout::Rect};
use crossterm::event::KeyEvent;
use crate::app::Message;

pub trait Component {
    /// Render the component to the frame
    fn render(&self, frame: &mut Frame, area: Rect);

    /// Handle keyboard input, return optional message
    fn handle_key(&mut self, key: KeyEvent) -> Option<Message>;

    /// Check if component can handle input (is focused)
    fn is_focused(&self) -> bool { false }
}

pub trait StatefulComponent {
    type State;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State);
    fn handle_key(&mut self, key: KeyEvent, state: &mut Self::State) -> Option<Message>;
}
```

---

## 6. UI Layout Design

### 6.1 Main Layout Structure

```
┌─────────────────────────────────────────────────────────────────┐
│ [Header: Profile | Region | Service Name]              HH:MM:SS │ <- 3 rows
├──────────────┬──────────────────────────────────────────────────┤
│              │                                                  │
│  [Sidebar]   │  [Main Content Area]                             │
│              │                                                  │
│  ● EC2       │  ┌─────────────────────────────────────────────┐ │
│    S3        │  │ Instance ID │ Name     │ State   │ Type     │ │
│    RDS       │  ├─────────────┼──────────┼─────────┼──────────┤ │
│    DynamoDB  │  │ i-abc123    │ WebSrv   │ running │ t3.micro │ │
│    Lambda    │  │ i-def456    │ Database │ stopped │ r5.large │ │
│    VPC       │  │ ...         │ ...      │ ...     │ ...      │ │
│    IAM       │  └─────────────────────────────────────────────┘ │
│    Backup    │                                                  │
│    CloudTr   │  [Detail Panel - toggleable]                     │
│              │                                                  │
├──────────────┴──────────────────────────────────────────────────┤
│ [Action Bar: j/k:Navigate  Enter:Select  s:Start  S:Stop  ?:Help│ <- 2 rows
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Layout Code

```rust
fn render_main_layout(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(10),    // Body
        Constraint::Length(2),  // Action bar
    ])
    .split(frame.area());

    render_header(frame, chunks[0], app);
    render_body(frame, chunks[1], app);
    render_action_bar(frame, chunks[2], app);

    // Render modal on top if present
    if let Some(modal) = &app.modal {
        render_modal(frame, modal);
    }
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::horizontal([
        Constraint::Length(16), // Sidebar
        Constraint::Min(40),    // Main content
    ])
    .split(area);

    render_sidebar(frame, chunks[0], app);

    // Split main area for list + details if detail panel visible
    if app.detail_visible {
        let main_chunks = Layout::vertical([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(chunks[1]);

        render_resource_list(frame, main_chunks[0], app);
        render_detail_panel(frame, main_chunks[1], app);
    } else {
        render_resource_list(frame, chunks[1], app);
    }
}
```

---

## 7. Keybinding Scheme

### 7.1 Global Keys
| Key | Action |
|-----|--------|
| `q` / `Ctrl+c` | Quit application |
| `?` | Show help modal |
| `Tab` | Cycle focus (sidebar ↔ main) |
| `r` | Refresh current view |
| `Esc` | Cancel / Close modal / Go back |
| `1-9` | Quick switch to service (1=EC2, 2=S3, etc.) |

### 7.2 Navigation Keys
| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `g` / `Home` | Go to first item |
| `G` / `End` | Go to last item |
| `Enter` | Select / Drill down |
| `Backspace` | Go back / Up one level |
| `d` | Toggle detail panel |

### 7.3 Service-Specific Actions

#### EC2
| Key | Action |
|-----|--------|
| `s` | Start instance |
| `S` | Stop instance |
| `R` | Reboot instance |
| `T` | Terminate instance (with confirm) |

#### S3  
| Key | Action |
|-----|--------|
| `Enter` | Browse bucket / Download object |
| `E` | Empty bucket (with confirm) |
| `D` | Delete bucket (with confirm) |

#### RDS
| Key | Action |
|-----|--------|
| `s` | Start instance |
| `S` | Stop instance |
| `R` | Reboot instance |
| `n` | Create snapshot |

---

## 8. Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- [ ] Project setup with Cargo.toml
- [ ] Terminal setup/restore with panic hooks
- [ ] Basic App state machine
- [ ] Event handling system (keyboard + async)
- [ ] Main layout (header, sidebar, content, action bar)
- [ ] Navigation between services
- [ ] AWS client initialization with profile support
- [ ] CLI argument parsing (--profile, --region)

### Phase 2: Core UI Components (Week 2-3)
- [ ] Sidebar component with service selection
- [ ] Generic resource table component
- [ ] Detail panel component
- [ ] Modal dialog component
- [ ] Loading indicator
- [ ] Error display
- [ ] Help screen

### Phase 3: EC2 Implementation (Week 3-4)
- [ ] EC2 data models
- [ ] EC2 list instances
- [ ] EC2 instance details view
- [ ] EC2 actions (start, stop, reboot, terminate)
- [ ] Confirmation dialogs for destructive actions

### Phase 4: S3 Implementation (Week 4-5)
- [ ] S3 data models
- [ ] S3 list buckets
- [ ] S3 bucket details
- [ ] S3 object browser (drill-down)
- [ ] S3 actions (empty bucket, delete bucket)

### Phase 5: Additional Services (Week 5-8)
- [ ] RDS (list, start/stop, snapshots)
- [ ] DynamoDB (list tables, scan, delete)
- [ ] Lambda (list, invoke)
- [ ] VPC (read-only views)
- [ ] IAM (list users/roles/policies)
- [ ] Backup (list plans/jobs, start backup)
- [ ] CloudTrail (list trails, events)

### Phase 6: Polish (Week 8-9)
- [ ] Theming system
- [ ] Configuration file support
- [ ] Pagination for large result sets
- [ ] Search/filter functionality
- [ ] Keyboard shortcut customization
- [ ] Error handling improvements
- [ ] Logging and debugging

### Phase 7: Testing & Documentation (Week 9-10)
- [ ] Unit tests for business logic
- [ ] Integration tests with localstack
- [ ] README and usage documentation
- [ ] Release packaging

---

## 9. Key Design Decisions

### 9.1 Why Component Pattern + TEA?
- **Component Pattern**: Encapsulates rendering and input handling per UI element
- **TEA (Messages)**: Provides predictable state updates and action dispatching
- **Hybrid**: Best of both - modular components with centralized state management

### 9.2 Async Strategy
- All AWS operations run in background tasks via `tokio::spawn`
- Results sent back via `mpsc` channels as `AwsEvent` messages
- UI remains responsive during network operations
- Loading indicators shown during pending operations

### 9.3 Error Handling
- Use `anyhow::Result` for application-level errors
- Use `thiserror` for domain-specific error types
- Display errors in UI with dismissable notifications
- Log detailed errors for debugging

### 9.4 State Ownership
- Single `App` struct owns all application state
- Components receive immutable references for rendering
- State mutations happen only in `App::update(Message)` method

---

## 10. Example: Main Application Loop

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::init();

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Initialize AWS clients
    let aws_clients = AwsClients::new(args.profile.as_deref(), &args.region).await?;

    // Create app state
    let mut app = App::new(aws_clients, args);

    // Create event handler
    let mut events = EventHandler::new(250); // 250ms tick rate
    let event_tx = events.sender();

    // Initial data load
    app.load_initial_data(event_tx.clone()).await;

    // Main loop
    while !app.should_quit {
        // Render
        terminal.draw(|frame| app.render(frame))?;

        // Handle events
        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) => {
                    if let Some(msg) = app.handle_key(key) {
                        app.update(msg, event_tx.clone()).await;
                    }
                }
                Event::Tick => {
                    app.on_tick();
                }
                Event::Aws(aws_event) => {
                    app.handle_aws_event(aws_event);
                }
            }
        }
    }

    // Restore terminal
    restore_terminal(&mut terminal)?;

    Ok(())
}
```

---

*Document created: 2025-12-10*
*Target: Rust 1.75+, Ratatui 0.29, AWS SDK for Rust*
