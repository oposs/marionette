---
phase: 9
slug: crm-listmonk
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (backend) |
| **Config file** | Cargo default |
| **Quick run command** | `cd backend && cargo test -p crm-demo` |
| **Full suite command** | `cd backend && cargo test -p crm-demo && cargo clippy -p crm-demo -- -D warnings` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd backend && cargo test -p crm-demo`
- **After every plan wave:** Run full suite with clippy
- **Max feedback latency:** 15 seconds

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Listmonk sync actually works | CRM-15 | Requires running Listmonk instance | Start Listmonk, set env vars, sync a contact, verify subscriber created |
| Mailing history displays | CRM-16 | Requires Listmonk with campaign data | Send campaign in Listmonk, view contact, verify history shows |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
