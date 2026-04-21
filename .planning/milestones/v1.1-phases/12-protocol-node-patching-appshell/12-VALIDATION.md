---
phase: 12
slug: protocol-node-patching-appshell
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-10
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Filled from 12-RESEARCH.md § Validation Architecture by the planner.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend unit + store), Playwright component tests (browser-test.ts), Playwright E2E (frontend/tests/e2e), `cargo test` (backend crates), ajv + js-yaml schema validation (frontend/tests/helpers/schema-validator.ts) |
| **Config file** | `frontend/vitest.config.ts`, `frontend/playwright.config.ts`, `backend/Cargo.toml`, `frontend/tests/helpers/schema-validator.ts` |
| **Quick run command** | `pnpm -C frontend test` (vitest unit) + `cargo test -p marionette-protocol` |
| **Full suite command** | `pnpm -C frontend test && pnpm -C frontend test:browser && pnpm -C frontend test:e2e -- --grep "protocol|appshell|focus" && cargo test --workspace` |
| **Estimated runtime** | ~120–180 seconds full suite |

---

## Sampling Rate

- **After every task commit:** Run `pnpm -C frontend test` (if frontend touched) OR `cargo test -p <affected-crate>` (if backend touched)
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green, including the focus-preservation browser test and the protocol-conformance E2E spec
- **Max feedback latency:** 30 seconds for the quick run, <3 minutes full suite

---

## Per-Task Verification Map

*The planner will populate this table as tasks are defined. Each row must map a task to a concrete REQ-ID, test type, and automated command.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| *TBD by planner* | | | | | | | | | |

---

## Wave 0 Requirements

Derived from RESEARCH.md (confirmed gaps). Wave 0 must land these before any test suite can cover PATCH/SHELL requirements:

- [ ] `frontend/src/lib/components/shell/AppShell.svelte` — scaffold file (new directory `shell/`)
- [ ] `frontend/src/lib/components/shell/AppShell.browser-test.ts` — test stub for AppShell rendering
- [ ] `frontend/src/lib/components/core/SurfaceMount.svelte` — new leaf component
- [ ] `frontend/src/lib/components/core/SurfaceMount.browser-test.ts` — test stub for nested Surface recursion
- [ ] `frontend/src/lib/store/surfaces.browser-test.ts` — test stub for focus preservation under node patches (MANDATORY per D-A6)
- [ ] `backend/crates/marionette/src/builders/app_shell.rs` — new hand-written builder file
- [ ] `backend/crates/marionette-protocol/tests/patch_operation_enum.rs` — unit tests for new tagged `PatchOperation` variants (set/set-node/delete-node/set-children/insert-child/remove-child), round-trip JSON
- [ ] `backend/crates/marionette-protocol/tests/patch_message_surface.rs` — unit tests for required `surface` field on `PatchMessage`
- [ ] `frontend/tests/e2e/node-patch-conformance.spec.ts` — E2E protocol-conformance spec for node patches (extends existing protocol-conformance.spec.ts)
- [ ] Verify `frontend/src/lib/components/ui/sidebar/` is installed via `npx shadcn-svelte add sidebar`; if missing, Wave 0 installs it
- [ ] Verify shadcn Toast (Radix) primitive is installed in `frontend/src/lib/components/ui/`; if missing, Wave 0 installs it
- [ ] Rename `--sidebar-background` → `--sidebar` (and audit all `bg-sidebar-background` usages) in `frontend/src/app.css` + `frontend/src/lib/components/nav/SideNav.svelte` + any other sites identified by grep

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual polish of sidebar collapse animation on mobile | SHELL-01 | Animation timing is subjective | Resize viewport <768px, click hamburger, observe sheet slide-in behaviour matches shadcn demo |
| Theme token visual check (--sidebar-* applied) | SHELL-03 | CSS visual | Toggle dark mode, confirm sidebar background/foreground/accent colors match tokens |
| CRM country-select demo flow (visual) | PATCH-02 / success criterion 8 | End-user UX check | Open Contact form, type in Name field, switch country, verify new fields appear and Name focus/cursor is preserved |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s quick-run
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
