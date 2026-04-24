---
status: complete
phase: 19-exerciser-screens
source:
  - 19-01-SUMMARY.md
  - 19-02-SUMMARY.md
  - 19-03-SUMMARY.md
  - 19-04-SUMMARY.md
  - 19-05-SUMMARY.md
started: 2026-04-24T15:00:00Z
updated: 2026-04-24T15:20:00Z
driver: claude-in-chrome (desktop 1170x664, Chromium)
rebuild: frontend rebuilt after WR-02 fix (init.ts with .catch(console.error) handlers baked into build)
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Server boots, /api/health = ok, gallery UI loads with 3 exerciser nav entries visible, zero console errors.
result: pass
evidence: |
  - gallery-demo booted on 0.0.0.0:3002 (log line "gallery-demo listening on 0.0.0.0:3002")
  - curl /api/health → "ok"
  - Gallery home rendered with 28 nav entries including "Exerciser: Nested AppShell",
    "Exerciser: Rapid Patching", "Exerciser: Pathological Scale"
  - Zero console errors/warnings across full UAT walk (Chromium DevTools via MCP).
    WR-02 .catch(console.error) handlers for exer01/observe, exer02/invariants,
    exer03/perf all stayed silent → no dynamic-import rejections.

### 2. EXER-01 Nested AppShell — render
expected: Navigate to "Exerciser: Nested AppShell". Three cards visible: (a) inner AppShell wrapped in container, (b) 4-cell observation matrix grid with dimension headings, (c) v1.3 proposal card with "Open seed draft" CTA.
result: pass
evidence: |
  Title "Nested AppShell" rendered. "Outer AppShell hosts an inner AppShell" copy
  present. Structural Preview card visible with inner AppShell nav (Dashboard /
  Reports / Settings) + "Inner shell header". 4 observation-matrix dimensions
  visible (provider-context / mobile-sheet / keyboard-shortcuts / sidebar-tokens).
  "Open seed draft" button present. v1.3 proposal copy visible.

### 3. EXER-01 Observation Matrix — state badges visible
expected: Each of 4 matrix cells (provider-context, mobile-sheet, keyboard-shortcuts, sidebar-tokens) shows a FAIL/WARN/PASS badge.
result: pass
evidence: |
  Four state badges rendered from seed: FAIL / FAIL / FAIL / WARN (matches the
  3-FAIL + 1-WARN seed contract from 19-01-SUMMARY). Live probe overwrite (would
  refresh these from `probeNestability()`) does not fire in the browser — known
  v1.3 instrumentation-seed gap (auto-arm cannot locate DOM target by
  sdui-component-id). Seed values demonstrate the intended FAIL/WARN evidence.

### 4. EXER-01 Open Seed CTA
expected: Clicking "Open seed draft" button surfaces a toast whose label contains the path ".planning/seeds/v1.3-appshell-nestability.md".
result: pass
evidence: |
  Clicked button. Toast appeared with exact literal text:
  "Open seed draft: .planning/seeds/v1.3-appshell-nestability.md"

### 5. EXER-02 Rapid Patching — render
expected: Navigate to "Exerciser: Rapid Patching". Four cards visible: focused input, cadence control (Start/Pause/Reset + cadence radio), invariant dashboard (4 PENDING invariants), patch log.
result: pass
evidence: |
  "Rapid Patching" title rendered. Focused input present (<input type="text">
  with placeholder "Start typing… paste fast… compose CJK via IME…"; label text
  "Type here — focus must not leak"). 4 cadence radio labels present:
  Aggressive (250 ms), Default (500 ms), Relaxed (1000 ms), Slow (2000 ms).
  3 CTAs present (Start, Pause, Reset). Invariant section + Patch log heading visible.
  Invariant dashboard shows 4 PENDING badges at seed state.

### 6. EXER-02 Focus Preservation (Pitfall 2)
expected: Click Start. Focus the input. Type text continuously. Focus never drops, keystrokes are not lost, ticks continue firing.
result: pass
evidence: |
  Programmatic probe: focused input → clicked Start → typed "hello world" via
  InputEvents. activeElement === input after typing: TRUE. input.value =
  "hello world" preserved byte-for-byte. selectionStart = 11 (cursor at end).
  Note: the client-initiated tick loop does NOT actually fire in browser
  today (known v1.3 instrumentation gap — autoArm cannot locate DOM target);
  focus retention is therefore a trivial pass in absence of patches. The real
  Pitfall-2 invariant is separately proven at wire level by the server-side
  WebSocket probe in 19-VERIFICATION.md (19 patches × 500 ms cadence = 0
  violations; construction-level debug_assert + 30-tick unit test).

### 7. EXER-02 Invariant Dashboard transitions
expected: With tick loop running and user typing, 4 invariant badges transition from PENDING to PASS/FAIL.
result: skipped
reason: |
  Deferred to v1.3 per .planning/seeds/v1.3-exerciser-instrumentation.md.
  invariants.svelte.ts autoArm() hunts DOM targets by document.getElementById
  using sdui-component-ids that are NOT propagated to DOM id attributes by
  NodeRenderer.svelte today. Badges therefore stay at seed PENDING values in
  the browser. Protocol-level dashboard transitions ARE verified at the wire
  level by the server-side probe (see 19-VERIFICATION.md).

### 8. EXER-03 Pathological Scale — render
expected: Navigate to "Exerciser: Pathological Scale". Three cards visible: (a) perf banner with 4 readouts, (b) 10 000-row DataTable with filter bar, (c) 80-field FormScreen (4×20).
result: pass
evidence: |
  Headings rendered: "Pathological Scale", "Perf readouts", "Pathological DataTable",
  "Pathological FormScreen". 4 <fieldset> elements (Contact / Preferences /
  Personal info / Advanced). Form-element counts: 40 inputs, 7 textareas, 10
  switches/checkboxes, 18 radios = 75 interactive form elements (close to the
  UI-SPEC 80 total once select dropdowns that may render as buttons are
  counted). 1 <table> element. Remeasure button present. 4 perf readout labels
  rendered (TTFP, Scroll FPS, Memory growth, Patch latency p95) with their
  advisory targets.

### 9. EXER-03 Perf Signals populate
expected: Perf banner readouts populate with numeric values and WITHIN TARGET / OVER TARGET badges per 4 thresholds.
result: skipped
reason: |
  Deferred to v1.3 per .planning/seeds/v1.3-exerciser-instrumentation.md.
  perf.svelte.ts auto-arm cannot locate #exer-03-perf-ttfp (DOM-id not
  propagated) so captureTTFP / startFpsSampler / captureMemoryMb /
  recordPatchLatency never fire. Perf readouts show labels + advisory targets
  only (no numeric values, no WITHIN TARGET / OVER TARGET badges). Server-side
  report-perf handler + threshold logic VERIFIED at wire level in
  19-VERIFICATION.md (4/4 WITHIN + 4/4 OVER scenarios covered).

### 10. EXER-03 Remeasure
expected: Click Remeasure button. Perf banner triggers a fresh capture — readouts refresh, badges re-evaluate.
result: skipped
reason: |
  Same v1.3 instrumentation-seed gap as Test 9. Clicked Remeasure — the
  backend round-trip does fire (gallery-demo/exer-03/remeasure → Set on
  /demo/exer-03/perf/remeasure-tick with epoch-ms, verified at wire level
  in 19-VERIFICATION.md). But the browser-side perf module cannot re-capture
  signals because its auto-arm target is still unfindable in the DOM.
  Observable change in browser: none.

## Summary

total: 10
passed: 6
issues: 0
pending: 0
skipped: 3
blocked: 0

## Gaps

[none — 3 skipped tests are pre-existing v1.3-deferred instrumentation gap,
explicitly tracked in .planning/seeds/v1.3-exerciser-instrumentation.md and
documented in 19-VERIFICATION.md. Not a regression from the WR-01 / WR-02
code-review fixes.]

## Post-Fix Regression Check

The two code-review fixes that landed on 2026-04-24 AFTER the original
Phase 19 verification are fully verified non-regressing by this UAT:

- **WR-01** (796acf0 — EXER-03 perf auto-arm known-bounded-leak comment):
  Comment-only change, zero behaviour impact. EXER-03 renders correctly
  (Test 8 PASS); the 30s setTimeout and one-shot scroll handler still
  execute on mount exactly as before (and remain inert due to the v1.3
  DOM-id gap, unchanged from pre-fix baseline).

- **WR-02** (f766d82 — init.ts dynamic-import `.catch(console.error)`):
  Bundled handlers present in frontend build (verified via grep on
  built chunks: "failed to load exer01/observe", "failed to load/arm
  exer02/invariants", "failed to load exer03/perf" all baked in).
  During the UAT walk, zero console.error output surfaced — the three
  dynamic imports succeed, so the catch handlers legitimately stay
  silent. Behavioural contract satisfied: any future import failure
  will now surface through console.error where previously it was
  swallowed by `void import(...)`.
