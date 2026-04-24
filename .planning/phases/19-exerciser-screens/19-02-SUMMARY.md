---
phase: 19-exerciser-screens
plan: 02
subsystem: ui
tags: [exerciser, appshell, sidebar-provider, svelte, sdui, vitest, observation-matrix, framework-seed]

# Dependency graph
requires:
  - phase: 19-01
    provides: exerciser module scaffold, GalleryState data subtree, 7 exer-01/02/03 route stubs (incl. exer-01/report + exer-01/open-seed placeholders), 17 lucide icons
  - phase: 17-gallery-crate-skeleton-colocated-built-in-demos
    provides: G-02 diagnosis of AppShell nestability + Phase 17 static-preview workaround (which EXER-01 pointedly does NOT repeat)
provides:
  - Real nested AppShell demo at gallery nav "Exerciser: Nested AppShell" (exactly one AppShell::new() invocation inside a Container wrap inside the outer gallery AppShell's content slot)
  - 4-dimension observation matrix Card with FAIL/WARN/PASS badges bound to /demo/exer-01/matrix/{dim}/details
  - v1.3 proposal Card with "Open seed draft" CTA that toasts the seed path
  - Backend handler handle_exer01_report (strict serde, 8 Set ops) + handle_exer01_open_seed (2-op toast patch)
  - Frontend probeNestability() module that captures 4 dimensions at runtime and reports via sendAction
  - DEV-gated window.__mrnExer01OuterSidebar hook on Sidebar.Provider exposing outer state to the probe
  - v1.3 framework-extension seed at .planning/seeds/v1.3-appshell-nestability.md (Problem / Proposed scope / Acceptance) — hand-off artifact that Plan 19-02 is the direct producer of per D-1
affects: [19-03-rapid-patching, 19-04-pathological-scale, 19-05 (phase close-out), future v1.3 "scoped surface name" framework work]

# Tech tracking
tech-stack:
  added:
    - jsdom (vitest env for frontend probe tests; dev-dep already declared in package.json, activated here via @vitest-environment docblock)
    - uuid (already present; used for toast id generation — verbatim pattern from handlers/toast.rs)
  patterns:
    - "Module-level MutationObserver auto-arm for one-shot DOM probes — no SDUI-tree coordination, no Svelte component, self-contained in a plain TS module"
    - "DEV-gated window hook with first-mount guard for nested-provider diagnostics (write-once so the inner mount doesn't clobber the outer handle)"
    - "Inline toast-emission pattern: SetNode(Button) + InsertChild(toasts-root) surface=toasts — byte-for-byte copy of handlers/toast.rs::handle_toast_fire (not extracted to a helper; only 2 call sites today)"
    - "Strict serde Deserialize on action payloads with map_err→ActionError::BadPayload as the T-19-02-01 Tampering mitigation"
    - "Exerciser that is DELIBERATELY BROKEN: the demo renders the known-broken nested-AppShell collision and ships a v1.3 seed instead of a v1.2 fix (per 19-CONTEXT.md §D-1)"

key-files:
  created:
    - frontend/src/lib/exer01/observe.svelte.ts
    - frontend/src/lib/exer01/observe.test.ts
    - .planning/seeds/v1.3-appshell-nestability.md
  modified:
    - backend/crates/gallery-demo/src/exerciser/nested_appshell.rs (Task 1: replaced Plan 19-01 stub with full EXER-01 implementation)
    - backend/crates/gallery-demo/src/handlers/exer01.rs (Task 2: replaced stub with handle_exer01_report + handle_exer01_open_seed + 4 tests)
    - backend/crates/gallery-demo/src/handlers/mod.rs (added gallery-demo/exer-01/open-seed route registration)
    - frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte (DEV-gated window.__mrnExer01OuterSidebar hook with first-mount guard)
    - .planning/phases/19-exerciser-screens/19-VALIDATION.md (Per-Task Verification Map rows for 19-02-T1 and 19-02-T2)

key-decisions:
  - "sidebar-provider.svelte hosts the __mrnExer01OuterSidebar hook (not AppShell.svelte) — AppShell delegates to Sidebar.Provider for the context, so the hook must sit in the file that owns setContext(Symbol.for('scn-sidebar'))"
  - "MutationObserver on document.body is the mount strategy (not a Svelte component, not +layout route detection, not a framework-crate onMount hook) — keeps the probe self-contained in the observe.svelte.ts module"
  - "First-mount guard via `if (w.__mrnExer01OuterSidebar === undefined)` ensures the inner Sidebar.Provider's setSidebar does NOT clobber the outer handle — Svelte mounts outer before inner"
  - "@vitest-environment jsdom docblock instead of changing the repo-wide vite.config.ts test.environment (which is 'node' by design) — keeps the jsdom overhead scoped to the one file that needs it"
  - "Inline toast-emission (not a toast helper extracted from handlers/toast.rs) — only 2 call sites today; extraction is premature"

patterns-established:
  - "Nested-provider diagnostics pattern: outer provider writes state handle to window under DEV, inner provider leaves it alone via first-mount guard, probe compares identity"
  - "One-shot DOM probe without framework coupling: MutationObserver → setTimeout(100ms settle) → probe → disconnect"
  - "Exerciser-as-evidence-capture: observation matrix bound to /demo/{screen}/matrix/{dim}/details paths that the probe overwrites on mount — Text.bind drives the DOM update"

requirements-completed: [EXER-01]

# Metrics
duration: ~38min (Task 1 + Task 2 end-to-end across the split sessions)
completed: 2026-04-24
---

# Phase 19 Plan 02: Nested AppShell Exerciser Summary

**Real nested AppShell demo + 4-dimension observation matrix probe + v1.3 scoped-surface-name framework-extension seed — EXER-01 ships the broken-nesting collision as live evidence and hands off to v1.3 instead of attempting a v1.2 fix.**

## Performance

- **Duration:** ~38 min (end-to-end; two-session continuation: first agent completed Task 1 + drafted Task 2 artifacts, second agent finished Task 2 validation + committed)
- **Started:** Task 1 committed at `f46a865` (prior session)
- **Completed:** 2026-04-24T10:05:22Z (SUMMARY write)
- **Tasks:** 2
- **Files modified:** 6 (2 new backend impl, 2 new frontend impl, 1 seed, 1 existing svelte file)
- **Commits:** 2 task commits (f46a865, 154c036)

## Accomplishments

- **Task 1 (nested_appshell.rs):** 3-Card exerciser screen with exactly one `AppShell::new()` invocation (inner shell wrapped in `#exer-01-inner-wrap` container; outer gallery AppShell hosts it via content slot). 4-cell observation matrix grid + v1.3 proposal Card with "Open seed draft" CTA targeting `gallery-demo/exer-01/open-seed`. 7 cargo tests green (root id, outer class, exactly-one-AppShell guard, 4 matrix dimension ids, CTA action string, registered_demos hookup, locked copy).
- **Task 2 (handlers + probe + seed):**
  - `handle_exer01_report` deserializes a 4-dimension `ObservationReport` (serde rename with hyphens) and emits 8 Set ops — 4 dimension roots + 4 `/details` subpaths (the latter is the `bind` target of each findings Text node). Rejects malformed payloads with `ActionError::BadPayload` (T-19-02-01 mitigation).
  - `handle_exer01_open_seed` emits a 2-op toast patch (SetNode Button + InsertChild into `toasts-root`) whose label carries the seed path. Copied verbatim from `handlers/toast.rs::handle_toast_fire`.
  - `observe.svelte.ts` probes 4 dimensions at mount: provider-context identity (`getContext(Symbol.for('scn-sidebar'))` vs `window.__mrnExer01OuterSidebar`), `--sidebar-width` inheritance, synthetic `Ctrl+B` keyboard-shortcut scoping, viewport < 768 mobile check. Auto-armed via MutationObserver on `document.body`.
  - `sidebar-provider.svelte` exposes its `sidebar` state on `window.__mrnExer01OuterSidebar` with a DEV gate + first-mount guard.
  - `.planning/seeds/v1.3-appshell-nestability.md` written with Problem / Proposed scope / Acceptance sections proposing per-surface context keys, CSS token scoping via `:where([data-surface])`, surface-named mobile-sheet portals, and active-surface-aware keyboard shortcuts.

## Task Commits

1. **Task 1: Build EXER-01 nested_appshell.rs** — `f46a865` (feat)
2. **Task 2: handlers + observe probe + sidebar-provider hook + v1.3 seed** — `154c036` (feat)
3. **Plan metadata (SUMMARY + VALIDATION update):** — pending next commit

## Files Created/Modified

**Created:**

- `frontend/src/lib/exer01/observe.svelte.ts` — probeNestability() + auto-arming MutationObserver
- `frontend/src/lib/exer01/observe.test.ts` — vitest happy-path with mocked sendAction + getContext
- `.planning/seeds/v1.3-appshell-nestability.md` — v1.3 framework-extension seed (scoped-surface-name proposal)

**Modified:**

- `backend/crates/gallery-demo/src/exerciser/nested_appshell.rs` — stub → full EXER-01 (3-Card, 7 tests)
- `backend/crates/gallery-demo/src/handlers/exer01.rs` — stub → handle_exer01_report + handle_exer01_open_seed + 4 tests
- `backend/crates/gallery-demo/src/handlers/mod.rs` — added `gallery-demo/exer-01/open-seed` route
- `frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte` — DEV-gated `window.__mrnExer01OuterSidebar` hook
- `.planning/phases/19-exerciser-screens/19-VALIDATION.md` — populated 19-02-T1 and 19-02-T2 verification rows

## Decisions Made

- **AppShell hook location:** Plan's `files_modified` list named `frontend/src/lib/components/layout/AppShell.svelte`, but 19-02-PLAN Task 2 Part C body explicitly says "most likely `frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte`" if AppShell doesn't itself call `setContext`. AppShell delegates to `Sidebar.Provider`, which is the actual owner of the sidebar context. Hook ships in `sidebar-provider.svelte`. See "Deviations from Plan" below for the file-drift flag.
- **First-mount guard semantics:** `if (w.__mrnExer01OuterSidebar === undefined)` guarantees the OUTER provider writes its handle and the INNER provider leaves it alone. Svelte lifecycle mounts outer first (top-down), so the first-to-write is always the outer.
- **Vitest jsdom scoping:** Added `// @vitest-environment jsdom` docblock at the top of `observe.test.ts` instead of changing the repo-wide `vite.config.ts` test environment. Rationale: `vite.config.ts` ships `environment: 'node'` as the default; flipping it to jsdom globally would slow every existing test file. Per-file opt-in via docblock is the recommended vitest idiom.
- **MutationObserver mount strategy over Svelte-component alternatives:** Evaluated (1) ProbeMount.svelte + registered component type, (2) Inner-AppShell onMount hook via framework-crate change, (3) `+layout.svelte` route detection. All three were rejected as too invasive for a one-shot probe. MutationObserver on `document.body` is self-contained to the observe.svelte.ts module.
- **Inline toast emission:** `handle_exer01_open_seed` emits the 2-op toast patch inline (verbatim copy of `handlers/toast.rs::handle_toast_fire`) rather than extracting a `toast_helper` module. With only 2 call sites today, a helper is premature; extraction can happen when a third call site lands.

## Deviations from Plan

### File Drift (Flag for Verifier)

**1. [Advisory — file-drift flag] `sidebar-provider.svelte` modified instead of `AppShell.svelte`**

- **Found during:** Task 2 Part C
- **Issue:** Plan 19-02 frontmatter `files_modified` lists `frontend/src/lib/components/layout/AppShell.svelte`, but the plan body (lines 757-768) explicitly notes that if AppShell doesn't call `getContext` directly, the hook moves to the file that owns `setContext(Symbol.for('scn-sidebar'))` — which is `sidebar-provider.svelte`. AppShell.svelte delegates the sidebar to `<Sidebar.Provider>`, so sidebar-provider.svelte IS the correct target.
- **Fix:** Hook placed in `sidebar-provider.svelte` (the correct file per the plan body). No change to `AppShell.svelte`. The plan's `files_modified` list is stale; the plan body is authoritative.
- **Files modified:** `frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte` (10-line DEV-gated block at end of `<script>` section, outside `onMount` — runs at component construction time during reactive `$state` setup, which happens for every provider instance).
- **Verification:** `grep -rn __mrnExer01OuterSidebar frontend/src/lib/` matches 3 files (sidebar-provider.svelte + observe.svelte.ts + observe.test.ts) — acceptance criterion requires ≥ 2, so 3 passes. Unit test passes without needing AppShell.svelte to be modified. Clippy + svelte-check + 4 cargo handler tests all green.
- **Committed in:** `154c036`

### Rule 3 — Blocking Issue (Auto-fixed)

**2. [Rule 3 — Blocking] jsdom environment not default; test failed with `ReferenceError: window is not defined`**

- **Found during:** Task 2 verification (running `pnpm exec vitest run src/lib/exer01/observe.test.ts`)
- **Issue:** Repo's `vite.config.ts` sets `test.environment: 'node'`, so the test couldn't access `window` or `document`.
- **Fix:** Added `// @vitest-environment jsdom` docblock as the first line of `observe.test.ts`. This opts the one file into jsdom without touching global config.
- **Files modified:** `frontend/src/lib/exer01/observe.test.ts`
- **Verification:** Test passes (`1 passed (1)`) in ~2s.
- **Committed in:** `154c036`

**3. [Rule 3 — Blocking] MutationObserver callback fired after vitest teardown, emitting `ReferenceError: document is not defined` as unhandled error**

- **Found during:** Task 2 first vitest run (test passed but unhandled error surfaced at the end)
- **Issue:** Module-level MutationObserver auto-arm outlives the test's `document` reference. When vitest tears down jsdom between tests, a pending mutation callback referenced `document` and threw.
- **Fix:** Added `if (typeof document === 'undefined') return;` guard inside the MutationObserver callback. Defensive — costs nothing in production (`document` is always defined in the browser) but prevents the vitest race.
- **Files modified:** `frontend/src/lib/exer01/observe.svelte.ts`
- **Verification:** Re-run vitest — clean pass, zero unhandled errors.
- **Committed in:** `154c036`

### Rule 2 — Missing Critical Functionality (Auto-added)

**4. [Rule 2 — Missing doc] clippy `missing_panics_doc` on `handle_exer01_report`**

- **Found during:** Task 2 clippy run
- **Issue:** `cargo clippy -- -D warnings` failed: `handle_exer01_report` contains `.expect("MatrixEntry serializes")`, which clippy flags as a possible panic lacking a `# Panics` doc section.
- **Fix:** Added a `/// # Panics` doc comment explaining the invariant: `MatrixEntry` is `{state: String, details: String}`, both of which serialize unconditionally, so the `.expect()` asserts an invariant rather than a fallible operation.
- **Files modified:** `backend/crates/gallery-demo/src/handlers/exer01.rs`
- **Verification:** `cargo clippy -p gallery-demo --features gallery --all-targets -- -D warnings` exits 0.
- **Committed in:** `154c036`

---

**Total deviations:** 4 (1 advisory file-drift flag, 2 Rule 3 blocking issues, 1 Rule 2 missing doc)

**Impact on plan:** None of the deviations change the plan's contract. The file-drift is a plan-body-vs-frontmatter mismatch the executor resolved correctly per the plan body's explicit contingency text. The two blocking issues were vitest-environment and test-teardown-race plumbing; fixing them was required to satisfy the plan's acceptance criteria verbatim. The missing panics doc was a clippy lint the plan's acceptance criteria required clean.

## Issues Encountered

- **`pnpm-lock.yaml` untracked:** Running `pnpm install` in the worktree created `frontend/pnpm-lock.yaml` which was not previously tracked (never been tracked in main). Left untracked — not part of Plan 19-02 scope, and the worktree will be destroyed on teardown. If the repo wants to start tracking the lockfile, that's a separate (cross-cutting) change.
- **Svelte 5 runes gotchas for outer-state comparison:** The probe compares `innerState === outerState` using object identity. Svelte 5 `$state`-wrapped objects retain a stable identity across re-renders, so `===` is correct. No `$state.snapshot` or unwrapping needed; the comparison works because the inner provider stores its own freshly-constructed `new Sidebar(...)` under the global key, which differs from the outer's instance.
- **File drift: plan frontmatter vs plan body:** See Deviation #1. Flagged for verifier.

## User Setup Required

None — no external services or env vars involved.

## Next Phase Readiness

- **Plan 19-03 (EXER-02 rapid-patching)** can proceed — its handler-stub scaffolding (shipped in Plan 19-01) is untouched by this plan, and the probe infrastructure (MutationObserver + sendAction) is a reference pattern 19-03 can reuse if it needs a similar DOM-observation harness.
- **v1.3 seed:** `.planning/seeds/v1.3-appshell-nestability.md` is planted and ready for a future v1.3 phase kickoff. It contains enough technical detail (builder API shape, Svelte Provider refactor, CSS scoping approach, web-platform alternatives weighed) that no re-research is required.
- **Manual UAT (Chrome MCP):** Out of scope for this plan's automated verify; captured in 19-VALIDATION.md §Manual-Only Verifications. The gallery nav entry "Exerciser: Nested AppShell" is discoverable via linkme `registered_demos()`, which is verified by cargo test. Chrome MCP end-to-end walkthrough is owed by Plan 19-05 (phase close-out).

## TDD Gate Compliance

Plan 19-02's tasks are marked `tdd="true"` but the plan's execution pattern was "write tests AND implementation together in one commit per task" (not the strict RED → GREEN two-commit cycle). Per 19-02-PLAN.md `<task>` bodies, the `<behavior>` block lists the intended test assertions and the `<action>` block writes both the test module and the production code in the same file. This matches the project's pattern for other gallery-demo exercisers (e.g., `catalog/buttons.rs:157-234` embeds tests in the same file as the demo).

Verified in commit history:
- `f46a865` is a `feat(...)` commit carrying both production code AND the 7 nested_appshell tests.
- `154c036` is a `feat(...)` commit carrying both production code AND the 4 handler tests + 1 vitest frontend test.

No explicit RED `test(...)` commit exists. Flagging for the verifier: this plan's pattern (test-and-impl-together) is consistent with Phase 18 and earlier exercisers, but it does deviate from the strict RED/GREEN gate sequence. Plan 19-02 frontmatter marks `type: execute` (not `type: tdd`), so plan-level TDD gate enforcement does NOT apply. Per-task `tdd="true"` is aspirational rather than gated.

## Known Stubs

None. All artifacts are production-ready. Four /demo/exer-01/matrix/{dim}/details paths initialize as empty Text nodes that the probe overwrites on mount — not a stub, the intended data-binding behavior.

## Threat Flags

None beyond those already in 19-02-PLAN.md `<threat_model>` (T-19-02-01, T-19-02-02, T-19-02-03 all mitigated inline per the handler implementations).

## Self-Check: PASSED

**Files created (verified via `test -f`):**

- `backend/crates/gallery-demo/src/handlers/exer01.rs` — FOUND
- `backend/crates/gallery-demo/src/handlers/mod.rs` — FOUND (modified)
- `frontend/src/lib/exer01/observe.svelte.ts` — FOUND
- `frontend/src/lib/exer01/observe.test.ts` — FOUND
- `frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte` — FOUND (modified)
- `.planning/seeds/v1.3-appshell-nestability.md` — FOUND
- `.planning/phases/19-exerciser-screens/19-VALIDATION.md` — FOUND (updated)

**Commits verified via `git log`:**

- `f46a865` — FOUND (Task 1)
- `154c036` — FOUND (Task 2)

**Test results:**

- `cd backend && cargo test -p gallery-demo --features gallery exer01` — 4 handler + 7 nested_appshell tests, all green (11/11)
- `cd frontend && pnpm exec vitest run src/lib/exer01/observe.test.ts` — 1 passed (1)
- `cd frontend && pnpm check` — 0 ERRORS 0 WARNINGS
- `cd backend && cargo clippy -p gallery-demo --features gallery --all-targets -- -D warnings` — exits 0
- `cd backend && cargo build -p gallery-demo --features gallery` — clean build

**Acceptance criteria (from 19-02-PLAN.md):**

- **Semantic** "exactly one AppShell invocation" acceptance criterion: `grep -c "AppShell::new()" backend/crates/gallery-demo/src/exerciser/nested_appshell.rs` returns **5** (one real call site at line 171; four occurrences in module doc, field comment, in-line comment, and a test-assertion string literal). The semantically-correct guard is the `tree_contains_exactly_one_app_shell` cargo test which counts `props.type == "app-shell"` nodes in the emitted `Vec<Node>` and asserts `== 1`: PASS. Plan's acceptance-criteria grep is a rough proxy; the cargo test is the authoritative check.
- `grep -E 'fn (build_structural_preview_card|build_observation_matrix_card|build_v13_proposal_card|build_matrix_cell)' ... | wc -l` = 4: PASS
- `grep -c 'exer-01-matrix-' ...` ≥ 4: PASS
- `grep "gallery-demo/exer-01/open-seed" ... | wc -l` = 1 per file (nested_appshell.rs + handlers/exer01.rs + handlers/mod.rs): PASS
- `grep "pub async fn handle_exer01_report" handlers/exer01.rs` matches 1: PASS
- `grep "pub async fn handle_exer01_open_seed" handlers/exer01.rs` matches 1: PASS
- `grep "__mrnExer01OuterSidebar" frontend/src/lib/` (recursive) ≥ 2 files: PASS (3 files)
- `grep "export async function probeNestability" frontend/src/lib/exer01/observe.svelte.ts` matches 1: PASS
- `test -f .planning/seeds/v1.3-appshell-nestability.md`: PASS
- `grep -E "^## (Problem|Proposed scope|Acceptance)" .planning/seeds/v1.3-appshell-nestability.md | wc -l` = 3: PASS

---

*Phase: 19-exerciser-screens*
*Plan: 02*
*Completed: 2026-04-24*
