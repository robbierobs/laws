# Codebase Refactoring Opportunities

## Analysis Summary
A deep analysis of the codebase reveals several opportunities for refactoring, optimization, and code quality improvements.

## High Priority Issues

### 1. Large Enum Variants (Clippy Warning)
**Location**: `src/event.rs`
**Issue**: `Event::Aws(AwsEvent)` and `AwsEvent` variants have large size differences.
**Impact**: Memory inefficiency - every `Event` instance allocates space for the largest variant.

**Solution**: Box the large variant:
```rust
// Before
pub enum Event {
    Aws(AwsEvent),
    ...
}

// After
pub enum Event {
    Aws(Box<AwsEvent>),
    ...
}
```

### 2. Monolithic Input Handler (`src/app/input.rs` - 1042 lines)
**Issue**: The `handle_key` function is extremely large (700+ lines) with deeply nested match statements.
**Impact**: Hard to maintain, test, and reason about.

**Solution**: Extract modal-specific input handlers into separate modules:
- `src/app/input/modals/s3_viewer.rs`
- `src/app/input/modals/bucket_creation.rs`
- `src/app/input/modals/profile_switcher.rs`
- `src/app/input/modals/global_search.rs`
- `src/app/input/modals/ecs_editor.rs`

Pattern:
```rust
// Instead of inline match blocks:
if self.services.s3.show_object_viewer {
    return self.handle_s3_viewer_input(key);
}
```

### 3. Repetitive Delete Handlers
**Location**: `src/app/update/iam.rs`
**Issue**: `handle_delete_iam_user`, `handle_delete_iam_role`, `handle_delete_iam_policy` have nearly identical structure.

**Solution**: Create a generic delete handler:
```rust
fn handle_delete_resource<S, F, M>(
    &mut self,
    resource_name: String,
    resource_type: &str,
    task_key: &str,
    service_fn: S,
    delete_fn: F,
    event_tx: EventSender,
) where
    S: Fn(&AwsClients) -> M + Send + 'static,
    F: Fn(&M, &str) -> impl Future<Output = Result<(), Error>> + Send + 'static,
    M: Send + 'static,
{...}
```

### 4. Repetitive EC2 Instance Actions
**Location**: `src/aws/ec2.rs`
**Issue**: `start_instance`, `stop_instance`, `reboot_instance`, `terminate_instance` are nearly identical.

**Solution**: Consolidate into a single method:
```rust
pub async fn execute_action(&self, action: InstanceAction, instance_id: &str) -> AppResult<()>
```

### 5. ViewMode Boilerplate Across Services
**Issue**: Each `ViewMode` enum (VpcViewMode, IamViewMode, BackupViewMode, etc.) implements similar methods.
**Status**: Already using `ViewMode` trait with default implementations. ✓
**Improvement**: Macro to derive common ViewMode implementations:
```rust
#[derive(ViewMode)]
#[view_mode(main_tabs = [Vpcs, Subnets, SecurityGroups])]
pub enum VpcViewMode { ... }
```

---

## Medium Priority Issues

### 6. Function With Too Many Arguments
**Location**: Multiple places including ECS update functions
**Clippy Warning**: Functions have 8+ arguments.

**Solution**: Use builder pattern or parameter structs:
```rust
pub struct EcsServiceUpdateParams {
    pub cluster_arn: String,
    pub service_name: String,
    pub task_definition: Option<String>,
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub force_new_deployment: bool,
}
```

### 7. Messages Module Size (`src/app/messages.rs` - 1191 lines)
**Issue**: Contains all message types, action enums, view modes, and factory functions.

**Solution**: Split into:
- `src/app/messages/mod.rs` - Core Message enum
- `src/app/messages/service.rs` - Service enum
- `src/app/messages/actions.rs` - Service action enums
- `src/app/messages/view_modes.rs` - ViewMode enums
- `src/app/messages/factories.rs` - Message factory functions

### 8. Duplicate Filter Logic in Models
**Issue**: Each model type implements `matches_filter` with similar patterns.

**Solution**: Create a `Filterable` trait and macro:
```rust
pub trait Filterable {
    fn searchable_fields(&self) -> Vec<&str>;
    
    fn matches_filter(&self, filter: &str) -> bool {
        let filter = filter.to_lowercase();
        self.searchable_fields()
            .iter()
            .any(|f| f.to_lowercase().contains(&filter))
    }
}
```

### 9. Clamp-like Pattern Without Using clamp
**Location**: Navigation code with scroll offsets
**Clippy Warning**: Manual min/max bounds checking.

**Solution**: Use `.clamp()`:
```rust
// Before
let scroll = if val > max { max } else if val < min { min } else { val };

// After
let scroll = val.clamp(min, max);
```

---

## Low Priority / Style Improvements

### 10. Acronym Names in Enums
**Clippy Warning**: `VPC`, `IAM`, `ECS`, `ECR` contain fully capitalized acronyms.
**Rust Convention**: Prefer `Vpc`, `Iam`, `Ecs`, `Ecr`.

**Recommendation**: This is a style choice. The current names are clear and commonly used. Could suppress with `#[allow(clippy::upper_case_acronyms)]` or rename to match Rust convention.

### 11. Field Assignment Outside Default::default()
**Location**: State initialization code

**Solution**: Use struct update syntax:
```rust
// Before
let mut state = Default::default();
state.field = value;

// After
let state = SomeStruct {
    field: value,
    ..Default::default()
};
```

---

## Architectural Improvements

### 12. Service Handler Trait
**Current**: Each service has methods scattered across input, update, render modules.
**Proposal**: Unified `ServiceHandler` trait per service:

```rust
pub trait ServiceHandler {
    type State;
    type Action;
    
    fn handle_input(&mut self, state: &mut Self::State, key: KeyEvent) -> InputResult;
    fn handle_action(&mut self, state: &mut Self::State, action: Self::Action, tx: EventSender);
    fn render(&self, state: &Self::State, frame: &mut Frame, area: Rect);
}
```

### 13. Error Notification System
**Current**: `app.error_message: Option<String>` - single error display.
**Proposal**: Notification queue with severity levels and auto-dismiss:

```rust
pub struct Notification {
    pub message: String,
    pub severity: Severity,
    pub expires_at: Option<Instant>,
}

pub struct NotificationCenter {
    queue: VecDeque<Notification>,
}
```

---

## Implementation Notes

### Boxing AwsEvent Complexity
**Warning**: Boxing `AwsEvent` requires updating 100+ call sites across the codebase. Each `Event::Aws(AwsEvent::X)` must become `Event::Aws(Box::new(AwsEvent::X))`. Additionally, multiline cases (struct literals, format! calls) require careful handling of closing parentheses.

**Recommended Approach**: Use a script to handle all transformations at once, then manually verify. The changes are:
1. Update `src/event.rs`: `Aws(Box<AwsEvent>)` 
2. Update all creation sites: `Event::Aws(Box::new(...))`
3. Update main.rs match: `Event::Aws(aws_event) => app.handle_aws_event(*aws_event)`

---

## Implementation Priority

| Priority | Issue | Effort | Impact | Status |
|----------|-------|--------|--------|--------|
| 1 | Box AwsEvent variant | **High** | High (memory) | ⏸️ 100+ call sites - needs scripted approach |
| 2 | Extract modal input handlers | Medium | High (maintainability) | 📋 Planned |
| 3 | Consolidate EC2 actions | Low | Medium | ✅ Done - Added `InstanceAction` enum |
| 4 | Parameter structs for ECS | Low | Medium | 📋 Planned |
| 5 | Use .clamp() | Low | Low (code quality) | ✅ Done |
| 6 | Split messages module | Medium | Medium | 📋 Planned |
| 7 | Generic delete handler | Medium | Medium | 📋 Planned |
| 8 | Filterable trait | Low | Low | Already partially done |

---

## Completed Refactorings (2025-12-17)

### 1. EC2 Instance Actions Consolidated
Added `InstanceAction` enum in `src/aws/ec2.rs` with a unified `execute_action()` method. 
The existing `start_instance`, `stop_instance`, `reboot_instance`, `terminate_instance` 
methods now delegate to this single implementation.

### 2. Clamp Pattern Cleanup
Replaced manual `min().max()` patterns with `.clamp()` in `src/ui/components/modal.rs`.
This eliminates 2 clippy warnings.

### 3. Field Assignment Cleanup
Fixed `field assignment outside of initializer` warning in `src/app/update/s3.rs` by using
struct update syntax: `S3BucketDetails { loading: true, ..Default::default() }`.

### 4. Too Many Arguments Suppressed
Added `#![allow(clippy::too_many_arguments)]` to:
- `src/app/update/ecs/services.rs` - ECS handlers that mirror action enum structure
- `src/ui/components/detail_panel.rs` - UI rendering utility functions

---

## Remaining Warnings (7)

| Warning | Count | Action |
|---------|-------|--------|
| Capitalized acronyms (RDS, VPC, IAM, ECS, ECR) | 5 | Style choice - could suppress |
| Large enum variants (Event, AwsEvent) | 2 | Needs Box refactoring (100+ sites) |
