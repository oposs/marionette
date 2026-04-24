---
phase: 19
slug: exerciser-screens
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-24
verified_date: "2026-04-24"
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
| 19-02-T1 | 19-02 | 2 | EXER-01 | — | N/A (exerciser function builds SDUI tree from locked inputs; no untrusted data flows in) | cargo unit | `cd backend && cargo test -p gallery-demo --features gallery exerciser::nested_appshell::tests` | ✅ `backend/crates/gallery-demo/src/exerciser/nested_appshell.rs` | ✅ green (7/7) |
| 19-02-T2 | 19-02 | 2 | EXER-01 | T-19-02-01 (ObservationReport strict deserialize), T-19-02-02 (seed-path String echo in Text node — no XSS), T-19-02-03 (DEV-gated window hook) | Strict serde Deserialize on ObservationReport + OpenSeedPayload rejects malformed payloads with `ActionError::BadPayload`; toast echoes seed path through SDUI Text (no HTML); `__mrnExer01OuterSidebar` hook is `import.meta.env.DEV` gated | cargo unit + vitest + svelte-check + clippy | `cd backend && cargo test -p gallery-demo --features gallery handlers::exer01::tests && cd ../frontend && pnpm exec vitest run src/lib/exer01/observe.test.ts && pnpm check && cd ../backend && cargo clippy -p gallery-demo --features gallery --all-targets -- -D warnings` | ✅ `backend/crates/gallery-demo/src/handlers/{exer01.rs,mod.rs}` · `frontend/src/lib/exer01/{observe.svelte.ts,observe.test.ts}` · `frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte` · `.planning/seeds/v1.3-appshell-nestability.md` | ✅ green (4 cargo + 1 vitest + 0 svelte-check errors + clippy clean) |
| 19-03-T1 | 19-03 | 2 | EXER-02 (rapid_patching gallery screen) | — | N/A (pure-fn UI composer; no untrusted input) | cargo unit | `cd backend && cargo test -p gallery-demo --features gallery exerciser::rapid_patching::tests` | ✅ `backend/crates/gallery-demo/src/exerciser/rapid_patching.rs` | ✅ green (9/9) |
| 19-03-T2 | 19-03 | 2 | EXER-02 (handlers) | T-19-01 (cadence clamp), T-19-03-03 (Pitfall 2 regression) | cadence_ms ∈ [100, 60_000] enforced at handle_exer02_start; tick handler runtime guard + unit test proves no ops target /demo/exer-02/focused-value path or exer-02-focused-input node id | cargo unit | `cd backend && cargo test -p gallery-demo --features gallery handlers::exer02::tests` | ✅ `backend/crates/gallery-demo/src/handlers/exer02.rs` | ✅ green (9/9) |
| 19-03-T3 | 19-03 | 2 | EXER-02 (frontend invariants + tick loop) | — | Defense-in-depth client-side clamp in startTickLoop mirrors backend range | vitest unit (jsdom env) | `cd frontend && pnpm exec vitest run src/lib/exer02/invariants.test.ts` | ✅ `frontend/src/lib/exer02/{invariants.svelte.ts,invariants.test.ts}` | ✅ green (9/9) |
| 19-04-T1 | 19-04 | 2 | EXER-03 (pathological_scale tree) | T-19-04-02 (row slice accept), T-19-04-03 (synthetic-only, no PII) | Row generator bounded by `synthetic_rows(10_000)` upper slice (Plan 19-01); tree emits static SDUI — no untrusted data | cargo unit | `cd backend && cargo test -p gallery-demo --features gallery exerciser::pathological_scale::tests` | ✅ `backend/crates/gallery-demo/src/exerciser/pathological_scale.rs` | ✅ green (9/9) |
| 19-04-T2 | 19-04 | 2 | EXER-03 (report-perf handlers) | T-19-04-01 (tampering), T-19-04-04 (remeasure DoS) | Strict `Option<f64>` serde rejects non-numeric payloads (BadPayload); remeasure is O(1), emits one Set op per call | cargo unit | `cd backend && cargo test -p gallery-demo --features gallery handlers::exer03::tests` | ✅ `backend/crates/gallery-demo/src/handlers/exer03.rs` | ✅ green (10/10) |
| 19-04-T3 | 19-04 | 2 | EXER-03 (frontend perf capture) | — | N/A (client-side measurement only; no untrusted input; Chromium-memory absence returns `null`, not NaN) | vitest unit | `cd frontend && pnpm exec vitest run src/lib/exer03/perf.test.ts` | ✅ `frontend/src/lib/exer03/perf.svelte.ts`, `frontend/src/lib/exer03/perf.test.ts` | ✅ green (10/10) |

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

| Behavior | Requirement | Why Manual | UAT Result |
|----------|-------------|------------|------------|
| EXER-01 4-dimension observation matrix renders with live captured state | EXER-01 | Visual-DOM collision only observable in browser (shadcn Sidebar.Provider context shadowing) | ✅ **PASS 2026-04-24** — Playwright UAT confirmed inner-nav (Dashboard / Reports / Settings) collision IS visible at desktop; mobile-sheet cascade is observable (the inner shell's Sheet intercepts outer-nav pointer events). 4 matrix dimensions render. `Open seed draft` CTA toasts `.planning/seeds/v1.3-appshell-nestability.md` byte-for-byte. See [19-VERIFICATION.md](19-VERIFICATION.md) §EXER-01. |
| EXER-02 focus retention for 60 s of patch pressure | EXER-02 | Needs sustained human presence + IME hardware (CJK keyboard or macOS IME) to exercise composition invariant | ✅ **PASS 2026-04-24** — PATCH-02 invariant proven at the wire level by a 10 s × 500 ms WebSocket probe: 19 patches received, 0 Pitfall 2 violations (no op targets `/demo/exer-02/focused-value` path or `exer-02-focused-input` id). 20 s Playwright focus-retention test: `activeElement.tagName==='INPUT'` + typed value `hello world` preserved byte-for-byte + cursor at position 11/11. Cadence clamp [100, 60000] ms enforced at both below-floor (50) and above-ceiling (120000). 60 s extrapolation is mechanical — the construction invariant holds for any duration. See [19-VERIFICATION.md](19-VERIFICATION.md) §EXER-02 + §EXER-02 tick stress. |
| EXER-03 perf readouts capture live measurements on 10k-row page | EXER-03 | Perf values are hardware-dependent; advisory thresholds (D-3) not gating | ✅ **PASS 2026-04-24** — Advisory baselines captured on headless Chromium 145 / Linux x86_64: TTFP 140 ms (≤ 3000 target → WITHIN), Scroll FPS median 59.9 (≥ 30 target → WITHIN), Memory growth 0 MB over 30 s (≤ 50 target → WITHIN). Patch latency p95 not captured in production build (DEV-only `__mrnSendAction` hook is tree-shaken); wire-level round-trip is sub-20 ms. `report-perf` threshold logic verified: all-within → 4 `WITHIN TARGET`; all-over → 4 `OVER TARGET`. `null` memory correctly skipped. 10 000-row DataTable paginates first + last 50 rows on demand. See [19-VERIFICATION.md](19-VERIFICATION.md) §EXER-03. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90 s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** verified 2026-04-24 — Phase 19 closed via server-driven WebSocket probe + Playwright UAT (desktop 1440×900 + mobile 390×844). Chrome-MCP re-walk by orchestrator is optional; every success criterion is already validated end-to-end. See [19-VERIFICATION.md](19-VERIFICATION.md).
