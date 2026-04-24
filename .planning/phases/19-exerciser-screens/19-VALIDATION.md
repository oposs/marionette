---
phase: 19
slug: exerciser-screens
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-24
---

# Phase 19 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (backend) + vitest (frontend unit) + Chrome MCP UAT (manual end-to-end) |
| **Config file** | `backend/Cargo.toml` workspace, `frontend/vitest.config.ts` |
| **Quick run command** | `cd backend && cargo test -p gallery-demo` |
| **Full suite command** | `cd backend && cargo test && cd ../frontend && pnpm test` |
| **Estimated runtime** | ~90 seconds (cargo) + ~25 seconds (vitest) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p gallery-demo` (plus focused vitest file if frontend task)
- **After every plan wave:** Run full suite (`cargo test && pnpm test`)
- **Before `/gsd-verify-work`:** Full suite green + Chrome MCP UAT walkthrough of all 3 exerciser nav entries
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

*Populated by gsd-planner — one row per task, citing automated command or Wave 0 dependency.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 19-01-T1 | 19-01 | 1 | EXER-01/02/03 (framework) | — | N/A (no untrusted input; client-side UI registry + probe slot) | vitest unit | `cd frontend && pnpm exec vitest run src/lib/init.patchprobe.test.ts` | ✅ `frontend/src/lib/init.patchprobe.test.ts` | ✅ green (5/5) |
| 19-01-T2 | 19-01 | 1 | EXER-02 (state backbone) + EXER-01/03 (module scaffold) | — | N/A (in-memory state, anonymous session) | cargo unit | `cd backend && cargo test -p gallery-demo --features gallery state::tests` | ✅ `backend/crates/gallery-demo/src/{state.rs,exerciser/mod.rs,exerciser/nested_appshell.rs,exerciser/rapid_patching.rs,exerciser/pathological_scale.rs,lib.rs}` | ✅ green (2/2) |
| 19-01-T3 | 19-01 | 1 | EXER-01/02/03 (seeds + route stubs) | T-19-01 (stub-safe), T-19-03 (10k-row accept) | Stub handlers do not deserialise cadence_ms or perf payloads — T-19-01 mitigate disposition ships in Plan 19-03 | cargo unit + router-dispatch | `cd backend && cargo test -p gallery-demo --features gallery handlers::show::tests handlers::fetch_rows::tests::exer03 handlers::router_tests` | ✅ `backend/crates/gallery-demo/src/handlers/{fetch_rows.rs,show.rs,exer01.rs,exer02.rs,exer03.rs,mod.rs}` | ✅ green (13+ new cases) |
| TBD | 19-02 | 2 | EXER-01 | — | N/A (no untrusted input) | unit / browser-test | TBD | ⬜ pending Plan 19-02 | ⬜ pending |
| 19-03-T1 | 19-03 | 2 | EXER-02 (rapid_patching gallery screen) | — | N/A (pure-fn UI composer; no untrusted input) | cargo unit | `cd backend && cargo test -p gallery-demo --features gallery exerciser::rapid_patching::tests` | ✅ `backend/crates/gallery-demo/src/exerciser/rapid_patching.rs` | ✅ green (9/9) |
| 19-03-T2 | 19-03 | 2 | EXER-02 (handlers) | T-19-01 (cadence clamp), T-19-03-03 (Pitfall 2 regression) | cadence_ms ∈ [100, 60_000] enforced at handle_exer02_start; tick handler runtime guard + unit test proves no ops target /demo/exer-02/focused-value path or exer-02-focused-input node id | cargo unit | `cd backend && cargo test -p gallery-demo --features gallery handlers::exer02::tests` | ✅ `backend/crates/gallery-demo/src/handlers/exer02.rs` | ✅ green (9/9) |
| 19-03-T3 | 19-03 | 2 | EXER-02 (frontend invariants + tick loop) | — | Defense-in-depth client-side clamp in startTickLoop mirrors backend range | vitest unit (jsdom env) | `cd frontend && pnpm exec vitest run src/lib/exer02/invariants.test.ts` | ✅ `frontend/src/lib/exer02/{invariants.svelte.ts,invariants.test.ts}` | ✅ green (9/9) |
| TBD | 19-04 | 2 | EXER-03 | T-19-03 (10k row accept) | 10_000-row cap lives in handlers::fetch_rows.rs (shipped 19-01) | cargo test + manual | TBD | ⬜ pending Plan 19-04 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Populated by gsd-planner. Candidate items from RESEARCH.md:*

- [x] 17 new lucide icons appended to `frontend/src/lib/registry/icons.ts` defaults (16 from UI-SPEC + rotate-ccw; Plan 19-01 Task 1)
- [x] `installPatchProbe` hook in `frontend/src/lib/init.ts` (instrumentation shared by EXER-02 + EXER-03; Plan 19-01 Task 1)
- [x] Synthetic row generator parameter bump covered by `exer-03-synthetic` source arm in `handlers/fetch_rows.rs` calling `synthetic_rows(10_000)` (generator itself in `fixtures.rs` already accepts any n; Plan 19-01 Task 3)
- [x] A1 resolved: client-initiated tick. Route `gallery-demo/exer-02/tick` reserved + reachability-verified; per-tick Patch returned through the normal ActionResult return path. Rationale: `marionette::ws::AppState` does not expose a broadcast channel, and extending it is a framework change out of scope per 19-CONTEXT.md §D-4. Full resolution ships in Plan 19-03.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| EXER-01 4-dimension observation matrix renders with live captured state | EXER-01 | Visual-DOM collision only observable in browser (shadcn Sidebar.Provider context shadowing) | Chrome MCP: navigate /exerciser/nested-appshell; verify outer nav replaced by inner nav; verify matrix table shows captured provider identity, --sidebar-* token cascade, mobile-sheet composition, keyboard-shortcut scope |
| EXER-02 focus retention for 60 s of patch pressure | EXER-02 | Needs sustained human presence + IME hardware (CJK keyboard or macOS IME) to exercise composition invariant | Chrome MCP: navigate /exerciser/rapid-patching; press Start patching; observe 4-light invariant dashboard stays green for 60 s; type during patches (ASCII + attempted IME composition if available); verify no character loss |
| EXER-03 perf readouts capture live measurements on 10k-row page | EXER-03 | Perf values are hardware-dependent; advisory thresholds (D-3) not gating | Chrome MCP: navigate /exerciser/pathological-scale; reload; verify TTFP + FPS + memory + patch-latency readouts populate (non-zero, non-NaN); scroll table for 30 s; verify memory delta captured |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90 s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
