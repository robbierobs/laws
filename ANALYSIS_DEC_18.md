# Codebase Analysis & Optimization Report - Dec 18, 2025

## 1. Memory Optimization: Large Enum Variants
**Issue**: `Event` and `AwsEvent` have very large variants, causing excessive memory usage for every event in the queue.
- `Event` is >500 bytes because it contains `AwsEvent`.
- `AwsEvent` is >500 bytes because of `ProfileRegionSwitched` and `EcsTaskDefinitionLoaded`.

**Solution**: 
- Box `AwsEvent` inside `Event::Aws(Box<AwsEvent>)`.
- Box large fields in `AwsEvent` variants (e.g., `EcsTaskDefinitionLoaded(Box<EcsTaskDefinition>)`).

## 2. Refactoring: Service Action Dispatch
**Issue**: `App::handle_service_action` in `src/app/update/mod.rs` is a giant, flat match statement that handles every single action for every service. This makes the file hard to maintain and violates encapsulation.

**Solution**:
- Implement `handle_action` methods on each service state struct (e.g., `Ec2State::handle_action`).
- Update `App::handle_service_action` to simply delegate:
  ```rust
  match action {
      ServiceAction::Ec2(a) => self.services.ec2.handle_action(a, tx),
      // ...
  }
  ```

## 3. Standardizing Service Interaction
**Issue**: Refreshing services, auto-selecting items, and collecting search results are handled via repetitive match statements in `input.rs` and `global.rs`.

**Solution**:
- Expand the `AwsService` or create a new `ServiceInternal` trait for `ServiceState` structs.
- Unified methods for:
  - `refresh(&self, clients: &AwsClients, tx: EventSender)`
  - `auto_select(&mut self)`
  - `get_search_results(&self) -> Vec<SearchResult>` (Partially done)

## 4. Reducing Boilerplate in Refresh Logic
**Issue**: `handle_refresh_data` and `refresh_all_services_for_search` in `src/app/update/global.rs` duplicate a lot of task-spawning logic.

**Solution**:
- Create a registry or an array of services that can be iterated over for "all service" operations.

## 5. UI Consistency: ViewMode Trait
**Status**: Mostly implemented.
**Observation**: Ensure all services use the `ViewMode` trait for tab navigation. Some services like `VPC` and `IAM` have complex sub-views that might benefit from more standardization.

---

# Implementation Plan

### Step 1: Boxing `AwsEvent` (High Priority)
- Update `src/event.rs`.
- Fix all call sites (approx 50-100).
- Fix `src/app/events.rs` to handle boxed event.

### Step 2: Delegate Service Actions
- Move logic from `src/app/update/*.rs` (App methods) to `src/app/states/*.rs` (State methods).
- Simplify `src/app/update/mod.rs`.

### Step 3: Standardize Refresh & Selection
- Implement `auto_select` on all service states.
- Clean up `src/app/input.rs`.
