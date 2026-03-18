---
phase: 1
slug: project-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.0 (frontend), cargo test (backend) |
| **Config file** | `frontend/vite.config.ts` (inline), none (backend) |
| **Quick run command** | `cd frontend && npx vitest --run && cd ../backend && cargo test` |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `make lint && make test`
- **After every plan wave:** Run `make build && make test && make lint`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01 | 01 | 1 | INFRA-02 | smoke | `test -d frontend/src/lib && test -d backend/crates/marionette-protocol` | ❌ W0 | ⬜ pending |
| 01-02 | 01 | 1 | INFRA-01 | smoke | `make build && make test && make lint` | ❌ W0 | ⬜ pending |
| 01-03 | 01 | 1 | INFRA-05 | smoke | `make lint` | ❌ W0 | ⬜ pending |
| 01-04 | 01 | 1 | INFRA-04 | manual-only | `make dev` + manual browser check | N/A | ⬜ pending |
| 01-05 | 01 | 1 | INFRA-03 | manual-only | Push to branch, observe GitHub Actions | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `frontend/` — entire SvelteKit project needs scaffolding
- [ ] `backend/` — Cargo workspace and all crates need scaffolding
- [ ] `spec/` — placeholder directory needed
- [ ] `Makefile` — needs creation
- [ ] `.github/workflows/ci.yml` — needs creation

*All directories and config files must exist before any validation can run.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Dev servers start concurrently | INFRA-04 | Requires running processes + browser | Run `make dev`, verify Vite on :5173 and cargo on :3001 |
| CI workflow runs on PR | INFRA-03 | Requires GitHub Actions runner | Push branch, open PR, verify jobs pass |
| Vite proxy forwards /api/* | INFRA-04 | Requires both servers running | `make dev` then `curl localhost:5173/api/health` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
