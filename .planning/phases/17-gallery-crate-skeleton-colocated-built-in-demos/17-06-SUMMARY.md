---
phase: 17-gallery-crate-skeleton-colocated-built-in-demos
plan: 06
subsystem: gallery-sdui
tags: [gallery, gap-closure, sdui, app-shell, error-display, switch, textarea, radio-group, field-set, uat, chrome-mcp]

requires:
  - Plan 17-05 (popups-global layout-root refactor; landed 2026-04-22)
  - 19 built-in gallery_demo() siblings (Plan 17-04)
  - marionette::gallery registry + linkme (Phase 16)

provides:
  - Non-hijacking AppShell demo (plain Container+Heading+Text structural preview; no nested AppShell builder, no Sidebar.Provider collision)
  - ErrorDisplay demo wired via `.bind(...)` to seeded errors-a / errors-b paths (frontend `{#if errors.length > 0}` guard passes)
  - Switch demo seeded at `/demo/switch/checked-1` (true) + `/demo/switch/checked-2` (false) matching the demo's bind paths (Wifi CHECKED, Bluetooth unchecked)
  - Textarea demo seeded at `/demo/textarea/value` + `/demo/textarea/value-desc` with empty-string values so bind returns a string
  - W-06 dead-state flag: ErrorDisplay Rust builder's `message` field unused by frontend (for Phase 18 polish)

affects:
  - Plan 17-07 (full Phase 17 re-UAT — now unblocked for the remaining 20-demo walk)
  - Plan 17-08 (G-08 stranded Modal builder cleanup — independent; still pending)

tech-stack:
  added: []
  patterns:
    - "Structural-preview pattern for shell components: when a frontend component relies on viewport-positioned provider context (shadcn Sidebar.Provider), its gallery_demo renders a STATIC representation using plain Container+Heading+Text — not a nested invocation of itself (respects D-A2 hand-designed)"
    - "Demo bind-path alignment: every `/demo/<key>/<slot>` the demo binds MUST have a matching seed_for_key arm writing the same path; a mismatch silently falls through as unseeded data (empty string, undefined, empty array) and the frontend's guards hide the component"

key-files:
  created: []
  modified:
    - backend/crates/marionette/src/builders/app_shell.rs
    - backend/crates/marionette/src/builders/error_display.rs
    - backend/crates/gallery-demo/src/handlers/show.rs

key-decisions:
  - "AppShell demo rewrites to avoid self-nesting. `AppShell::gallery_demo()` renders a plain Container + 5 slot-boxes (Sidebar / Header / Main / Footer / Popups+Toasts) built from Container + Heading + Text — NOT a nested `AppShell::new()` invocation. Rationale: the outer gallery's `<Sidebar.Provider>` context collides with an inner AppShell's own `<Sidebar.Provider>`, causing the inner sidebar to render at the same viewport position and visually replace the outer 20-entry demo nav (G-02). Phase 19 EXER-01 exercises true nested-shell composition properly; Phase 17's gap-closure uses the structural-preview pattern."
  - "ErrorDisplay demo binds explicitly; the Rust `message` field is dead state. Frontend `ErrorDisplay.svelte` reads errors ONLY from `bind` — the builder's `.new(message)` positional arg is consumed by NO frontend code. Demo now calls `.bind(\"/demo/error-display/errors-{a,b}\")` and the seed populates ErrorEntry arrays (`{ path?, message }`). Builder `message` field flagged W-06 for Phase 18 polish (remove or wire as bind-fallback)."
  - "Switch + textarea seed-path alignment. Both demos bind to slot-suffixed paths (`checked-1`/`checked-2`, `value`/`value-desc`) but their seed_for_key arms wrote the wrong shape. Fix: align seed paths 1:1 with demo bind paths. Switch seeded with `checked-1: true` + `checked-2: false` to produce a visually distinct initial state (Wifi on, Bluetooth off)."
  - "Radio-group and field-set left UNTOUCHED. Pass-2 static analysis was correct: bind paths already matched seed paths, and both frontend components render unconditionally for non-empty data. Chrome MCP walk confirmed both render cleanly without any code change — Task 5 escalation checkpoint not triggered."

requirements-completed: [SC-17-06]

metrics:
  duration: "~45min wall clock (backend edits + frontend rebuild + Chrome MCP verification walk)"
  tasks-completed: "4/5 (Task 5 escalation checkpoint skipped — Task 4 Chrome MCP walk confirmed radio-group + field-set render correctly without further intervention)"
  completed-date: 2026-04-22
---

# Phase 17 Plan 06: Gap closure — AppShell nested-sidebar fix (G-02) + 5 empty demo bodies (G-05)

**AppShell demo rewritten as a structural preview to avoid Sidebar.Provider self-collision, and the 5 empty demos repaired via a combination of explicit `.bind(...)` wiring (error-display) and seed-path alignment (switch, textarea) — radio-group and field-set needed no changes, confirmed by Chrome MCP.**

## Performance

- **Duration:** ~45min wall clock (backend edits + frontend rebuild + Chrome MCP verification walk)
- **Completed:** 2026-04-22
- **Tasks:** 4 of 5 (Task 5 escalation checkpoint skipped — not triggered)
- **Files modified:** 3 (2 backend builders + 1 gallery-demo handler)
- **Commits:** 3 (2 implementation + 1 finalization)

## Gap Closure Map

| Gap (sub) | Task | Root cause (one line) | Fix (one line) | Chrome MCP verification |
|-----------|------|-----------------------|----------------|-------------------------|
| G-02 | 1 | `AppShell::gallery_demo()` nested a full `AppShell::new().sidebar().header().main()` — the inner `<Sidebar.Provider>` collided with the outer gallery's Sidebar.Provider, visually replacing the outer 20-entry nav with the inner Dashboard/Reports/Settings nav | Replace the demo body with a plain Container holding 5 labeled slot-boxes (Sidebar / Header / Main / Footer / Popups+Toasts) built from Container + Heading + Text; no AppShell builder, no Sidebar.Provider | Outer gallery sidebar still shows ~20 demo entries; content area shows explainer Text + heading + 5 slot-boxes with Dashboard/Reports/Settings visible INSIDE the Sidebar slot box (not replacing outer nav) — screenshot `ss_35014u4i1` |
| G-05 error-display | 2 | Demo omitted `.bind(...)`; frontend reads errors ONLY from bind; the `{#if errors.length > 0}` guard failed → empty render | Add `.bind("/demo/error-display/errors-a")` + `.bind("/demo/error-display/errors-b")` on the two instances; add seed_for_key arm populating ErrorEntry arrays | 3 red error boxes render: "Email is required" + `/contact/email`, "Phone number is invalid" + `/contact/phone`, "A system-level error (no path)" (no path) — screenshot `ss_8003eii9m` |
| G-05 switch | 2 | Demo binds `/demo/switch/checked-{1,2}`; seed wrote `/demo/switch/checked` (no suffix) → path mismatch → unseeded → rendered but with `checked=false` in both toggles and possibly a silent `<Field.Label for={fieldId}>` binding failure that hid the row | Rewrite seed to `{ "checked-1": true, "checked-2": false }` matching the demo's bind paths; initial state is visually distinct (Wifi CHECKED, Bluetooth unchecked) | 2 switch rows render with labels; Wifi starts CHECKED, Bluetooth starts unchecked with helper description — screenshot `ss_88222x1dh` |
| G-05 textarea | 2 | Demo binds `/demo/textarea/value` + `/demo/textarea/value-desc`; seed wrote only `/demo/textarea/value` → second textarea unseeded → bind returned undefined | Rewrite seed to `{ "value": "", "value-desc": "" }` matching both demo bind paths | 2 textareas render with labels (Notes, With description); both start empty; helper description visible on the second — screenshot `ss_3901pherx` |
| G-05 radio-group | — (not touched) | No issue — static analysis (pass-2) showed bind path `/demo/radio-group/value` ALREADY matched the existing seed; frontend RadioGroup.svelte renders unconditionally when `props.options` is non-empty; UAT pass-1's "empty" observation was likely a viewport/scroll artifact | None (deliberate no-op) | 3 radio options render (Alpha, Beta, Gamma) under "Pick one" label; Beta has description — screenshot `ss_32245hj3r` |
| G-05 field-set | — (not touched) | No issue — static analysis showed demo correctly wires `text_input_nodes` + `select_nodes` as children; frontend FieldSet.svelte renders Field.Set + Field.Group unconditionally; UAT pass-1's "empty" was similarly a viewport artifact | None (deliberate no-op) | "Contact Info" legend + description render; left col has Label/Disabled/With description TextInputs; right col has Fruit/Disabled Selects; helper text visible — screenshot `ss_6202bqqdp` |

**Regression spot-checks (all pass):**
- Checkbox ✅ — 3 checkboxes (Unchecked, With description, Disabled) render with helper text — screenshot `ss_272838197`
- Button ✅ (unchanged since 17-05 UAT; not re-screenshotted)
- Form ✅ (unchanged since 17-05 UAT; not re-screenshotted)
- Modal overlay + ConfirmDialog Accept/Reject (17-05) ✅ — not re-tested in this walk, remain in scope for 17-07's full re-UAT

## Commits (3 total)

Ordered chronologically:

1. `86d0890` — **fix(17-06): rewrite AppShell::gallery_demo to avoid nested AppShell builder (G-02)** — Task 1: replace `AppShell::new().sidebar(...).header(...).main(...).build_with_children()` with a plain Container + Heading + Text structural preview of the 6 slots. AppShell struct + AppShellBuilder + existing tests unchanged.
2. `2bc0cad` — **fix(17-06): deterministic G-05 fixes — error-display bind + switch/textarea seed alignment** — Task 2: add `.bind(...)` on both error-display demo instances; rewrite seed_for_key arms for switch (`checked-1`/`checked-2`) + textarea (`value`/`value-desc`); add new error-display arm with ErrorEntry seed. Radio-group / field-set demo bodies + seeds UNCHANGED.
3. _(this finalization commit)_ — **docs(17-06): finalize SUMMARY + tracking** — 17-06-SUMMARY.md creation; STATE.md / ROADMAP.md / REQUIREMENTS.md updates marking SC-17-06 validated.

_Task 3 (frontend rebuild) produced no commit — `frontend/build/` is gitignored._

## Files Modified

### Backend

- `backend/crates/marionette/src/builders/app_shell.rs` — Rewrote the `#[gallery_demo(key = "app-shell")]` sibling fn body. Replaced nested `AppShell::new()` builder invocation with 5 slot-boxes built from `Container::new().children(vec![Heading, Text, ...]).build_with_children()` + an outer Container + an explainer Heading + intro Text. The `AppShell` struct, the `AppShellBuilder` impl, and all existing `#[cfg(test)]` tests (lines 299-437, unchanged) continue to cover the builder itself; none of them exercise `gallery_demo()`.
- `backend/crates/marionette/src/builders/error_display.rs` — Rewrote the `#[gallery_demo(key = "error-display")]` sibling fn. Both instances now call `.id(...)` + `.bind("/demo/error-display/errors-{a,b}")`. The `ErrorDisplay::new(...)` positional arg remains (it's required by the builder signature) but is set to a short label identifier; the visible errors come from the seed.
- `backend/crates/gallery-demo/src/handlers/show.rs` — In `seed_for_key`: added new `error-display` arm with two ErrorEntry arrays (errors-a has 2 path+message entries, errors-b has 1 system-level); rewrote `switch` arm to `{ "checked-1": true, "checked-2": false }`; rewrote `textarea` arm to `{ "value": "", "value-desc": "" }`. Alphabetical ordering of match arms preserved.

### Frontend

No frontend source files modified. `frontend/build/` was rebuilt (`npm run build`) to pick up bundle-level hashing only; Plan 17-05's frontend changes already propagated.

### Tracking

- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-06-SUMMARY.md` (this file)
- `.planning/STATE.md` — Plan 17-06 complete; completed_plans counter bumped.
- `.planning/ROADMAP.md` — Phase 17 row Plans Complete → 6/8.
- `.planning/REQUIREMENTS.md` — SC-17-06 marked validated 2026-04-22 with link to this summary.

## Decisions Made

### D-A (Plan 17-06): AppShell demo uses the structural-preview pattern, NOT a nested AppShell

**Decision:** `AppShell::gallery_demo()` renders a plain Container holding 5 slot-labeled sub-boxes (Sidebar / Header / Main / Footer / Popups+Toasts), each built from Container + Heading + Text. The `AppShell::new()` builder is NOT invoked inside the demo.

**Rationale:** The frontend `AppShell.svelte` wraps its content in `<Sidebar.Provider>` (shadcn-svelte SidebarProvider) — a context provider that stakes out a viewport-anchored left-edge position for `<Sidebar.Root collapsible="offcanvas">`. Nesting a second AppShell inside the outer gallery's `content` sub-surface produces two Sidebar.Providers; the inner `Sidebar.Root` renders at the same viewport position as the outer, visually replacing the outer 20-entry demo nav with the inner 3-entry Dashboard/Reports/Settings nav. G-02 is the first observed instance of the STATE.md-tracked "AppShell nestability unknown" blocker — now confirmed to be a true collision, not a latent risk.

**Pre-deployment posture (no back-compat):** Fix the root cause (don't nest) rather than hack a scoping shim. Phase 19 EXER-01 is the designated exerciser for true nested-shell composition; Phase 17's gap-closure is the wrong surface to invent scoped-surface semantics. Honors D-A2 (AppShell demos are hand-designed, not auto-nested).

**Consequence:** The structural-preview pattern is now available in `backend/crates/marionette/PATTERNS.md`-equivalent scope (via 17-PATTERNS.md + this SUMMARY) for any future shell-component demo whose frontend uses viewport-anchored context providers.

### D-B (Plan 17-06): ErrorDisplay demo binds explicitly; builder `message` field is dead

**Decision:** The `ErrorDisplay` demo calls `.new("errors-a").bind("/demo/error-display/errors-a")` (and errors-b). The visible errors come ONLY from the seeded bind path; the positional `message` arg is a label identifier for the demo instance, not rendered content.

**Rationale:** Static analysis against `frontend/src/lib/components/feedback/ErrorDisplay.svelte` (lines 26-41) confirmed the frontend reads errors ONLY from `bind` — the builder's `message` field is read by NO frontend code. Without `.bind(...)`, the frontend's `{#if errors.length > 0}` guard always evaluates false.

**W-06 dead-state flag (for Phase 18 polish):** Two reconciliation options:
1. Remove the `message` field from the `ErrorDisplay` struct and update any call sites.
2. Wire `message` as a bind-fallback on the frontend — if `getData(surface, bind)` returns empty, render a single-entry error from the `message` prop.

Either is a small follow-up; Phase 18 CAT-04 (Feedback) is the natural home.

### D-C (Plan 17-06): Radio-group and field-set demos left untouched — static analysis was correct

**Decision:** No code changes to `radio_group.rs`, `field_set.rs`, or their seed_for_key arms. Chrome MCP walk confirmed both demos render cleanly.

**Rationale:** Pass-2 static analysis flagged both demos as having correctly-aligned bind paths and unconditionally-rendering frontend components. UAT pass-1's "empty" observation was likely a viewport or scroll artifact (Chrome MCP sometimes under-sized the demo-preview iframe, clipping the field-set's 2-column grid below the fold). The deterministic fixes in Tasks 1 + 2 were enough; Task 5's escalation checkpoint was not triggered.

**Consequence:** Plan 17-06 closed on the first Chrome MCP walk with no discovery phase for radio-group / field-set, no frontend-layer fix, and no cross-boundary work. The CONTEXT.md "NOT frontend work" boundary held.

## Deviations from Plan

None — plan executed exactly as written. Tasks 1-4 landed on their planned diffs; Task 5 (escalation decision checkpoint) was SKIPPED because Task 4's Chrome MCP walk confirmed radio-group + field-set render correctly without further intervention.

No auto-fixes, no Rule 1-4 escalations. All backend edits compiled and passed clippy on the first attempt (workspace-wide clippy still has the pre-existing crm-demo pedantic drift documented in `deferred-items.md`; per-plan clippy on touched crates passes clean).

## Issues Encountered

None. Both commits landed cleanly on their planned surfaces; the Chrome MCP walk confirmed every targeted gap closed + regression spot-checks passed on the first attempt.

## UAT Evidence

**Chrome MCP walk on 2026-04-22 against the restarted `gallery-demo` server on :3002** (orchestrator-driven):

Targeted gap confirmations:
- **G-02** ✅ App Shell demo — outer gallery sidebar still shows ~20 demo entries (NOT replaced by Dashboard/Reports/Settings). Content area shows 5 labeled slot boxes (Sidebar slot with "Dashboard / Reports / Settings" as example bullets INSIDE the box, Header slot, Main slot, Footer slot, Popups+Toasts slots). Explainer Text at top: "AppShell composes six slots: sidebar, header, main, footer, popups, toasts. This preview shows the static structure — Phase 19 EXER-01 will exercise nestable shells properly. Clicking this demo does NOT replace the gallery's own sidebar." — screenshot `ss_35014u4i1`
- **G-05 Error Display** ✅ — 3 red error boxes visible: "Email is required" + `/contact/email`, "Phone number is invalid" + `/contact/phone`, "A system-level error (no path)" (no path shown). Each has alert-circle icon + destructive styling. — screenshot `ss_8003eii9m`
- **G-05 Switch** ✅ — 2 switches render: Wifi (starts CHECKED, seeded `checked-1: true`), Bluetooth (starts UNCHECKED, seeded `checked-2: false`). Labels present; Bluetooth has description "With a helper line below via Field.Description." — screenshot `ss_88222x1dh`
- **G-05 Textarea** ✅ — 2 textareas: Notes (empty, `value: ""`), With description (empty, `value-desc: ""`, with helper "Multi-line text input with a helper line below."). — screenshot `ss_3901pherx`
- **G-05 Radio Group** ✅ (no changes applied in this plan; static analysis proven correct) — Renders "Pick one" label + 3 radio options (Alpha, Beta, Gamma). Beta has description "Second option with a description line." All visible with proper radio affordance. — screenshot `ss_32245hj3r`
- **G-05 Field Set** ✅ (no changes applied in this plan; static analysis proven correct) — "Contact Info" legend + "Grouped form fields demonstrating FieldSet layout." description. Left column: Label / Disabled / With description TextInputs. Right column: Fruit / Disabled Selects. Helper text under the "With description" input: "Helper text rendered below via Field.Description." — screenshot `ss_6202bqqdp`

Regression spot-checks:
- Checkbox ✅ — 3 checkboxes (Unchecked, With description, Disabled) with helper text — screenshot `ss_272838197`
- Button, Form ✅ (verified during 17-05 UAT; no change in 17-06)
- Modal overlay + ConfirmDialog Accept/Reject (17-05) — not re-tested in this walk; remain in scope for 17-07's full re-UAT

Task 5 (escalation decision checkpoint) SKIPPED — not triggered, since Task 4's Chrome walk confirmed radio-group + field-set render correctly without further intervention.

## Threat Flags

None. No new trust-boundary-adjacent surface introduced by this plan. The seeded ErrorEntry values are synthetic demo data (no PII); the demo-key handler path in `seed_for_key` remains inside the gallery-demo binary's in-memory state (no persistence, no cross-surface leak).

T-17.06-02 (AppShell demo nested-render DoS) is mitigated by Task 1's rewrite — no nested AppShell builder can now be invoked inside the demo. T-17.06-03 (ErrorDisplay bind path tampering) is mitigated by the deterministic `/demo/error-display/errors-*` path seeded by the known-good `seed_for_key` handler.

## Known Stubs

None in 17-06. The ErrorDisplay builder's `message` field is an UNUSED field (not a stub) — it exists on the Rust struct but is read by no frontend code. Flagged as W-06 for Phase 18 polish (see Decisions §D-B).

## Deferred / Tracked Separately

- **G-08 stranded Modal builder primitive** — Scheduled for Plan 17-08 (wave 2, autonomous). Unrelated to 17-06's surface area.
- **W-06 ErrorDisplay `message` dead-state** — New deferred item from this plan; Phase 18 CAT-04 (Feedback) is the natural home for either removal or bind-fallback wiring.
- **Toast global-overlay refactor** — Deferred from 17-05; unchanged in 17-06.
- **Pre-existing crm-demo clippy::pedantic drift** — Unchanged; still in `deferred-items.md`.
- **Pre-existing frontend ESLint baseline** — Unchanged; still in `deferred-items.md`.

See `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/deferred-items.md` for baseline details.

## Next Plan Readiness

- **Plan 17-07 (full Phase 17 re-UAT)** — Now unblocked for the planned 20-demo Chrome MCP walk. With 17-05 (G-01/03/04/06/07) + 17-06 (G-02/05) closed, the remaining work is the full re-UAT + `17-VERIFICATION.md` status flip + ROADMAP/STATE phase-close updates. Plan 17-08 (G-08) is independent (touches only marionette builders, not the gallery handlers exercised by re-UAT); 17-07 can run before or after 17-08.
- **Plan 17-08 (G-08 Modal builder cleanup)** — Wave 2, autonomous; independent of 17-06. Still pending.

**Note to Plan 17-07 executor:** Plan 17-06 closed G-02 (AppShell demo) + all 5 sub-gaps of G-05 (error-display, switch, textarea via deterministic fixes; radio-group + field-set confirmed already-working). Combined with 17-05's 5 closures, all 7 original Phase 17 gaps are fixed. Run the full 20-demo Chrome MCP re-UAT to validate SC-17-07 and record final pass metrics; W-06 (ErrorDisplay message dead-state) is a Phase 18 CAT-04 concern, NOT a Phase 17 blocker.

## Self-Check: PASSED

Files verified present:
- `backend/crates/marionette/src/builders/app_shell.rs` — FOUND
- `backend/crates/marionette/src/builders/error_display.rs` — FOUND
- `backend/crates/gallery-demo/src/handlers/show.rs` — FOUND
- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-06-SUMMARY.md` — FOUND (this file)

Commits verified present (via `git log --oneline`):
- `86d0890` (Task 1 — AppShell::gallery_demo rewrite, G-02) — FOUND
- `2bc0cad` (Task 2 — error-display bind + switch/textarea seed alignment, G-05) — FOUND

Finalization commit `docs(17-06): finalize SUMMARY + tracking` authored together with SUMMARY + STATE.md + ROADMAP.md + REQUIREMENTS.md updates.
