# Refactoring Roadmap

This roadmap consolidates findings from all analysis subtasks (app/, aws/, models/, ui/, utils/, tests/, error handling). Items are prioritized by impact and risk, with effort estimates and quick-win highlights.

## Priority Definitions

- **P0 (Critical)**: Bugs or correctness issues that can break UX/data safety.
- **P1 (High)**: High-leverage refactors that reduce complexity and maintenance cost.
- **P2 (Medium)**: Quality improvements with moderate payoff.
- **P3 (Low/Future)**: Nice-to-haves or longer-term architecture work.

## Effort Estimates

- **S**: ≤ 1 day
- **M**: 2–4 days
- **L**: 1–2 weeks
- **XL**: 2+ weeks

## Quick Wins (High Value, Low Effort)

1. **Fix ECS table state cloning bug** (UI) — selection doesn’t persist. **P0 / S**
2. **Stop silent error swallowing in aws_profiles parsing** (utils). **P1 / S**
3. **Add shared tag + name tag helpers** (models). **P1 / S**
4. **Normalize modal centering via helper** (ui). **P2 / S**
5. **Add tests for utils/error.rs**. **P2 / S**

---

## Roadmap (Prioritized)

### P0 — Critical Bugs & Safety

| Area | Finding | Action | Effort |
| --- | --- | --- | --- |
| ui/ | ECS screens clone table state; selection never persists | Use shared TableState, avoid cloning, add regression test | S |
| aws/ | Silent errors in S3::get_bucket_details & DynamoDB::list_tables | Return/propagate errors to AwsEvent::Error | M |
| aws/ | Event polling ignores errors with unwrap_or(false) | Surface errors or log, ensure failure state visible | S |

### P1 — High-Value Refactors

| Area | Finding | Action | Effort |
| --- | --- | --- | --- |
| app/ | EC2 & RDS state/input/update handlers duplicated | Introduce shared instance-state trait + generic input handler + generic update helper | L |
| aws/ | Pagination loops duplicated in IAM, Lambda, SecretsManager, ECS | Create pagination helper in utils/pagination.rs and refactor services | M |
| aws/ | N+1 DynamoDB describe_table calls | Batch where possible or make parallel with bounded concurrency | M |
| models/ | Tag extraction, name tags duplicated | Add tag helpers in models module (shared functions) | S |
| models/ | state_color duplicated across 10+ models | Extract status->color helper (enum or fn) | M |
| error | Many channel send errors ignored via .ok() | Centralize send helpers that log on failure | M |
| tests | Characterization tests missing for service states/update layer | Add baseline tests for key state transitions | L |

### P2 — Quality & Consistency

| Area | Finding | Action | Effort |
| --- | --- | --- | --- |
| ui/ | Modal centering duplicated (filter_modal, CloudTrail, SecretsManager) | Create `render_centered_modal` helper in ui/components | S |
| ui/ | Inconsistent error display across screens | Standardize error panel component or helper | M |
| ui/ | Budgets screen bypasses render_table helper | Replace manual table build with render_table | S |
| utils/ | Size formatting duplicated in s3/dynamodb | Create utils/formatting helpers and reuse | S |
| utils/ | Empty modules (formatting.rs, pagination.rs) | Implement or remove to avoid dead code | S |
| tests | Brittle timing tests in event.rs | Replace with deterministic mocks/time control | M |

### P3 — Future/Strategic Refactors

| Area | Finding | Action | Effort |
| --- | --- | --- | --- |
| models/ | Introduce resource ID newtypes | Reduce mixups, improve type safety across services | L |
| aws/ | Reduce SDK struct clones in from_aws | Consider borrow-based conversions or small owned structs | L |
| app/ | Shared Searchable/AutoSelectable helpers | Reduce duplication across services | M |
| tests | Expand coverage for AWS wrapper error edge cases | Add failure-case tests with mocks | L |

---

## Suggested Phases

### Phase 1 (Stability & Quick Wins)
- Fix ECS selection bug (P0/S)
- Propagate silent AWS errors (P0/M)
- Tag + name-tag helpers (P1/S)
- Modal centering helper (P2/S)

### Phase 2 (Duplication Reduction)
- EC2/RDS shared input/update abstractions (P1/L)
- Pagination helper in aws/ + refactor services (P1/M)
- state_color helper consolidation (P1/M)

### Phase 3 (Testing & Consistency)
- Characterization tests for service states/update layer (P1/L)
- Fix brittle timing tests (P2/M)
- Standardize error display component (P2/M)

### Phase 4 (Strategic Enhancements)
- Resource ID newtypes (P3/L)
- Reduce SDK clone usage (P3/L)
- Broader wrapper edge-case tests (P3/L)

---

## Notes & Dependencies

- **P0 work should block new feature delivery** until addressed.
- **EC2/RDS consolidation** should align with **Searchable/AutoSelectable helper** design to avoid churn.
- **Pagination helper** in utils/pagination.rs should be shared with a consistent error model to reduce future drift.
- **Testing additions** should prioritize update layer behavior and event handling to catch regressions early.

---

## Success Criteria

- No silent error paths in aws/ or event handling.
- Shared helper abstractions reduce duplication in app/models/ui.
- Tests cover key state transitions and error handling paths.
- UI behavior consistent across services (modals, errors, tables).
