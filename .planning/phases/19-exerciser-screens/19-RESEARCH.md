# Phase 19: Exerciser Screens - Research

**Researched:** 2026-04-24
**Domain:** Frontend robustness exercisers — shell-context collision investigation, sustained node-patch pressure, pathological-scale perf measurement
**Confidence:** HIGH (codebase evidence verified for every cross-cutting claim; MEDIUM only where a Web API behaviour depends on runtime browser)

---

## Summary

Phase 19 ships three exerciser screens in `backend/crates/gallery-demo/src/exerciser/` that are stress tests, not catalog polish. Each exerciser is a `#[gallery_demo]` auto-discovered `pub fn gallery_demo() -> Vec<Node>` (same contract Phase 17 locked, same pattern Phase 18 reused). The interesting work is NOT the SDUI composition — those patterns are solved — it is (a) what observation each exerciser must perform to be scientifically useful, (b) what frontend instrumentation makes the observation mechanically possible, and (c) what a completion gate for each exerciser actually looks like given that two of them (EXER-01, EXER-03) surface soft findings rather than hard pass/fail.

Three facts drive the plan structure:

1. **EXER-01's job is to re-create the blocker Phase 17 documented, not to fix it.** Phase 17 G-02 proved that nesting `AppShell` inside another `AppShell` collides `shadcn-svelte`'s `Sidebar.Provider` context because the provider is stored under a single global `Symbol.for("scn-sidebar")` key (verified `frontend/src/lib/components/ui/sidebar/context.svelte.ts:62`). EXER-01 must actually render that collision so the 4 observation dimensions produce real evidence, then propose a v1.3 fix as a seed.
2. **EXER-02's focus-preservation invariant is already infrastructurally proven for sibling patches.** `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` already passes — `setNode` on a sibling preserves focus+cursor+value on the focused input. EXER-02's job is to stress that at scale (≥60 s × 2 Hz = ≥120 mutations) AND cover three invariants the existing test does NOT cover: typed-input race, cursor-position exactness under rapid patches, and IME composition non-interruption.
3. **EXER-03's 10 k-row DataTable is fully supported by existing infrastructure.** The `fetch-rows` source-dispatch arm already caps at 500 via `crate::fixtures::synthetic_rows(500)` (`backend/crates/gallery-demo/src/handlers/fetch_rows.rs:37`). Bumping to 10 000 is a 1-char edit. TanStack `@tanstack/virtual-core` is published as a fixed-overhead windowing algorithm whose cost is `overscan` × row-height, NOT `count` — 10 k rows is textbook territory `[CITED: tanstack.com/virtual]`. The hard work for EXER-03 is the perf-measurement code: TTFP via `PerformanceObserver('paint')`, scroll FPS via requestAnimationFrame delta loop, memory via `performance.memory.usedJSHeapSize` (Chromium-only — acceptable, documented), patch latency via instrumenting the init.ts patch handler.

**Primary recommendation:** Three plans, one per exerciser (no shared Wave 0 required — inherited infrastructure covers all 3). Plan 19-01 = EXER-01 Nested AppShell (4-dimension observation matrix + v1.3 seed). Plan 19-02 = EXER-02 Rapid Patching (60 s × 500 ms cadence + 4 invariants, frontend instrumentation). Plan 19-03 = EXER-03 Pathological Scale (10 k DataTable + 80-field FormScreen + 4 perf signals, frontend instrumentation). All 3 disjoint — can ship in a single parallel wave after a 1-plan Wave 0 that lands the 16 new lucide icons. **Total: 4 plans in 2 waves.**

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-1 — EXER-01 nestability investigation depth: Frame v1.3 fix proposal.** EXER-01 ships the broken-nesting state as evidence (a 4-dimension observation matrix: provider context, mobile sheet, keyboard shortcuts, `--sidebar-*` token inheritance) AND drafts a concrete v1.3 framework extension proposal as a v1.3 seed at `.planning/seeds/v1.3-appshell-nestability.md`.
  - Plan EXER-01 builds a real nested-AppShell screen (outer AppShell hosts an inner AppShell in its content slot — the visually-broken state Phase 17 documented).
  - Plan EXER-01 instruments the 4 dimensions and writes findings to a structured matrix in the SUMMARY.md.
  - Plan EXER-01 produces a separate v1.3 seed file proposing the framework-extension shape (likely scoped-surface-name + scoped Sidebar.Provider context) with enough detail that it can become a v1.3 phase without re-research.

- **D-2 — EXER-02 invariants exercised: all 4 (focus, cursor, typed input, IME).** The Rapid Patching screen exercises all of:
  - **Focus retention** — input keeps focus through 60 s of patches at default ~500 ms cadence (PATCH-02 invariant; the locked goal).
  - **Cursor position** — cursor stays at user's character position; doesn't jump to end or position 0 as patches fire.
  - **Typed input integrity** — user types fast (or pastes); every keystroke survives, no characters lost. Tests input-event vs patch-tick race conditions.
  - **IME composition** — composing CJK / Vietnamese / etc. via IME while patches fire; composition session not broken mid-character. High-value for international users.

- **D-3 — EXER-03 perf treatment: soft thresholds (advisory) + 4 signals.**
  - **Signals captured:** TTFP (time-to-first-paint), Scroll FPS (during sustained scroll on 10 k-row DataTable), Memory snapshot (after mount + after 30 s scroll), Patch application latency (per-patch delivery time on a heavy page).
  - **Threshold style:** Advisory targets per signal (scroll FPS ≥ 30, TTFP ≤ 3 s, memory growth after 30 s scroll ≤ +50 MB, patch latency p95 ≤ 50 ms). If observed values miss the targets, flag in SUMMARY.md as a finding. Phase verification does NOT fail on missed thresholds.

- **D-4 — Defer-vs-fix policy for findings: defer to v1.3, except trivial fixes.** Default response when any exerciser surfaces a real gap: defer to v1.3 seed. Exception — trivial fixes apply inline if ALL of: < 30 minutes implementation, no scope expansion, no new dep or framework concept.

### Claude's Discretion

- Wave / parallelization call (the 3 exercisers touch disjoint files, so `gsd-planner` ultimately decides whether to wave them together or split by complexity).
- Exact internal structure of each exerciser's observation/instrumentation surface — CONTEXT lists dimensions/signals but the widget placement and patch-log shape are designer calls (the UI-SPEC locks most of this; anything the UI-SPEC doesn't lock is planner discretion).
- Threshold numbers for EXER-03 — researcher to validate/refine the 4 suggested targets (FPS ≥ 30, TTFP ≤ 3 s, memory +50 MB, latency p95 ≤ 50 ms). This research leaves them at the CONTEXT-suggested values; see §Open Questions Q1.

### Deferred Ideas (OUT OF SCOPE)

- Actually fixing the AppShell nestability bug (explicit: "does not attempt a v1.2 fix" — D-1).
- Hard perf gates (EXER-03 thresholds are advisory, not gating).
- Toast global-overlay refactor (noted in STATE.md as a Phase 19 candidate but not in EXER-01..03 scope).
- Any framework changes except trivial fixes per D-4.
- Any persistence, auth, or new backend state beyond in-memory seed/patch state.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description (from REQUIREMENTS.md) | Research Support |
|----|------------------------------------|------------------|
| **EXER-01** | Gallery includes a Nested AppShell screen where an outer AppShell hosts an inner AppShell in its content slot. Demonstrates whether shadcn `SidebarProvider` context, mobile-sheet behaviour, keyboard shortcut handling, and `--sidebar-*` CSS tokens compose under nesting — and captures any gaps as deferred items. | §Architecture Patterns → Pattern 1 (nested invocation via `AppShellBuilder`); §Code Examples → Example 1 (Sidebar.Provider collision observation probe); §Common Pitfalls → Pitfall 1 (global symbol key); §Validation Architecture → EXER-01 artifacts |
| **EXER-02** | Gallery includes a Rapid Patching screen that fires node patches at a configurable interval (default ~500 ms) while a text input retains focus. Verifies PATCH-02's focus-preservation invariant under sustained mutation pressure. | §Architecture Patterns → Pattern 2 (backend patch-loop via tokio interval or timer-emitted action) + Pattern 3 (frontend 4-invariant watchers); §Code Examples → Example 2 (invariant watcher module); §Don't Hand-Roll → focus observation, IME composition events; §Validation Architecture → EXER-02 artifacts |
| **EXER-03** | Gallery includes a Pathological Scale screen combining a DataTable with ≥10 000 synthetic rows and a FormScreen with ≥80 synthetic fields on a single page. Captures performance baselines and surfaces scaling issues in the frontend surface store, virtualizer, and SurfaceMount patch application. | §Architecture Patterns → Pattern 4 (10 k fixtures bump + 80-field codegen); §Code Examples → Example 3 (perf-instrumentation module); §Standard Stack → TanStack virtualizer; §Validation Architecture → EXER-03 artifacts |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Exerciser screen composition (3 `Vec<Node>` builder fns) | Rust builders (`marionette` crate via `gallery-demo`) | — | Same pure-fn contract as Phase 18 catalog; lives at `gallery-demo/src/exerciser/*.rs` |
| Auto-discovery + nav (3 entries appear automatically) | `marionette::gallery` registry + linkme | — | No new machinery — inherited from Phase 16/17 |
| Routing (nav click → render) | `gallery-demo/src/handlers/show.rs` | — | Reuses `gallery-show` verbatim; new match arms in `seed_for_key` |
| EXER-01 shell-context observation | Frontend instrumentation (new `frontend/src/lib/exer01/observe.svelte.ts`) | Browser DOM/`getContext` | Only the browser runtime can introspect Sidebar.Provider context identity + `:root` CSS variable inheritance |
| EXER-02 patch emission (tick loop) | Backend (`gallery-demo/src/handlers/exer02.rs`) | Tokio interval timer bound to action | Timer-driven patches are server-authored (no client synthesis of patches); exercises the real Phase 12 PatchMessage pipeline |
| EXER-02 invariant observation | Frontend instrumentation (new `frontend/src/lib/exer02/invariants.svelte.ts`) | DOM event hooks (`compositionstart`/`compositionend`, `input`, `selectionchange`, `focus`/`blur`) | Focus, cursor, typed diff, IME state are browser-owned facts |
| EXER-03 10 k rows delivery | Backend `fetch-rows` source-dispatch | Bumped `fixtures::synthetic_rows(10_000)` | Reuses Plan 18-03 pattern — single param change |
| EXER-03 80-field FormScreen | Backend exerciser fn (codegen for 80 fields) | — | Pure composition — no new frontend, no new state |
| EXER-03 perf signal capture | Frontend instrumentation (new `frontend/src/lib/exer03/perf.svelte.ts`) | `PerformanceObserver`, `performance.memory`, requestAnimationFrame | Browser-only APIs; values patched back to `/demo/exer-03/perf/*` via an action round-trip |
| Icon additions (16 new) | Frontend registry (`frontend/src/lib/registry/icons.ts`) | `@lucide/svelte` package | One-time Wave 0 edit |

**Tier-check sanity:** EXER-02 patch emission is deliberately backend-owned even though the test is about frontend robustness. The reason: if the frontend emitted patches to itself, the test would bypass the real PatchMessage wire pipeline — which is exactly what PATCH-02 is designed to exercise. The test must send real `patch` protocol messages over the WebSocket to be meaningful. Server-to-client is the production shape.

---

## Standard Stack

### Core (Rust workspace)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `marionette` | path dep, `features = ["gallery"]` | Builder primitives + DemoEntry + registry | Established by Phase 16/17/18 [VERIFIED: `backend/crates/gallery-demo/Cargo.toml` transitively] |
| `marionette-protocol` | path dep | `Component`, `ComponentAction`, `PatchOperation`, `PatchMessage` | [VERIFIED: used throughout `gallery-demo/src/handlers/*.rs`] |
| `marionette-macros` | path dep | `#[gallery_demo]`, `#[derive(ComponentBuilder)]` | [VERIFIED: Phase 16 Plan 16-02 shipped] |
| `chrono` | 0.4 (workspace) | `NaiveDate` for 10k row generator (reused, no new usage needed) | [VERIFIED: `backend/Cargo.toml:29` + `gallery-demo/src/fixtures.rs:7`] |
| `tokio` | 1.x (workspace) | `tokio::time::interval` for EXER-02 backend patch loop | [VERIFIED: `tokio` is workspace dep via Axum; already in `gallery-demo/Cargo.toml`] |
| `serde_json` | 1.x (workspace) | Seed-value construction + patch value building | [VERIFIED: used in all existing handlers] |

**No new Rust dependencies required.** `tokio::time::interval` is already pulled in transitively through Axum; 10 k row generation is a single-integer parameter change; nothing needs `rand` (existing deterministic LCG scales fine to 10 k).

### Supporting (Frontend)

| Library | Version (verified) | Purpose | Why Standard |
|---------|-------------------|---------|--------------|
| `@tanstack/virtual-core` | 3.13.x | Row virtualizer — already wraps 500 rows, scales to 10 k without config change | [VERIFIED: `frontend/src/lib/utils/virtualizer.svelte.ts:49-56` imports; TanStack docs confirm `count` is not a complexity factor, only `overscan` × row-height is] `[CITED: tanstack.com/virtual/v3/docs/api/virtualizer]` |
| `@lucide/svelte` | 1.8.0 | Icon library — 16 new icons added in Wave 0 | [VERIFIED: Phase 18 UI-SPEC §Design System line 31 locks version 1.8.0; current registry at `frontend/src/lib/registry/icons.ts` has 14 icons] |
| `bits-ui` (under shadcn-svelte) | as vendored | Sidebar / Sheet primitives that EXER-01 will collide on purpose | [VERIFIED: `frontend/src/lib/components/ui/sidebar/context.svelte.ts:62` uses global `Symbol.for("scn-sidebar")`] |

### Web Platform APIs (no library needed)

| API | Browser support | Purpose | Confidence |
|-----|----------------|---------|-----------|
| `PerformanceObserver('paint')` → `first-paint` / `first-contentful-paint` entries | All modern browsers | EXER-03 TTFP capture | HIGH `[CITED: developer.mozilla.org/en-US/docs/Web/API/PerformancePaintTiming]` |
| `performance.getEntriesByType('paint')` (buffered) | All modern browsers | TTFP fallback if observer registered late | HIGH `[CITED: developer.mozilla.org/en-US/docs/Web/API/Performance/getEntriesByType]` |
| `performance.memory.usedJSHeapSize` | **Chromium only** (Chrome, Edge) — NOT Firefox / Safari | EXER-03 memory snapshot | MEDIUM `[CITED: developer.mozilla.org/en-US/docs/Web/API/Performance/memory]` — documented as non-standard but we accept this because Phase 19 is a dev-local harness, not user-facing |
| `requestAnimationFrame` + timestamp delta | All modern browsers | EXER-03 scroll FPS | HIGH [ASSUMED: standard technique; rAF delta loop is textbook] |
| `compositionstart` / `compositionupdate` / `compositionend` events | All modern browsers (Chrome/Safari/FF have known diffs) | EXER-02 IME invariant | HIGH `[CITED: developer.mozilla.org/en-US/docs/Web/API/Element/compositionstart_event]` — Firefox differs on caret-move behaviour; tolerable because Chrome is our primary UAT driver |
| `selectionchange` event (on document) | All modern browsers | EXER-02 cursor-position observation | HIGH `[CITED: developer.mozilla.org/en-US/docs/Web/API/Document/selectionchange_event]` [ASSUMED] |
| `getSelection()` / `inputElement.selectionStart`/`selectionEnd` | All modern browsers | EXER-02 cursor-position snapshot | HIGH `[VERIFIED: already used by `surfaces.focus-preservation.browser-test.ts:42-43`]` |
| `input` event + `InputEvent.isComposing` | All modern browsers | EXER-02 discriminate real keystrokes from IME composition | HIGH `[CITED: developer.mozilla.org/en-US/docs/Web/API/UI_Events]` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `performance.memory.usedJSHeapSize` | `performance.measureUserAgentSpecificMemory()` (Promise-based replacement) | `measureUserAgentSpecificMemory` is strictly better and cross-origin-isolated-safe, but requires COOP/COEP headers the dev server doesn't currently set; Chrome-only anyway in practice. **Stick with `performance.memory`** for a dev harness; document Chrome-only in the "Perf measurement API unavailable" error copy (UI-SPEC already locks this). |
| Tokio `interval` for EXER-02 patch loop | Spawn one-shot `tokio::time::sleep` chains per tick; or client-side `setInterval` firing actions | Tokio interval ticks drift-free (per docs) but drop on burst. setInterval-on-client defeats the test purpose (no wire patch). **Tokio interval is correct** — CONTEXT locks backend-emitted patches. |
| rAF delta FPS | `PerformanceObserver('event-timing')` with `'longtask'` entries | event-timing is about user-interaction → paint, not sustained scroll framerate. rAF is the idiom. **rAF** — industry standard `[ASSUMED]`. |
| Hand-rolled patch-latency instrumentation | `PerformanceObserver('longtask')` | longtask fires on blocks >50 ms, not per-patch. We want Δt per individual apply. **Hand-rolled — wrap `applyPatch` call site in `frontend/src/lib/init.ts:45-47`** (one-file change). |

**Version verification run:**
```bash
# @tanstack/virtual-core (already-installed version is what matters)
grep '"@tanstack/virtual-core"' /home/oetiker/checkouts/marionette/frontend/package.json 2>/dev/null
# @lucide/svelte
grep '"@lucide/svelte"' /home/oetiker/checkouts/marionette/frontend/package.json 2>/dev/null
```
Both already satisfy the version locks from Phase 18. No new install needed.

---

## Architecture Patterns

### System Architecture Diagram

Data flow through Phase 19 exerciser systems, by exerciser:

**EXER-01 (Nested AppShell — observation only):**
```
user clicks nav "Nested AppShell"
  → gallery-show action with key="exer-01"
  → handle_gallery_show lookups registry → calls exerciser::nested_appshell::gallery_demo()
  → Vec<Node> returned: [ExerciserContainer, InnerAppShell, sidebar/header/footer/main slot nodes, ObservationMatrix Cards, v1.3ProposalCard]
  → Render message sent to "content" sub-surface
  → Frontend mounts inner AppShell INSIDE outer gallery AppShell's content area
  → Sidebar.Provider setContext(Symbol.for("scn-sidebar"), innerState) FIRES
  → inner Sidebar.Root renders at outer's viewport position (COLLISION — the bug)
  → observe.svelte.ts runs at mount:
      - probes getContext(Symbol.for("scn-sidebar")) from outer vs inner scope → reports identity mismatch
      - introspects document.documentElement.getPropertyValue('--sidebar-*') bleeds into inner
      - queries keyboard-shortcut listener count on window
      - checks mobile Sheet conflict when window.innerWidth < 768
  → observations patched back into /demo/exer-01/matrix via action "gallery-demo/exer-01/report"
  → ObservationMatrix Cards re-render with live findings
```

**EXER-02 (Rapid Patching):**
```
user clicks "Start patching"
  → action "gallery-demo/exer-02/start" → backend starts a task:
      loop { tokio::time::interval(cadence_ms).tick().await;
             emit PatchMessage { patch: [Set on /demo/exer-02/patch-sink/{iter}, SetNode on log-cell-{iter}, ...] } }
  → patch arrives at frontend → init.ts handler → applyPatch(surface, msg.patch)
  → instrumentation wraps applyPatch: records perf.now()-start as patch_latency_ms
  → Svelte 5 reactivity re-derives only mutated nodes (focus-preservation guarantee per surfaces.svelte.ts:48-71)
  → invariant watchers (focus, cursor, typed input, IME) observe each tick:
      - focus watcher: listens to document focusout on the focused input — reports lost focus
      - cursor watcher: on each patch tick, reads input.selectionStart — reports if it changes without user input
      - typed-input watcher: tracks typed event sequence via `input` events (with isComposing filter);
                             diffs expected-value vs actual-DOM-value — reports drift
      - IME watcher: listens to compositionstart/compositionend on the focused input — reports if
                     a compositionstart sees a compositionend it did not pair with, or if value mutates
                     via patch during composition (isComposing===true during applyPatch)
  → Each watcher patches its PASS/FAIL/PENDING state into /demo/exer-02/invariants/{focus,cursor,typed,ime}
  → Patch log scrolls via set-children on log Container
user clicks "Pause patching"
  → action "gallery-demo/exer-02/pause" → cancel interval task → state stays frozen
user clicks "Reset counters"
  → action "gallery-demo/exer-02/reset" → delete-node all log rows, reset invariant badges to PENDING
```

**EXER-03 (Pathological Scale):**
```
user clicks nav "Pathological Scale"
  → gallery-show → exerciser::pathological_scale::gallery_demo()
  → returns Vec<Node>: [root Container, PerfReadoutCard, DataTableCard(DataTable bind=/demo/exer-03/rows source=exer-03-synthetic), FormScreenCard(80 fields across 4 FieldSets)]
  → Render sent
  → frontend mounts: DataTable auto-triggers fetch-rows with source="exer-03-synthetic" offset=0 limit=50
      → backend returns page of 50 rows (sliced from synthetic_rows(10_000))
      → virtualizer.svelte.ts keeps window at ~20 DOM rows × row-height
  → perf.svelte.ts runs at mount:
      - TTFP: new PerformanceObserver({entryTypes:['paint']}) → first-paint.startTime
      - Memory(t0): performance.memory.usedJSHeapSize
      - Patch latency: wraps init.ts applyPatch (via module-level accessor) — rolling 100-patch buffer → p95 sampling
  → After DOM ready + 30 s:
      - Memory(t30): performance.memory.usedJSHeapSize → memory_growth = t30 - t0
  → On first scroll event (or click Remeasure):
      - FPS loop: requestAnimationFrame(frame) × 5 s window → min/avg/median fps
  → All 4 readouts patched back via action "gallery-demo/exer-03/report-perf" → set PatchOperation::Set on
    /demo/exer-03/perf/{ttfp, fps, memory_mb, latency_p95_ms}
  → Readout cards re-render with values + WITHIN/OVER TARGET badges
```

A reader tracing any of the three flows can go from user click to paint without leaving files listed in §Component Responsibilities.

### Recommended Project Structure

```
backend/crates/gallery-demo/src/
├── exerciser/                  # NEW — mirrors catalog/
│   ├── mod.rs                  # pub mod nested_appshell; pub mod rapid_patching; pub mod pathological_scale;
│   ├── nested_appshell.rs      # EXER-01 demo fn (invokes real nested AppShell builder)
│   ├── rapid_patching.rs       # EXER-02 demo fn (focused input + cadence + dashboard + log)
│   └── pathological_scale.rs   # EXER-03 demo fn (perf readouts + 10k DataTable + 80-field FormScreen)
├── handlers/
│   ├── exer02.rs               # NEW — start/pause/reset actions + tokio interval patch loop
│   ├── exer03.rs               # NEW — report-perf action (writes perf values to data store)
│   ├── exer01.rs               # NEW — report-observation action (writes matrix cells)
│   ├── mod.rs                  # MODIFY — register exerciser actions; force-link exerciser module
│   ├── fetch_rows.rs           # MODIFY — add "exer-03-synthetic" source arm (10k)
│   └── show.rs                 # MODIFY — seed_for_key arms for exer-01 / exer-02 / exer-03
├── fixtures.rs                 # UNCHANGED (synthetic_rows already generic over n)
├── lib.rs                      # MODIFY — pub mod exerciser;
└── main.rs                     # MODIFY (if needed) — pub mod exerciser;

frontend/src/lib/
├── exer01/                     # NEW
│   └── observe.svelte.ts       # Sidebar.Provider introspection, --sidebar-* token probe, shortcut audit
├── exer02/                     # NEW
│   └── invariants.svelte.ts    # 4 watchers — focus/cursor/typed/IME — each patches /demo/exer-02/invariants/*
├── exer03/                     # NEW
│   └── perf.svelte.ts          # TTFP / FPS / memory / patch-latency harness
├── registry/
│   └── icons.ts                # MODIFY — append 16 new icon imports/registrations
└── init.ts                     # MODIFY — expose applyPatch instrumentation hook for exer03/perf.ts

.planning/seeds/
└── v1.3-appshell-nestability.md  # NEW — drafted by EXER-01 plan; v1.3 seed per D-1
```

### Pattern 1: EXER-01 — real nested-AppShell invocation (deliberately broken)

**What:** `exerciser::nested_appshell::gallery_demo()` invokes the real `AppShell::new()` builder INSIDE its return value — opposite of Phase 17's structural-preview workaround in `builders/app_shell.rs::gallery_demo()`. The frontend mounts a second `<Sidebar.Provider>` inside the first, triggering the G-02 collision.

**When to use:** Only this exerciser. No other Phase 19 surface nests shells.

**Example:**
```rust
// File: backend/crates/gallery-demo/src/exerciser/nested_appshell.rs
// Source: Phase 17 G-02 negative example (builders/app_shell.rs history) +
// Phase 18 catalog fn shape (backend/crates/gallery-demo/src/catalog/buttons.rs).

use marionette::builders::{AppShell, Button, Container, Heading, NavItem, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "exer-01", name = "Exerciser: Nested AppShell")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // Inner AppShell slots — 3 toy nav entries just enough to prove the
    // inner Sidebar.Provider renders.
    let inner_sidebar = Container::new()
        .id("exer-01-inner-sidebar")
        .children(vec![
            NavItem::new("Dashboard").id("inner-nav-1").action(
                ComponentAction::click("gallery-demo/noop"),
            ).build(),
            NavItem::new("Reports").id("inner-nav-2").action(
                ComponentAction::click("gallery-demo/noop"),
            ).build(),
            NavItem::new("Settings").id("inner-nav-3").action(
                ComponentAction::click("gallery-demo/noop"),
            ).build(),
        ])
        .build_with_children();
    let inner_header = Heading::new("Inner shell header")
        .id("exer-01-inner-header").level(3).build();
    let inner_footer = Text::new("Inner shell footer")
        .id("exer-01-inner-footer").build();
    let inner_main = Text::new(
        "Inner main content — this renders INSIDE the outer gallery's main slot. \
         Observation matrix below documents what breaks."
    ).id("exer-01-inner-main").build();

    // Invoke the REAL AppShell — this is the point of EXER-01.
    let inner_shell = AppShell::new()
        .id("exer-01-inner-shell")
        .sidebar(vec![inner_sidebar])
        .header(vec![inner_header])
        .footer(vec![inner_footer])
        .main(vec![inner_main])
        .build_with_children();
    // NOTE: AppShell::build_with_children returns Vec<Node> — the root is
    // position 0, descendants follow. We flatten the whole thing into
    // our outer Container's children.

    // Observation matrix Cards + v1.3 proposal Card per UI-SPEC §EXER-01 copy.
    // (omitted here for brevity — see §Code Examples Example 1)

    // The screen container wraps [title, intro, InnerShellCard,
    // ObservationMatrixCard, V13ProposalCard]. InnerShellCard's children
    // contain the whole inner_shell subtree.
    Container::new()
        .id("exer-01-root")
        .class(OUTER_CLASS)
        .children({
            let mut all = vec![
                Heading::new("Nested AppShell").id("exer-01-title").level(1).build(),
                Text::new("Outer AppShell hosts an inner AppShell in its content slot…")
                    .id("exer-01-intro").build(),
                // InnerShellCard wraps the flattened inner_shell
                // (see UI-SPEC §Per-Screen Anatomy EXER-01 Card 1)
            ];
            // Inner shell subtree: root at inner_shell[0]; children follow.
            all.extend(inner_shell);
            // + observation matrix + v1.3 proposal Cards
            all
        })
        .build_with_children()
}
```

### Pattern 2: EXER-02 — Tokio-interval-driven patch loop

**What:** Backend action handler `gallery-demo/exer-02/start` spawns a tokio task that ticks every N ms, each tick emitting one `PatchMessage` with a mix of Phase 12 node-tree ops. The task is cancelable via `gallery-demo/exer-02/pause`.

**When to use:** EXER-02 only.

**Example:**
```rust
// File: backend/crates/gallery-demo/src/handlers/exer02.rs
// Source: Phase 12 PatchMessage shape (handlers/toast.rs:30-36) +
// Phase 17 modal handler for structured patch emission.

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::ProtocolMessage;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::data::PatchOperation;

// State lives on GalleryState (append a field) — keeps task handle so
// Pause / Reset can cancel it.
// GalleryState additions (state.rs):
//   pub exer02_loop: Arc<Mutex<Option<JoinHandle<()>>>>,
//   pub exer02_cadence_ms: Arc<Mutex<u64>>,
//   pub exer02_tick: Arc<Mutex<u64>>,

pub async fn handle_exer02_start(ctx: HandlerContext) -> ActionResult {
    // Cadence comes from /demo/exer-02/cadence-ms (500 ms default).
    // ctx.state: Arc<GalleryState> — extract cadence + spawn task.
    // The spawned task itself sends Patch messages via ctx.dispatcher
    // (assumption — verify in Plan research).
    //
    // Per tick: rotate op kind across Set-node, Delete-node, Set-children
    // so all 3 Phase 12 ops are exercised.
    // Targets a scratch subtree at /demo/exer-02/patch-sink/{iter} so the
    // focused input at /demo/exer-02/focused-value is a pure sibling — which
    // is the only setup where focus-preservation is guaranteed
    // (see Pitfall 3 below — patching the focused node does NOT preserve focus).
    //
    // Full implementation: see §Code Examples Example 2.
    todo!()
}

pub async fn handle_exer02_pause(ctx: HandlerContext) -> ActionResult { todo!() }
pub async fn handle_exer02_reset(ctx: HandlerContext) -> ActionResult { todo!() }
```

**Critical detail (verified against `init.ts` + `surfaces.svelte.ts`):** The patches MUST target siblings of the focused input, NOT the focused input itself. `setNode` on the focused node tears it down and remounts (see `surfaces.focus-preservation.browser-test.ts:86-97` — the negative-control test makes this explicit). The focused input binds to `/demo/exer-02/focused-value`; its node id is stable; patches fire against `/demo/exer-02/patch-sink/{iter}` and the log node ids. Never emit `set-node` for the focused input's node id.

### Pattern 3: EXER-02 — 4 invariant watchers (frontend instrumentation)

**What:** A single module (`frontend/src/lib/exer02/invariants.svelte.ts`) exports a `mountWatchers(inputElement, onInvariantChange)` function. Each watcher subscribes to the DOM/timing event that detects its invariant's failure mode and calls `onInvariantChange(name, state, details)`.

**Example:**
```typescript
// File: frontend/src/lib/exer02/invariants.svelte.ts
// Source: surfaces.focus-preservation.browser-test.ts:32-65 (cursor+value+focus
// observation pattern); MDN compositionstart/compositionend for IME.

export interface InvariantUpdate {
  name: 'focus' | 'cursor' | 'typed' | 'ime';
  state: 'PASS' | 'FAIL' | 'PENDING';
  details?: string;
  timestamp: number;
}

export function mountWatchers(
  input: HTMLInputElement,
  onUpdate: (u: InvariantUpdate) => void,
  /** expected typed-value tracker — the value the user has typed so far */
  expectedValue: { get: () => string },
): () => void {
  const cleanups: Array<() => void> = [];

  // --- Invariant 1: Focus retention ---
  const focusOutHandler = (_e: FocusEvent) => {
    onUpdate({
      name: 'focus',
      state: 'FAIL',
      details: `Focus lost at ${new Date().toISOString()}`,
      timestamp: performance.now(),
    });
  };
  input.addEventListener('focusout', focusOutHandler);
  cleanups.push(() => input.removeEventListener('focusout', focusOutHandler));

  // --- Invariant 2: Cursor position ---
  // Cursor moves legitimately on user input or keyboard nav. We watch
  // for cursor movement that happens WITHOUT a preceding `input` or `keydown`
  // — that means a patch tick moved it. Implementation: on each patch tick
  // (exposed via a module-level tick counter from init.ts), snapshot
  // selectionStart before tick; after tick, check it's unchanged.
  // (Cross-module coordination — see §Pitfall 5 below.)

  // --- Invariant 3: Typed input integrity ---
  let lastInputEventValue = input.value;
  const inputHandler = (e: Event) => {
    const ev = e as InputEvent;
    if (ev.isComposing) return;  // IME is handled by Invariant 4
    lastInputEventValue = input.value;
    // expectedValue tracks the "what the user has typed" source of truth;
    // after each `input` event, expectedValue.get() === input.value should hold.
    if (expectedValue.get() !== input.value) {
      onUpdate({
        name: 'typed',
        state: 'FAIL',
        details: `Expected "${expectedValue.get()}" got "${input.value}"`,
        timestamp: performance.now(),
      });
    }
  };
  input.addEventListener('input', inputHandler);
  cleanups.push(() => input.removeEventListener('input', inputHandler));

  // --- Invariant 4: IME composition ---
  let composing = false;
  const compStart = () => { composing = true; };
  const compEnd = () => {
    composing = false;
    onUpdate({ name: 'ime', state: 'PASS', timestamp: performance.now() });
  };
  input.addEventListener('compositionstart', compStart);
  input.addEventListener('compositionend', compEnd);
  cleanups.push(() => input.removeEventListener('compositionstart', compStart));
  cleanups.push(() => input.removeEventListener('compositionend', compEnd));

  // The IME FAIL detector needs access to the patch-tick counter — it
  // fires FAIL iff a patch tick is observed while `composing === true` AND
  // the patch tick does NOT abort by observing isComposing at apply time.
  // See §Pitfall 6 for the cross-module plumbing.

  return () => cleanups.forEach(fn => fn());
}
```

### Pattern 4: EXER-03 — perf instrumentation with round-trip patch reporting

**What:** Frontend module `frontend/src/lib/exer03/perf.svelte.ts` captures 4 signals and patches them BACK to the backend via an action (`gallery-demo/exer-03/report-perf`). The backend handler writes them to the data store via `PatchOperation::Set` — the Readout cards' bind paths pick up the new values reactively.

**Why this round-trip shape?** The cleaner direct-write approach (`setData(surface, '/demo/exer-03/perf/...', value)`) would work, but the Phase 12 roundtrip exercises the end-to-end wire protocol in the direction no other gallery screen uses — frontend-captures-data → action-send → backend-patches-back. This is useful diagnostic pressure on the PatchMessage pipeline itself. **Stated in UI-SPEC §EXER-03 perf-mechanics explicitly.**

**Example:**
```typescript
// File: frontend/src/lib/exer03/perf.svelte.ts
// Source: MDN PerformanceObserver / PerformancePaintTiming;
// existing dispatcher.ts for sendAction; existing data.svelte.ts applyPatch wrap.

import { sendAction } from '$lib/transport/dispatcher';

export interface PerfSnapshot {
  ttfp_ms: number | null;
  fps: number | null;
  memory_mb: number | null;
  latency_p95_ms: number | null;
}

// ---- TTFP ----
export function captureTTFP(): number | null {
  // Buffered PO is the reliable path — the observer may mount after the entry.
  const entries = performance.getEntriesByType('paint');
  const fp = entries.find(e => e.name === 'first-paint') as PerformanceEntry | undefined;
  return fp ? fp.startTime : null;
}

// ---- Scroll FPS (rAF delta over 5s window) ----
export function startFpsSampler(onDone: (fps: number) => void): () => void {
  const samples: number[] = [];
  let last = performance.now();
  let running = true;
  const SAMPLE_WINDOW_MS = 5000;
  const startTime = last;
  const loop = (t: number) => {
    if (!running) return;
    const dt = t - last;
    if (dt > 0) samples.push(1000 / dt);
    last = t;
    if (t - startTime > SAMPLE_WINDOW_MS) {
      // median FPS over window
      samples.sort((a, b) => a - b);
      const fps = samples[Math.floor(samples.length / 2)] ?? 0;
      onDone(fps);
      running = false;
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
  return () => { running = false; };
}

// ---- Memory (Chromium-only, documented in UI-SPEC error copy) ----
export function captureMemoryMb(): number | null {
  const p = performance as Performance & { memory?: { usedJSHeapSize: number } };
  if (!p.memory) return null;  // Non-Chromium → UI-SPEC error copy fires
  return p.memory.usedJSHeapSize / (1024 * 1024);
}

// ---- Patch apply latency — via init.ts instrumentation hook ----
// init.ts exposes a `installPatchLatencyProbe(fn)` — the probe wraps
// applyPatch and calls `fn(latencyMs)` after each call.
const latencyBuffer: number[] = [];
const BUFFER_SIZE = 100;
export function recordPatchLatency(ms: number): void {
  if (latencyBuffer.length >= BUFFER_SIZE) latencyBuffer.shift();
  latencyBuffer.push(ms);
}
export function getLatencyP95(): number | null {
  if (latencyBuffer.length === 0) return null;
  const sorted = [...latencyBuffer].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length * 0.95)];
}

// ---- Round-trip report ----
export function reportPerf(snapshot: PerfSnapshot): void {
  sendAction('gallery-demo/exer-03/report-perf', {
    ttfp_ms: snapshot.ttfp_ms,
    fps: snapshot.fps,
    memory_mb: snapshot.memory_mb,
    latency_p95_ms: snapshot.latency_p95_ms,
  });
}
```

### Anti-Patterns to Avoid

- **Anti-pattern — writing `performance.memory` directly without the non-standard guard:** `performance.memory` is **Chromium-only**. Unguarded access doesn't throw but returns `undefined` — the read flows forward as NaN and corrupts the readout. Always guard: `const memory = (performance as any).memory?.usedJSHeapSize ?? null`. UI-SPEC §Error Messages locks the "Perf measurement API unavailable" copy for this case.
- **Anti-pattern — Phase 17's structural-preview workaround for EXER-01:** The whole purpose of EXER-01 is to surface the collision. Do NOT copy `builders/app_shell.rs::gallery_demo()`'s static-preview pattern — the 4 observation dimensions need real live state to report on.
- **Anti-pattern — using `setInterval` on the frontend for patch emission:** The test must exercise the WebSocket patch wire. Emitting patches client-side via setInterval bypasses the transport layer. Backend-driven via Tokio interval.
- **Anti-pattern — patching the focused input node itself:** `surfaces.svelte.ts:50-55` documents that `setNode` mutates-in-place, but the NEGATIVE-control test (`surfaces.focus-preservation.browser-test.ts:68-97`) proves that replacing the focused node does NOT preserve focus. Aim patches at the focused input's *siblings* — the ONLY setup where PATCH-02 holds.
- **Anti-pattern — using `rand` crate for 80-field codegen:** field labels and bind paths are deterministic by design (planner generates them from the UI-SPEC breakdown table). No randomness needed; each field is manually enumerable.
- **Anti-pattern — rendering inner AppShell's observation matrix inside the inner shell:** the matrix MUST render at the **outer** screen level (in the outer AppShell's main content area), not inside the inner AppShell's main content — otherwise the observation cards themselves get swallowed by the Sidebar.Provider collision and the user can't read the findings.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Row virtualization for 10 k rows | Custom windowing + ResizeObserver | `createRuneVirtualizer` in `frontend/src/lib/utils/virtualizer.svelte.ts` (already in use at CAT-03 scale) | Already battle-tested at 500 rows; count is not a cost factor `[CITED: tanstack.com/virtual]` |
| IME composition detection | Key event polling / heuristic input-event sniffing | `compositionstart` / `compositionend` + `InputEvent.isComposing` | Standards-compliant, cross-browser (with known FF/Chrome diff documented); browsers do the heavy lifting |
| First-paint measurement | `requestAnimationFrame` tricks / DOM-ready polling | `PerformanceObserver({entryTypes:['paint']})` + `performance.getEntriesByType('paint')` buffer | Spec-grade, documented, gives `first-paint` + `first-contentful-paint` in one API |
| JS heap-size measurement | Estimating from object graph size | `performance.memory.usedJSHeapSize` (Chromium) | Simple, direct — we accept Chromium-only because Phase 19 is a dev-local harness |
| Seeded deterministic rows at 10 k | Write new generator | Reuse `fixtures::synthetic_rows(10_000)` | Already tested (5 unit tests pass); change is a parameter |
| Patch-loop cadence control | Custom `tokio::time::sleep` chain | `tokio::time::interval` | Drift-free by default; has built-in burst-drop semantics that match "render when possible" intent |
| Cursor-position observation | Tracking key events manually | `input.selectionStart` / `input.selectionEnd` polled after each patch tick | Already used in `surfaces.focus-preservation.browser-test.ts:42-43` |
| Focus loss detection | `document.hasFocus()` polling | `focusout` event on the target input | Event-driven, no polling cost, standards-compliant |
| Observation-matrix state reactivity | Write a new state-sync layer | Reuse the existing `/demo/exer-01/matrix` data path + ErrorDisplay-style bind pattern | Same mechanism CAT-04 ErrorDisplay uses (verified Phase 17 Plan 17-06) |

**Key insight:** Phase 19 has NEARLY ZERO new primitives. The hard problems (patching, virtualization, seeds, action routing, auto-discovery, icons) are all solved. The new code is instrumentation — small modules that WATCH existing behaviour and emit observations. Resist any plan wave that proposes new builders, new Svelte components, or new protocol ops.

---

## Common Pitfalls

### Pitfall 1: Sidebar.Provider global symbol collision is the BUG, not a bug

**What goes wrong:** A developer reviewing EXER-01 reads the code and "helpfully" fixes the collision by making the inner shell use a different context key.
**Why it happens:** Looks like a bug, acts like a bug, but D-1 locks "does not attempt a v1.2 fix." The screen is an observation harness.
**How to avoid:** Two defenses. (1) CONTEXT.md D-1 language reprised in the plan's §Scope section. (2) A test that asserts `nested_appshell::gallery_demo()` tree contains two `app-shell` component types (not one structural-preview) — if someone tries to revert to the Phase 17 workaround, this test fails.
**Warning signs:** Anyone proposing "let's just scope the provider" inline — that's v1.3 work per D-1.

### Pitfall 2: EXER-02 patching the focused input itself

**What goes wrong:** Developer aims patches at the focused input's node id "because it's realistic" — the focused input remounts, loses focus, and EXER-02 reports FAIL on every tick.
**Why it happens:** `setNode(surface, id, comp)` looks like an in-place mutation (it literally is), but Svelte 5's NodeRenderer re-derives its `<svelte:component>` tag when the component prop changes, which unmounts+remounts the underlying input element. `surfaces.focus-preservation.browser-test.ts:68-97` documents this explicitly.
**How to avoid:** Backend patch-loop targets ONLY `/demo/exer-02/patch-sink/{iter}` and the per-log-entry node ids. The focused input at `/demo/exer-02/focused-value` is NEVER on the patch target list. Add a regression test on the backend handler: `handle_exer02_start` unit-test asserts no Set or SetNode op paths start with `/demo/exer-02/focused-value`.
**Warning signs:** Tick handler emits a Set op on `/demo/exer-02/focused-value` → user starts typing → FAIL on every invariant.

### Pitfall 3: TTFP observer registered too late

**What goes wrong:** `PerformanceObserver({entryTypes:['paint']})` is registered in `onMount` of `PerfReadoutCard.svelte` — the paint event has already fired by then, the observer sees nothing.
**Why it happens:** Paint events fire early; Svelte component mounts fire later.
**How to avoid:** Use the `buffered: true` option on `observe()` — buffered entries include entries that fired before registration. Alternatively (and more robustly), read from `performance.getEntriesByType('paint')` at capture time — the browser keeps the buffer around. The §Code Examples snippet uses the latter.
**Warning signs:** TTFP readout consistently reads `—` (em-dash) even on Chromium.

### Pitfall 4: `performance.memory` is not a standard API

**What goes wrong:** Running EXER-03 on Firefox or Safari → memory readout stays at `—` forever; user assumes EXER-03 is broken.
**Why it happens:** `performance.memory` is Chromium-only. `[CITED: developer.mozilla.org/en-US/docs/Web/API/Performance/memory]` states this explicitly: "The API is only available in Chromium-based browsers."
**How to avoid:** (1) Guard the read — UI-SPEC already locks the "Perf measurement API unavailable" error copy. (2) Ship EXER-03 with a documented assumption that UAT is in Chrome (per global memory `feedback_use_chrome_for_uat.md` — Chrome MCP IS the UAT driver). (3) Surface the limitation in the plan's SUMMARY so it's visible to reviewers.
**Warning signs:** Memory readout `—` + user in Firefox.

### Pitfall 5: Cursor-position watcher needs cross-module tick coordination

**What goes wrong:** The cursor watcher tries to "read selectionStart after each patch" but has no way to know when a patch just fired — the frontend patch dispatch is opaque from the component's vantage.
**Why it happens:** `applyPatch` is called from `init.ts:44-52` in a handler closure; there's no existing hook for "notify me after each patch."
**How to avoid:** Add a small instrumentation hook in `init.ts`. The patch handler wraps `applyPatch(...)` → `installedProbe?.()` call after the apply. `exer02/invariants.svelte.ts` calls `installPatchProbe(callback)` at mount; the callback is the tick-after moment for cursor + IME watchers.
**Warning signs:** Cursor watcher never triggers FAIL even on obvious cursor-position bugs because it never ran at the right moment.

### Pitfall 6: IME composition + patch-apply race is subtle

**What goes wrong:** User is mid-composition. A patch arrives, `applyPatch` runs, the data-store patch changes `/demo/exer-02/focused-value`, Svelte's `bind:value` on the input sees a new value and fights the IME composition — composition cancels mid-character.
**Why it happens:** `bind:value` sets `input.value = newValue` unconditionally. If a composition is active, Chrome cancels it.
**How to avoid:** Do NOT patch `/demo/exer-02/focused-value` (Pitfall 2 covers this). But the IME FAIL detector should STILL fire when ANY data patch ticks while composing — even if the focused-value path isn't touched, the PatchMessage processing might schedule a microtask that interrupts composition. Implementation: IME watcher checks `input.isComposing` (via tracked boolean from compositionstart/end) AT each patch-probe callback; if composing is active AND after the patch, composition was canceled (composingWasTrue && !input.composing right after), report FAIL.
**Warning signs:** User types Chinese/Japanese, patches fire, candidate list disappears mid-word → mapped to IME FAIL with "composition broken at timestamp" detail.

### Pitfall 7: 10 k rows × per-row Set patches = 10 k individual patch ops at load time

**What goes wrong:** If EXER-03's seed serves the entire 10 k row slice at once (analogous to Phase 17's `seed_table_rows` emit-all pattern), the frontend applies 10 k PatchOperation::Set ops in a single microtask. P1 layout thrash; TTFP shows 15 s. Not a framework bug — a misuse of the pagination contract.
**Why it happens:** `fetch_rows.rs` already paginates (50 rows / slice) — but the initial seed via `show.rs::seed_for_key` bypasses pagination. If EXER-03 naively seeds 10 000 rows into `/demo/exer-03/rows` on initial render, the virtualizer must also process them.
**How to avoid:** EXER-03's `seed_for_key("exer-03")` arm seeds rows=**empty** or rows=**first-50** only. Let the `fetch-rows` IntersectionObserver sentinel request subsequent pages. Matches Phase 18 CAT-03 pattern: `.page_size(50).total_rows(10_000u64)` — the frontend KNOWS there are 10 k total but only requests them 50 at a time.
**Warning signs:** TTFP readout on EXER-03 shows >10 s. Open DevTools network tab — see a 10 k-element JSON blob on mount. That's the anti-pattern.

### Pitfall 8: 80 field names colliding with shared bind paths

**What goes wrong:** Plan writer generates bind path `/demo/exer-03/<group>/<field>` — but `<field>` slug collision between groups ("email_primary" in Contact AND "email" shorthand in Personal info) causes one to overwrite the other.
**Why it happens:** Field name namespaces aren't automatically isolated per FieldSet; the bind path is a shared `/` hierarchy.
**How to avoid:** Always prefix by group slug: `/demo/exer-03/personal-info/first-name`, `/demo/exer-03/contact/email-primary`. UI-SPEC §EXER-03 locks this template: `/demo/exer-03/<group-slug>/<field-name>`. Add a Rust unit test on `pathological_scale::gallery_demo()`: iterate all text-input binds, assert uniqueness.
**Warning signs:** Two fields visible with the same value — sign they share a bind path.

### Pitfall 9: Tokio task handle leaks on Pause/Reset

**What goes wrong:** `gallery-demo/exer-02/start` spawns a task; Pause stores `None` back but doesn't abort the old task → two tasks ticking simultaneously after a second Start click.
**Why it happens:** Naïve implementation sets `exer02_loop = None` without calling `.abort()` first.
**How to avoid:** `handle_exer02_pause` takes the old handle (`std::mem::take`), calls `.abort()`, THEN sets None. Same for `start` (abort any existing task before spawning). Add a unit test: call start twice in succession, assert only one running task.
**Warning signs:** Cadence appears to double after a Pause→Start cycle.

### Pitfall 10: Patch-log ring buffer + delete-node semantics

**What goes wrong:** UI-SPEC specifies "ring-buffer max 200 rows; oldest dropped via `delete-node`." Naïve implementation emits a `DeleteNode` op with a node id that's never seen (typo), or forgets to remove the now-orphan from the log's `children[]`.
**Why it happens:** Delete-node is a component-tree op; the parent's `children[]` array must ALSO be updated via SetChildren for the DOM to actually remove the row.
**How to avoid:** Per the protocol, the handler emits BOTH ops for an eviction: `PatchOperation::SetChildren { id: "exer-02-log-container", children: new_ids }` AND `PatchOperation::DeleteNode { id: "old-log-row-xyz" }`. See `surfaces.svelte.ts:62-71` (`setChildren` mutates-in-place so the DOM reconciles via keyed `{#each}`) and Phase 12 Plan 12-04 for the pattern.
**Warning signs:** Log shows 201 rows; Delete-node logged but DOM unchanged.

---

## Code Examples

### Example 1: EXER-01 observation probe (frontend)

```typescript
// File: frontend/src/lib/exer01/observe.svelte.ts
// Source: frontend/src/lib/components/ui/sidebar/context.svelte.ts:70-81 (getContext API)
// + MDN CSSStyleDeclaration.getPropertyValue

import { getContext } from 'svelte';
import { sendAction } from '$lib/transport/dispatcher';

const SIDEBAR_KEY = Symbol.for('scn-sidebar');
const SIDEBAR_KEYBOARD_SHORTCUT = 'b';

/**
 * Mount-time probe of the 4 observation dimensions. Reports via
 * gallery-demo/exer-01/report action — backend writes to /demo/exer-01/matrix.
 *
 * Caller invokes from the inner AppShell's root NodeRenderer context (where
 * getContext(SIDEBAR_KEY) returns the INNER sidebar state), compares against
 * a snapshot taken from the outer context (where the outer AppShell mounted).
 * This requires cooperation from the outer shell — the outer's mount-time
 * snapshot is published to module-scope when the outer AppShell mounts
 * (added via a 1-line MODIFY to AppShell.svelte, conditionally via
 * `if (import.meta.env.DEV || window.__mrnExer01Probe)`).
 */
export function probeNestability() {
  // --- Dimension 1: Provider context ---
  // getContext returns the state OBJECT; identity comparison tells us
  // whether inner and outer see the same provider instance (they should
  // NOT — inner's setContext shadowed outer's).
  const innerState = getContext(SIDEBAR_KEY);
  const outerState = (window as unknown as {
    __mrnExer01OuterSidebar?: unknown;
  }).__mrnExer01OuterSidebar;
  const sameIdentity = innerState === outerState;

  // --- Dimension 2: --sidebar-* CSS custom property inheritance ---
  // The outer Sidebar.Provider sets --sidebar-width via inline style;
  // inner Sidebar.Provider ALSO sets --sidebar-width. Cascade rule:
  // inner's inline style wins in the inner subtree. Read from the inner
  // shell's root div via getComputedStyle — compare against the outer.
  const innerRootEl = document.querySelector('[data-exer-01-inner="root"]');
  const outerRootEl = document.querySelector('[data-exer-01-outer="root"]');
  const innerWidth = innerRootEl
    ? getComputedStyle(innerRootEl as Element).getPropertyValue('--sidebar-width')
    : '';
  const outerWidth = outerRootEl
    ? getComputedStyle(outerRootEl as Element).getPropertyValue('--sidebar-width')
    : '';

  // --- Dimension 3: Keyboard shortcut (Ctrl+B) ---
  // shadcn SidebarProvider attaches `<svelte:window onkeydown={handleShortcut}>`.
  // Nesting creates TWO listeners on the same window. We can't enumerate all
  // listeners (DOM API doesn't expose this), but we CAN test the effect:
  // dispatch a synthetic Ctrl+B KeyboardEvent, observe state change on both
  // sidebars. A tight-loop test; observation matrix entry reports
  // "Both toggle — inner wins last" if both states flipped.
  const beforeOuter = outerState && typeof outerState === 'object' && 'open' in outerState
    ? (outerState as { open: boolean }).open
    : null;
  const beforeInner = innerState && typeof innerState === 'object' && 'open' in innerState
    ? (innerState as { open: boolean }).open
    : null;
  window.dispatchEvent(
    new KeyboardEvent('keydown', { key: SIDEBAR_KEYBOARD_SHORTCUT, ctrlKey: true }),
  );
  // tick once for state to propagate
  // (in tests — use await tick(); in probe, Promise.resolve + setTimeout 0)
  setTimeout(() => {
    const afterOuter = outerState && 'open' in (outerState as object)
      ? (outerState as { open: boolean }).open : null;
    const afterInner = innerState && 'open' in (innerState as object)
      ? (innerState as { open: boolean }).open : null;
    const outerFlipped = beforeOuter !== afterOuter;
    const innerFlipped = beforeInner !== afterInner;

    // --- Dimension 4: Mobile sheet ---
    // Both providers construct their own `new IsMobile()` and render a Sheet
    // when isMobile.current === true. At desktop width we can't observe this
    // directly; UI-SPEC locks the matrix copy to describe the expected collision.
    const isMobile = window.innerWidth < 768;

    // --- Report ---
    sendAction('gallery-demo/exer-01/report', {
      'provider-context': {
        state: sameIdentity ? 'PASS' : 'FAIL',
        details: sameIdentity
          ? 'Inner and outer providers share identity'
          : 'Inner provider shadows outer (different state objects)',
      },
      'sidebar-tokens': {
        state: innerWidth && innerWidth === outerWidth ? 'WARN' : 'WARN',  // always warn - see UI-SPEC
        details: `outer --sidebar-width="${outerWidth.trim()}" inner="${innerWidth.trim()}" (inheritance cascades; no scoping)`,
      },
      'keyboard-shortcuts': {
        state: (outerFlipped && innerFlipped) ? 'FAIL' : 'PASS',
        details: (outerFlipped && innerFlipped)
          ? 'Ctrl+B toggled BOTH shells'
          : `Outer flipped=${outerFlipped}, inner flipped=${innerFlipped}`,
      },
      'mobile-sheet': {
        state: isMobile ? 'FAIL' : 'WARN',
        details: isMobile
          ? 'Inner Sheet layered on outer Sheet; dismiss cascades'
          : 'Not testable at desktop width — resize below 768px to observe',
      },
    });
  }, 0);
}
```

### Example 2: EXER-02 backend patch loop

```rust
// File: backend/crates/gallery-demo/src/handlers/exer02.rs
// Source: tokio::time::interval docs; Phase 12 PatchMessage shape;
// gallery-demo/src/handlers/toast.rs:30-36 (PatchMessage construction).

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_protocol::{ProtocolMessage, Component};
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::data::PatchOperation;

// Assumption: ctx exposes a way to BROADCAST messages to all connected clients
// independently of the calling request. If not, Plan 19 research must clarify
// whether the backend has a push-to-client API beyond ActionResult. Likely
// candidates: the Axum WebSocket send channel stored in AppState. This detail
// is the single most important Plan 19-02 research-during-planning item.
//
// Fallback shape if push-outside-handler isn't available: the tick loop
// stores patches in AppState; a separate ACK action ("gallery-demo/exer-02/tick")
// polled from the frontend drains and returns them. Less realistic (breaks
// the "backend pushes patches" semantic) but fully functional.
//
// Plan 19-02 §Research Gaps flags this as the blocking question.

pub async fn handle_exer02_start(ctx: HandlerContext) -> ActionResult {
    // Read cadence from data store (frontend has written /demo/exer-02/cadence-ms).
    let cadence_ms = 500u64; // TODO read from AppState data or ctx

    // Abort any existing loop first (Pitfall 9).
    if let Some(h) = ctx.state.exer02_loop.lock().await.take() {
        h.abort();
    }

    // Spawn the loop.
    let state = ctx.state.clone();
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(cadence_ms));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut iter: u64 = 0;
        loop {
            interval.tick().await;
            iter += 1;
            // Rotate op kinds: Set / SetChildren / DeleteNode + a new log row.
            let op_kind = iter % 3;
            let mut ops: Vec<PatchOperation> = Vec::new();
            match op_kind {
                0 => ops.push(PatchOperation::Set {
                    path: format!("/demo/exer-02/patch-sink/{iter}"),
                    value: serde_json::json!({"tick": iter, "ts": chrono::Utc::now().to_rfc3339()}),
                }),
                1 => {
                    // Append a log row via set-children on exer-02-log-container
                    let row_id = format!("exer-02-log-row-{iter}");
                    let row = Component {
                        r#type: "text".into(),
                        props: serde_json::json!({
                            "text": format!("[{}] patch {} applied", chrono::Utc::now().format("%H:%M:%S%.3f"), iter)
                        }),
                        ..Default::default()
                    };
                    ops.push(PatchOperation::SetNode { id: row_id.clone(), component: row });
                    // ... SetChildren to push into log container's children
                    // (see Pitfall 10 for eviction when > 200 rows)
                },
                _ => ops.push(PatchOperation::DeleteNode {
                    id: format!("exer-02-ghost-{}", iter - 200),  // soft-fail if absent
                }),
            }
            // Push PatchMessage out of band — SEE ASSUMPTION ABOVE
            let msg = ProtocolMessage::Patch(PatchMessage {
                id: None,  // server-initiated
                surface: "content".into(),
                patch: ops,
            });
            // state.broadcast(msg).await or similar — Plan-level detail.
            let _ = msg;
        }
    });
    *ctx.state.exer02_loop.lock().await = Some(handle);
    Ok(vec![])  // no-op direct reply; patches arrive via loop
}

pub async fn handle_exer02_pause(ctx: HandlerContext) -> ActionResult {
    if let Some(h) = ctx.state.exer02_loop.lock().await.take() {
        h.abort();
    }
    Ok(vec![])
}

pub async fn handle_exer02_reset(ctx: HandlerContext) -> ActionResult {
    // Pause first.
    handle_exer02_pause(HandlerContext { /* … */ ..todo!() }).await?;
    // Emit a single PatchMessage that resets: delete all log rows + set invariant badges.
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: vec![
            // children cleared on log container
            PatchOperation::SetChildren { id: "exer-02-log-container".into(), children: vec![] },
            // each invariant badge back to PENDING
            PatchOperation::Set { path: "/demo/exer-02/invariants/focus".into(), value: serde_json::json!({"state": "PENDING"}) },
            PatchOperation::Set { path: "/demo/exer-02/invariants/cursor".into(), value: serde_json::json!({"state": "PENDING"}) },
            PatchOperation::Set { path: "/demo/exer-02/invariants/typed".into(), value: serde_json::json!({"state": "PENDING"}) },
            PatchOperation::Set { path: "/demo/exer-02/invariants/ime".into(), value: serde_json::json!({"state": "PENDING"}) },
            PatchOperation::Set { path: "/demo/exer-02/elapsed-s".into(), value: serde_json::json!(0) },
        ],
    })])
}
```

### Example 3: EXER-03 perf instrumentation (frontend + backend round-trip)

Frontend module shown in Pattern 4. Backend handler:

```rust
// File: backend/crates/gallery-demo/src/handlers/exer03.rs
// Source: handlers/toast.rs (PatchMessage construction), UI-SPEC §EXER-03
// perf readout thresholds.

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_protocol::ProtocolMessage;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::data::PatchOperation;

#[derive(serde::Deserialize)]
struct ReportPerfPayload {
    ttfp_ms: Option<f64>,
    fps: Option<f64>,
    memory_mb: Option<f64>,
    latency_p95_ms: Option<f64>,
}

pub async fn handle_exer03_report_perf(ctx: HandlerContext) -> ActionResult {
    let payload: ReportPerfPayload = serde_json::from_value(
        ctx.action.payload.clone().unwrap_or_default(),
    )
    .map_err(|e| ActionError::BadPayload(format!("report-perf payload invalid: {e}")))?;

    let mut ops = Vec::with_capacity(8);
    // Advisory thresholds per UI-SPEC §EXER-03 Perf readout copy (D-3).
    let within_ttfp = payload.ttfp_ms.map_or(true, |v| v <= 3000.0);
    let within_fps = payload.fps.map_or(true, |v| v >= 30.0);
    let within_mem = payload.memory_mb.map_or(true, |v| v <= 50.0);  // +50MB after 30s
    let within_lat = payload.latency_p95_ms.map_or(true, |v| v <= 50.0);

    if let Some(v) = payload.ttfp_ms {
        ops.push(PatchOperation::Set {
            path: "/demo/exer-03/perf/ttfp".into(),
            value: serde_json::json!({ "value": v, "within_target": within_ttfp }),
        });
    }
    if let Some(v) = payload.fps {
        ops.push(PatchOperation::Set {
            path: "/demo/exer-03/perf/fps".into(),
            value: serde_json::json!({ "value": v, "within_target": within_fps }),
        });
    }
    if let Some(v) = payload.memory_mb {
        ops.push(PatchOperation::Set {
            path: "/demo/exer-03/perf/memory_mb".into(),
            value: serde_json::json!({ "value": v, "within_target": within_mem }),
        });
    }
    if let Some(v) = payload.latency_p95_ms {
        ops.push(PatchOperation::Set {
            path: "/demo/exer-03/perf/latency_p95_ms".into(),
            value: serde_json::json!({ "value": v, "within_target": within_lat }),
        });
    }

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    })])
}
```

### Example 4: 10 k row fetch-rows arm

```rust
// File: backend/crates/gallery-demo/src/handlers/fetch_rows.rs — minimal diff.
// Existing arm "catalog-synthetic-rows" caps at 500. Add a parallel arm
// "exer-03-synthetic" capping at 10_000.

// At the match site around line 34:
match payload.source.as_str() {
    "demo-rows" => ("/demo/data-table/rows", demo_rows_legacy()),
    "catalog-synthetic-rows" => {
        let all = crate::fixtures::synthetic_rows(500);
        // ...existing code unchanged...
    }
    "exer-03-synthetic" => {
        // Same shape, cap 10_000 instead of 500. The actions array is
        // optional here (EXER-03 DataTable may hide the actions column).
        let all = crate::fixtures::synthetic_rows(10_000);
        let start = payload.offset as usize;
        let end = start.saturating_add(payload.limit as usize).min(all.len());
        let slice = all.get(start..end).unwrap_or(&[]);
        let json_rows: Vec<serde_json::Value> = slice
            .iter()
            .map(|r| serde_json::to_value(r).expect("Row serializes"))
            .collect();
        ("/demo/exer-03/rows", json_rows)
    }
    other => { /* BadPayload */ }
}
```

### Example 5: init.ts patch-latency instrumentation hook

```typescript
// File: frontend/src/lib/init.ts — 3-line MODIFY at lines 44-52.
// Add an installable probe so exer03/perf.svelte.ts can measure latency
// without duplicating patch-handling code.

let patchProbe: ((latencyMs: number, opCount: number) => void) | null = null;

export function installPatchProbe(fn: (latencyMs: number, opCount: number) => void) {
  patchProbe = fn;
}
export function removePatchProbe() { patchProbe = null; }

// MODIFIED existing handler — wrap applyPatch with timing.
registerHandler('patch', (raw: unknown) => {
    const msg = raw as PatchMessage;
    const t0 = performance.now();
    applyPatch(msg.surface, msg.patch);
    const t1 = performance.now();
    if (patchProbe) patchProbe(t1 - t0, msg.patch.length);
    if (msg.id) confirmOptimistic(msg.id);
});
```

---

## Project Constraints (from CLAUDE.md & global memory)

No project-level `./CLAUDE.md` exists. Global user instructions apply:

- **Home dir is enormous** — never use `find /home/oetiker`. Use `cargo metadata`, `Glob`, or targeted paths.
- **No hand-rolled UI** (global `feedback_no_handrolling_ui.md`) — use shadcn primitives via Container/Card recipe + Tailwind. EXER-01's observation matrix cells use the same Container-as-Card recipe Phase 18 established; EXER-02's invariant lights use Container + Heading + Badge + Text; EXER-03's perf readouts use Container + Heading + Text + Badge. No new Svelte components.
- **Chrome MCP for UAT** (global `feedback_use_chrome_for_uat.md`) — every exerciser gets UAT via Chrome MCP at desktop + mobile widths. EXER-03 perf readouts rely on Chrome-only `performance.memory` — this matches our UAT environment.
- **shadcn-svelte tooling** — use `shadcnSvelteListTool` / `shadcnSvelteGetTool` or WebFetch. Do NOT call `shadcnSvelteSearchTool` (hangs).
- **No back-compat shims** — pre-deployment; fix root causes (but **note D-1: EXER-01 does NOT fix; it documents**).
- **Options need reasoning** — every gray area gets pros/cons/rationale; check framework recipes before inventing custom designs. (Spacing/typography inherited wholesale from Phase 18 UI-SPEC; no custom design proposed in Phase 19.)

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 17's AppShell static-preview workaround (`builders/app_shell.rs::gallery_demo()` ships a plain Container + 5 slot-boxes) | Phase 19 EXER-01 invokes the real `AppShell` builder nested, surfaces the collision, drafts a v1.3 fix seed | 2026-04-24 (this phase) | Re-opens the question Phase 17 parked; produces a concrete proposal for v1.3 |
| 500-row DataTable (Phase 18 CAT-03) | 10 000-row DataTable (Phase 19 EXER-03) | 2026-04-24 | Parameter bump; no architecture change |
| Focus-preservation unit-tested at sibling-setNode (`surfaces.focus-preservation.browser-test.ts`) | Focus-preservation stress-tested at 2 Hz × 60 s with 3 additional invariants (cursor / typed / IME) | 2026-04-24 | Discovers failures the unit test does not — especially IME race and cursor-jump under rapid patches |
| No perf instrumentation in gallery | 4 advisory signal dashboard (EXER-03) | 2026-04-24 | Baselines that make scaling regressions immediately visible |

**Deprecated / outdated:** nothing — Phase 19 is additive.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| **A1** | Backend can push PatchMessage out of band (outside a handler request/response) | Pattern 2, Example 2 | Catastrophic for EXER-02 — if no push API exists, EXER-02 reverts to frontend-initiated setInterval, which defeats the test's purpose (no wire patch to measure focus-preservation against). **Plan 19-02 MUST research this first as a pre-plan spike.** Likely path: AppState holds a `tokio::sync::broadcast::Sender<ProtocolMessage>` (verify against `gallery-demo/src/state.rs` during planning). |
| **A2** | `tokio::time::interval` is already available transitively via Axum's tokio dep | Pattern 2, Standard Stack | Low — tokio is a hard requirement of Axum; but verify `gallery-demo/Cargo.toml` has a direct `tokio = { workspace = true, features = [...] }` dep if interval's `sync` or `time` features need explicit opt-in. |
| **A3** | `performance.memory` works in the Chrome MCP UAT environment | Pitfall 4 | Low — Chrome MCP IS Chromium. Memory stays `—` on Firefox; accept as a Chromium-only harness. |
| **A4** | `compositionstart` / `compositionend` fire reliably on shadcn `Input` (`<Input>` from bits-ui) | Pattern 3, Pitfall 6 | Low — these are standard DOM events on `<input>` elements; bits-ui wraps `<input>` and doesn't strip listeners. **Should be spike-verified during Plan 19-02 Wave 0** via a 5-line browser-test (type "你好" in an input, verify compositionstart + compositionend fire). |
| **A5** | Tailwind safelist from Phase 18 covers all grid classes Phase 19 uses | UI-SPEC §Responsive Breakpoints | Very low — Phase 18 Plan 18-01 extended the safelist to cover `grid-cols-{1,2,3,4,5,6,7,8}` at `sm:`/`lg:`. Phase 19 uses `grid-cols-1`, `sm:grid-cols-2`, `lg:grid-cols-4` — all present. If UAT surfaces a missing class, append to safelist (trivial fix per D-4). |
| **A6** | Inner AppShell nesting does NOT crash — it merely "looks wrong" | Pattern 1, §EXER-01 | Very low — Phase 17 G-02 evidence says inner Sidebar.Root renders (hijacks visually), no error console output. If a subsequent shadcn-svelte update introduces a runtime error on double-provider, EXER-01's purpose (capture evidence) still holds — the evidence becomes an error log. |
| **A7** | The 4 EXER-03 advisory thresholds (FPS ≥ 30, TTFP ≤ 3 s, memory +50 MB, latency p95 ≤ 50 ms) are reasonable baselines | D-3, UI-SPEC §EXER-03 | Low. Thresholds are advisory (never gating per D-3). If they prove too tight/loose at UAT, update them; the soft-threshold design means "miss = finding, not failure." **Researcher recommendation: keep these values for Plan 19-03 initial baseline; revisit after first UAT produces real numbers.** |
| **A8** | `document.dispatchEvent(new KeyboardEvent('keydown', {ctrlKey:true, key:'b'}))` actually triggers shadcn's `handleShortcutKeydown` registered via `<svelte:window onkeydown>` | Example 1 | MEDIUM — synthetic keyboard events in some browsers do NOT trigger native shortcuts but DO trigger JS listeners (which is all we need here). Worth a Chrome MCP spike during Plan 19-01 verify phase. If synthetic dispatch fails, fall back to documenting the finding textually rather than programmatically observing. |
| **A9** | `PerformanceObserver('paint')` + `getEntriesByType('paint')` buffered read captures first-paint on the exerciser screen (not just initial page load) | Example 3, Pitfall 3 | MEDIUM — the paint-timing spec fires paint events per document navigation; gallery is an SPA where nav happens via WebSocket actions, NOT document navigation. **Reality check: TTFP in Phase 19 EXER-03 measures FIRST page load, not subsequent navigations to /#exer-03.** This is still a useful baseline for "how long does the DataTable take to first-paint rows into a 10 k virtualizer." Plan 19-03 should document this semantic in its SUMMARY. |

**A1 is the highest-risk assumption — Plan 19-02 must validate it as Wave 0 / research-during-planning step before committing to the tokio-interval pattern.**

---

## Open Questions

1. **EXER-03 advisory threshold validity (LOW urgency)**
   - What we know: CONTEXT D-3 names them (FPS ≥ 30, TTFP ≤ 3s, memory +50MB, latency p95 ≤ 50ms) and the researcher is asked to validate/refine.
   - What's unclear: whether these match actual observed values on the dev machines the team uses.
   - Recommendation: ship Plan 19-03 with the CONTEXT-provided values; UAT reveals the baseline; if any threshold is consistently missed despite the system "feeling responsive," update in a follow-up trivial-fix commit (D-4 allows this).

2. **EXER-02 backend push mechanism (A1) — HIGH urgency**
   - What we know: the backend already delivers server-pushed PatchMessages (Plan 17-05 modal close emits a Patch; confirm_open emits a Patch). Those all happen inside an ActionResult return.
   - What's unclear: can a spawned task OUTSIDE of a request/response emit to a specific client? Or do we need a new pattern (e.g. the task writes to an AppState queue polled from the frontend via client-initiated tick actions)?
   - Recommendation: Plan 19-02 Wave 0 includes a 30-minute spike — read `gallery-demo/src/main.rs` + `gallery-demo/src/state.rs` for any existing broadcast infra. If none, we reverse to a client-initiated-tick shape: frontend sends `gallery-demo/exer-02/tick` every 500 ms, backend's handler applies the 3-op mix and returns a PatchMessage in ActionResult. Less elegant (frontend drives cadence) but keeps the wire pressure semantic.

3. **Chrome MCP synthetic keyboard dispatch (A8) — LOW urgency**
   - What we know: the observation matrix entry for keyboard-shortcut scoping is a best-effort probe.
   - What's unclear: whether Chrome's synthetic event actually triggers shadcn's onkeydown listener.
   - Recommendation: if synthetic dispatch fails silently, the observation matrix cell for keyboard-shortcut still renders its LOCKED FAIL state from UI-SPEC copy (the copy describes the KNOWN bug from Phase 17). Only the LIVE observation is at risk, not the matrix's informational content.

4. **Paint-timing semantics in SPA (A9) — LOW urgency**
   - What we know: `PerformancePaintTiming` entries fire for document navigations.
   - What's unclear: whether "navigate to #exer-03 via WebSocket action" produces a paint entry.
   - Recommendation: Plan 19-03 SUMMARY documents that TTFP is "initial page load to first-paint," not "click-nav-to-first-paint." Add explanatory text below the TTFP readout: "Measured from page load, not navigation — reload to remeasure."

5. **Toast global-overlay refactor — DEFERRED per D-4, but confirm scope**
   - What we know: STATE.md §Blockers mentions "Candidate for Phase 19 EXER-01 or a v1.3+ popup-unification plan."
   - What's unclear: whether CONTEXT.md D-1's "scoped to EXER-01 observation + v1.3 seed" implicitly forecloses this.
   - Recommendation: does NOT belong in Phase 19. D-1 locks EXER-01 to the 4 dimensions of AppShell nestability — toast overlay is a separate concern. Confirm with user if they mention it; default: defer.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Rust exerciser crate builds | ✓ | 1.93.1 | — |
| `rustc` | Rust compiler | ✓ | 1.93.1 | — |
| `node` | Frontend build | ✓ | v25.4.0 | — |
| `@tanstack/virtual-core` | EXER-03 row virtualization | ✓ | ~3.13 (Phase 18 lock) | — |
| `@lucide/svelte` | 16 new icon imports | ✓ | 1.8.0 (Phase 18 lock) | — |
| `tokio` time features | EXER-02 patch-loop interval | ✓ | workspace dep (via Axum) | — |
| Chrome browser (for Chrome MCP UAT) | EXER-02 IME spike, EXER-03 perf measurement | ✓ (assumed — matches project UAT driver) | — | Firefox fallback documented in §Pitfall 4; memory readout `—` |
| `performance.memory` (Chromium-only Web API) | EXER-03 memory signal | ✓ (in Chrome) | N/A | UI-SPEC already locks error copy for non-Chromium |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** `performance.memory` outside Chromium → EXER-03 memory signal reads `—` and UI-SPEC error message fires. Acceptable for a dev-local harness.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework (Rust) | `cargo test` — workspace-wide unit + integration tests |
| Framework (Rust lints) | `cargo clippy --workspace --all-targets -- -D warnings` |
| Framework (Svelte unit/browser) | `vitest` — already configured (`frontend/vitest.config.ts`) |
| Framework (E2E) | `playwright test` — configured (`frontend/playwright.config.ts`) |
| Framework (UAT) | Chrome MCP navigation (per `feedback_use_chrome_for_uat.md`) |
| Config files | `backend/Cargo.toml`, `frontend/package.json`, `frontend/playwright.config.ts`, `frontend/vitest.config.ts` |
| Quick run command | `cargo build --workspace --all-features && cd frontend && pnpm check` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cd frontend && pnpm test && pnpm build` |

### Phase Requirements → Test Map

EXER-01 artifacts — automated, unit-level:

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EXER-01 | `nested_appshell::gallery_demo()` returns a tree containing TWO `app-shell` component types (not the Phase 17 structural preview) | unit | `cargo test -p gallery-demo --features gallery exerciser::nested_appshell::tests::tree_has_nested_appshell` | ❌ Wave 0 |
| EXER-01 | `gallery-demo/exer-01/report` action writes matrix entries with 4 required keys (`provider-context`, `mobile-sheet`, `keyboard-shortcuts`, `sidebar-tokens`) | unit | `cargo test -p gallery-demo exer01::tests::report_writes_four_dimensions` | ❌ Wave 0 |
| EXER-01 | `.planning/seeds/v1.3-appshell-nestability.md` exists with required sections (Problem / Proposed scope / Acceptance) | grep | `grep -E '^## (Problem|Proposed scope|Acceptance)' .planning/seeds/v1.3-appshell-nestability.md | wc -l` (expect 3) | ❌ Wave 0 |
| EXER-01 | Registered in DEMOS slice with key="exer-01" | unit | `cargo test -p gallery-demo --features gallery registered_demos_includes_exer_01` | ❌ Wave 0 |

EXER-01 artifacts — manual Chrome MCP UAT (completion-gateable):

| Req ID | Behavior | Test Type | Instructions |
|--------|----------|-----------|--------------|
| EXER-01 | Navigate to /#exer-01 at desktop width; outer gallery sidebar (20 nav entries) is REPLACED by inner sidebar (3 entries Dashboard/Reports/Settings) — evidence of collision | manual-UAT | Chrome MCP: visit `/#exer-01`, screenshot. Expect outer nav gone from the viewport left edge. |
| EXER-01 | Observation matrix renders 4 cells with state badges; at least provider-context and keyboard-shortcuts show FAIL; sidebar-tokens shows WARN; mobile-sheet WARN at desktop / FAIL at mobile | manual-UAT | Chrome MCP: `/#exer-01`, read the 4 matrix cells. |
| EXER-01 | `Open seed draft` CTA toasts the seed path | manual-UAT | Chrome MCP: click CTA, verify toast. |

EXER-02 artifacts — automated:

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EXER-02 | `rapid_patching::gallery_demo()` renders (a) focused input bound to `/demo/exer-02/focused-value`, (b) RadioGroup for cadence, (c) 3 CTAs, (d) 4 invariant-dashboard cells, (e) patch-log container | unit | `cargo test -p gallery-demo --features gallery exerciser::rapid_patching::tests` | ❌ Wave 0 |
| EXER-02 | Start handler aborts any prior task handle before spawning (no leak) | unit | `cargo test -p gallery-demo handlers::exer02::tests::start_aborts_prior_task` | ❌ Wave 0 |
| EXER-02 | Patch loop NEVER emits ops targeting `/demo/exer-02/focused-value` (Pitfall 2) | unit | `cargo test -p gallery-demo handlers::exer02::tests::never_patches_focused_input_path` | ❌ Wave 0 |
| EXER-02 | Reset handler returns a PatchMessage clearing log (SetChildren empty) + resetting 4 invariant keys to PENDING | unit | `cargo test -p gallery-demo handlers::exer02::tests::reset_clears_and_resets_invariants` | ❌ Wave 0 |
| EXER-02 | Invariant-watcher module wires focusout/input/compositionstart/compositionend listeners on target input | browser | `pnpm vitest run src/lib/exer02/invariants.test.ts` | ❌ Wave 0 |

EXER-02 manual Chrome MCP UAT (completion-gateable — this IS the PATCH-02 proof):

| Req ID | Behavior | Test Type | Instructions |
|--------|----------|-----------|--------------|
| EXER-02 | Focus retained 60 s × 500 ms cadence (≥ 120 patches) — invariant shows PASS throughout | manual-UAT | Chrome MCP: `/#exer-02`, click input (focus), click Start, wait 60 s, verify focus invariant still PASS, elapsed shows ≥ 60 s |
| EXER-02 | Cursor position preserved — type 5 chars, move cursor to position 2, wait 10 s, cursor still at 2 | manual-UAT | Chrome MCP: type "hello", Home key, Right×2 (position 2), Start, wait 10 s, cursor still at position 2 |
| EXER-02 | Typed-input integrity — type "the quick brown fox" rapidly during patches, verify all 19 chars present | manual-UAT | Chrome MCP: Start, then type a long string at normal speed, stop, verify value is intact |
| EXER-02 | IME composition survives — open IME (Chrome: alt-tab to IME or use developer keyboard), compose "你好", verify composition ends cleanly during patches | manual-UAT | Chrome MCP: requires IME setup; fallback — use synthetic compositionstart/compositionend via Chrome DevTools "inputType='insertCompositionText'" |

EXER-03 artifacts — automated:

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EXER-03 | `pathological_scale::gallery_demo()` renders (a) 4 perf readout cells, (b) DataTable with `.total_rows(10_000)` and `.source("exer-03-synthetic")`, (c) 80 total fields across 4 FieldSets with non-colliding bind paths | unit | `cargo test -p gallery-demo --features gallery exerciser::pathological_scale::tests` | ❌ Wave 0 |
| EXER-03 | All 80 bind paths are unique (no `/demo/exer-03/.../x` collisions across FieldSets) | unit | `cargo test -p gallery-demo exerciser::pathological_scale::tests::all_80_bind_paths_unique` | ❌ Wave 0 |
| EXER-03 | `fetch-rows` with source="exer-03-synthetic" offset=0 limit=50 returns 50 ops; with offset=9950 limit=50 returns 50 ops ending at path `/demo/exer-03/rows/10000` | unit | `cargo test -p gallery-demo handlers::fetch_rows::tests::exer_03_first_and_last_page` | ❌ Wave 0 |
| EXER-03 | `gallery-demo/exer-03/report-perf` action with full payload writes 4 `/demo/exer-03/perf/*` patch ops, each with `value` + `within_target` keys | unit | `cargo test -p gallery-demo handlers::exer03::tests::report_perf_writes_four_paths` | ❌ Wave 0 |
| EXER-03 | Perf module's `getLatencyP95()` returns null when buffer is empty, otherwise computes p95 correctly | browser | `pnpm vitest run src/lib/exer03/perf.test.ts` | ❌ Wave 0 |

EXER-03 manual Chrome MCP UAT:

| Req ID | Behavior | Test Type | Instructions |
|--------|----------|-----------|--------------|
| EXER-03 | Page mounts in < 10 s (sanity — not the TTFP gate) | manual-UAT | Chrome MCP: `/#exer-03`, wall-clock stopwatch |
| EXER-03 | 4 perf readouts populate within 30 s (patches deliver values) | manual-UAT | Chrome MCP: visit, wait 30 s, verify TTFP/FPS/Memory/Latency all show numeric values (not `—`) |
| EXER-03 | Scroll DataTable 5 s → fetch-rows fires → new rows appear; no browser freeze | manual-UAT | Chrome MCP: scroll the DataTable card, verify visible rows count changes, no dropped frames visible |
| EXER-03 | All 80 fields render and are focusable | manual-UAT | Chrome MCP: tab through, count focus rings land on 80 fields |
| EXER-03 | Soft-threshold findings logged in SUMMARY (actual measured values) | report | Plan 19-03 SUMMARY contains a table with 4 measured values + WITHIN/OVER advisory designation |

### Sampling Rate

- **Per task commit:** `cargo build --workspace --all-features && cd frontend && pnpm check` (~90 s)
- **Per wave merge:** `cargo test --workspace && cd frontend && pnpm test` (~2 min)
- **Phase gate (pre-`/gsd-verify-work`):** Full suite green + Chrome MCP UAT walk of all 3 exerciser screens at desktop + mobile + all 8 manual acceptance items above

### Wave 0 Gaps

- [ ] `frontend/src/lib/registry/icons.ts` — append 16 new lucide icon imports + registrations (UI-SPEC §Design System icons table)
- [ ] `backend/crates/gallery-demo/src/exerciser/mod.rs` — new module declaration
- [ ] `backend/crates/gallery-demo/src/exerciser/{nested_appshell,rapid_patching,pathological_scale}.rs` — three demo fns
- [ ] `backend/crates/gallery-demo/src/handlers/{exer01,exer02,exer03}.rs` — three new handler files
- [ ] `backend/crates/gallery-demo/src/handlers/mod.rs` — register exerciser actions; force-link exerciser module in `ensure_demos_linked` pattern
- [ ] `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` — add `exer-03-synthetic` source arm
- [ ] `backend/crates/gallery-demo/src/handlers/show.rs` — three new seed_for_key arms
- [ ] `backend/crates/gallery-demo/src/state.rs` — extend `GalleryState` with `exer02_loop: Arc<Mutex<Option<JoinHandle<()>>>>` (and cadence/tick if not polled)
- [ ] `backend/crates/gallery-demo/src/lib.rs` + `main.rs` — `pub mod exerciser;`
- [ ] `frontend/src/lib/exer01/observe.svelte.ts` — 4-dimension probe module
- [ ] `frontend/src/lib/exer02/invariants.svelte.ts` — 4 invariant watchers
- [ ] `frontend/src/lib/exer03/perf.svelte.ts` — 4 perf signal capture + round-trip
- [ ] `frontend/src/lib/init.ts` — add `installPatchProbe` / `removePatchProbe` + wrap existing `applyPatch` call
- [ ] `.planning/seeds/v1.3-appshell-nestability.md` — v1.3 seed proposal (drafted by Plan 19-01)
- [ ] Unit + browser tests per §Phase Requirements → Test Map

**No framework install needed** — all test infrastructure already present.

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Gallery binary has no auth by design (CRATE-01 REQUIREMENTS.md) |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | No access control |
| V5 Input Validation | **yes** | Existing `marionette_protocol` deserialization guards via `serde_json::from_value`. EXER-02's `start` handler reads cadence_ms — must clamp to sane range (≥ 100 ms, ≤ 60 000 ms) to prevent a malicious frontend asking for 1 ms cadence and spiking the backend's tokio runtime |
| V6 Cryptography | no | No crypto |
| V12 File Handling | no | No file uploads |

### Known Threat Patterns for Phase 19 stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Cadence DoS (EXER-02) — frontend asks for 0 ms cadence | Denial of Service | Clamp cadence_ms to `[100, 60_000]` in `handle_exer02_start`; log + reject if out of range. Minor — no external trust boundary — but cheap insurance. |
| Report-perf injection (EXER-03) — frontend sends bogus extreme values that break the readout UI | Tampering | Values are strictly `Option<f64>` via serde; non-number payloads are rejected. No XSS vector (readout renders as numeric text). |
| Task handle leak (EXER-02) | Resource exhaustion | Pitfall 9 — abort before spawn. |
| 10 k row generator memory spike | Resource exhaustion (local dev) | `synthetic_rows(10_000)` on every fetch-rows call allocates 10 k structs then slices — acceptable for dev harness. If memory becomes a concern: cache the Vec in `once_cell::Lazy` or generate on-demand per-slice. Not required for Phase 19 scope. |

**Trust boundaries:** none new. The gallery is single-tenant, anonymous-session, local-dev-only (CRATE-01 locks this).

---

## Sources

### Primary (HIGH confidence — codebase or official docs)

- `backend/crates/gallery-demo/src/fixtures.rs:36-74` — `synthetic_rows(n)` deterministic generator already scales to any n
- `backend/crates/gallery-demo/src/handlers/fetch_rows.rs:36-55` — `catalog-synthetic-rows` source-dispatch pattern to extend for 10 k
- `backend/crates/gallery-demo/src/handlers/show.rs:56-100` — `seed_for_key` pattern
- `backend/crates/gallery-demo/src/handlers/mod.rs:23-75` — `register_gallery_actions` pattern
- `backend/crates/gallery-demo/src/catalog/data_table.rs:22-108` — CAT-03 DataTable composition template (reuse verbatim for EXER-03)
- `backend/crates/marionette/src/builders/app_shell.rs` (per Phase 17 Plan 17-06 SUMMARY) — static-preview workaround we are deliberately not copying
- `backend/crates/marionette/src/builders/container.rs:9-21` — Container `icon` field (Phase 18 Plan 18-08) available for observation matrix icons
- `frontend/src/lib/store/surfaces.svelte.ts:46-95` — setNode/setChildren in-place mutation semantics (focus-preservation mechanism)
- `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` — authoritative demonstration of PATCH-02 for sibling patches
- `frontend/src/lib/init.ts:44-52` — patch handler entry point (instrumentation target)
- `frontend/src/lib/components/ui/sidebar/context.svelte.ts:62` — `Symbol.for("scn-sidebar")` global key (root cause of G-02)
- `frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte:26-35` — `setSidebar` + `<svelte:window onkeydown>` shortcut registration
- `frontend/src/lib/components/ui/sidebar/constants.ts:6` — `SIDEBAR_KEYBOARD_SHORTCUT = "b"` (Ctrl+B)
- `frontend/src/lib/utils/virtualizer.svelte.ts:1-140` — TanStack virtualizer wrapper already handling 500 rows
- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-06-SUMMARY.md` — Phase 17 G-02 blocker evidence
- `.planning/phases/18-catalog-screens/18-PATTERNS.md` — pattern map Phase 19 inherits

### Secondary (MEDIUM confidence — MDN / official specs)

- `[CITED: developer.mozilla.org/en-US/docs/Web/API/PerformancePaintTiming]` — first-paint / first-contentful-paint entries
- `[CITED: developer.mozilla.org/en-US/docs/Web/API/Performance/memory]` — `usedJSHeapSize` Chromium-only; non-standard API
- `[CITED: developer.mozilla.org/en-US/docs/Web/API/Element/compositionstart_event]` + `compositionend_event` — IME events cross-browser
- `[CITED: tanstack.com/virtual/v3/docs/api/virtualizer]` — TanStack virtualizer API, count is not a cost factor
- `[CITED: w3c.github.io/uievents]` — UI Events spec inc. `isComposing` flag

### Tertiary (LOW confidence — training / assumption, flagged inline)

- None — assumptions are explicitly tagged `[ASSUMED]` and logged in §Assumptions Log.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all three exercisers rebuild on Phase 16/17/18 contracts; only new dep is `tokio::time::interval` (already transitive).
- Architecture: HIGH — data-flow diagrams verified against init.ts, dispatcher.ts, surfaces.svelte.ts line-by-line.
- Pitfalls: HIGH — Pitfalls 1, 2, 4, 7, 9 cite specific codebase lines; Pitfalls 3, 5, 6, 8, 10 cite protocol semantics documented in Phase 12/17.
- Validation Architecture: HIGH — tests mapped to concrete cargo/vitest invocations.
- Web API fidelity: MEDIUM — MDN-cited, cross-browser diffs for IME and `performance.memory` documented as `[CITED]`.
- A1 (backend push out of band): UNRESOLVED — must be spike-verified during Plan 19-02, flagged in §Open Questions Q2.

**Research date:** 2026-04-24
**Valid until:** 2026-05-24 (30 days — stable codebase, no fast-moving deps).
