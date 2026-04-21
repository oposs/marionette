---
phase: 16
slug: framework-hooks
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-21
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (unit + integration) + `trybuild` for compiler-error fixtures |
| **Config file** | `backend/Cargo.toml` workspace manifest; per-crate `Cargo.toml` in `marionette`, `marionette-macros`, `gallery-smoke` |
| **Quick run command** | `cargo test -p marionette-macros --lib && cargo test -p marionette --lib` |
| **Full suite command** | `cargo test --workspace --all-features` |
| **Symbol test (FRAME-03)** | `cargo test -p marionette --test no_gallery_symbols` (requires built rlib under `backend/target/`) |
| **Clippy gate** | `cargo clippy -p marionette-macros -p marionette -p gallery-smoke -- -D warnings` |
| **Estimated runtime** | Quick: ~15s · Full: ~45s · Symbol test: ~20s (includes two subprocess `cargo build` invocations with isolated `--target-dir`) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <touched-crate> --lib` (fast — affected crate only)
- **After every plan wave:** Run `cargo test --workspace --all-features` + `cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite + symbol test must be green
- **Max feedback latency:** 20 seconds per-task; 60 seconds per-wave

---

## Per-Task Verification Map

Filled by the planner during plan creation (one row per `<task>` in every PLAN.md). Requirement column maps FRAME-01/02/03 and decision IDs (D-A1..D-D4) from CONTEXT.md. Threat Ref column cross-references `<threat_model>` entries in each PLAN.md (populated once plans exist). Until plans land, this table is a stub.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 16-01-01 | 01 | 1 | FRAME-02, D-A1, D-A2, D-A3, D-A4, D-B1, D-B3, D-B4, D-C2 | T-16-01 | `DEMOS` static is absent under default build; collision check panics in debug | unit + integration | `cargo test -p marionette --lib` | ❌ W0 | ⬜ pending |
| 16-02-01 | 02 | 1 | FRAME-01, D-B1, D-C1, D-C3, D-C4 | T-16-02 | Macro rejects non-`pub`, non-`fn() -> Node` items at compile time | proc-macro unit tests + trybuild | `cargo test -p marionette-macros --lib && cargo test -p gallery-smoke --test ui_errors` | ❌ W0 | ⬜ pending |
| 16-03-01 | 03 | 2 | FRAME-03, FRAME-04, D-D1, D-D2, D-D4 | T-16-03 | FRAME-03 symbol-grep: zero `gallery_demo`/`DEMOS` symbols under default build; matches under `--features gallery` | integration + symbol | `cargo test -p marionette --test no_gallery_symbols && cargo test -p gallery-smoke --test registry_roundtrip` | ❌ W0 | ⬜ pending |
| 16-04-01 | 04 | 3 | D-A1 (Key Decision log) | — | Docs state registration-library decision + rationale | doc-link check | `grep -q 'linkme' .planning/PROJECT.md && grep -q 'registration-library' .planning/STATE.md` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Rows refined by planner; task IDs and commands align once PLAN.md files land.*

---

## Wave 0 Requirements

- [ ] `backend/crates/marionette/tests/no_gallery_symbols.rs` — FRAME-03 symbol-table grep (uses `nm`; isolated `--target-dir` per subtest to avoid rlib cache thrash — see RESEARCH §4)
- [ ] `backend/crates/gallery-smoke/Cargo.toml` — new workspace member, inherits `workspace.dependencies.linkme`
- [ ] `backend/crates/gallery-smoke/src/lib.rs` — `#[gallery_demo(key = "smoke", name = "Smoke Check")] pub fn smoke() -> Node`
- [ ] `backend/crates/gallery-smoke/tests/registry_roundtrip.rs` — asserts `"smoke"` entry present in `registered_demos()` with correct `display_name`
- [ ] `backend/crates/gallery-smoke/tests/ui/` — trybuild fixtures: `fail_not_pub.rs`, `fail_wrong_signature.rs`, `fail_wrong_return.rs`, `fail_applied_to_struct.rs` (+ matching `.stderr`)
- [ ] `backend/crates/gallery-smoke/tests/ui_errors.rs` — trybuild harness invoking `TestCases::compile_fail("tests/ui/fail_*.rs")`
- [ ] `backend/Cargo.toml` — register `crates/gallery-smoke` workspace member + add `linkme = { version = "0.3", optional = false }` to `[workspace.dependencies]`
- [ ] `backend/crates/marionette/Cargo.toml` — `[features] gallery = ["dep:linkme"]`, `linkme = { workspace = true, optional = true }`
- [ ] `backend/crates/marionette/src/gallery.rs` (or `gallery/mod.rs`) — `DemoEntry`, `DEMOS` (cfg-gated), `registered_demos()` with memoized sort + collision check
- [ ] `backend/crates/marionette-macros/src/gallery_demo.rs` — attribute-macro impl (darling for args, syn for signature validation)

*Every bullet above must exist before the corresponding automated command can run green.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| PROJECT.md Key Decisions row for linkme registration-library choice | FRAME-02 (choice logged with rationale) | Documentation quality — autofixable but meaningful review is by human | Inspect `.planning/PROJECT.md` §Key Decisions for a new row naming `linkme` with one-sentence rationale |
| `cargo doc -p marionette --features gallery` renders public API cleanly | D-C2 (DemoEntry), D-B4 (registered_demos always compiled) | rustdoc rendering is visual | Run `cargo doc -p marionette --features gallery --open`; confirm `marionette::gallery::{DemoEntry, registered_demos}` appear with doc comments |
| Clippy pedantic stays clean for new code | Project convention (`#![warn(clippy::pedantic)]` in marionette-macros) | Pedantic churn is occasionally context-dependent | `cargo clippy -p marionette-macros -p marionette -p gallery-smoke --all-features -- -D warnings -W clippy::pedantic` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s per wave
- [ ] `nyquist_compliant: true` set in frontmatter after planner fills Per-Task Verification Map

**Approval:** pending
