---
phase: 4
slug: backend-toolkit
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-20
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) |
| **Config file** | None needed (Cargo default) |
| **Quick run command** | `cd backend && cargo test` |
| **Full suite command** | `cd backend && cargo test && cargo clippy -- -D warnings` |
| **Estimated runtime** | ~30 seconds (first build), ~10 seconds (incremental) |

---

## Sampling Rate

- **After every task commit:** Run `cd backend && cargo test`
- **After every plan wave:** Run `cd backend && cargo test && cargo clippy -- -D warnings`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| protocol-types | TBD | 1 | BACK-03 | unit | `cargo test -p marionette-protocol` | ❌ W0 | ⬜ pending |
| proc-macros | TBD | 1 | BACK-02 | unit | `cargo test -p marionette-macros` | ❌ W0 | ⬜ pending |
| axum-handlers | TBD | 2 | BACK-01,04 | unit+integration | `cargo test -p marionette` | ❌ W0 | ⬜ pending |
| websocket | TBD | 2 | BACK-06 | integration | `cargo test -p marionette` | ❌ W0 | ⬜ pending |
| sea-orm | TBD | 2 | BACK-05 | integration | `cargo test -p marionette` | ❌ W0 | ⬜ pending |
| auth | TBD | 2 | BACK-07 | unit | `cargo test -p marionette` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `backend/crates/marionette-protocol/` — protocol types crate (stub exists)
- [ ] `backend/crates/marionette-macros/` — proc macro crate (stub exists)
- [ ] `backend/crates/marionette/` — main library crate (stub exists)
- [ ] SeaORM + SQLite dependencies added to workspace

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WebSocket message flow | BACK-06 | Requires live connection | Start crm-demo, connect WS client, verify hello message |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
