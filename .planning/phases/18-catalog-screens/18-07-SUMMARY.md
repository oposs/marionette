---
phase: 18-catalog-screens
plan: 07
subsystem: gallery-catalog-feedback
tags: [gallery, catalog, feedback, toast, modal, confirm, cat-04]
dependency_graph:
  requires:
    - marionette::builders::{Button, Container, ErrorDisplay, Heading, Spinner, Text}
    - marionette_protocol::ComponentAction
    - marionette_macros::gallery_demo (linkme registration)
    - marionette::gallery::{registered_demos, Node, DemoEntry}
    - Phase 17 handlers — gallery-demo/toast-fire, gallery-demo/modal-open, gallery-demo/confirm-open (no new handlers)
  provides:
    - catalog-feedback gallery_demo (registered via linkme #[distributed_slice])
    - seed_for_key("catalog-feedback") — single synthetic ErrorEntry at /demo/catalog-feedback/errors
  affects:
    - Nav auto-discovery — `Catalog: Feedback` now appears in AppShell sidebar between CAT-03 DataTable and future CAT-05 Typography
    - Gallery-demo total registered demo count (+1)
tech-stack:
  added: []
  patterns:
    - build_tree() flattening — each Card helper returns (root, descendants); outer fn splices descendants into a single flat Vec<Node>. Mirrors catalog/buttons.rs + catalog/forms.rs established pattern.
    - bind-alignment regression guard — sibling test in handlers/show.rs iterates registered_demos(), pulls the ErrorDisplay's `bind` field from the rendered tree, asserts it matches the seed path (Phase 17 G-05 lesson hardened into a test).
key-files:
  created:
    - backend/crates/gallery-demo/src/catalog/feedback.rs
  modified:
    - backend/crates/gallery-demo/src/catalog/mod.rs
    - backend/crates/gallery-demo/src/handlers/show.rs
decisions:
  - Kept pragmatic TDD split: RED commit = tests + stub returning single-empty-Container (5 of 6 tests fail); GREEN commit = real composition (all 6 pass). No REFACTOR commit needed — the GREEN implementation was already structured on the established catalog/buttons.rs pattern.
  - Rule 1 deviation — fixed two test assertions in the plan's prescribed test code that referenced `toast["props"]["action"]` and `err["props"]["bind"]`; the actual marionette_protocol::Component serializes `action` and `bind` as top-level fields (siblings of `type`). Tests now use `toast["action"]["name"]` and `err["bind"]`. Behaviour unchanged.
  - Did NOT fix the W-06 ErrorDisplay `message`-field dead-state (per D-2-C). Positional arg `ErrorDisplay::new("errors")` still passes a short label; visible errors come from `.bind(...)`. Flagged below for a future polish plan.
  - Did NOT modify the three Phase 17 trigger handlers despite observed drift from UI-SPEC §Copywriting Contract (see §Observed Handler Drift below). Plan 18-07 explicitly scopes the catalog screen + seed; Plan 18-08 (UAT) decides whether to open a polish plan.
metrics:
  duration_seconds: 575
  duration_human: 9m 35s
  tasks_completed: 2
  tests_added: 8
  files_modified: 3
  commits: 3
  completed: 2026-04-23T17:57:39Z
---

# Phase 18 Plan 07: CAT-04 Feedback Catalog Screen Summary

CAT-04 ships two side-by-side Cards composed from Marionette builders — Card 1 "Trigger surfaces" wires three Buttons to the existing Phase 17 feedback handlers (toast-fire, modal-open, confirm-open) and Card 2 "Placeholder states" renders empty / loading / error mini-Cards statically, with the error placeholder bound to a newly seeded single-entry errors array so it lights up on first paint.

## Card Inventory

### Card 1 — Trigger surfaces (`catalog-feedback-card1`)
| Child id | Type | Purpose |
|----------|------|---------|
| `catalog-feedback-card1-heading` | heading h2 | "Trigger surfaces" |
| `catalog-feedback-trigger-grid` | container (grid grid-cols-1 sm:grid-cols-3 gap-3) | Responsive 3-column grid |
| `catalog-feedback-toast-trigger` | button | Label "Fire toast", action click `gallery-demo/toast-fire` |
| `catalog-feedback-modal-trigger` | button | Label "Open modal", action click `gallery-demo/modal-open` |
| `catalog-feedback-confirm-trigger` | button | Label "Open confirm dialog", action click `gallery-demo/confirm-open` |

### Card 2 — Placeholder states (`catalog-feedback-card2`)
| Child id | Type | Purpose |
|----------|------|---------|
| `catalog-feedback-card2-heading` | heading h2 | "Placeholder states" |
| `catalog-feedback-placeholder-grid` | container (grid grid-cols-1 sm:grid-cols-3 gap-3) | Responsive 3-column grid |
| `catalog-feedback-empty` | container (border-dashed + text-muted-foreground) | Empty placeholder chrome |
| &nbsp;&nbsp;↳ `catalog-feedback-empty-h` | heading h4 | "No data yet" |
| &nbsp;&nbsp;↳ `catalog-feedback-empty-body` | text | Locked copy: "Start by adding your first item — empty states should always tell users what to do next." |
| `catalog-feedback-loading` | container (border + centered) | Loading placeholder chrome |
| &nbsp;&nbsp;↳ `catalog-feedback-loading-spinner` | spinner (size=md) | Visible throbber |
| &nbsp;&nbsp;↳ `catalog-feedback-loading-label` | text | "Loading…" |
| `catalog-feedback-error` | error-display (bind `/demo/catalog-feedback/errors`) | Renders seeded errors array |

### Outer frame
- `catalog-feedback-root` (container, `flex flex-col gap-6 p-6`) — top-level root
- `catalog-feedback-title` (heading h1 "Feedback")
- `catalog-feedback-intro` (text — locked copy)

## Trigger Action → Handler Table

| Button id | Label | Fires | Handler lives at |
|-----------|-------|-------|------------------|
| `catalog-feedback-toast-trigger` | Fire toast | `gallery-demo/toast-fire` (click) | `backend/crates/gallery-demo/src/handlers/toast.rs::handle_toast_fire` |
| `catalog-feedback-modal-trigger` | Open modal | `gallery-demo/modal-open` (click) | `backend/crates/gallery-demo/src/handlers/modal.rs::handle_modal_open` |
| `catalog-feedback-confirm-trigger` | Open confirm dialog | `gallery-demo/confirm-open` (click) | `backend/crates/gallery-demo/src/handlers/confirm.rs::handle_confirm_open` |

No new handlers were added. All three are Phase 17 handlers, unmodified by Plan 18-07.

## Seed Arm (handlers/show.rs)

```rust
"catalog-feedback" => serde_json::json!({
    "demo": { "catalog-feedback": {
        "errors": [
            {
                "message": "Sample error: failed to load resource. Retry or check your connection.",
                "path": null,
            },
        ],
    }},
}),
```

Matches ErrorDisplay.svelte's ErrorEntry contract (`{ path?: string, message: string }`). Only one entry — the locked synthetic sample from UI-SPEC §Copywriting §Empty/Loading/Error placeholder copy.

## Tests Added

### `catalog::feedback` (6 new)
1. `root_id` — `gallery_demo()[0].0 == "catalog-feedback-root"`
2. `three_trigger_buttons_with_locked_actions` — all 3 buttons have the correct top-level `action.name` + type="click"
3. `empty_placeholder_has_border_dashed_class` — class contains `border-dashed` + `text-muted-foreground`
4. `loading_placeholder_has_spinner` — node `catalog-feedback-loading-spinner` has `type == "spinner"`
5. `error_display_bound_to_seeded_path` — node `catalog-feedback-error` has `type == "error-display"` + `bind == "/demo/catalog-feedback/errors"`
6. `registered_demos_includes_catalog_feedback` — linkme registration + `display_name == "Catalog: Feedback"`

### `handlers::show` (2 new)
7. `catalog_feedback_seed_has_one_sample_error` — exactly one entry; `message` starts with "Sample error"; `path` is null
8. `catalog_feedback_error_bind_aligns_with_demo_tree` — G-05 regression guard: reads `bind` from the rendered demo tree's `catalog-feedback-error` node, asserts it equals the seed path; confirms seed array is non-empty

Total gallery-demo test count: 52 → 60.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Plan's test assertions used `props.action` / `props.bind`, but these fields serialize at Component top level**

- **Found during:** Task 1 GREEN phase (tests failed after implementation was correct)
- **Issue:** The plan's prescribed test block asserted `toast["props"]["action"]["name"]` and `err["props"]["bind"]`. Verification against `backend/crates/marionette-protocol/src/component.rs` showed `pub action: Option<ComponentAction>` and `pub bind: Option<String>` are both direct fields of `Component`, serialized as siblings of `type` — not nested under `props`.
- **Fix:** Changed assertions to `toast["action"]["name"]`, `toast["action"]["type"] == "click"`, and `err["bind"] == "/demo/catalog-feedback/errors"`. Added an explanatory comment pointing at `Component`'s serde definition so future maintainers don't re-introduce the bug.
- **Files modified:** `backend/crates/gallery-demo/src/catalog/feedback.rs`
- **Commit:** b735b9b (GREEN phase — the fix landed alongside the real implementation)

## Observed Handler Drift (documented per plan Task 2 `read_first` — NOT fixed)

The plan explicitly directs documenting (not fixing) any drift between the existing Phase 17 trigger handlers and UI-SPEC §Copywriting Contract. Plan 18-08 (UAT) decides whether to open a polish plan.

| Handler | UI-SPEC expectation | Actual Phase 17 output | Disposition |
|---------|--------------------|-------------------------|-------------|
| `gallery-demo/toast-fire` (`toast.rs:14`) | Toast text "Toast fired from the Feedback catalog." (default variant) | Emits a shadcn Button-shaped toast labelled "Demo toast from gallery-demo/toast-fire" wired to `dismiss-toast` | Visual-UX drift. Functional toast still appears and dismisses. |
| `gallery-demo/modal-open` (`modal.rs:21-49`) | Modal body per UI-SPEC §CAT-04 "Modal body copy" (with form + button row) | Emits Heading "Example modal" + Text "Clicking X or the backdrop dismisses this dialog." — no form, no button row | Content drift. Modal opens/closes correctly; body copy is a placeholder. |
| `gallery-demo/confirm-open` (`confirm.rs:33-39`) | ConfirmDialog with title "Delete this item?", message about demo record, confirm_label="Delete", cancel_label="Cancel", destructive=true, cancel_action=`gallery-demo/confirm-reject` | Title="Demo confirm", message="Choose an option.", confirm_label="Accept", cancel_label="Reject", **no `destructive=true` flag**, cancel_action is correct | Copy + destructive-flag drift. Confirm/cancel wiring works; visual variant is neutral not destructive. |

All three handlers are **structurally correct** (the frontend widget renders and the round-trip works). Drift is in locked copy + the destructive visual variant only. Fixing any of these is a one-file edit + no test changes, but is explicitly out of Plan 18-07 scope (D-2-C: "trigger Cards reuse existing Phase 17 feedback handlers verbatim").

## Deferred Items

- **W-06 ErrorDisplay `message`-field dead-state** — remains deferred per D-2-C. The Rust `ErrorDisplay::new("errors")` positional arg is still dead weight (frontend reads errors only from `bind`). A future polish plan should either remove the field or wire it as a bind-fallback when `getData(surface, bind)` returns empty. Carry the note into STATE.md Blockers/Concerns unchanged.

- **UI-SPEC §Copywriting drift on the three trigger handlers** — see §Observed Handler Drift above. Candidate for a single-commit polish plan post-18-08 UAT if the drift is visually jarring.

## Verification

- ✅ `cargo test -p gallery-demo` — 60 tests passing (was 52; +8 net)
- ✅ `cargo clippy -p gallery-demo --all-targets -- -D warnings` — clean
- ✅ `cargo build --workspace --all-features` — succeeds
- ✅ All Task 1 + Task 2 acceptance-criteria shell commands return the required counts
- ⏳ Manual Chrome MCP UAT — formal in Plan 18-08

## TDD Gate Compliance

This plan declared `tdd_mode: opportunistic` + Task 1 used `tdd="true"`. The git log shows the proper gate sequence:

| Gate | Commit | Content |
|------|--------|---------|
| RED   | `12ef21f` `test(18-07): add failing tests for CAT-04 Feedback catalog screen` | 6 unit tests + stub implementation (empty Container). 5 of 6 tests fail as expected; the `registered_demos_includes_catalog_feedback` test passes because linkme registration works on the stub's `#[gallery_demo]` annotation alone. |
| GREEN | `b735b9b` `feat(18-07): implement CAT-04 Feedback catalog screen` | Real composition with two Cards + all 6 tests pass. Rule 1 test-assertion bug fix bundled in. |
| REFACTOR | — | Skipped. GREEN landed on the established catalog/buttons.rs `build_tree`/flatten pattern on the first pass; no cleanup warranted. |

Task 2 was not a TDD task (no `tdd="true"` attribute) — committed once as `feat(18-07): seed CAT-04 catalog-feedback errors + bind-alignment guard` (`862ca3a`) with seed + regression guard tests together.

## Commits

| Hash | Message |
|------|---------|
| `12ef21f` | test(18-07): add failing tests for CAT-04 Feedback catalog screen |
| `b735b9b` | feat(18-07): implement CAT-04 Feedback catalog screen |
| `862ca3a` | feat(18-07): seed CAT-04 catalog-feedback errors + bind-alignment guard |

## Self-Check: PASSED

- ✅ `backend/crates/gallery-demo/src/catalog/feedback.rs` — exists
- ✅ `backend/crates/gallery-demo/src/catalog/mod.rs` — modified (added `pub mod feedback;`)
- ✅ `backend/crates/gallery-demo/src/handlers/show.rs` — modified (new `catalog-feedback` arm + 2 tests)
- ✅ Commit `12ef21f` — exists (RED)
- ✅ Commit `b735b9b` — exists (GREEN)
- ✅ Commit `862ca3a` — exists (Task 2)
- ✅ Full gallery-demo test suite: 60 passed, 0 failed
- ✅ `cargo clippy -p gallery-demo --all-targets -- -D warnings` exits 0
- ✅ `cargo build --workspace --all-features` exits 0
- ✅ CAT-04 requirement covered
