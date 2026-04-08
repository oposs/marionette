---
phase: 04-backend-toolkit
plan: 02
subsystem: api
tags: [rust, proc-macro, derive, builder-pattern, darling, syn, quote]

requires:
  - phase: 04-backend-toolkit
    plan: 01
    provides: Protocol types (Component, ComponentAction) used by macro-generated code
provides:
  - ComponentBuilder derive macro generating fluent builders from struct fields
  - action attribute macro generating action name constants
  - requires attribute macro generating AuthRequirement metadata
  - All 18 standard component builders (Button through ErrorDisplay)
  - ComponentAction helper constructors (submit, click, change)
  - AuthRequirement enum in marionette-protocol
affects: [04-backend-toolkit, 05-crm-demo]

tech-stack:
  added: [darling, syn, quote, proc-macro2]
  patterns: [derive macro with darling for attribute parsing, fluent builder pattern with required/optional fields, fully-qualified paths in generated code for edition 2024]

key-files:
  created:
    - backend/crates/marionette-macros/src/component_builder.rs
    - backend/crates/marionette-macros/src/action.rs
    - backend/crates/marionette-macros/src/requires.rs
    - backend/crates/marionette/src/builders/mod.rs
    - backend/crates/marionette/src/builders/node.rs
    - backend/crates/marionette/src/builders/standard.rs
    - backend/crates/marionette/tests/macro_tests.rs
  modified:
    - backend/crates/marionette-macros/Cargo.toml
    - backend/crates/marionette-macros/src/lib.rs
    - backend/crates/marionette/Cargo.toml
    - backend/crates/marionette/src/lib.rs
    - backend/crates/marionette-protocol/src/common.rs
    - backend/crates/marionette-protocol/src/component.rs

key-decisions:
  - "ComponentAction helper constructors live in marionette-protocol (orphan rule)"
  - "AuthRequirement enum added to marionette-protocol common.rs for cross-crate use"
  - "Fully qualified paths in macro output (::marionette_protocol::, ::serde_json::, ::uuid::) for edition 2024"
  - "darling needless_continue clippy allow at crate level (darling-generated code)"

patterns-established:
  - "ComponentBuilder: derive on struct with #[component(type = ...)] and #[builder(optional)] field attrs"
  - "Builder new() takes required fields, optional fields are setter methods, build() returns (id, Component)"
  - "build_with_children() returns flat Vec of all nodes for adjacency list insertion"
  - "action macro: #[action(name = ...)] generates UPPER_SNAKE constant alongside function"
  - "requires macro: #[requires(authenticated)] or #[requires(role = ...)] generates FN_AUTH constant"

requirements-completed: [BACK-02, BACK-07, BACK-10]

duration: 5min
completed: 2026-03-20
---

# Phase 04 Plan 02: Macros and Builders Summary

**ComponentBuilder derive macro with darling, action/requires attribute macros, and all 18 standard component builders with 12 passing tests**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-20T15:02:57Z
- **Completed:** 2026-03-20T15:08:00Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments
- ComponentBuilder derive macro generates typed fluent builders from struct fields with required/optional distinction
- action and requires attribute macros generate compile-time constants for action names and auth metadata
- All 18 standard component types (Button, TextInput, Select, Checkbox, Container, Grid, Heading, Text, SideNav, NavItem, NavGroup, Form, DataTable, Modal, Toast, ConfirmDialog, Spinner, ErrorDisplay)
- 12 tests passing (9 builder unit tests + 3 macro integration tests), clippy clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement ComponentBuilder, action, and requires proc macros** - `626fdc3` (feat)
2. **Task 2: Standard component builders and macro tests** - `8e7f770` (test)

## Files Created/Modified
- `backend/crates/marionette-macros/Cargo.toml` - Added syn, quote, proc-macro2, darling dependencies
- `backend/crates/marionette-macros/src/lib.rs` - Proc macro entry points for all three macros
- `backend/crates/marionette-macros/src/component_builder.rs` - ComponentBuilder derive implementation using darling
- `backend/crates/marionette-macros/src/action.rs` - action attribute macro generating name constants
- `backend/crates/marionette-macros/src/requires.rs` - requires attribute macro generating auth metadata
- `backend/crates/marionette/Cargo.toml` - Added uuid dependency
- `backend/crates/marionette/src/lib.rs` - Module declarations and re-exports
- `backend/crates/marionette/src/builders/mod.rs` - Builder module re-exports
- `backend/crates/marionette/src/builders/node.rs` - Node type alias and node_id helper
- `backend/crates/marionette/src/builders/standard.rs` - All 18 component structs with derive + tests
- `backend/crates/marionette/tests/macro_tests.rs` - Integration tests for action and requires macros
- `backend/crates/marionette-protocol/src/common.rs` - Added AuthRequirement enum
- `backend/crates/marionette-protocol/src/component.rs` - Added ComponentAction helper constructors

## Decisions Made
- ComponentAction helper constructors (submit, click, change) placed in marionette-protocol rather than marionette due to Rust orphan rule
- AuthRequirement enum added to marionette-protocol/common.rs so both marionette-macros and marionette can reference it
- Used fully qualified paths in all macro-generated code for edition 2024 compatibility
- Allowed clippy::needless_continue at crate level for darling-generated code

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed edition 2024 reference pattern matching in component_builder.rs**
- **Found during:** Task 1
- **Issue:** `ref p` patterns in `if let syn::Type::Path(ref p) = ty` are errors in edition 2024
- **Fix:** Removed `ref` keyword, using direct patterns as edition 2024 requires
- **Files modified:** backend/crates/marionette-macros/src/component_builder.rs
- **Committed in:** 626fdc3 (Task 1 commit)

**2. [Rule 3 - Blocking] Moved ComponentAction impl from marionette to marionette-protocol**
- **Found during:** Task 2
- **Issue:** Rust orphan rule prevents adding inherent impl for ComponentAction outside its defining crate
- **Fix:** Added submit/click/change constructors directly in marionette-protocol/src/component.rs
- **Files modified:** backend/crates/marionette-protocol/src/component.rs, backend/crates/marionette/src/builders/node.rs
- **Committed in:** 8e7f770 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes necessary for compilation. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All macros ready for use by action routing (Plan 03) and WebSocket session management (Plan 04)
- Standard builders ready for CRM demo (Phase 05)
- AuthRequirement enum ready for authorization middleware

---
*Phase: 04-backend-toolkit*
*Completed: 2026-03-20*
