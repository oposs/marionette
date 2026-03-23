---
phase: 7
slug: crm-core
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (backend integration) |
| **Config file** | Cargo default |
| **Quick run command** | `cd backend && cargo test -p crm-demo` |
| **Full suite command** | `cd backend && cargo test -p crm-demo && cargo clippy -p crm-demo -- -D warnings` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd backend && cargo test -p crm-demo`
- **After every plan wave:** Run full suite with clippy
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Contact/company form layout | CRM-05 | Visual assessment | Open browser, create a contact, verify form fields render |
| Virtual scroll performance | CRM-04 | Subjective smoothness | Load 100+ contacts, scroll through table |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
