---
phase: 16
plan: "04"
status: complete
requirements: [FRAME-02]
completed: 2026-04-21
---

# Phase 16 Plan 04 — Docs Closure

## Objective

Close the documentation loop on Phase 16. FRAME-02's "choice logged in PROJECT.md Key Decisions with rationale" clause needed a concrete Key Decisions row. STATE.md's "Registration library selection" blocker needed to be marked resolved. Phase 17's planner needed an explicit hand-off note warning about the D-C1 default-from-fn-ident key collision at scale when every built-in's demo fn shares the `gallery_demo()` ident.

## Requirements Addressed

- **FRAME-02** — "registration-library choice recorded in a Key Decisions entry with rationale." Satisfied by the new PROJECT.md Key Decisions row naming `linkme` with rationale (type-safe distributed-slice, zero runtime cost, explicit mental model, + the stable-iteration-order design decision to sort at iteration time rather than delegate to linkme).

## What Shipped

### `.planning/PROJECT.md`

- Appended one row to the Key Decisions table (after the `Pure fn() -> Node demo contract` row) naming `linkme over inventory for gallery-demo registry` with explicit rationale and outcome `✓ Good — v1.2 Phase 16`. Row cites `CONTEXT.md D-A1` as the decision record for traceability.

### `.planning/STATE.md`

Three surgical edits:
1. **Blockers/Concerns:** "Registration library selection" blocker rewritten as a resolved entry (prefixed with ✅) pointing at CONTEXT.md D-A1 + 16-01-PLAN.md for the implementation detail.
2. **Accumulated Context → Decisions:** appended a new `[v1.2 Phase 16]` decision bullet capturing the linkme choice + stable-iteration-order ownership.
3. **Accumulated Context:** inserted a new `### Phase 17 hand-off (from Phase 16)` subsection warning the next planner that Phase 17's DEMO-01 convention (every built-in's demo fn named `gallery_demo()`) would mass-collide on default-derived keys, so every `#[gallery_demo]` annotation in `backend/crates/marionette/src/builders/` MUST use an explicit `key = "..."` override. The natural convention is to match each builder's `#[component(type = "…")]` string (e.g. `#[gallery_demo(key = "button")]`). Also flags that `gallery-demo` becomes the 6th workspace crate (not 5th), since `gallery-smoke` already occupies the 5th slot from Phase 16.
4. **Session Continuity:** `Stopped at:` and `Resume:` updated to reflect Phase 16 shipment and the natural next commands (`/gsd-verify-work 16` → `/gsd-plan-phase 17`).

## Commits

1. `<hash-1>` — `docs(16-04): add linkme Key Decisions row (FRAME-02 choice logged)`
2. `<hash-2>` — `docs(16-04): close registration-library blocker + Phase 17 hand-off note`

*(Hashes filled in by git; see commit log for exact values.)*

## Verification

Every acceptance criterion from the plan is green:

| Check | Result |
|-------|--------|
| `grep -q 'linkme over inventory for gallery-demo registry' .planning/PROJECT.md` | ✓ |
| `grep -q 'D-A1' .planning/PROJECT.md` | ✓ |
| `grep -qi 'Registration library selection (resolved' .planning/STATE.md` | ✓ |
| `grep -q 'Phase 17 hand-off' .planning/STATE.md` | ✓ |
| `grep -q 'key = "button"' .planning/STATE.md` | ✓ |
| `! grep -qi 'inventory vs linkme decision deferred' .planning/STATE.md` | ✓ (old text removed) |
| `grep -q '16-01-PLAN.md' .planning/STATE.md` | ✓ |
| `grep -qi 'Phase 16 shipped' .planning/STATE.md` | ✓ |

## Key-Files Modified

- `.planning/PROJECT.md` (one new row)
- `.planning/STATE.md` (three surgical edits: blocker closure, accumulated-context append + hand-off section, session-continuity refresh)

## Deviations

None. Plan 04 was trivial documentation work; executed inline without spawning a subagent (no code, no tests, no build artifacts — inline execution is more efficient than spawning-to-worktree-to-merge for 2 doc-only tasks).
