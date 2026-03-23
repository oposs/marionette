---
phase: 5
slug: integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Playwright (E2E), cargo test (backend integration) |
| **Config file** | `frontend/playwright.config.ts`, Cargo default |
| **Quick run command** | `cd backend && cargo test -p crm-demo` |
| **Full suite command** | `make build && cd frontend && npx playwright test tests/e2e/` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd backend && cargo test -p crm-demo`
- **After every plan wave:** Run full E2E suite
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| static-serve | TBD | 1 | INTEG-01 | integration | `cargo test -p crm-demo` | ❌ W0 | ⬜ pending |
| ws-roundtrip | TBD | 1 | INTEG-02 | E2E | `npx playwright test` | ❌ W0 | ⬜ pending |
| conformance | TBD | 2 | INTEG-03 | E2E | `npx playwright test` | ❌ W0 | ⬜ pending |

---

## Wave 0 Requirements

- [ ] crm-demo wired with Axum router serving static files + WebSocket
- [ ] Playwright E2E test infrastructure (already exists from Phase 3)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual rendering in browser | INTEG-02 | Subjective visual check | `make dev`, open browser, verify components render |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
