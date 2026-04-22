---
phase: 17-gallery-crate-skeleton-colocated-built-in-demos
plan: 01
subsystem: framework
tags: [gallery, linkme, proc-macro, syn, trybuild, rust]

# Dependency graph
requires:
  - phase: 16-framework-hooks
    provides: "linkme-backed DEMOS slice, #[gallery_demo] proc macro, gallery cargo feature gate, gallery-smoke permanent regression crate"
provides:
  - "DemoEntry.render field typed fn() -> Vec<Node> (flat tree: root at index 0, descendants after)"
  - "Macro validator return_type_is_vec_node that accepts Vec<Node>, std::vec::Vec<Node>, and rejects non-Vec or Vec<!Node> with Vec<Node>-specific error"
  - "gallery-smoke::smoke() returns vec![Text::new(...).build()] demonstrating the new shape"
  - "trybuild .stderr fixture updated to assert the Vec<Node> rejection message"
affects: [17-02, 17-03, 17-04, 18, 19]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Render field carries the full descendant subtree (flat Vec<Node>, root at index 0) — unblocks composite demos (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast, AppShell)"
    - "Macro-level type validation extended from simple ident matching to nested path+generic inspection (Vec<T> with inner-segment check)"

key-files:
  created: []
  modified:
    - backend/crates/marionette/src/gallery.rs
    - backend/crates/marionette-macros/src/gallery_demo.rs
    - backend/crates/gallery-smoke/src/lib.rs
    - backend/crates/gallery-smoke/tests/registry_roundtrip.rs
    - backend/crates/gallery-smoke/tests/ui/fail_wrong_return.stderr

key-decisions:
  - "Flat Vec<Node> convention (index 0 = root, rest = descendants) matches the existing build_with_children() return shape — zero adapter code needed in plan 04's composite demos"
  - "Clean-cut rename of return_type_is_node → return_type_is_vec_node with no deprecated alias per feedback_pre_deployment_no_backcompat"
  - "Nested generic-argument inspection in the macro validator accepts path-qualified variants (std::vec::Vec<Node>, ::alloc::vec::Vec<crate::gallery::Node>) — future-proof against different import styles in downstream plans"

patterns-established:
  - "Pattern: render contract — pure fn() -> Vec<Node>, no I/O, no external state; first element is the root of a flat adjacency list"
  - "Pattern: macro validator destructures syn::Type::Path → PathArguments::AngleBracketed → GenericArgument::Type → syn::Type::Path to enforce Vec<T> with constrained T"

requirements-completed: []

# Metrics
duration: 24min
completed: 2026-04-22
---

# Phase 17 Plan 01: Gallery DemoEntry.render → Vec<Node> Summary

**Signature flip `DemoEntry.render: fn() -> Node` → `fn() -> Vec<Node>` across 4 code sites + 1 trybuild fixture, unlocking composite demos in plan 04 (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast, AppShell)**

## Performance

- **Duration:** ~24 min
- **Started:** 2026-04-22T09:36:46Z
- **Completed:** 2026-04-22T10:00:00Z
- **Tasks:** 3 (2 code tasks + 1 verification)
- **Files modified:** 5

## Accomplishments

- `DemoEntry.render` flipped to `fn() -> Vec<Node>` with the "index 0 = root, remaining are descendants" convention documented in the struct doc comment
- `#[gallery_demo]` proc macro now enforces the new contract via `return_type_is_vec_node` — rejects non-Vec or `Vec<!Node>` returns with a Vec<Node>-specific error message
- `gallery-smoke::smoke()` migrated to return `vec![Text::new("gallery-smoke").build()]`; the four registry_roundtrip integration tests pass unchanged (force-link reference updated in step-with)
- trybuild fixture `fail_wrong_return.stderr` updated to match the new rejection text; the input `.rs` fixture (`pub fn wrong_return() -> Vec<u32> { ... }`) is unchanged and still correctly rejected because `u32 != Node`
- Workspace-wide verification: `cargo build --workspace --exclude crm-demo --features gallery` green, `cargo test --workspace --exclude crm-demo --features gallery` green (all test results report 0 failures), clippy `-D warnings` green on marionette lib + marionette-macros + gallery-smoke

## Task Commits

Each task was committed atomically:

1. **Task 1: Flip DemoEntry.render to Vec<Node> and update gallery.rs + macro validator + gallery-smoke body** — `9efe5cb` (refactor)
2. **Task 2: Update fail_wrong_return trybuild .stderr fixture** — `c9fdbfe` (test)
3. **Task 3: Workspace verification sweep (build + test + clippy)** — no commit, pure verification task per plan

_Task 3 modifies no files; it is a pass/fail checkpoint. Plan's own text: "No files are modified in this task — it is a pure verification checkpoint."_

## Files Created/Modified

- `backend/crates/marionette/src/gallery.rs` — flipped `render` field type, updated doc comment, renamed `tests::minimal_node` → `minimal_nodes` returning `Vec<Node>`, updated `tests::leak_entry` reference
- `backend/crates/marionette-macros/src/gallery_demo.rs` — renamed `return_type_is_node` → `return_type_is_vec_node`; new body destructures `syn::Type::Path → PathArguments::AngleBracketed → GenericArgument::Type → syn::Type::Path` and checks the inner path's last segment ident is `Node`; updated two error messages (unit-return branch + non-matching-type branch)
- `backend/crates/gallery-smoke/src/lib.rs` — `smoke()` returns `Vec<Node>` wrapping the Text tuple
- `backend/crates/gallery-smoke/tests/registry_roundtrip.rs` — force-link reference at line 22 now typed `fn() -> Vec<marionette::gallery::Node>`
- `backend/crates/gallery-smoke/tests/ui/fail_wrong_return.stderr` — expected error message now references `Vec<Node>`, em-dash phrasing "index 0 is the root, remaining entries are descendants"

## Decisions Made

- Clean-cut rename of the macro helper (no deprecated `return_type_is_node` alias) per user global memory `feedback_pre_deployment_no_backcompat`
- The trybuild input `fail_wrong_return.rs` was deliberately left unchanged: `Vec<u32>` is still wrong under the new rule (inner segment `u32` != `Node`), so the fixture continues to serve its purpose without a second edit
- The plan's explicit directive to leave the arg-count error message (`"#[gallery_demo] fn must be \`fn() -> Node\` with zero arguments (found N)"`) alone was honored, even though that message now references the old signature. This is a cosmetic inconsistency — it only triggers when an unrelated validator (arg count) fires — and the `fail_wrong_signature.stderr` trybuild fixture exercises/pins it. Noted in "Issues Encountered" below as a follow-up candidate.

## Deviations from Plan

None — plan executed exactly as written.

- All 3 tasks ran in the order specified
- All verifications passed on first run (no retry loops, no fix attempts needed)
- No files outside the `files_modified` list were touched (`git log --stat 5ef2408..HEAD` shows exactly 5 files)
- No scope creep: the arg-count validator's stale "`fn() -> Node`" string was noted but left in place per plan Task 1 Step 2 explicit instruction

## Issues Encountered

- **Cosmetic stale string in arg-count error message** (not in this plan's scope): `gallery_demo.rs` line 104 still reads `"#[gallery_demo] fn must be \`fn() -> Node\` with zero arguments (found {})"`. Under the new contract the canonical signature is `fn() -> Vec<Node>`. This only fires if someone adds an arg (unrelated validator), and the existing `fail_wrong_signature.stderr` fixture pins this exact text. Fixing it would require a second trybuild fixture update and was explicitly deferred by the plan. Recommended follow-up: pick this up in plan 17-04 (which already touches the macro surface) or a dedicated cosmetic-cleanup micro-plan.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **Plan 17-03 unblocked:** handler consumption pattern `let nodes_vec = entry.render(); let root_id = nodes_vec[0].0.clone(); let nodes_map: HashMap<_, _> = nodes_vec.into_iter().collect();` now compiles against the shipped `Vec<Node>` signature
- **Plan 17-04 unblocked:** composite gallery demos can return `Container::new().children(...).build_with_children()` directly — that builder already returns `Vec<(String, Component)>` which aliases exactly to `Vec<Node>`
- **No blockers introduced:** the linkme backbone, `gallery` feature gate, explicit `key = "..."` requirement, pure-fn contract, and alphabetical iteration order are all unchanged from Phase 16

## Self-Check: PASSED

Verification of claims before returning:

**Files claimed modified:**
- `backend/crates/marionette/src/gallery.rs` — FOUND (changed in 9efe5cb)
- `backend/crates/marionette-macros/src/gallery_demo.rs` — FOUND (changed in 9efe5cb)
- `backend/crates/gallery-smoke/src/lib.rs` — FOUND (changed in 9efe5cb)
- `backend/crates/gallery-smoke/tests/registry_roundtrip.rs` — FOUND (changed in 9efe5cb)
- `backend/crates/gallery-smoke/tests/ui/fail_wrong_return.stderr` — FOUND (changed in c9fdbfe)

**Commits claimed:**
- `9efe5cb` (Task 1: refactor) — FOUND in `git log`
- `c9fdbfe` (Task 2: test) — FOUND in `git log`

**Grep-based acceptance criteria (all from plan Task 1 + Task 2):**
- `pub render: fn() -> Vec<Node>` at gallery.rs:33 — FOUND
- old `render: fn() -> Node,` pattern — NOT FOUND (good)
- `fn return_type_is_vec_node` at gallery_demo.rs:157 — FOUND
- old `fn return_type_is_node` — NOT FOUND (good)
- `return_type_is_vec_node(ty)` call site at gallery_demo.rs:140 — FOUND
- `pub fn smoke() -> Vec<Node>` at lib.rs:25 — FOUND
- `vec![Text::new("gallery-smoke").build()]` at lib.rs:26 — FOUND
- `fn() -> Vec<marionette::gallery::Node>` at registry_roundtrip.rs:22 — FOUND
- `Vec<Node>` in fail_wrong_return.stderr — FOUND
- old `fn must return \`Node\`` in fail_wrong_return.stderr — NOT FOUND (good)

**Automated verifications (all green):**
- `cargo test -p marionette --lib gallery` — 4/4 passing
- `cargo test -p gallery-smoke --test registry_roundtrip` — 4/4 passing
- `cargo test -p marionette-macros --lib title_case` — 7/7 passing
- `cargo test -p gallery-smoke --test ui_errors` — 4/4 trybuild fixtures compile-fail with expected text
- `cargo build --workspace --exclude crm-demo --features gallery` — exit 0
- `cargo test --workspace --exclude crm-demo --features gallery` — all `test result: ok` with zero failures
- `cargo clippy -p marionette-macros -p gallery-smoke --all-features --tests -- -D warnings` — exit 0
- `cargo clippy -p marionette --lib --all-features -- -D warnings` — exit 0

---
*Phase: 17-gallery-crate-skeleton-colocated-built-in-demos*
*Completed: 2026-04-22*
