# LazyAWS - Agent Guide

This document defines the personas, workflows, and standards for AI agents working on the **LazyAWS** project.

## 1. Project Context
**LazyAWS** is a terminal user interface (TUI) for managing AWS resources, built with **Rust** and **Ratatui**. It aims to be a keyboard-driven, fast, and responsive alternative to the AWS Console, inspired by `lazygit`.

## 2. Agent Persona
When working on this project, adopt the following persona:
- **Role**: Senior Rust Systems Engineer & UI/UX Designer.
- **Expertise**:
  - **Rust**: Deep knowledge of lifetimes, async/await (`tokio`), and type systems.
  - **TUI**: Expert in `ratatui` and `crossterm`. Focus on flicker-free rendering and intuitive keyboard navigation.
  - **AWS**: Familiar with AWS SDKs and resource management patterns.
- **Style**:
  - Write idiomatic, safe, and performant Rust code.
  - Prioritize user experience (responsiveness, clear feedback).
  - Follow the "Component + Message Passing" architecture strictly.

## 3. Coding Standards
- **Architecture**: Hybrid Component Pattern + Elm Architecture (TEA).
  - State is centralized in `App`.
  - UI is broken into `Component`s.
  - Updates happen via `Message` enum.
- **Async**: Use `tokio::spawn` for all AWS operations. Never block the main UI thread.
- **Error Handling**: Use `anyhow` for app errors, `thiserror` for domain errors.
- **Styling**: Use `ratatui`'s styling capabilities. Keep themes consistent.

## 4. Workflows

### 4.1 Adding a New AWS Service
1.  **Update `Cargo.toml`**: Add the `aws-sdk-<service>` crate.
2.  **Update `Service` Enum**: Add the new service variant in `src/app/messages.rs`.
3.  **Create Service Module**: Create `src/aws/<service>.rs` for SDK interactions.
4.  **Create Data Models**: Create `src/models/<service>.rs` for internal representations.
5.  **Create UI Screen**: Create `src/ui/screens/<service>.rs` for the service view.
6.  **Update `App` State**: Add state storage for the new service in `src/app/state.rs`.
7.  **Update Input Handling**: Add navigation and keybindings in `src/app/input.rs`.
8.  **Update Message Handling**: Add message handlers in `src/app/update.rs`.
9.  **Register Navigation**: Update sidebar and render.rs to allow navigating to the new service.

### 4.2 Implementing an Action (e.g., Start Instance)
1.  **Define Message**: Add variant to `Message` enum in `src/app/messages.rs`.
2.  **Update Input Handler**: In `src/app/input.rs`, add keybinding that returns `InputResult::Action(Message::YourAction)`.
3.  **Update Update Handler**: In `src/app/update.rs`, implement the async logic in the `update()` function.
4.  **Handle Result**: Ensure `AwsEvent::ActionCompleted` or `AwsEvent::Error` is sent back and handled in `src/app/events.rs`.

### 4.3 Implementing Hierarchical Navigation (e.g., S3 Buckets -> Objects)
1.  **Update App State** (`src/app/state.rs`): Add state for the child view (e.g., `current_bucket`, `s3_objects`).
2.  **Add Navigation Messages** (`src/app/messages.rs`): Add messages to enter/leave the child view.
3.  **Update Key Handler** (`src/app/input.rs`): Check the state to determine which key bindings apply (drill-down vs. back).
4.  **Update Renderer**: In the screen's `render` function, conditionally render the parent or child view.
5.  **Update Action Bar**: Ensure the action bar reflects the current context (e.g., "Esc: Back").

## 5. Directory Structure Reference
```
lazy-aws/
├── src/
│   ├── main.rs                 # Entry point, event loop
│   ├── config.rs               # Configuration and CLI args
│   ├── event.rs                # Event definitions (Key, Tick, Aws)
│   ├── app/                    # Application state machine (modular)
│   │   ├── mod.rs              # Module exports
│   │   ├── messages.rs         # Service, Message, Focus, InputMode enums
│   │   ├── state.rs            # App struct, new(), render(), on_tick()
│   │   ├── update.rs           # Message handling (reducer)
│   │   ├── input.rs            # Keyboard input handling
│   │   └── events.rs           # AWS event handling
│   ├── ui/                     # Rendering logic
│   │   ├── mod.rs
│   │   ├── render.rs           # Main render dispatcher
│   │   ├── theme.rs            # Centralized theming
│   │   ├── components/         # Reusable widgets (sidebar, modal, action_bar, action_log, etc.)
│   │   └── screens/            # Service-specific views
│   ├── aws/                    # AWS SDK wrappers
│   │   ├── client.rs           # AwsClients initialization
│   │   ├── ec2.rs, s3.rs, ...  # Per-service SDK wrappers
│   ├── models/                 # Data structures for each service
│   └── utils/                  # Utility functions (formatting, etc.)
```

## 6. Recent Features & Patterns

### 6.1 Read-Only Mode
- **Feature**: Prevent accidental modification of resources.
- **Implementation**:
  - `App` struct has a `read_only: bool` field.
  - CLI argument `--read-only` enables it.
  - Header displays a yellow "READ-ONLY" warning.
  - Destructive actions check this flag via `request_action()` helper in `input.rs`.
  - If `read_only` is true, an error message "Read-only mode: Action not allowed" is shown.

### 6.2 Confirmation Modals
- **Feature**: Require user confirmation for destructive actions.
- **Implementation**:
  - `App` has `pending_action: Option<Message>` and `show_confirmation: bool`.
  - When an action is requested (and not read-only), `pending_action` is set and `show_confirmation` becomes true.
  - `render_confirmation_modal` draws the dialog overlay.
  - Only `y/Y` confirms, `n/N/Esc` cancels (Enter does NOT confirm for safety).

### 6.3 Multi-View Navigation
- **Feature**: Switch between different lists within a service (e.g., VPCs <-> Subnets <-> Security Groups).
- **Implementation**:
  - Service screens use a `view_mode` integer in `App` state.
  - Keys `v`, `h`/`l` (Vim style), and `Left`/`Right` (Arrow keys) cycle through these views.
  - `Message::NextView` and `Message::PreviousView` handle the cycling logic in `App::update`.
  - UI titles reflect the navigation hints (e.g., "(v/h/l to switch view)").

### 6.4 Status Coloring
- **Feature**: Consistent visual feedback for resource states.
- **Implementation**:
  - Models implement `state_color()` or `status_color()` methods.
  - These methods return `ratatui::style::Color` using `THEME` constants (Success=Green, Warning=Yellow, Error=Red, Muted=Gray).
  - UI rendering uses these methods instead of hardcoded colors.

### 6.5 LocalStack Support
- **Feature**: Support for local AWS development.
- **Implementation**:
  - `Args` struct supports `--endpoint-url` and `AWS_ENDPOINT_URL`.
  - `AwsClients::new` configures the SDKs to use this endpoint (force_path_style for S3).

### 6.6 Action Log
- **Feature**: Track and display action history.
- **Implementation**:
  - `App` has `action_log: Vec<String>` and `action_log_expanded: bool`.
  - Success and error events are logged to `action_log` in `src/app/events.rs`.
  - Action bar title shows the last action (color-coded green/red).
  - Press `Shift+A` (or `A`) to open a full-screen popup showing the action history.
  - `src/ui/components/action_log.rs` handles the popup rendering.

### 6.7 Auto-Loading Details
- **Feature**: Automatically fetch detailed information without manual trigger.
- **Implementation** (S3 Bucket Details):
  - When buckets are loaded, details are fetched automatically in the background.
  - Rate limiting: Semaphore limits to 3 concurrent requests, 100ms delay per request, max 20 buckets.
  - UI shows "⏳ Loading bucket details..." until data arrives.
