---
phase: 10
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-08
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (existing) + manual build check |
| **Config file** | `frontend/vitest.config.ts` |
| **Quick run command** | `cd frontend && npx vite build 2>&1 | tail -5` |
| **Full suite command** | `cd frontend && npm run build && npx vitest run` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend && npx vite build 2>&1 | tail -5`
- **After every plan wave:** Run `cd frontend && npm run build && npx vitest run`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | FOUND-01 | — | N/A | build | `cd frontend && npx vite build` | ✅ | ⬜ pending |
| 10-02-01 | 02 | 1 | FOUND-02 | — | N/A | build+grep | `cd frontend && npx vite build && ! grep -r flowbite src/app.css` | ✅ | ⬜ pending |
| 10-03-01 | 03 | 2 | FOUND-03 | — | N/A | grep | `! grep -r 'flowbite' frontend/package.json` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements. Build validation and grep checks suffice.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual rendering of stubs | FOUND-01 | Stubs render visual output that requires human eye-check | Start dev server, navigate to demo page, confirm components render |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
