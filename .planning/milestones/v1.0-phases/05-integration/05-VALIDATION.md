---
phase: 5
slug: integration
status: draft
nyquist_compliant: true
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
| **Config file** | `frontend/playwright.e2e.config.ts` (E2E), `frontend/playwright.config.ts` (component/visual, unchanged) |
| **Quick run command** | `cd backend && cargo test -p crm-demo` |
| **Full suite command** | `cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/` |
| **Estimated runtime** | ~30 seconds (backend tests), ~60 seconds (E2E with build) |

---

## Sampling Rate

- **After every task commit:** Run `cd backend && cargo test -p crm-demo`
- **After every plan wave:** Run full E2E suite with `--config playwright.e2e.config.ts`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds (backend tests), 120 seconds (E2E with build)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-Task1 | 05-01 | 1 | INTEG-01, INTEG-02 | unit/build | `cd backend && cargo test -p crm-demo && cargo clippy -p crm-demo -- -D warnings` | Yes (in-plan) | pending |
| 05-01-Task2 | 05-01 | 1 | INTEG-01, INTEG-02 | integration | `cd backend && cargo test -p crm-demo -- --nocapture` | W0 | pending |
| 05-02-Task1 | 05-02 | 2 | INTEG-03 | deps/helpers | `cd frontend && node -e "require('js-yaml'); require('ajv')" && test -f playwright.e2e.config.ts` | W0 | pending |
| 05-02-Task2 | 05-02 | 2 | INTEG-01, INTEG-02, INTEG-03 | E2E | `cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/` | W0 | pending |

---

## Wave 0 Requirements

- [ ] crm-demo wired with Axum router serving static files + WebSocket (05-01-Task1)
- [ ] Backend integration tests including SPA fallback assertion (05-01-Task2)
- [ ] Separate playwright.e2e.config.ts for E2E tests (05-02-Task1)
- [ ] E2E test helpers: ws-capture.ts, schema-validator.ts (05-02-Task1)

---

## Requirement Coverage

| Requirement | Backend Test | E2E Test | Specific Assertion |
|-------------|-------------|----------|-------------------|
| INTEG-01 (static + SPA) | 05-01-Task2: spa_fallback test | 05-02-Task2: "SPA fallback serves app for deep routes" | GET /some/deep/route returns index.html |
| INTEG-02 (round-trip) | 05-01-Task2: navigate + demo_click tests | 05-02-Task2: "navigate action triggers render", "button click sends action" | Render message with components, patch with data |
| INTEG-03 (conformance) | N/A | 05-02-Task2: protocol-conformance.spec.ts | AJV validates hello/render/action/patch against OpenAPI schemas |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual rendering in browser | INTEG-02 | Subjective visual check | `make dev`, open browser, verify components render |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s for backend tests
- [x] `nyquist_compliant: true` set in frontmatter
- [x] Task IDs match actual plan task names

**Approval:** pending
