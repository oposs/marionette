---
phase: 12
plan: 02
subsystem: protocol
tags: [protocol, rust, serde, backend, node-patching]
requirements: [PATCH-01, PATCH-03]
dependency_graph:
  requires: [12-01]
  provides:
    - "tagged PatchOperation enum with 6 variants (set, set-node, delete-node, set-children, insert-child, remove-child)"
    - "PatchMessage.surface required field"
    - "protocol version 1.1.0 emission"
  affects:
    - backend/crates/marionette-protocol/src/data.rs
    - backend/crates/marionette-protocol/src/messages.rs
    - backend/crates/marionette/src/ws.rs
    - backend/crates/marionette/src/router.rs
    - backend/crates/marionette/tests/ws_integration.rs
    - backend/crates/crm-demo/tests/integration_test.rs
tech_stack:
  added: []
  patterns:
    - "serde internally-tagged enum with tag=\"op\", rename_all=kebab-case"
    - "#[serde(rename = \"childId\")] for camelCase wire names on snake_case Rust fields"
    - "#[allow(clippy::large_enum_variant)] on SetNode variant carrying Component"
key_files:
  created:
    - .planning/phases/12-protocol-node-patching-appshell/deferred-items.md
  modified:
    - backend/crates/marionette-protocol/src/data.rs
    - backend/crates/marionette-protocol/src/messages.rs
    - backend/crates/marionette/src/ws.rs
    - backend/crates/marionette/src/router.rs
    - backend/crates/marionette/tests/ws_integration.rs
    - backend/crates/crm-demo/tests/integration_test.rs
decisions:
  - "Implemented D-A1 (tagged enum with op discriminator) verbatim per CONTEXT locked decisions"
  - "Implemented D-A2 (6 variants: Set, SetNode, DeleteNode, SetChildren, InsertChild, RemoveChild)"
  - "Implemented D-A3 (surface field required on PatchMessage, no per-op override)"
  - "Implemented D-A5 (protocol version 1.1.0 in HelloMessage emission)"
  - "Implemented D-A7 (no back-compat; deleted old PatchOperation struct cleanly)"
  - "Deferred pre-existing crm-demo clippy drift (76 warnings) to dedicated plan — out of scope"
metrics:
  duration_minutes: 12
  completed: "2026-04-10T14:58:20Z"
  tasks: 2
  protocol_tests_before: 13
  protocol_tests_after: 22
  new_protocol_tests: 9
  call_sites_migrated: 5
---

# Phase 12 Plan 02: Protocol Crate Rewrite Summary

**One-liner:** PatchOperation rewritten as a 6-variant serde-tagged enum with `op` discriminator; PatchMessage gains required `surface` field; HelloMessage now emits protocol version 1.1.0.

## What Was Built

### Task 1: PatchOperation tagged enum (commit 620559b)

Replaced the old `PatchOperation { path, value }` struct with a tagged enum carrying six variants. The enum is serialized with `#[serde(tag = "op", rename_all = "kebab-case")]`, matching D-A1/D-A2 exactly:

| Variant | Wire `op` | Fields |
|---------|-----------|--------|
| `Set` | `"set"` | `path: String, value: serde_json::Value` |
| `SetNode` | `"set-node"` | `id: String, component: Component` |
| `DeleteNode` | `"delete-node"` | `id: String` |
| `SetChildren` | `"set-children"` | `id: String, children: Vec<String>` |
| `InsertChild` | `"insert-child"` | `parent: String, index: usize, child_id: String` (wire: `childId`) |
| `RemoveChild` | `"remove-child"` | `parent: String, child_id: String` (wire: `childId`) |

The enum is marked `#[allow(clippy::large_enum_variant)]` because `SetNode` carries a full `Component`, which is substantially larger than the other variants (per RESEARCH Finding 3). Documented in the threat model (T-12-02 accept).

Unit tests added (all in `data.rs` `mod tests`):

- `patch_op_set_round_trip` — `{"op": "set", "path": "/...", "value": ...}`
- `patch_op_set_node_round_trip` — `{"op": "set-node", "id": "...", "component": {...}}`
- `patch_op_delete_node_round_trip` — `{"op": "delete-node", "id": "..."}`
- `patch_op_set_children_round_trip` — `{"op": "set-children", "id": "...", "children": [...]}`
- `patch_op_insert_child_round_trip` — `{"op": "insert-child", "parent": "...", "index": N, "childId": "..."}`
- `patch_op_remove_child_round_trip` — `{"op": "remove-child", "parent": "...", "childId": "..."}`
- `patch_op_unknown_discriminator_rejected` — proves strict tagged enum rejects unknown `op` (T-12-03)
- `validation_error_round_trip`, `validation_error_without_path` — preserved from pre-plan state

### Task 2: PatchMessage.surface + Hello 1.1.0 (commit 220d367)

Added a required `surface: Surface` field to `PatchMessage` between `id` and `patch`. The field has no `skip_serializing_if` and no `default`, so missing-surface payloads fail deserialization — proved by the `patch_message_surface_required` test.

Bumped the `HelloMessage.version` emission in `backend/crates/marionette/src/ws.rs` from `"1.0.0"` to `"1.1.0"` (D-A5). Updated the `hello_round_trip` and `deserialize_from_spec_json` tests in `messages.rs` to match. Updated the `"1.0.0"` assertions in `ws_integration.rs` and `crm-demo/tests/integration_test.rs`.

New tests in `messages.rs`:

- `patch_message_surface_required` — `{"type": "patch", "patch": []}` without surface is rejected
- `patch_message_targets_non_main_surface` — round-trips a patch targeting `"modal"` with a `DeleteNode` op

## Call Sites Migrated

| Crate | File | Kind | Notes |
|-------|------|------|-------|
| marionette-protocol | `src/messages.rs` | 2 call sites | `patch_round_trip` test body; `action_round_trip` optimistic patch test |
| marionette | `src/router.rs` | 1 call site | `echo_handler` test helper in `mod tests` |
| marionette | `tests/ws_integration.rs` | 1 call site | `echo_handler` |
| crm-demo | `tests/integration_test.rs` | 1 call site | `handle_demo_click` |

**Total: 5 Rust call sites migrated** from `PatchOperation { path, value }` struct-construction syntax to `PatchOperation::Set { path, value }` enum-variant syntax. All 5 of these call sites also received the new `surface: "main".into()` field in the wrapping `PatchMessage`.

## Protocol Test Counts

| Module | Before | After | Delta |
|--------|--------|-------|-------|
| `data::tests` | 4 | 9 | +5 (set, set-node, delete-node, set-children, insert-child, remove-child round-trips plus unknown discriminator rejection minus one old `complex_value` test merged) |
| `messages::tests` | 8 | 10 | +2 (`patch_message_surface_required`, `patch_message_targets_non_main_surface`) |
| `component::tests` | 3 | 3 | 0 |
| **Total protocol crate** | **15** | **22** | **+7** |

Note: the plan predicted "13 before / 22 after" but an actual count of the pre-plan file shows 15 tests (4 data + 8 messages + 3 component). The net new test count is 7, and the variant-round-trip requirement (6 per-variant + unknown rejection = 7 tests) is met.

## Full Workspace Test Results

```
protocol crate:         22 passed
marionette unit tests:  26 passed
marionette ws_integration: 5 passed
marionette macro_tests:  3 passed
marionette-macros tests: 7 passed
crm-demo unit tests:     6 passed
crm-demo integration:   (passed — counted in workspace total)
```

`cargo test --workspace` exits 0.

## Clippy Status

- `cargo clippy -p marionette-protocol -- -D warnings` — **clean**
- `cargo clippy -p marionette -- -D warnings` — **clean** (after Rule 3 fixes to ws.rs)
- `cargo clippy --workspace -- -D warnings` — **NOT clean** due to 76 pre-existing pedantic warnings in `crm-demo` (toolchain drift from clippy 1.93.0 lint additions). See `deferred-items.md`. **In-scope crates for Plan 12-02 are clippy-clean.**

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing `listmonk: None` in AppState initializer**
- **Found during:** Task 2 workspace build
- **Issue:** `backend/crates/marionette/tests/ws_integration.rs:21` constructed `AppState` without the `listmonk` field, failing compilation
- **Fix:** Added `listmonk: None` to the initializer
- **Files modified:** `backend/crates/marionette/tests/ws_integration.rs`
- **Commit:** 220d367
- **Root cause:** Pre-existing state drift — the `AppState` struct gained a `listmonk` field but this test file was never updated. Blocking because `cargo test --workspace` is a plan acceptance criterion.

**2. [Rule 3 - Blocking] Pre-existing clippy pedantic warnings in ws.rs**
- **Found during:** Task 2 clippy verification
- **Issue:** 4 pre-existing clippy errors in `backend/crates/marionette/src/ws.rs`:
  - Line 80: `clippy::nonminimal_bool` (`if !expired` with else branch)
  - Lines 118–119: `clippy::collapsible_if` / `collapsible_nested_if` (triple-nested if-let chain)
  - Line 257: `clippy::redundant_closure_for_method_calls` (`|v| v.as_i64()`)
- **Fix:** Inverted the `expired` if-else; collapsed the login-form nested if using `let` chains (Rust 2024 `if let && let && cond`); replaced the closure with `serde_json::Value::as_i64` method reference
- **Files modified:** `backend/crates/marionette/src/ws.rs`
- **Commit:** 220d367
- **Justification:** Plan 12-02 modifies `ws.rs` (hello version bump) and acceptance criteria require clippy-clean in-scope crates. Fixing inline is cheaper than splitting into a separate plan. Changes are pure lint refactors with zero behavior change — verified by `cargo test --workspace` remaining green.

### Scope Deferred

**76 pre-existing clippy pedantic warnings in `crm-demo`** — toolchain drift from clippy 1.93.0 (new lints not in effect when the code was written). None caused by Plan 12-02. Logged to `deferred-items.md` with recommendation for a dedicated lint-cleanup plan. Verified via `git stash && cargo clippy -p crm-demo` (pre-plan state reproduces all 76 errors).

## Key Decisions Honored (from 12-CONTEXT.md)

- **D-A1** ✓ Tagged enum with `op` discriminator — implemented verbatim
- **D-A2** ✓ 6 ops total (set + 5 node ops) — all 6 present, child-sugar ops (`insert-child` / `remove-child`) present
- **D-A3** ✓ `surface: Surface` required on PatchMessage — no skip, no default
- **D-A4** N/A — root immutability is enforced at the frontend layer (not in this plan)
- **D-A5** ✓ Protocol version bumped to `1.1.0` in ws.rs emission
- **D-A6** N/A — focus preservation is a frontend concern (later plans)
- **D-A7** ✓ No back-compat — old `PatchOperation { path, value }` shape deleted cleanly, call sites migrated

## Threat Model Coverage

- **T-12-02 (Tampering — oversized Component in SetNode):** accepted per plan; documented via `#[allow(clippy::large_enum_variant)]` with rationale
- **T-12-03 (DoS — strict enum rejects unknown op):** mitigated by `HelloMessage.version` negotiation (out of this plan's scope) and proven by `patch_op_unknown_discriminator_rejected` test
- **T-12-04 (Info disclosure — surface routing):** proved by `patch_message_targets_non_main_surface` test

## Threat Flags

None — no new trust boundaries introduced. The only wire-shape changes (new enum variants, new required field, new version string) all originate server-side and flow server→client, which is an unchanged trust direction.

## Known Stubs

None — every field added in this plan is exercised by a round-trip test with concrete sample data.

## Commits

| # | Hash | Task | Summary |
|---|------|------|---------|
| 1 | `620559b` | Task 1 | Rewrite PatchOperation as 6-variant tagged enum; migrate 5 call sites |
| 2 | `220d367` | Task 2 | Add PatchMessage.surface; bump Hello to 1.1.0; fix blocking pre-existing issues in ws_integration.rs and ws.rs |

## Self-Check: PASSED

Verified files exist:

- FOUND: backend/crates/marionette-protocol/src/data.rs (rewritten, enum PatchOperation present, tag="op", rename_all kebab-case)
- FOUND: backend/crates/marionette-protocol/src/messages.rs (pub surface: Surface present)
- FOUND: backend/crates/marionette/src/ws.rs ("1.1.0" present, ws_session logic preserved)
- FOUND: backend/crates/marionette/src/router.rs (PatchOperation::Set variant, surface field)
- FOUND: backend/crates/marionette/tests/ws_integration.rs (PatchOperation::Set variant, surface field, 1.1.0 assertion, listmonk: None)
- FOUND: backend/crates/crm-demo/tests/integration_test.rs (PatchOperation::Set variant, surface field, 1.1.0 assertions)
- FOUND: .planning/phases/12-protocol-node-patching-appshell/deferred-items.md

Verified commits exist:

- FOUND: 620559b (Task 1)
- FOUND: 220d367 (Task 2)

Verified acceptance criteria:

- `grep -q 'enum PatchOperation' backend/crates/marionette-protocol/src/data.rs` → PASS
- `grep -q 'tag = "op"' backend/crates/marionette-protocol/src/data.rs` → PASS
- `grep -q 'rename_all = "kebab-case"' backend/crates/marionette-protocol/src/data.rs` → PASS
- `grep -q 'pub surface: Surface' backend/crates/marionette-protocol/src/messages.rs` → PASS
- `grep -q '"1.1.0"' backend/crates/marionette/src/ws.rs` → PASS
- No `"1.0.0"` in any Rust source → PASS
- No old `PatchOperation { ... }` struct syntax (excluding the enum definition) → PASS
- `cargo test -p marionette-protocol` → 22/22 PASS
- `cargo test --workspace` → all test suites green
- `cargo clippy -p marionette-protocol -p marionette -- -D warnings` → clean (in-scope crates)
