---
phase: 2
slug: protocol-specification
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Redocly CLI 2.24.0 (OpenAPI validation), Spectral 6.15.0 (style linting) |
| **Config file** | `spec/package.json` (tooling), `spec/.redocly.yaml` (validation config) |
| **Quick run command** | `cd spec && npx @redocly/cli lint openapi.yaml` |
| **Full suite command** | `cd spec && npx @redocly/cli lint openapi.yaml && npx @redocly/cli bundle openapi.yaml -o /dev/null` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd spec && npx @redocly/cli lint openapi.yaml`
- **After every plan wave:** Run full suite + `npx @redocly/cli bundle openapi.yaml -o /dev/null`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01 | 01 | 1 | PROT-01,02,03 | lint | `cd spec && npx @redocly/cli lint openapi.yaml` | ❌ W0 | ⬜ pending |
| 02-02 | 01 | 1 | PROT-04,05 | lint | `cd spec && npx @redocly/cli lint openapi.yaml` | ❌ W0 | ⬜ pending |
| 02-03 | 02 | 1 | PROT-06,07,08,09,10 | lint | `cd spec && npx @redocly/cli lint openapi.yaml` | ❌ W0 | ⬜ pending |
| 02-04 | 02 | 1 | PROT-11,12 | lint | `cd spec && npx @redocly/cli lint openapi.yaml` | ❌ W0 | ⬜ pending |
| 02-05 | 03 | 2 | PROT-13,14 | lint | `cd spec && npx @redocly/cli lint openapi.yaml` | ❌ W0 | ⬜ pending |
| 02-06 | 04 | 2 | DOC-01 | bundle | `cd spec && npx @redocly/cli bundle openapi.yaml -o /dev/null` | ❌ W0 | ⬜ pending |
| 02-07 | 04 | 2 | DOC-02 | manual | Review spec/PROTOCOL.md for completeness | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `spec/package.json` — Redocly CLI + Spectral dev dependencies
- [ ] `spec/.redocly.yaml` — Validation configuration
- [ ] `spec/openapi.yaml` — Entry point file (even if minimal)

*All spec tooling must exist before validation can run.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Protocol manual readability | DOC-02 | Requires human judgment on clarity | Read spec/PROTOCOL.md, verify a developer could implement from it |
| Swagger UI rendering | DOC-01 | Requires visual browser check | Run `npx @redocly/cli preview-docs openapi.yaml`, verify renders |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
