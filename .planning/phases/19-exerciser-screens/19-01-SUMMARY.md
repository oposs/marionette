---
phase: 19-exerciser-screens
plan: 01
subsystem: infra
tags: [lucide-icons, svelte, vitest, tokio, rust, gallery-demo, linkme, action-router, patch-probe]

# Dependency graph
requires:
  - phase: 17-gallery-crate-skeleton-colocated-built-in-demos
    provides: "gallery-demo crate + linkme DEMOS registry + ActionRouter builder pattern"
  - phase: 18-catalog-screens
    provides: "synthetic_rows(n) generator + Container.icon() primitive + gallery_demo() fn contract"
provides:
  - "17 lucide icons registered in frontend icon registry (16 UI-SPEC + rotate-ccw)"
  - "installPatchProbe() hook in frontend init.ts — one-slot callback fired with per-patch latencyMs"
  - "GalleryState extended with exer02_loop / exer02_cadence_ms / exer02_tick fields"
  - "state() once-cell singleton returning &'static GalleryState — handler-accessible without HandlerContext plumbing"
  - "pub mod exerciser scaffold + 3 #[gallery_demo] stubs (exer-01/02/03) registered in linkme DEMOS"
  - "7 ActionRouter routes registered on gallery-demo (exer-01/report, exer-02/start|pause|reset|tick, exer-03/report-perf|remeasure)"
  - "exer-03-synthetic fetch-rows source arm at /demo/exer-03/rows (10_000-row cap)"
  - "3 seed_for_key arms (exer-01 observation matrix, exer-02 4 PENDING invariants, exer-03 4×20=80 form fields + empty rows)"
  - "router-dispatch reachability guard test for gallery-demo/exer-02/tick (Plan 19-01 -> 19-03 handoff contract)"
affects: [19-02-PLAN, 19-03-PLAN, 19-04-PLAN, 19-05-PLAN]

# Tech tracking
tech-stack:
  added:
    - "tokio::task::JoinHandle (already in tree; new usage shape for EXER-02 loop handle)"
    - "std::sync::OnceLock singleton pattern in gallery-demo (new in this crate)"
  patterns:
    - "patchProbe single-slot callback hook: module-local null-default, replace via setter, invoked post-applyPatch with performance.now delta"
    - "state() singleton Option B per 19-PATTERNS.md §AppState integration gap — dev-local state bypass for AppState extension"
    - "router-dispatch integration test in #[cfg(test)] mod router_tests — build via register_gallery_actions, dispatch ActionMessage through real ActionRouter, assert Ok(vec![]) = empty Vec<ProtocolMessage>"
    - "Wave 1 framework-polish plan pattern: bundle all shared-file edits into a single sequential plan so Wave 2 parallel plans touch disjoint files"

key-files:
  created:
    - "frontend/src/lib/init.patchprobe.test.ts"
    - "backend/crates/gallery-demo/src/exerciser/mod.rs"
    - "backend/crates/gallery-demo/src/exerciser/nested_appshell.rs"
    - "backend/crates/gallery-demo/src/exerciser/rapid_patching.rs"
    - "backend/crates/gallery-demo/src/exerciser/pathological_scale.rs"
    - "backend/crates/gallery-demo/src/handlers/exer01.rs"
    - "backend/crates/gallery-demo/src/handlers/exer02.rs"
    - "backend/crates/gallery-demo/src/handlers/exer03.rs"
  modified:
    - "frontend/src/lib/registry/icons.ts (14 icons → 31)"
    - "frontend/src/lib/init.ts (installPatchProbe + wrapped patch handler)"
    - "backend/crates/gallery-demo/src/lib.rs (+pub mod exerciser)"
    - "backend/crates/gallery-demo/src/state.rs (3 new fields + state() singleton)"
    - "backend/crates/gallery-demo/src/handlers/fetch_rows.rs (+exer-03-synthetic arm + 2 tests)"
    - "backend/crates/gallery-demo/src/handlers/show.rs (+3 seed arms + seed_exer_03 helper + 4 tests)"
    - "backend/crates/gallery-demo/src/handlers/mod.rs (+3 pub mod + 7 .action() regs + 3 router-dispatch tests)"
    - ".planning/phases/19-exerciser-screens/19-VALIDATION.md (Per-Task Verification Map populated)"

key-decisions:
  - "17 icons registered (not 16): the UI-SPEC §Design System table includes rotate-ccw as a row but its summary line says 'Total new icons: 16'. The plan explicitly ships rotate-ccw too since UI-SPEC references it at lines 223 and 277-278 as the Reset / Remeasure CTA icon. 17 is the correct contract — UI-SPEC summary line is a typo."
  - "Once-cell state() singleton chosen over AppState extension (19-PATTERNS.md Option B). marionette::ws::AppState cannot gain a gallery-specific slot without a framework-crate edit (out of scope per 19-CONTEXT.md §D-4). Dev-local singleton is a targeted, reversible bypass."
  - "Client-initiated tick for EXER-02 (A1 resolution). Verified in Plan 19-01 research: AppState has no broadcast channel; extending it is out of scope. Frontend will sendAction('gallery-demo/exer-02/tick') every cadence_ms; each tick receives one PatchMessage via the normal ActionResult return path. Route reserved + reachability-verified in this plan; full loop logic ships in Plan 19-03."
  - "seed_exer_03 extracted as helper fn (not inlined). 80 field defaults across 4 groups = ~70 lines; inlining would have pushed seed_for_key past clippy::too_many_lines (already at the limit). Helper keeps seed_for_key readable as a registry dispatch."
  - "#[allow(clippy::too_many_lines)] on seed_for_key — registry-style dispatch with short parallel arms; splitting into per-key fns would obscure the per-key seed correspondence which is already hard-contracted against bind paths (Phase 17 G-05 lesson)."

patterns-established:
  - "Wave-1 framework-polish plan: bundles all shared-file edits (handlers/mod.rs, show.rs, fetch_rows.rs, lib.rs, state.rs) into a single sequential plan so Wave 2 parallel plans touch only disjoint files. Contention-free Wave 2 is the payoff."
  - "Stub handler + route registration as handoff contract: Plan N-01 registers routes whose handlers return Ok(vec![]); Plan N-03 replaces the handler body without touching handlers/mod.rs again. Router-dispatch test is the reachability guard."
  - "patchProbe pattern: one-slot module-local callback, invoked post-applyPatch with performance.now delta. Consumers (EXER-02 invariants, EXER-03 perf) pass-through, never poll."

requirements-completed: [EXER-01, EXER-02, EXER-03]

# Metrics
duration: 20m
completed: 2026-04-24
---

# Phase 19 Plan 19-01: Framework Polish Summary

**17 lucide icons + installPatchProbe hook (frontend); 3-field GalleryState extension + state() once-cell singleton + exerciser module scaffold with 3 registered stubs; 7 ActionRouter routes + 3 seed_for_key arms + exer-03-synthetic fetch-rows arm at 10 000-row cap — all Wave 2 plans unblocked, disjoint-file.**

## Performance

- **Duration:** 20m (from execution start to final commit; includes first-time pnpm install + cold cargo build ≈ 4 min)
- **Started:** 2026-04-24T09:13:35Z
- **Completed:** 2026-04-24T09:33:53Z
- **Tasks:** 3
- **Files created:** 8
- **Files modified:** 8 (+1 doc: 19-VALIDATION.md)

## Accomplishments

- Frontend icon registry now carries 31 total icons (14 pre-existing + 17 new). All 16 UI-SPEC §Design System icons + rotate-ccw are registered; the unknown-icon fallback path correctly returns CircleHelp.
- `installPatchProbe(fn|null)` exported from init.ts. The patch handler now measures per-patch latencyMs via `performance.now()` and forwards to the installed probe — zero overhead when no probe is installed (single null check), no probe storms when the probe slot is cleared.
- `GalleryState` extended with Phase 19 EXER-02 fields: `exer02_loop: Arc<Mutex<Option<JoinHandle<()>>>>`, `exer02_cadence_ms: Arc<Mutex<u64>>` (default 500), `exer02_tick: Arc<Mutex<u64>>`. Defaults verified by a unit test.
- `state() -> &'static GalleryState` once-cell accessor added. Ptr-identity test confirms a single instance. This is Option B from 19-PATTERNS.md §AppState integration gap — avoids a marionette framework-crate edit that 19-CONTEXT.md §D-4 puts out of scope.
- `pub mod exerciser` + 3 sibling stub files (`nested_appshell.rs`, `rapid_patching.rs`, `pathological_scale.rs`) each carrying a `#[gallery_demo(key = "exer-0N")]` fn. The linkme `DEMOS` registry now surfaces all three keys; the existing `navigate_shell_render_includes_one_nav_item_per_registered_demo` integration test stays green, which means the gallery nav auto-surfaces the three exerciser entries on first boot.
- `handlers/fetch_rows.rs` gained the `exer-03-synthetic` source arm — path `/demo/exer-03/rows`, cap `synthetic_rows(10_000)`, same row-shape + injected `actions` array as `catalog-synthetic-rows`. First-page (1..=50) and last-page (9951..=10_000) tests both pass. Unknown sources still reject with `ActionError::BadPayload` (regression guard test unchanged).
- `handlers/show.rs::seed_for_key` gained three arms: `exer-01` (observation matrix with 3 FAIL + 1 WARN dimensions), `exer-02` (4 PENDING invariants + 500 ms cadence + empty focused value + elapsed-s = 0), and `exer-03` (empty rows, nullable perf readouts, 4 groups × 20 fields = 80 total). All 4 new seed tests pass including the field-count=80 UI-SPEC contract.
- `handlers/mod.rs` gained 3 `pub mod` declarations + 7 `.action(...)` route registrations: `gallery-demo/exer-01/report`, `gallery-demo/exer-02/start|pause|reset|tick`, `gallery-demo/exer-03/report-perf|remeasure`. All routes map to stub handlers returning `Ok(vec![])` — Plans 19-02/03/04 replace the bodies without touching mod.rs again.
- Added a `router_tests` module with three tokio tests that build the router via `register_gallery_actions(ActionRouter::new())` and dispatch real `ActionMessage`s: `exer_02_tick_route_is_reachable_and_returns_empty` (the explicit 19-01 → 19-03 handoff guard), `exer_02_tick_route_is_not_a_not_found_error` (belt-and-suspenders), and `all_seven_phase19_exerciser_routes_are_reachable` (broader Wave 2 reachability guard).
- Resolved 19-RESEARCH.md §Open Question A1 (HIGH-urgency UNRESOLVED): **client-initiated tick**. Verified `AppState` in `backend/crates/marionette/src/ws.rs:24-33` has no broadcast channel; each connection has a private `mpsc::Sender<ProtocolMessage>` (ws.rs:61). Extending `AppState` would modify the framework crate, which 19-CONTEXT.md §D-4 puts out of scope. Resolution: the frontend will call `sendAction('gallery-demo/exer-02/tick')` every cadence_ms after Start patching; `handle_exer02_tick` returns the per-tick `PatchMessage` via the normal `ActionResult` return path. The wire still carries real `PatchMessage`s (not frontend-synthesized ones), so PATCH-02's WebSocket-patch-wire acceptance is satisfied. Route `gallery-demo/exer-02/tick` reserved + reachability-verified in this plan; full loop logic ships in Plan 19-03.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add 16 lucide icons and install patchProbe hook** — `f27d680` (feat)
2. **Task 2: Extend GalleryState + state() singleton + exerciser module scaffold** — `4844410` (feat)
3. **Task 3: exer-03-synthetic fetch-rows arm + 3 seed arms + 3 stub handlers + 7 route regs** — `9b73ef9` (feat)

_No TDD RED/GREEN/REFACTOR split required — each task already shipped tests alongside production code in a single commit, and the plan's TDD `tdd="true"` flag was honored inside each commit (tests + implementation written together)._

## Files Created/Modified

### Created

- `frontend/src/lib/init.patchprobe.test.ts` — 5 vitest cases covering probe fire/silence + 17-icon registration smoke test
- `backend/crates/gallery-demo/src/exerciser/mod.rs` — module root, 3 `pub mod` declarations
- `backend/crates/gallery-demo/src/exerciser/nested_appshell.rs` — `#[gallery_demo(key = "exer-01")]` stub
- `backend/crates/gallery-demo/src/exerciser/rapid_patching.rs` — `#[gallery_demo(key = "exer-02")]` stub
- `backend/crates/gallery-demo/src/exerciser/pathological_scale.rs` — `#[gallery_demo(key = "exer-03")]` stub
- `backend/crates/gallery-demo/src/handlers/exer01.rs` — `handle_exer01_report` stub
- `backend/crates/gallery-demo/src/handlers/exer02.rs` — `handle_exer02_start|pause|reset|tick` stubs
- `backend/crates/gallery-demo/src/handlers/exer03.rs` — `handle_exer03_report_perf|remeasure` stubs

### Modified

- `frontend/src/lib/registry/icons.ts` — 17 new lucide imports + registrations (14 → 31 total)
- `frontend/src/lib/init.ts` — `installPatchProbe(fn|null)` export + wrapped patch handler with `performance.now()` latency probe
- `backend/crates/gallery-demo/src/lib.rs` — `pub mod exerciser;` between catalog and fixtures (alphabetical)
- `backend/crates/gallery-demo/src/state.rs` — 3 new fields on `GalleryState` + `fn state() -> &'static GalleryState` singleton + 2 unit tests
- `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` — `exer-03-synthetic` arm (10 000-row cap) + 2 tests (first page / last page)
- `backend/crates/gallery-demo/src/handlers/show.rs` — 3 seed arms (exer-01/02/03) + `seed_exer_03()` helper + 4 tests
- `backend/crates/gallery-demo/src/handlers/mod.rs` — 3 `pub mod` + 7 `.action(...)` registrations + `router_tests` module (3 dispatch tests)
- `.planning/phases/19-exerciser-screens/19-VALIDATION.md` — populated Per-Task Verification Map with Plan 19-01 rows; checked off Wave 0 requirements now satisfied

## Decisions Made

- **17 icons, not 16.** UI-SPEC §Design System lists `rotate-ccw` as a table row (Reset / Remeasure CTA) but the summary line below says "Total new icons: 16" — this is a UI-SPEC typo. The plan's grep counts explicitly expect 17; shipping 17 matches the plan + UI-SPEC table literal.
- **Once-cell `state()` singleton instead of AppState extension** per 19-PATTERNS.md §AppState integration gap (Option B). Rationale: extending `marionette::ws::AppState` would require a framework-crate edit, which 19-CONTEXT.md §D-4 puts out of scope. The singleton is a dev-local, reversible targeted bypass.
- **Client-initiated tick for EXER-02 (A1 resolution)** — see Accomplishments bullet above. Resolves the HIGH-urgency open question from 19-RESEARCH.md without modifying the `marionette` framework crate.
- **`seed_exer_03()` extracted as a helper fn** rather than inlined in `seed_for_key`. 80 field defaults spread across 4 `json!` blocks is ~70 lines; inlining pushed `seed_for_key` past clippy's `too_many_lines` threshold. The helper also keeps `seed_for_key` readable as a pure registry dispatch.
- **`#[allow(clippy::too_many_lines)]` on `seed_for_key`** — even after extracting `seed_exer_03`, `seed_for_key` is 131 lines because each catalog/exerciser arm carries doc comments explaining its bind-path contract (Phase 17 G-05 hard-learned lesson). Splitting the dispatch into per-key fns would obscure the per-key seed correspondence; scoped allow with rationale comment.
- **Router-dispatch test module in `handlers/mod.rs`, not a separate `tests/routes.rs`.** The tests need `super::register_gallery_actions` visibility without re-exporting it; an in-file `#[cfg(test)] mod router_tests` is the minimum-friction location. No new integration test binary needed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Install frontend node_modules in worktree**
- **Found during:** Task 1 verification (`pnpm exec vitest` failed with "Command not found")
- **Issue:** The worktree is a freshly-cloned git worktree without a `frontend/node_modules/` directory; vitest is not installed.
- **Fix:** Ran `pnpm install` (no frozen lockfile because the repo has never committed a `pnpm-lock.yaml`). Install produced `frontend/pnpm-lock.yaml` as a side effect; left uncommitted to honor the repo's existing convention of not tracking the lockfile.
- **Files modified:** `frontend/pnpm-lock.yaml` (generated, NOT committed)
- **Verification:** `pnpm exec vitest run src/lib/init.patchprobe.test.ts` now exits 0 with 5/5 passing.
- **Committed in:** N/A — side-effect artifact left uncommitted, mirrors repo convention.

**2. [Rule 3 - Blocking] Clippy `similar_names` error on `router`/`routes` locals**
- **Found during:** Task 3 clippy verification (`cargo clippy --features gallery --all-targets -- -D warnings`).
- **Issue:** `similar_names` fires on `let router = ...; let routes = [...]` — the two locals differ by only one character.
- **Fix:** Renamed to `dispatcher` and `route_names`.
- **Files modified:** `backend/crates/gallery-demo/src/handlers/mod.rs` (router_tests module)
- **Verification:** `cargo clippy --features gallery --all-targets -- -D warnings` exits 0.
- **Committed in:** `9b73ef9` (Task 3 commit; applied inline before the commit, not a separate fix-up commit).

**3. [Rule 3 - Blocking] Clippy `too_many_lines` error on `seed_for_key`**
- **Found during:** Task 3 clippy verification.
- **Issue:** `seed_for_key` grew to 131 lines with the three new Phase 19 arms — over clippy's 100-line threshold.
- **Fix:** Added scoped `#[allow(clippy::too_many_lines)]` with a rationale comment: the function is a registry-style dispatch where per-key seed correspondence is more valuable than line-count hygiene. Also extracted `seed_exer_03()` as a helper fn so the `seed_for_key` growth is limited to a single-arm call site.
- **Files modified:** `backend/crates/gallery-demo/src/handlers/show.rs`
- **Verification:** `cargo clippy --features gallery --all-targets -- -D warnings` exits 0.
- **Committed in:** `9b73ef9` (Task 3 commit).

---

**Total deviations:** 3 auto-fixed (all Rule 3 — blocking issues, no architectural change needed).
**Impact on plan:** None. Deviation 1 is a worktree-hygiene artifact (missing node_modules). Deviations 2 and 3 are clippy lints the plan's action script didn't anticipate — mechanical fixes applied inline without changing the public API or acceptance contract. Zero scope creep; all three unblocked cargo/clippy/vitest verification paths.

## Issues Encountered

- **`pnpm install` took ~50 s** on first run inside the worktree (cold cache). Not a plan issue — simply the first-time cost of a fresh worktree. Subsequent executor runs in this worktree should skip.
- **`cargo build -p gallery-demo --features gallery`** cold-built for ~3 min on first run (Task 2 verification); warm rebuilds ran in ~10 s after that. Again, first-time-worktree cost.
- **No breaking changes** to existing cargo/vitest tests. Before Plan 19-01: 70 unit tests passing. After Plan 19-01: 79 unit tests passing (70 + 9 new: 2 state, 4 show seed, 2 fetch_rows exer03, + 3 router_tests dispatches). The `navigate_shell_render_includes_one_nav_item_per_registered_demo` integration test is green with the three new exerciser keys automatically surfacing in auto-nav.

## Authentication Gates

None — the plan executes against a dev-local gallery harness with `AuthRequirement::None` on every new route (matches the crate-wide single-tenant anonymous-session posture from CRATE-01).

## User Setup Required

None — no external services, no environment variables, no dashboard configuration needed. Verify locally:

```bash
cd backend && cargo test -p gallery-demo --features gallery
cd frontend && pnpm exec vitest run src/lib/init.patchprobe.test.ts
cd backend && cargo clippy -p gallery-demo --features gallery --all-targets -- -D warnings
```

## Threat Flags

None new. The plan's threat register (T-19-01 through T-19-04) is accounted for:

- T-19-01 (DoS via cadence_ms): Plan 19-01 stubs do not read cadence_ms — stub-safe. Clamp ships with Plan 19-03 per the mitigation disposition.
- T-19-02 (Tampering on report_perf payload): stubs do not deserialise the payload. Accept disposition still valid — serde's `Option<f64>` contract enforced by Plan 19-04.
- T-19-03 (10k row allocation): `exer-03-synthetic` arm uses `crate::fixtures::synthetic_rows(10_000)` which allocates 10 000 structs per fetch-rows call. Accept disposition per plan.
- T-19-04 (info disclosure): deterministic LCG; no PII. No change.

## Self-Check: PASSED

Verification of claims in this SUMMARY (executed after writing):

**Files claimed created — all present:**
```
FOUND: frontend/src/lib/init.patchprobe.test.ts
FOUND: backend/crates/gallery-demo/src/exerciser/mod.rs
FOUND: backend/crates/gallery-demo/src/exerciser/nested_appshell.rs
FOUND: backend/crates/gallery-demo/src/exerciser/rapid_patching.rs
FOUND: backend/crates/gallery-demo/src/exerciser/pathological_scale.rs
FOUND: backend/crates/gallery-demo/src/handlers/exer01.rs
FOUND: backend/crates/gallery-demo/src/handlers/exer02.rs
FOUND: backend/crates/gallery-demo/src/handlers/exer03.rs
```

**Commits claimed — all present in worktree branch:**
```
FOUND: f27d680  feat(19-01): add 17 lucide icons + installPatchProbe hook (Task 1)
FOUND: 4844410  feat(19-01): extend GalleryState + exerciser module scaffold (Task 2)
FOUND: 9b73ef9  feat(19-01): exer-03 fetch-rows arm + 3 seed arms + 7 route regs (Task 3)
```

**Tests claimed passing — confirmed:**
```
cargo test -p gallery-demo --features gallery: 79 unit passed + 1 nav_auto_discovery + 1 smoke_boot (81 total, 0 failed)
cargo clippy -p gallery-demo --features gallery --all-targets -- -D warnings: exit 0
pnpm exec vitest run src/lib/init.patchprobe.test.ts: 5/5 passed
pnpm check: 0 errors 0 warnings
```

## Next Phase Readiness

- Wave 2 plans 19-02 (EXER-01), 19-03 (EXER-02), 19-04 (EXER-03) are **unblocked**. Each can now run in parallel, touching disjoint files:
  - 19-02 edits `exerciser/nested_appshell.rs` + `handlers/exer01.rs` body.
  - 19-03 edits `exerciser/rapid_patching.rs` + `handlers/exer02.rs` body.
  - 19-04 edits `exerciser/pathological_scale.rs` + `handlers/exer03.rs` body.
  - Shared files (`handlers/mod.rs`, `state.rs`, `show.rs`, `fetch_rows.rs`, `lib.rs`) are now immutable for Wave 2.
- **A1 resolution locked**: Plan 19-03 builds on client-initiated tick (route `gallery-demo/exer-02/tick` + `state()` singleton). No framework-crate edits needed.
- **patchProbe consumers** (Plans 19-03 + 19-04) can `import { installPatchProbe } from '$lib/init'` and pass a callback — the probe slot is live from this plan's ship.
- **No blockers carried forward.** Worktree-local `frontend/pnpm-lock.yaml` is a hygiene artifact, not a plan concern.

---
*Phase: 19-exerciser-screens*
*Plan: 19-01*
*Completed: 2026-04-24*
