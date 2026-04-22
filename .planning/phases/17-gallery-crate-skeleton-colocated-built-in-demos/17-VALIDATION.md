---
phase: 17
slug: gallery-crate-skeleton-colocated-built-in-demos
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-22
---

# Phase 17 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (workspace-native). Frontend: existing vitest for component tests. E2E: Chrome MCP drives UAT per `feedback_use_chrome_for_uat.md`. |
| **Config file** | `backend/Cargo.toml` (test targets auto-discovered). No separate test config file. |
| **Quick run command** | `cd backend && cargo test -p gallery-demo` |
| **Full suite command** | `cd backend && cargo test --workspace --features gallery` |
| **Estimated runtime** | ~60s quick (one crate), ~240s full workspace |

---

## Sampling Rate

- **After every task commit:** Run `cd backend && cargo test -p gallery-demo` (or the nearest-affected crate if the task only touched `marionette` / `marionette-macros` / `gallery-smoke`)
- **After every plan wave:** Run `cd backend && cargo test --workspace --features gallery`
- **Before `/gsd-verify-work`:** Full workspace test suite must be green, AND Chrome MCP walks every `registered_demos()` key in the running gallery without surfacing any `ErrorMessage` in the dispatcher console
- **Max feedback latency:** ~60s for quick, ~240s for full

---

## Per-Task Verification Map

*Populated by the planner during plan writing — each task's `<automated>` block points at one of the test commands below.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 17-0X-0Y | Phase 16.5 refactor | 0 | D-Z1 | — | `DemoEntry.render: fn() -> Vec<Node>` signature change compiles, `gallery-smoke` still passes | unit | `cd backend && cargo test -p gallery-smoke --features gallery` | ✅ existing | ⬜ pending |
| 17-0X-0Y | Phase 16.5 refactor | 0 | D-Z1 | — | trybuild fixtures catch `fn() -> Node` (old sig) as an error | unit | `cd backend && cargo test -p gallery-smoke --test ui_errors` | ✅ existing (update) | ⬜ pending |
| 17-0X-0Y | gallery-demo skeleton | 1 | CRATE-01 | T-17-01 (V5 input validation) | App boots on :3002; `gallery-show` with unknown key returns `ActionError::NotFound` | integration | `cd backend && cargo test -p gallery-demo --test smoke_boot` | ❌ W0 — new file | ⬜ pending |
| 17-0X-0Y | gallery-demo skeleton | 1 | CRATE-01 | — | Thin backend: no `sea_orm::Migrator`, no `bcrypt` | grep | `grep -L 'sea_orm::Migrator\|bcrypt' backend/crates/gallery-demo/src/main.rs` | n/a (CLI) | ⬜ pending |
| 17-0X-0Y | gallery-demo skeleton | 1 | CRATE-01 | — | In-memory `Arc<RwLock<_>>` state only | grep | `grep -q 'Arc<RwLock' backend/crates/gallery-demo/src/main.rs` | n/a (CLI) | ⬜ pending |
| 17-0X-0Y | nav auto-discovery | 1 | CRATE-02 | — | Nav Render contains one NavItem per `registered_demos()` entry | integration | `cd backend && cargo test -p gallery-demo --test nav_auto_discovery` | ❌ W0 — new file | ⬜ pending |
| 17-0X-0Y | built-in coverage | 2 | DEMO-01 | — | Every in-scope builder has a `gallery_demo` sibling; skipped builders are NOT registered | unit | `cd backend && cargo test -p marionette --features gallery --lib gallery::builtin_coverage` | ❌ W0 — new module | ⬜ pending |
| 17-0X-0Y | docs | 3 | DEMO-02 | — | `GALLERY-DEMOS.md` exists and documents the pure-fn contract | grep | `[ -f backend/crates/marionette/GALLERY-DEMOS.md ] && grep -q 'pure fn\|no I/O' backend/crates/marionette/GALLERY-DEMOS.md` | ❌ W0 — new file | ⬜ pending |
| 17-0X-0Y | UAT | 3 | SC #5 (ROADMAP) | — | Every registered demo renders without producing an `ErrorMessage` in dispatcher console | manual | Chrome MCP walk (see UAT section below) | manual | ⬜ pending |

Note: the planner will assign exact Task IDs and plan numbers during plan writing. The table above is pre-populated with requirement coverage so the planner can map each requirement to at least one task's `<automated>` block.

---

## Wave 0 Requirements

New test files this phase introduces (required before any Wave-1 execution):

- [ ] `backend/crates/gallery-demo/tests/smoke_boot.rs` — covers CRATE-01 (boot path, port :3002 binding, static file serving from `../frontend/build`, unknown-key handling)
- [ ] `backend/crates/gallery-demo/tests/nav_auto_discovery.rs` — covers CRATE-02 (nav iteration from `registered_demos()`; asserts entry count + ordering matches registry output)
- [ ] `backend/crates/marionette/src/gallery.rs` — add `#[cfg(all(test, feature = "gallery"))] mod builtin_coverage_tests` that asserts the expected set of ~19 built-in keys is present in the registry (DEMO-01) and that documented-skip keys (`surface-mount`, `nav-item`, `nav-group`, `field-separator`, `side-nav`, `container`) are NOT present
- [ ] `backend/crates/marionette/GALLERY-DEMOS.md` — covers DEMO-02 (the doc itself is a deliverable)
- [ ] Chrome MCP walk protocol documented in phase UAT notes (not automated; manual verification per `feedback_use_chrome_for_uat.md`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Every nav entry produces a screen, not an error surface | ROADMAP SC #5 | Visual; the contract is "not an error surface" — subjective until automated per-demo Playwright tests land in Phase 18/19 | Drive Chrome MCP: navigate to `http://localhost:3002`, for each demo key in the nav, click the NavItem, wait for content Render, verify no `ErrorMessage` in dispatcher console, capture screenshot |
| Adding a new `#[gallery_demo]` auto-surfaces in nav | CRATE-02 | Confirms the full compile-time → link-time → runtime iteration pipeline works for new entries | Manual: add a throwaway `#[gallery_demo(key = "scratch")] pub fn gallery_demo() -> Vec<Node> { ... }` to any builder file; `cargo run -p gallery-demo`; observe "scratch" appears in nav |
| Composite demos (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast, AppShell) render meaningful mini-compositions | D-A1, D-A2 | Visual quality assessment; no automated test distinguishes "meaningful" from "minimal" | Chrome MCP walk + visual review |
| Modal demo's trigger + close flow works end-to-end | D-A4 | The `close-modal` frontend-hardcoded action must route to a registered backend handler that clears the modal sub-surface. Open Question #2 flagged in research | Chrome MCP: click "Open modal" trigger → modal opens → click close (×) → modal closes cleanly |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify pointing at a command from this doc, or a Wave 0 dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags (tests run one-shot, return exit code)
- [ ] Feedback latency < 240s for full workspace
- [ ] `nyquist_compliant: true` set in frontmatter after all tasks land and per-task verification is green

**Approval:** pending
