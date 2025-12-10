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
2.  **Update `Service` Enum**: Add the new service variant in `src/app.rs`.
3.  **Create Service Module**: Create `src/aws/<service>.rs` for SDK interactions.
4.  **Create Data Models**: Create `src/models/<service>.rs` for internal representations.
5.  **Create UI Screen**: Create `src/ui/screens/<service>.rs` for the service view.
6.  **Update `App` State**: Add state storage for the new service in `App` struct.
7.  **Register Navigation**: Update sidebar and input handling to allow navigating to the new service.

### 4.2 Implementing an Action (e.g., Start Instance)
1.  **Define Action**: Add variant to `Action` enum (if generic) or specific `Message`.
2.  **Update Handler**: Implement the logic in `src/actions/handlers.rs` or the service module.
3.  **Trigger Async Task**: Spawn a tokio task to execute the action and send a result message back.
4.  **Handle Result**: Update `App::update` to handle success/failure messages (e.g., refresh list, show error).

### 4.3 Implementing Hierarchical Navigation (e.g., S3 Buckets -> Objects)
1.  **Update App State**: Add state for the child view (e.g., `current_bucket`, `s3_objects`).
2.  **Add Navigation Messages**: Add messages to enter/leave the child view (e.g., `LoadS3Objects`, `LeaveS3Bucket`).
3.  **Update Key Handler**: In `handle_key`, check the state (e.g., `current_bucket.is_some()`) to determine which key bindings apply (drill-down vs. back).
4.  **Update Renderer**: In the screen's `render` function, conditionally render the parent or child view.
5.  **Update Action Bar**: Ensure the action bar reflects the current context (e.g., "Esc: Back").

## 5. Directory Structure Reference
```
lazy-aws/
├── src/
│   ├── main.rs                 # Entry point
│   ├── app.rs                  # State machine
│   ├── config.rs               # Configuration and CLI args
│   ├── event.rs                # Event loop
│   ├── ui/                     # Rendering logic
│   │   ├── mod.rs
│   │   ├── render.rs           # Main render dispatcher
│   │   ├── theme.rs            # Centralized theming
│   │   ├── components/         # Reusable widgets (sidebar, modal, etc.)
│   │   └── screens/            # Service-specific views
│   ├── aws/                    # AWS SDK wrappers
│   ├── models/                 # Data structures
│   └── actions/                # Command handlers
```

## 6. Recent Features & Patterns

### 6.1 Read-Only Mode
- **Feature**: Prevent accidental modification of resources.
- **Implementation**:
  - `App` struct has a `read_only: bool` field.
  - CLI argument `--read-only` enables it.
  - Header displays a yellow "READ-ONLY" warning.
  - Destructive actions (like `StartInstance`) check this flag in `handle_key` or `request_action` helper.
  - If `read_only` is true, an error message "Read-only mode: Action not allowed" is shown instead of prompting for confirmation.

### 6.2 Confirmation Modals
- **Feature**: Require user confirmation for destructive actions.
- **Implementation**:
  - `App` has `pending_action: Option<Message>` and `show_confirmation: bool`.
  - When an action is requested (and not read-only), `pending_action` is set and `show_confirmation` becomes true.
  - `render_confirmation_modal` draws the dialog overlay.
  - `handle_key` intercepts keys when `show_confirmation` is true (Enter to confirm, Esc to cancel).

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

