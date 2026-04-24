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
| TBD | TBD | TBD | EXER-01 | — | N/A (no untrusted input) | unit / browser-test | TBD | ⬜ W0 | ⬜ pending |
| TBD | TBD | TBD | EXER-02 | — | N/A | browser-test | TBD | ⬜ W0 | ⬜ pending |
| TBD | TBD | TBD | EXER-03 | — | N/A | cargo test + manual | TBD | ⬜ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Populated by gsd-planner. Candidate items from RESEARCH.md:*

- [ ] 16 new lucide icons appended to `frontend/src/lib/registry/icons.ts` defaults
- [ ] `installPatchProbe` hook in `frontend/src/lib/init.ts` (instrumentation shared by EXER-02 + EXER-03)
- [ ] Synthetic row generator parameter bump (`synthetic_rows(10_000)` path in `backend/crates/gallery-demo/src/fixtures.rs`)
- [ ] Spike of A1 — backend `PatchMessage` out-of-band push capability (gates EXER-02 Pattern 2; fallback is client-initiated tick)

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
