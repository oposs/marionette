---
phase: 3
slug: frontend-library
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-19
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.0 (unit), vitest-browser-svelte 2.1.0 (component), Playwright (visual/E2E) |
| **Config file** | `frontend/vite.config.ts` (Vitest inline), `frontend/playwright.config.ts` (visual) |
| **Quick run command** | `cd frontend && npx vitest --run` |
| **Full suite command** | `cd frontend && npx vitest --run && npx playwright test` |
| **Estimated runtime** | ~30 seconds (unit), ~60 seconds (browser+visual) |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend && npx vitest --run`
- **After every plan wave:** Run full suite including browser tests
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds (unit), 90 seconds (full)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| data-store | TBD | 1 | FRONT-01 | unit | `npx vitest --run src/lib/store` | ❌ W0 | ⬜ pending |
| registry | TBD | 1 | FRONT-02 | unit | `npx vitest --run src/lib/registry` | ❌ W0 | ⬜ pending |
| websocket | TBD | 1 | FRONT-03,05 | unit | `npx vitest --run src/lib/ws` | ❌ W0 | ⬜ pending |
| surface | TBD | 1 | FRONT-04 | browser | `npx vitest --run --browser` | ❌ W0 | ⬜ pending |
| optimistic | TBD | 1 | FRONT-06 | unit | `npx vitest --run src/lib/store` | ❌ W0 | ⬜ pending |
| dirty | TBD | 1 | FRONT-07 | unit | `npx vitest --run src/lib/store` | ❌ W0 | ⬜ pending |
| routing | TBD | 2 | FRONT-08 | unit | `npx vitest --run src/lib/router` | ❌ W0 | ⬜ pending |
| components | TBD | 2 | FRONT-10-16 | browser | `npx vitest --run --browser` | ❌ W0 | ⬜ pending |
| visual | TBD | 3 | FRONT-25-27 | visual | `npx playwright test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Vitest browser mode configured in vite.config.ts
- [ ] Playwright installed and configured for visual tests
- [ ] Test helper: mock WebSocket server for connection tests
- [ ] Test helper: mock store for component tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Flowbite styling matches design | FRONT-16 | Visual assessment | Open component test pages, verify Flowbite styles applied |
| Virtual scroll feels smooth | FRONT-13 | Subjective UX | Scroll large table, verify no jank |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (unit), < 90s (full)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
