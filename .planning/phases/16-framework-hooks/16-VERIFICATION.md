---
phase: 16-framework-hooks
verified: 2026-04-21T21:45:00Z
status: passed
score: 4/4 success criteria verified
overrides_applied: 0
re_verification:
  previous_status: none
  previous_score: null
  gaps_closed: []
  gaps_remaining: []
  regressions: []
---

# Phase 16: Framework Hooks Verification Report

**Phase Goal (ROADMAP.md):** The auto-discovery spine is in place — `#[gallery_demo]` proc macro, a stable registry-iteration API backed by `inventory` or `linkme`, and a `gallery` cargo feature gate that keeps production builds of `marionette` free of demo code.

**Verified:** 2026-04-21T21:45:00Z
**Status:** PASSED
**Re-verification:** No — initial verification
**Verifier:** Claude (gsd-verifier)

---

## Goal Achievement — Success Criteria

### SC1 — `#[gallery_demo]` proc macro + clear compile errors

**Status:** VERIFIED

**Evidence:**

- `backend/crates/marionette-macros/src/gallery_demo.rs` (229 LoC) implements the macro end-to-end:
  - `validate_item()` (lines 89–151) enforces, in order: visibility (`pub` required), zero args, non-async, no generics, no where-clause, `Node` return type. Each violation yields a distinct `syn::Error::new_spanned` error message that names the violated rule.
  - Non-`fn` items route through the `syn::parse2::<ItemFn>` failure branch (lines 37–44) producing `"#[gallery_demo] can only be applied to \`pub fn\` items (not structs, enums, modules, or other items)"`.
- `backend/crates/marionette-macros/src/lib.rs:96–99` exports `#[proc_macro_attribute] pub fn gallery_demo`.
- All 4 trybuild fixtures pass:

```
cargo test -p gallery-smoke --test ui_errors
    tests/ui/fail_applied_to_struct.rs ... ok
    tests/ui/fail_not_pub.rs ... ok
    tests/ui/fail_wrong_return.rs ... ok
    tests/ui/fail_wrong_signature.rs ... ok
test result: ok. 1 passed; 0 failed
```

- Error-message sampling confirms each names the violated rule:
  - `fail_not_pub.stderr:1` → `"#[gallery_demo] requires \`pub fn\` visibility (found private fn)"`
  - `fail_wrong_signature.stderr:1` → `"#[gallery_demo] fn must be \`fn() -> Node\` with zero arguments (found 1)"`
  - `fail_wrong_return.stderr:1` → `"#[gallery_demo] fn must return \`Node\` (an alias for \`(String, marionette_protocol::Component)\`)"`
  - `fail_applied_to_struct.stderr:1` → `"#[gallery_demo] can only be applied to \`pub fn\` items ..."`

### SC2 — `registered_demos()` API + logged choice

**Status:** VERIFIED

**Evidence:**

- `backend/crates/marionette/src/gallery.rs:25–36` defines `DemoEntry { key: &'static str, render: fn() -> Node, display_name: &'static str }` (matches D-C2 exactly).
- `gallery.rs:102–104` `pub fn registered_demos() -> impl Iterator<Item = &'static DemoEntry>` — always compiled, returns empty iterator under default (`build_sorted` at lines 87–90 returns `Vec::new()` when `gallery` feature off).
- Stable alphabetical-by-key ordering enforced at iteration time in `sort_entries()` (lines 57–79) with `OnceLock`-memoized cache at line 51.
- PROJECT.md line 132: `| linkme over inventory for gallery-demo registry | Type-safe \`#[distributed_slice]\` attribute binds element type to the static slice declaration; zero runtime cost (no ctor-style global initializers); explicit mental model ... Decision record: 16-CONTEXT.md D-A1. | ✓ Good — v1.2 Phase 16 |`
- Unit tests green:

```
cargo test -p marionette --lib gallery
  sort_entries_yields_alphabetical_order ... ok
  sort_entries_empty_input_yields_empty ... ok
  sort_entries_duplicate_panics_in_debug - should panic ... ok
  registered_demos_is_idempotent ... ok
test result: ok. 4 passed
```

### SC3 — Zero demo symbols under default build

**Status:** VERIFIED

**Evidence:**

- `backend/crates/marionette/tests/no_gallery_symbols.rs` shells `cargo build -p marionette` into two isolated `--target-dir`s and `nm --demangle`s the resulting `libmarionette.rlib`.
- Grep patterns (lines 83–88) probe the real markers: `gallery::DEMOS` (the `#[linkme::distributed_slice] pub static DEMOS`) and `__GALLERY_DEMO_` (per-entry static ident emitted by the proc macro). These are not trivially-matching patterns — they correspond to the exact symbol names emitted by the implementation in `gallery.rs:42` and `gallery_demo.rs:58–59`.
- `gallery.rs:40–42` — `DEMOS` is cfg-gated behind `#[cfg(feature = "gallery")]`.
- `gallery_demo.rs:71–83` — macro emits both the annotated fn AND its registration static under `#[cfg(feature = "gallery")]` (D-B1 satisfied).
- `marionette/Cargo.toml:22, 28–29` — `linkme = { workspace = true, optional = true }` + `[features] gallery = ["dep:linkme"]` (D-B3 idiom).
- Both subtests pass:

```
cargo test -p marionette --test no_gallery_symbols
  default_build_has_zero_demo_symbols ... ok
  gallery_feature_build_has_demo_symbols ... ok
test result: ok. 2 passed
```

### SC4 — End-to-end smoke test

**Status:** VERIFIED

**Evidence:**

- `backend/crates/gallery-smoke/` exists as a workspace member (registered at `backend/Cargo.toml:8`).
- `gallery-smoke/src/lib.rs:23–27` registers a real `#[gallery_demo(key = "smoke", name = "Smoke Check")] pub fn smoke() -> Node { Text::new("gallery-smoke").build() }` (not mocked).
- `tests/registry_roundtrip.rs` — 4 tests:
  - `force_link_smoke_demo` — references `smoke` as `fn() -> marionette::gallery::Node` to force linker retention.
  - `smoke_demo_is_registered` — iterates `registered_demos()`, asserts `key == "smoke"` present with `display_name == "Smoke Check"`.
  - `registry_is_alphabetically_ordered` — asserts keys yielded in sorted order.
  - `registry_iteration_is_idempotent` — asserts memoized iteration across two calls is deterministic.
- All pass:

```
cargo test -p gallery-smoke --test registry_roundtrip
  force_link_smoke_demo ... ok
  registry_is_alphabetically_ordered ... ok
  smoke_demo_is_registered ... ok
  registry_iteration_is_idempotent ... ok
test result: ok. 4 passed
```

---

## Success-Criteria Summary

| # | Criterion | Status | Score |
|---|-----------|--------|-------|
| SC1 | `#[gallery_demo]` proc macro + misuse errors name violated rule | VERIFIED | 1/1 |
| SC2 | `registered_demos()` stable-ordered iterator, linkme choice logged | VERIFIED | 1/1 |
| SC3 | Zero demo symbols under default build; present under `--features gallery` | VERIFIED | 1/1 |
| SC4 | End-to-end smoke test registers + iterates + asserts stable order | VERIFIED | 1/1 |

**Score: 4/4 success criteria verified**

---

## Requirements Coverage

| ID | Plan(s) | Description | Status | Evidence |
|----|---------|-------------|--------|----------|
| FRAME-01 | 16-02, 16-03 | `#[gallery_demo]` proc macro with signature/visibility validation; compile errors name violated rule | ✓ SATISFIED | `marionette-macros/src/gallery_demo.rs:89–151` (validate_item); 4 trybuild fixtures pass (`fail_not_pub`, `fail_wrong_signature`, `fail_wrong_return`, `fail_applied_to_struct`) |
| FRAME-02 | 16-01, 16-04 | `registered_demos()` iterator API backed by `linkme`; choice logged | ✓ SATISFIED | `marionette/src/gallery.rs:25–104`; PROJECT.md:132 Key Decisions row with rationale citing D-A1 |
| FRAME-03 | 16-01, 16-03 | `gallery` feature gate; zero demo symbols in default build | ✓ SATISFIED | `marionette/Cargo.toml:28–29`; `gallery.rs:40–42` cfg-gated DEMOS; `no_gallery_symbols.rs` 2/2 tests pass |

No orphaned requirements — REQUIREMENTS.md maps FRAME-01/02/03 to Phase 16 (table at line 70–72); all three delivered.

---

## CONTEXT.md Locked-Decision Compliance

| Decision | Claim | Implementation Evidence | Status |
|----------|-------|-------------------------|--------|
| D-A1 | linkme chosen (not inventory) | `Cargo.toml:34` workspace dep `linkme = "0.3"`; `marionette/Cargo.toml:22,29` feature gate; PROJECT.md:132 Key Decisions row | ✓ |
| D-A2 | Sort at iteration time (alphabetical) | `gallery.rs:57–79` `sort_entries()` does `v.sort_by_key(|e| e.key)` | ✓ |
| D-A3 | debug_assert on duplicate; log-and-keep-first in release | `gallery.rs:62–75` `debug_assert!` + `tracing::warn!` + `dedup_by` (keep-first) | ✓ |
| D-A4 | OnceLock memoization | `gallery.rs:51` `static SORTED_CACHE: OnceLock<Vec<&'static DemoEntry>>`; `get_or_init(build_sorted)` at line 103 | ✓ |
| D-B1 | Macro gates BOTH fn + registration behind `#[cfg(feature = "gallery")]` | `gallery_demo.rs:71–83` — both `#func` and the linkme static are under `#[cfg(feature = "gallery")]` | ✓ |
| D-B3 | `gallery` feature on marionette; pulls optional linkme via `dep:` | `marionette/Cargo.toml:28–29` `gallery = ["dep:linkme"]`; line 22 `linkme = { workspace = true, optional = true }` | ✓ |
| D-B4 | `registered_demos()` always compiled (empty under default) | `gallery.rs:102–104` (no cfg guard); `build_sorted` has both `#[cfg(feature = "gallery")]` (lines 81–85) and `#[cfg(not(feature = "gallery"))]` (lines 87–90) variants | ✓ |
| D-C1 | Key defaults from fn ident; `key = "..."` overrides | `gallery_demo.rs:54` `opts.key.unwrap_or_else(|| fn_ident.to_string())` | ✓ |
| D-C2 | DemoEntry = { key, render, display_name } | `gallery.rs:25–36` — exact fields, no `component_type`, no source-location | ✓ |
| D-C3 | display_name default = title-cased key | `gallery_demo.rs:55` + `title_case` (171–188); 7 unit tests in `gallery_demo.rs:194–227` all pass (hyphen/underscore/mixed-case-tail/empty/trailing-sep/double-sep) | ✓ |
| D-D1 | FRAME-03 via `nm` symbol-grep | `marionette/tests/no_gallery_symbols.rs` shells `cargo build` + `nm --demangle`, greps `gallery::DEMOS`/`__GALLERY_DEMO_` | ✓ |
| D-D2 | `gallery-smoke` is a workspace member | `backend/Cargo.toml:8` — `"crates/gallery-smoke"` listed | ✓ |
| D-D3 | `gallery-smoke` is permanent (no "retire in Phase 17") | `gallery-smoke/src/lib.rs:10–11` explicitly says "NOT retired after Phase 17"; 16-03-SUMMARY.md Hand-Off Notes reiterates | ✓ |
| D-D4 | 4 trybuild fixture pairs | `tests/ui/` has `fail_{not_pub,wrong_signature,wrong_return,applied_to_struct}.{rs,stderr}` — 4 pairs | ✓ |

**All 14 locked decisions implemented as specified.**

---

## Integration Sanity

| Check | Command | Result |
|-------|---------|--------|
| Default marionette build | `cargo build -p marionette` | ✓ green |
| Feature-on marionette build | `cargo build -p marionette --features gallery` | ✓ green |
| Workspace build (excl. crm-demo) | `cargo build --workspace --exclude crm-demo` | ✓ green |
| Workspace tests (excl. crm-demo) | `cargo test --workspace --exclude crm-demo` | ✓ all green (14 result buckets, 0 failures, including 7 marionette-macros title-case tests, 4 gallery unit tests, 4 registry_roundtrip tests, 2 no_gallery_symbols tests, 4 trybuild fixtures) |
| Clippy on Phase 16 code (macros + smoke) | `cargo clippy -p marionette-macros -p gallery-smoke --all-features --tests -- -D warnings` | ✓ zero warnings |
| Clippy on marionette lib + FRAME-03 test | `cargo clippy -p marionette --lib --all-features --test no_gallery_symbols -- -D warnings` | ✓ zero warnings |

---

## STATE.md / PROJECT.md Closure

| Check | Result |
|-------|--------|
| PROJECT.md Key Decisions row names `linkme` with rationale + D-A1 reference | ✓ line 132 |
| STATE.md "Registration library selection" blocker marked resolved (✅ prefix) | ✓ line 88 |
| STATE.md "Phase 17 hand-off" subsection exists warning about key-collision | ✓ line 71 |
| STATE.md [v1.2 Phase 16] decision bullet recorded | ✓ line 69 |

---

## Anti-Patterns Scan

| File | Pattern | Severity | Notes |
|------|---------|----------|-------|
| — | No TODO/FIXME/XXX/HACK/PLACEHOLDER comments in Phase 16 files | — | Clean |
| — | No stub returns (`return null`, empty arrays, `=> {}`) | — | All implementations substantive |
| — | No console.log-only handlers | — | N/A (Rust) |
| `gallery.rs:56` | `#[cfg_attr(not(any(feature = "gallery", test)), allow(dead_code))]` on `sort_entries()` | ℹ️ Info | Documented deviation in 16-01-SUMMARY.md — precisely scoped to configs where `sort_entries` is genuinely unreachable (only called via feature-gated `build_sorted` or test). Not a stub. |

No blocking anti-patterns. All implementations are substantive and wired.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| gallery-smoke's `smoke` demo appears in registry with correct display_name | `cargo test -p gallery-smoke --test registry_roundtrip smoke_demo_is_registered` | ok | ✓ PASS |
| Registry iteration is alphabetical | `cargo test -p gallery-smoke --test registry_roundtrip registry_is_alphabetically_ordered` | ok | ✓ PASS |
| Duplicate key panics in debug | `cargo test -p marionette --lib gallery sort_entries_duplicate_panics_in_debug` | ok (should_panic caught) | ✓ PASS |
| Default marionette.rlib has zero demo symbols | `cargo test -p marionette --test no_gallery_symbols default_build_has_zero_demo_symbols` | ok | ✓ PASS |
| Gallery-enabled marionette.rlib has demo symbols | `cargo test -p marionette --test no_gallery_symbols gallery_feature_build_has_demo_symbols` | ok | ✓ PASS |
| title_case rendering (7 cases) | `cargo test -p marionette-macros --lib title_case` | 7/7 ok | ✓ PASS |

All behavioral checks green.

---

## Open Issues

**None.**

Two pre-existing items noted for context (not Phase 16 gaps):
- `crm-demo` has pre-existing pedantic drift and was correctly excluded from the workspace test/build sweeps (out of Phase 16 scope, consistent with the verification prompt).
- Pre-existing `session.rs` private-link rustdoc warning (carried from prior phases, logged in 16-01-SUMMARY.md as out of scope).

---

## Gaps Summary

None. All four ROADMAP success criteria are verified against the actual codebase:

- The `#[gallery_demo]` macro exists, validates signatures, and produces crisp named-rule errors (SC1).
- `registered_demos()` delivers a stable alphabetically-sorted iterator of `DemoEntry { key, render, display_name }` (SC2).
- The `gallery` feature gate cleanly zero-izes demo symbols in the default `libmarionette.rlib` and brings them back when enabled (SC3).
- The `gallery-smoke` crate round-trips a real `#[gallery_demo]` registration through the linkme slice end-to-end across crate boundaries (SC4).

All 14 locked decisions (D-A1..D-D4) are reflected in the implementation. FRAME-01/02/03 are satisfied with concrete implementation + test evidence. PROJECT.md and STATE.md documentation-closure requirements are met.

---

*Verified: 2026-04-21T21:45:00Z*
*Verifier: Claude (gsd-verifier)*
