---
phase: 16-framework-hooks
plan: 02
subsystem: proc-macro
tags: [rust, proc-macro, darling, syn, attribute-macro, linkme, gallery, auto-discovery]

# Dependency graph
requires:
  - phase: 15-crm-migration-validation
    provides: v1.1 baseline (clean-break pre-deployment posture, edition-2024 toolchain, clippy::pedantic bar)
provides:
  - "#[gallery_demo] attribute proc macro exported from marionette_macros"
  - "Darling-based attribute argument parser for key/name overrides"
  - "Syn-based signature validation (pub, zero-arg, non-async, non-generic, no where-clause, returns Node)"
  - "Cfg-gated emission of fn + linkme distributed_slice static using ::marionette::gallery::{DemoEntry, DEMOS, __linkme} absolute paths"
  - "Title-case helper with 7 inline unit tests (ASCII split-on [- _], first-char uppercase, preserves mixed-case tails)"
affects: [16-01-registry-crate, 16-03-gallery-smoke, 17-builtin-demos, 18-catalog-screens, 19-exerciser-screens]

# Tech tracking
tech-stack:
  added: []  # No new deps — darling/syn/quote/proc-macro2 already present
  patterns:
    - "Attribute proc macro with darling::ast::NestedMeta::parse_meta_list + FromMeta opts struct"
    - "First-fail spanned error pattern using syn::Error::new_spanned for validation rules"
    - "Cfg-gated token emission with absolute path references to downstream consumer paths"

key-files:
  created:
    - backend/crates/marionette-macros/src/gallery_demo.rs
  modified:
    - backend/crates/marionette-macros/src/lib.rs

key-decisions:
  - "D-B1 locked: macro emits #[cfg(feature = \"gallery\")] on BOTH the annotated fn and the linkme static (zero demo symbols in default build)"
  - "D-C1 locked: key defaults to fn ident string; #[gallery_demo(key = \"...\")] overrides"
  - "D-C3 locked: display_name defaults to title-cased key (split on [- _], uppercase first ASCII char, preserve tail); #[gallery_demo(name = \"...\")] overrides"
  - "D-C4 locked: darling used for attribute argument parsing (consistent with component_builder.rs)"
  - "Macro routes linkme through ::marionette::gallery::__linkme re-export (Area E preference chosen) so consumer crates don't need their own linkme dep"

patterns-established:
  - "Pattern: darling::ast::NestedMeta::parse_meta_list(attr) -> FromMeta::from_list(&meta_list) for structured attr parsing"
  - "Pattern: explicit syn::parse2::<ItemFn>(item.clone()) with fallback compile error so non-fn items (struct/enum/mod) get a targeted message instead of a syn tokenizer error"
  - "Pattern: synthesize per-annotation static ident via format!(\"__GALLERY_DEMO_{fn_ident}\") with fn_ident.span() for good error locality"

requirements-completed: [FRAME-01]

# Metrics
duration: 2min
completed: 2026-04-21
---

# Phase 16 Plan 02: Gallery-Demo Proc Macro Summary

**`#[gallery_demo]` attribute macro with darling-based (key, name) arg parsing, syn-based signature/visibility validation, and cfg-gated emission of fn + linkme distributed_slice static routed through `::marionette::gallery::__linkme`.**

## Performance

- **Duration:** ~2 min (140s wall-clock)
- **Started:** 2026-04-21T20:31:54Z
- **Completed:** 2026-04-21T20:34:14Z
- **Tasks:** 1
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments

- `#[gallery_demo]` attribute proc macro shipped as `marionette_macros::gallery_demo`, with optional `key = "..."` / `name = "..."` arguments.
- Signature/visibility gate rejects non-`pub` fns, arg-having fns, `async` fns, generic fns, where-clause fns, and non-`Node`-returning fns with spanned error messages that name the violated rule.
- Item-kind gate produces a targeted compile error naming `pub fn` when applied to `struct`/`enum`/`mod` / non-fn items.
- Title-case helper implements D-C3: `"button"` -> `"Button"`, `"text-input"` -> `"Text Input"`, `"data_table"` -> `"Data Table"`, `"OKLCH-swatches"` -> `"OKLCH Swatches"`; 7 inline unit tests including edge cases (empty string, trailing separator, double separator).
- Emitted tokens reference `::marionette::gallery::DemoEntry`, `::marionette::gallery::DEMOS`, and `::marionette::gallery::__linkme::distributed_slice` — lexical-only at macro-crate compile time; resolve at consumer link time (per Plan 01's gallery submodule).
- `cargo build`, `cargo test --lib gallery_demo`, `cargo clippy -- -D warnings`, and `cargo doc --no-deps` all green on `marionette-macros`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement `#[gallery_demo]` attribute macro with darling + syn validation** — `fa8eed8` (feat)

## Files Created/Modified

- `backend/crates/marionette-macros/src/gallery_demo.rs` (created, 222 LoC) — `gallery_demo_impl` entry point, `GalleryDemoOpts` darling struct, `validate_item` signature/visibility gate, `return_type_is_node` helper, `title_case` ASCII title-casing helper, inline `tests` module with 7 unit tests.
- `backend/crates/marionette-macros/src/lib.rs` (modified, +32 LoC) — added `mod gallery_demo;` alphabetically, plus `#[proc_macro_attribute] pub fn gallery_demo(attr, item)` export with full rustdoc (Arguments, Example, misuse semantics).

## Decisions Made

- **Chose the darling `FromMeta` + `NestedMeta::parse_meta_list` idiom** (darling 0.23's current attribute-macro path) over an ad-hoc `syn::meta::ParseNestedMeta` parser. Matches `component_builder.rs`'s approach (darling is already a workspace dep, no new deps added).
- **Emitted the linkme path as `::marionette::gallery::__linkme::distributed_slice`** (the Area E "re-export form" preference) rather than `::linkme::distributed_slice` directly. Insulates consumer crates from `linkme` version drift and avoids forcing each demo-hosting crate to add a direct `linkme` dep.
- **Validation order:** visibility -> arguments -> async -> generics -> where-clause -> return type. First-fail wins; error message names the specific rule. Rationale: `#[gallery_demo]` on a wildly-wrong fn only produces one error, not a cascade.
- **Unique static ident = `__GALLERY_DEMO_<fn_ident>`** with `fn_ident.span()` so duplicate fn idents within the same scope collide naturally at the normal Rust name-resolution level (not a macro-specific failure mode).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed clippy::pedantic lints after initial implementation**
- **Found during:** Task 1 verification (`cargo clippy -- -D warnings`)
- **Issue:** Initial implementation (copied verbatim from plan's reference block) tripped two pedantic lints: `clippy::collapsible_if` on the nested `if let` in `return_type_is_node`, and `clippy::manual_pattern_char_comparison` on `|c: char| c == '-' || c == '_'` inside `title_case`.
- **Fix:**
  - Collapsed `if let syn::Type::Path(p) = ty { if let Some(last) = ... }` into `if let syn::Type::Path(p) = ty && let Some(last) = ...` (edition 2024 let-chains syntax, already used in `component_builder.rs::extract_option_inner`).
  - Replaced the closure with a char-array pattern: `key.split(['-', '_'])`.
- **Files modified:** `backend/crates/marionette-macros/src/gallery_demo.rs` (lines 150-154 and line 166)
- **Verification:** `cargo clippy -p marionette-macros -- -D warnings` now green; 7 unit tests still pass unchanged.
- **Committed in:** `fa8eed8` (part of Task 1 commit — fixes applied before commit)

---

**Total deviations:** 1 auto-fixed (1 clippy-pedantic bug)
**Impact on plan:** Minor — both lints are clippy::pedantic style preferences, not correctness issues. Behaviour identical to the plan's reference implementation. No scope creep. Fixes improve consistency with the sibling `extract_option_inner` pattern in `component_builder.rs`.

## Issues Encountered

None — plan executed cleanly. The clippy warnings discovered during verification were style-level pedantic nits (collapsible if, char-array pattern) resolved inline per Rule 1 before the first commit.

## User Setup Required

None — pure Rust proc-macro work, no external service configuration required.

## Next Phase Readiness

- **Plan 01 (in parallel):** Macro emits `::marionette::gallery::{DemoEntry, DEMOS, __linkme}` paths as tokens. These paths are lexical at macro-crate compile time and resolve at consumer link time once Plan 01's `marionette::gallery` submodule is merged. Plan 02 and Plan 01 are fully independent at the macro-crate build level (verified: `cargo build -p marionette-macros` succeeds without Plan 01).
- **Plan 03 (gallery-smoke):** Ready to consume `marionette_macros::gallery_demo` as `#[gallery_demo(key = "smoke")] pub fn smoke() -> Node`. Trybuild UI fixtures for the six error cases (non-pub, args, async, generics, where-clause, wrong return type, applied-to-struct) can assert the exact error strings emitted by this plan's `validate_item` function.
- **Phase 17 (built-in demos):** Every `gallery_demo()` fn will need an explicit `key = "..."` override (same fn ident across ~20 sites would mass-collide on the ident-derived default). Planner flag already raised in 16-CONTEXT.md §Specifics.
- **No blockers.** Clippy pedantic clean, rustdoc renders, unit tests pass.

## Self-Check: PASSED

Files verified to exist on disk:
- `backend/crates/marionette-macros/src/gallery_demo.rs` — FOUND (222 lines, exceeds 200-line min)
- `backend/crates/marionette-macros/src/lib.rs` — FOUND (modified to add `mod gallery_demo;` + export)

Commit verified in git log:
- `fa8eed8` — FOUND (`feat(16-02): implement #[gallery_demo] attribute proc macro`)

Symbol checks (via Grep):
- `pub fn gallery_demo_impl` in gallery_demo.rs — FOUND
- `fn title_case` in gallery_demo.rs — FOUND
- `fn validate_item` in gallery_demo.rs — FOUND
- `::marionette::gallery::DemoEntry` in gallery_demo.rs — FOUND
- `::marionette::gallery::DEMOS` in gallery_demo.rs — FOUND
- `::marionette::gallery::__linkme::distributed_slice` in gallery_demo.rs — FOUND
- `mod gallery_demo;` in lib.rs — FOUND
- `pub fn gallery_demo` in lib.rs — FOUND
- `#[proc_macro_attribute]` in lib.rs — 3 occurrences (action, requires, gallery_demo)

Verification commands (all green):
- `cargo build -p marionette-macros` — PASS
- `cargo test -p marionette-macros --lib gallery_demo` — 7/7 PASS
- `cargo clippy -p marionette-macros -- -D warnings` — PASS (0 warnings)
- `cargo doc -p marionette-macros --no-deps` — PASS

---
*Phase: 16-framework-hooks*
*Plan: 02*
*Completed: 2026-04-21*
