---
phase: 8
slug: crm-features
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 8 — Validation Strategy

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
| Interaction timeline visual | CRM-11 | Visual layout | Open contact, verify timeline entries with type icons |
| Tag chips display | CRM-08 | Visual rendering | View contact list, verify colored tag chips |
| Search + filter combination | CRM-07, CRM-09 | Interactive UX | Search + add filter, verify combined results |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
