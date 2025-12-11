# LazyAWS Architecture Documentation

This document describes the architectural patterns and design decisions in LazyAWS.

## Table of Contents
1. [Overview](#overview)
2. [Core Architecture](#core-architecture)
3. [Key Patterns](#key-patterns)
4. [Module Structure](#module-structure)
5. [Data Flow](#data-flow)
6. [Adding New Features](#adding-new-features)

## Overview

LazyAWS is built using a hybrid architecture combining:
- **Component Pattern**: UI elements are self-contained components with their own rendering and input handling
- **Elm Architecture (TEA)**: Unidirectional data flow with messages, update functions, and centralized state
- **Trait-based Polymorphism**: Service screens implement traits for consistent behavior

This architecture provides:
- **Modularity**: Each service and component is independent
- **Testability**: Components and message handlers can be tested in isolation
- **Scalability**: New services can be added without modifying core code
- **Type Safety**: Rust's type system prevents many classes of bugs

## Core Architecture

### State Management

The application state is centralized in the `App` struct, which is organized into:

```rust
pub struct App {
    // Global state
    pub should_quit: bool,
    pub current_service: Service,
    pub focus: Focus,
    pub input_mode: InputMode,
    
    // Service-specific state (consolidated)
    pub services: ServiceStates,
    
    // Task management
    pub tasks: TaskManager,
    
    // AWS clients
    pub aws_clients: Option<AwsClients>,
}
```

**Key Benefit**: Instead of having 50+ fields scattered across `App`, service-specific data is organized in `ServiceStates`:

```rust
pub struct ServiceStates {
    pub ec2: Ec2State,
    pub s3: S3State,
    pub rds: RdsState,
    // ... one state struct per service
}
```

### Message Flow (Elm Architecture)

```
User Input → KeyEvent → InputHandler → Message → Update Function → State Change → Render
```

1. **Input**: User presses a key
2. **Handler**: `App::handle_key()` or component's `handle_key()` translates it to a `Message`
3. **Update**: `App::update()` processes the message and modifies state
4. **Render**: `render()` function displays the new state

Messages are typed using enums:

```rust
pub enum Message {
    Global(GlobalMessage),     // App-wide operations
    Service(ServiceAction),    // Service-specific actions
}

pub enum ServiceAction {
    Ec2(Ec2Action),
    S3(S3Action),
    // ... per-service action enums
}
```

**Key Benefit**: Type-safe message dispatch prevents invalid operations (e.g., can't send an EC2 action to the S3 handler).

## Key Patterns

### 1. Component Trait

Components are reusable UI elements that handle their own rendering and input:

```rust
pub trait Component {
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, key: KeyEvent) -> Option<Message>;
}
```

**Usage**: Sidebar, modals, and other UI elements implement this trait.

**Example**: The sidebar handles j/k navigation internally and returns a `Navigate` message when Enter is pressed.

### 2. Screen Trait

Service screens implement the `Screen` trait for polymorphic rendering:

```rust
pub trait Screen {
    fn render(&self, frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App);
}
```

**Key Benefit**: The main render loop uses `get_screen(service).render(...)` instead of a massive match statement.

### 3. InputHandler Trait (Optional)

Screens can optionally implement `InputHandler` for delegated input processing:

```rust
pub trait InputHandler {
    fn handle_key(&self, key: KeyEvent, app: &mut App) -> InputResult;
}
```

**Key Benefit**: Reduces complexity in the main input handler by delegating service-specific logic to the appropriate screen.

### 4. ServiceStates

Service state is organized into per-service structs:

```rust
pub struct Ec2State {
    pub instances: Vec<Ec2Instance>,
    pub list_state: TableState,
}

impl Ec2State {
    pub fn selected_instance(&self) -> Option<&Ec2Instance> { /* ... */ }
    pub fn selected_instance_id(&self) -> Option<String> { /* ... */ }
}
```

**Key Benefit**: 
- Encapsulates service data and logic
- Provides type-safe accessors
- Easier to reason about than scattered fields

### 5. TaskManager

Async operations are tracked and cancellable:

```rust
pub struct TaskManager {
    active_tasks: HashMap<String, JoinHandle<()>>,
}

impl TaskManager {
    pub fn spawn(&mut self, key: String, handle: JoinHandle<()>) { /* ... */ }
    pub fn cancel(&mut self, key: &str) { /* ... */ }
}
```

**Key Benefit**:
- Prevents race conditions (duplicate fetches)
- Allows cancellation when switching services
- Clear lifecycle management

### 6. FilteredList

Efficient filtering with caching:

```rust
pub struct FilteredList<T> {
    items: Vec<T>,
    filter: String,
    filtered_indices: Vec<usize>,  // Cached!
    cache_dirty: bool,
}
```

**Key Benefit**:
- O(n) filter computation once
- O(1) access during rendering
- Handles thousands of items efficiently

### 7. Error Handling

Structured error types using `thiserror`:

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("AWS {service} error: {message}")]
    AwsApi { service: String, message: String, source: Option<...> },
    
    #[error("Resource not found: {resource_type} '{resource_id}'")]
    NotFound { resource_type: String, resource_id: String },
    // ...
}
```

**Key Benefit**:
- Type-safe error handling
- Context-rich error messages
- Easy debugging with source tracking

### 8. Centralized Configuration

All magic numbers in one place:

```rust
pub struct AppConfig {
    pub tick_rate_ms: u64,
    pub max_s3_objects: usize,
    pub sidebar_width: u16,
    // ... all configurable values
}
```

**Key Benefit**:
- Easy to customize without recompiling (can load from env vars or files)
- Consistent defaults
- Runtime validation

## Module Structure

```
src/
├── main.rs                 # Entry point, event loop
├── config/                 # Configuration module
│   ├── mod.rs
│   ├── args.rs            # CLI argument parsing
│   └── configuration.rs   # AppConfig with defaults
├── app/                   # Application logic
│   ├── mod.rs
│   ├── state.rs           # App struct
│   ├── messages.rs        # Message enums
│   ├── service_state.rs   # Per-service state structs
│   ├── update.rs          # Message handlers
│   ├── input.rs           # Input handling
│   ├── events.rs          # AWS event handling
│   ├── task_manager.rs    # Async task tracking
│   └── filtered_list.rs   # Filtered list with caching
├── ui/                    # UI rendering
│   ├── mod.rs
│   ├── render.rs          # Main render dispatcher
│   ├── screen.rs          # Screen & InputHandler traits
│   ├── theme.rs           # Centralized colors/styles
│   ├── components/        # Reusable UI components
│   │   ├── sidebar.rs
│   │   ├── modal.rs
│   │   └── ...
│   └── screens/           # Service-specific screens
│       ├── mod.rs
│       ├── ec2.rs
│       ├── s3.rs
│       └── ...
├── aws/                   # AWS SDK wrappers
│   ├── client.rs
│   ├── ec2.rs
│   └── ...
├── models/                # Data structures
│   ├── ec2.rs
│   ├── s3.rs
│   └── ...
├── utils/                 # Utility functions
├── error.rs              # Error types
└── event.rs              # Event definitions
```

## Data Flow

### Service Navigation Flow

```
User presses '1' 
  → Sidebar receives KeyEvent
  → Returns Message::Navigate(Service::EC2)
  → App::update() handles Navigate message
  → Sets app.current_service = Service::EC2
  → Spawns task to fetch EC2 instances
  → Render loop calls get_screen(EC2).render()
  → EC2 screen displays instances
```

### Action Confirmation Flow

```
User presses 's' (start instance)
  → handle_ec2_input() returns InputResult::Action(Message::ec2_start(...))
  → App::request_action() checks read_only flag
  → Sets pending_action and shows confirmation modal
  → User presses 'y'
  → Returns Message::ConfirmAction
  → App::update() executes pending action
  → Spawns async task to start instance
  → Updates action log on completion
```

### Async Operation Flow

```
Message::ec2_start(id)
  → App::update() dispatches to action handler
  → Spawns tokio task with TaskManager
  → Task calls AWS SDK
  → Sends AwsEvent::ActionCompleted via channel
  → Event handler updates app state
  → Adds message to action log
  → Render shows success/error
```

## Adding New Features

### Adding a New AWS Service

1. **Update `Service` enum** (`src/app/messages.rs`):
   ```rust
   pub enum Service {
       // ... existing
       NewService,
   }
   ```

2. **Create data models** (`src/models/newservice.rs`):
   ```rust
   pub struct NewServiceResource {
       pub id: String,
       pub name: Option<String>,
       // ...
   }
   ```

3. **Add service state** (`src/app/service_state.rs`):
   ```rust
   pub struct NewServiceState {
       pub resources: Vec<NewServiceResource>,
       pub list_state: TableState,
   }
   ```

4. **Create AWS SDK wrapper** (`src/aws/newservice.rs`):
   ```rust
   pub async fn list_resources(client: &Client) -> Result<Vec<NewServiceResource>> {
       // ...
   }
   ```

5. **Create UI screen** (`src/ui/screens/newservice.rs`):
   ```rust
   pub fn render(frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
       // ...
   }
   ```

6. **Implement Screen trait** (`src/ui/screens/mod.rs`):
   ```rust
   pub struct NewServiceScreen;
   
   impl Screen for NewServiceScreen {
       fn render(&self, frame: &mut Frame, list_area: Rect, detail_area: Option<Rect>, app: &mut App) {
           newservice::render(frame, list_area, detail_area, app);
       }
   }
   ```

7. **Add message handlers** (`src/app/messages.rs`, `src/app/update.rs`)

8. **Add input handlers** (`src/app/input.rs`)

### Adding a New Component

1. Create file in `src/ui/components/mycomponent.rs`
2. Implement `Component` trait
3. Export from `src/ui/components/mod.rs`
4. Use in render functions

### Adding a New Action

1. **Define action enum** in `messages.rs`:
   ```rust
   pub enum Ec2Action {
       // ... existing
       Terminate(String),
   }
   ```

2. **Add constructor** to `Message`:
   ```rust
   pub fn ec2_terminate(instance_id: String) -> Self {
       Message::Service(ServiceAction::Ec2(Ec2Action::Terminate(instance_id)))
   }
   ```

3. **Handle in input.rs**:
   ```rust
   KeyCode::Char('T') => {
       if let Some(id) = app.services.ec2.selected_instance_id() {
           return InputResult::Action(Message::ec2_terminate(id));
       }
   }
   ```

4. **Handle in update.rs**:
   ```rust
   Ec2Action::Terminate(id) => {
       // Spawn async task
   }
   ```

5. **Add AWS SDK call** in `src/aws/ec2.rs`

## Best Practices

1. **Use typed IDs**: Create newtype wrappers for IDs to prevent mixing (e.g., `InstanceId(String)`)

2. **Keep components small**: Each component should have a single responsibility

3. **Async operations**: Always use TaskManager to spawn tasks with a unique key

4. **Error handling**: Use `AppError` types, provide context, and display user-friendly messages

5. **Testing**: Write unit tests for message constructors, state accessors, and error handling

6. **Configuration**: Add new magic numbers to `AppConfig`, not as hardcoded constants

7. **Documentation**: Update this document when adding new architectural patterns

## Performance Considerations

- **Filtering**: Use `FilteredList` for lists with >100 items to cache filter results
- **Rendering**: Ratatui is fast; prefer simplicity over premature optimization
- **Async**: Spawn tasks in background; never block the UI thread
- **Rate limiting**: Use semaphores and delays for AWS API rate limits (see S3 details loading)

## Security Considerations

- **Read-only mode**: Always check `app.read_only` before destructive actions
- **Confirmation modals**: Require explicit 'y' confirmation for destructive actions
- **Error messages**: Don't leak sensitive information (AWS account IDs, keys, etc.)
- **AWS credentials**: Never log or display AWS credentials

## Future Improvements

Potential architectural enhancements:

1. **Plugin system**: Allow loading service modules dynamically
2. **Customizable keybindings**: Load from config file
3. **Theme system**: Support multiple color schemes
4. **Offline mode**: Cache AWS data locally
5. **Multi-region view**: Show resources across regions simultaneously
6. **Command palette**: Fuzzy search for all actions
7. **Session persistence**: Save/restore app state between runs

---

For questions or suggestions, please open an issue on GitHub.
