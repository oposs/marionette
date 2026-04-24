---
phase: 19-exerciser-screens
reviewed: 2026-04-24T13:30:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - backend/crates/gallery-demo/src/exerciser/mod.rs
  - backend/crates/gallery-demo/src/exerciser/nested_appshell.rs
  - backend/crates/gallery-demo/src/exerciser/pathological_scale.rs
  - backend/crates/gallery-demo/src/exerciser/rapid_patching.rs
  - backend/crates/gallery-demo/src/handlers/exer01.rs
  - backend/crates/gallery-demo/src/handlers/exer02.rs
  - backend/crates/gallery-demo/src/handlers/exer03.rs
  - backend/crates/gallery-demo/src/handlers/fetch_rows.rs
  - backend/crates/gallery-demo/src/handlers/mod.rs
  - backend/crates/gallery-demo/src/handlers/show.rs
  - backend/crates/gallery-demo/src/lib.rs
  - backend/crates/gallery-demo/src/state.rs
  - frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte
  - frontend/src/lib/exer01/observe.svelte.ts
  - frontend/src/lib/exer01/observe.test.ts
  - frontend/src/lib/exer02/invariants.svelte.ts
  - frontend/src/lib/exer02/invariants.test.ts
  - frontend/src/lib/exer03/perf.svelte.ts
  - frontend/src/lib/exer03/perf.test.ts
  - frontend/src/lib/init.patchprobe.test.ts
  - frontend/src/lib/init.ts
  - frontend/src/lib/registry/icons.ts
findings:
  critical: 0
  warning: 2
  info: 5
  total: 7
status: issues_found
---

# Phase 19: Code Review Report

**Reviewed:** 2026-04-24T13:30:00Z
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

Phase 19 introduces three exerciser screens (Nested AppShell, Rapid Patching, Pathological Scale) composed with existing marionette builders, seven new handler routes, and three frontend instrumentation modules. The code is well-structured, heavily documented, and thoroughly unit-tested. The threat model is appropriate for a localhost demo gallery (ASVS L1, no untrusted input paths).

**Strengths verified against the reviewer's checklist:**

- **Pitfall 2 (focus-preserving tick)** is enforced correctly in `handlers/exer02.rs::handle_exer02_tick`:
  construction guarantees no op path/id matches the focused input, a `debug_assert!` runtime guard over every emitted op (line 242-252), plus a 30-tick regression test `tick_never_targets_focused_input_path`. Frontend `invariants.svelte.ts` mirrors this — watchers only read from the focused input, never mutate it.
- **DEV gating** for `__mrnExer01OuterSidebar` is correct in `sidebar-provider.svelte:44` — wrapped in `import.meta.env.DEV` so production bundles drop the window hook entirely.
- **Chromium-only `performance.memory`** degrades gracefully in `perf.svelte.ts::captureMemoryMb` (line 103-108): returns `null` when the extension is absent, with unit-test coverage.
- **Strict serde deserialisation** holds for `ObservationReport` (exer01.rs:27-37 — `#[derive(Deserialize)]` with rename-matching; missing fields rejected), `PerfSnapshot` (all-optional fields, tested against `"not-a-number"`), and `StartPayload` (cadence clamp tested at both bounds).
- **No unwrap/panic hazards** in hot-path handlers. `exer01.rs:71` uses `.expect()` on `serde_json::to_value(MatrixEntry)` but the panic-safety doc-comment (lines 48-52) correctly notes the invariant: a two-String struct is infallible.
- **Pre-deployment posture** is respected — no back-compat shims, no migration code.

**Findings below** are two medium-severity hygiene issues around unbounded timers in the frontend perf module, plus five Info-level items. No critical bugs, security vulnerabilities, or invariant violations were found.

## Warnings

### WR-01: Unbounded 30-second `setTimeout` in perf auto-arm is never cancellable

**File:** `frontend/src/lib/exer03/perf.svelte.ts:201-210`
**Issue:**
The module-level auto-arm block installs a `setTimeout(..., 30_000)` that fires a `reportPerf` round-trip at t+30s to capture memory-growth delta. There is no way to cancel this timer if the user navigates away from the EXER-03 screen or from the gallery entirely. The timer will still fire, allocate a `PerfSnapshot`, and call `sendAction('gallery-demo/exer-03/report-perf', …)` — producing a backend PatchMessage writing into `/demo/exer-03/perf/memory_mb/*` on a surface the user is no longer viewing.

The same concern applies to the `scrollHandler` registered on `window` (line 220) — it's registered once at capture phase and only `removeEventListener`-ed on its own first invocation. If the user never scrolls, the listener stays active for the page lifetime.

This is bounded (one-shot timers, small handler) so it will not crash, leak significantly, or cause incorrect data in-screen. It only becomes visible during long gallery sessions or rapid navigation between exerciser screens, where stale reports surface on the backend log.

**Fix:**
Track handles in the `arm()` closure and expose a teardown path. Minimal change:
```typescript
if (typeof window !== 'undefined') {
	let armed = false;
	let memoryT0: number | null = null;
	let t30sHandle: ReturnType<typeof setTimeout> | null = null;
	let scrollHandler: (() => void) | null = null;

	const arm = () => {
		if (armed) return;
		if (!document.getElementById('exer-03-perf-ttfp')) return;
		armed = true;
		installPatchProbe((dt) => recordPatchLatency(dt));
		setTimeout(() => { /* mount snapshot */ }, 100);
		t30sHandle = setTimeout(() => { /* growth snapshot */ }, 30_000);
		scrollHandler = () => {
			if (scrollHandler) window.removeEventListener('scroll', scrollHandler, true);
			startFpsSampler((fps) => reportPerf({ ttfp_ms: null, fps, memory_mb: null, latency_p95_ms: null }));
		};
		window.addEventListener('scroll', scrollHandler, true);
	};

	// Expose teardown (parallel to EXER-02's autoArm cleanup pattern).
	// Consumer can call this when navigating away from EXER-03.
	// Alternatively: detect #exer-03-perf-ttfp removal via MutationObserver
	// and auto-disarm.
}
```
Or, if cancellability is out of scope for v1.2, add a code comment explicitly documenting the known-bounded leak so future maintainers don't read the sibling EXER-02 `autoArm()` cleanup pattern and assume symmetry.

### WR-02: `void import(...).then(...)` silently swallows module-load rejections

**File:** `frontend/src/lib/init.ts:129-133`
**Issue:**
```typescript
if (typeof window !== 'undefined') {
    void import('./exer01/observe.svelte');
    void import('./exer02/invariants.svelte').then((m) => m.autoArm());
    void import('./exer03/perf.svelte');
}
```

The `void` operator discards both the promise and any rejection. If any of these three dynamic imports fails (network error, bundle corruption, module-parse error, or — for the exer02 chain — `autoArm()` itself throwing), the error is silently swallowed. The gallery will continue running with the instrumentation missing, and no console output or telemetry will indicate the failure. UAT would see "EXER-02 invariants never fire PASS" and have to dig through DevTools to find the root cause.

**Fix:**
Attach a minimal `.catch` handler that logs to console.error, matching the rest of the init module's error hygiene:
```typescript
if (typeof window !== 'undefined') {
    import('./exer01/observe.svelte').catch((e) =>
        console.error('[marionette] failed to load exer01/observe', e)
    );
    import('./exer02/invariants.svelte')
        .then((m) => m.autoArm())
        .catch((e) => console.error('[marionette] failed to load/arm exer02/invariants', e));
    import('./exer03/perf.svelte').catch((e) =>
        console.error('[marionette] failed to load exer03/perf', e)
    );
}
```

## Info

### IN-01: `OpenSeedPayload::path` accepted without shape validation

**File:** `backend/crates/gallery-demo/src/handlers/exer01.rs:86-103`
**Issue:**
`handle_exer01_open_seed` deserialises `{ "path": String }` from the action payload and embeds the raw string into a Button label (`format!("Open seed draft: {}", payload.path)`). No validation that the path is inside `.planning/seeds/` or any allow-list. Because the server only echoes the string into an SDUI Button label and the SDUI renderer emits it as plain text (no HTML interpolation, confirmed by threat-model comment at lines 16-18), this is not a security vulnerability. However, a malicious/confused client could send `{"path": "/etc/passwd"}` and the toast would display "Open seed draft: /etc/passwd" — confusing UX for a demo that documents itself as "opens the v1.3 seed draft".

**Fix:**
Optionally reject paths not starting with `.planning/seeds/`:
```rust
if !payload.path.starts_with(".planning/seeds/") {
    return Err(ActionError::BadPayload(format!(
        "open-seed refused non-seed path: {}", payload.path
    )));
}
```
Or leave as-is with a comment noting the demo-gallery scope ("any string acceptable — this is localhost-only; the path is never opened as a file by the server").

### IN-02: `autoArm()` "already armed" branch returns a silent no-op cleanup

**File:** `frontend/src/lib/exer02/invariants.svelte.ts:277-281`
**Issue:**
```typescript
if (armed) {
    return () => {
        /* already armed — caller kept a prior cleanup */
    };
}
armed = true;
```
If a caller invokes `autoArm()` twice (e.g., HMR or test rerun without prior teardown), the second call returns a no-op cleanup. When the second caller runs that cleanup, `armed` remains `true` and the first caller's real cleanup is still live — confusing under double-teardown. The intent ("caller kept a prior cleanup") is correct per the architecture doc but the foot-gun is subtle: if a future refactor lets both callers forget to call cleanup, the module will stay armed forever.

**Fix:**
Either (a) document the invariant inline ("caller MUST retain the cleanup from the first successful arm"), or (b) use a reference-counting scheme where both callers get a real cleanup but only the last one actually tears down. Low priority — the existing behavior is correct for the single-caller case Plan 19-03 documents.

### IN-03: `handle_exer02_tick` pair of separate `lock().await` calls exposes a minor race

**File:** `backend/crates/gallery-demo/src/handlers/exer02.rs:170-174, 227`
**Issue:**
The tick handler acquires `exer02_tick.lock().await` (line 170) to increment and return `iter`, then later acquires `exer02_cadence_ms.lock().await` (line 227) to compute `elapsed_s = (iter * cadence) / 1000`. If two ticks A and B are in flight and their order interleaves as (A.iter=5, B.iter=6, A.cadence, B.cadence), then A and B both read the same cadence but embed different iter values — which is fine by itself, but if `exer02_cadence_ms` is mutated by a `start` call in between (e.g. user switches cadence radio), the two ticks will use two different cadences for their elapsed computation. Since the frontend drives tick cadence via `setInterval`, and cadence changes come from user clicks (not every tick), this race is practically unreachable under the EXER-02 harness. Not a correctness bug.

**Fix:**
Acknowledge the concurrent-tick assumption explicitly, or acquire both locks at the start in a consistent order (tick then cadence) to reduce the window. Alternative: combine into a single `Mutex<(u64, u64)>` holding `(tick, cadence)` if this ever becomes hot.

### IN-04: `compositionupdate` handler redundantly assigns `composing = true`

**File:** `frontend/src/lib/exer02/invariants.svelte.ts:128-133`
**Issue:**
```typescript
const compUpdate = () => {
    // compositionupdate is the intra-composition event. No state
    // transition needed here; listen to satisfy the test and to exercise
    // the full composition event lifecycle.
    composing = true;
};
```
The inline comment acknowledges this is a no-op state transition — `composing` was already `true` from `compositionstart`. Keeping the listener so tests can dispatch `compositionupdate` is fine, but the body could simply be empty (an explicit no-op) rather than a redundant assignment that reads as intentional-but-isn't.

**Fix:**
```typescript
const compUpdate = () => {
    // Intra-composition event — no-op; listener kept so the full
    // composition lifecycle is exercised in tests.
};
```

### IN-05: MutationObservers on `document.body` stay alive for page lifetime

**File:** `frontend/src/lib/exer03/perf.svelte.ts:223-226`, `frontend/src/lib/exer01/observe.svelte.ts:116-130`
**Issue:**
Both modules register `new MutationObserver(...)` at import time observing `document.body` with `childList: true, subtree: true`. The EXER-01 observer correctly calls `obs.disconnect()` after it fires once (line 127). The EXER-03 observer does NOT disconnect after `armed = true` — it will continue firing on every DOM mutation for the lifetime of the tab. Each callback is cheap (`if (armed) return` short-circuits), but this is a CPU cost paid by every unrelated gallery screen for the EXER-03 module's lifetime.

**Fix:**
Mirror `observe.svelte.ts`'s pattern — disconnect after arming:
```typescript
const obs = new MutationObserver(() => {
    arm();
    if (armed) obs.disconnect();
});
```

---

_Reviewed: 2026-04-24T13:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
