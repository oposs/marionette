---
phase: 6
slug: crm-auth-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (backend unit + integration), Playwright (E2E) |
| **Config file** | Cargo default, `frontend/playwright.e2e.config.ts` |
| **Quick run command** | `cd backend && cargo test -p crm-demo` |
| **Full suite command** | `cd backend && cargo test -p crm-demo && cd ../frontend && npx playwright test --config playwright.e2e.config.ts` |
| **Estimated runtime** | ~30 seconds (backend), ~60 seconds (E2E) |

---

## Sampling Rate

- **After every task commit:** Run `cd backend && cargo test -p crm-demo`
- **After every plan wave:** Run full suite including E2E
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| user-entity | TBD | 1 | CRM-12 | unit | `cargo test -p crm-demo` | ❌ W0 | ⬜ pending |
| login-flow | TBD | 1 | CRM-13 | integration | `cargo test -p crm-demo` | ❌ W0 | ⬜ pending |
| user-mgmt | TBD | 2 | CRM-12 | integration | `cargo test -p crm-demo` | ❌ W0 | ⬜ pending |
| audit-trail | TBD | 2 | CRM-14 | integration | `cargo test -p crm-demo` | ❌ W0 | ⬜ pending |
| e2e-auth | TBD | 3 | CRM-13 | E2E | `npx playwright test --config playwright.e2e.config.ts` | ❌ W0 | ⬜ pending |

---

## Wave 0 Requirements

- [ ] User + audit_log SeaORM entities and migrations in crm-demo
- [ ] Login HTTP endpoint wired in Axum router
- [ ] Default admin seed on first startup

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Login form visual appearance | CRM-13 | Subjective styling check | Open browser, verify login form renders with Flowbite styling |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
