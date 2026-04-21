# Phase 16: Framework Hooks - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 16 lays the **auto-discovery spine** that every subsequent v1.2 phase rides on. It delivers three things and nothing else:

1. **The `#[gallery_demo]` attribute proc macro** in `marionette-macros`, applicable to a `pub fn name() -> Node` item. Misapplication (wrong signature, wrong visibility, wrong item kind) produces a clear compiler error that names the violated rule (FRAME-01).

2. **A distributed-slice registry + iteration API** in `marionette`: `pub static DEMOS: [DemoEntry]` backed by `linkme`, plus `pub fn registered_demos() -> impl Iterator<Item = &'static DemoEntry>` returning entries in stable order (FRAME-02).

3. **A `gallery` cargo feature gate** on the `marionette` crate (default OFF) such that a default `cargo build -p marionette` compiles zero demo symbols and zero registry entries. Enabling `--features gallery` brings them in. Verified by a symbol-table test plus a cross-crate smoke test (FRAME-03, FRAME-04).

**What this phase is NOT:**

- **NOT any built-in component demos.** Phase 17 sweeps `backend/crates/marionette/src/builders/` and adds `gallery_demo()` siblings for every existing built-in (Button, TextInput, SelectInput, Checkbox, Textarea, RadioGroup, Switch, DataTable, AppShell, NavItem, Sidebar pieces, ModalSurface, ConfirmDialog, ToastSurface, FieldSet, FieldSeparator, Container, Heading, plus any others). Phase 16 exercises the macro with a toy demo in the `gallery-smoke` crate only.
- **NOT the `gallery-demo` binary crate.** That is CRATE-01 in Phase 17. Phase 16 introduces `gallery-smoke` as a test-only workspace member — a minimal crate that links against `marionette --features gallery`, registers a single toy demo, and asserts the registry round-trips end-to-end.
- **NOT a CI lint enforcing "every built-in must have a `gallery_demo()`".** That is GALLERY-LINT, deferred to v1.3+ per REQUIREMENTS.md §v1.3+ Requirements.
- **NOT a theme-editor, catalog screens, exerciser screens, or nested-AppShell work.** Those are Phases 18/19/20.
- **NOT a protocol change.** No new SDUI component types, no new message kinds, no version bump. The `marionette-protocol` crate is untouched.
- **NOT a revision to the "pure `fn() -> Node` contract".** The contract is locked by the architecture note (`.planning/notes/2026-04-21-gallery-demo-architecture.md`) and REQUIREMENTS §DEMO-02. Phase 16 builds the rails; the contract is enforced by convention (proc macro signature check) + documentation (Phase 17's `GALLERY-DEMOS.md`).
- **NOT an enforcement policy for composite demos.** Phase 16 makes nothing prevent a `gallery_demo()` fn from reaching into fixtures/state — the macro checks signature + visibility only. Phase 17 documents the convention; a future phase may CI-enforce it.

</domain>

<decisions>
## Implementation Decisions

### Area A — Registration library + iteration semantics

- **D-A1: `linkme` backs the distributed-slice registry.** Chosen over `inventory` primarily for type-safety (the `#[distributed_slice(DEMOS)]` attribute binds to a typed static), zero runtime cost (no ctor-style global initializers), and explicit mental model consistent with marionette's "clean framework" posture. Logged in PROJECT.md Key Decisions per FRAME-02's requirement. This supersedes the STATE.md blocker "Registration library selection".
- **D-A2: Stable iteration order is enforced by sorting at iteration time.** `linkme`'s native order is linker-defined (roughly source-file / link-unit order, not portable across platforms or rustc versions). `registered_demos()` sorts the slice alphabetically by `key` before yielding. One-time O(n log n) at first call (trivial for n ≈ 20–50). This satisfies FRAME-02's "stable iteration order" promise in the strongest sense — deterministic across Linux/macOS/CI.
- **D-A3: Duplicate-key collisions use runtime debug-assert, log-and-keep-first in release.** `registered_demos()` runs a one-time duplicate-key check the first time it's invoked. In debug builds, a duplicate key causes a panic with a clear message naming both sites. In release builds, the duplicate is logged via `tracing::warn!` and the first-registered entry wins. Rationale: `linkme` cannot detect collisions at compile time without lang-items hackery; debug-assert catches mistakes immediately during development; release path stays resilient if a future third-party demo crate collides with a marionette built-in.
- **D-A4: Sort + collision check memoize their result.** The sorted snapshot + collision scan run exactly once per process, behind a `std::sync::OnceLock<Vec<&'static DemoEntry>>` (or equivalent). Subsequent `registered_demos()` calls are zero-cost.

### Area B — Feature-gate strategy

- **D-B1: `#[gallery_demo]` gates both the annotated fn AND its registration.** The macro emits `#[cfg(feature = "gallery")]` on both the fn body and the `#[linkme::distributed_slice]` static. Under default `cargo build -p marionette`, neither the fn symbol nor the registry entry exists in `libmarionette.rlib`. FRAME-03's "zero demo symbols" promise is satisfied at the symbol level, not just at the registry-entry level.
- **D-B2: Composite demos are cfg-gated implicitly as a unit.** Phase 18's `FormScreen::gallery_demo()` will be itself annotated with `#[gallery_demo]` and therefore wrapped in the same `#[cfg(feature = "gallery")]`. Nested calls like `TextInput::gallery_demo()` inside `FormScreen::gallery_demo()` compile normally under `--features gallery` because they all share the same cfg. Phase 16 note only — nothing to enforce here now.
- **D-B3: `gallery` feature lives on `marionette` only; pulls `linkme` as an optional dependency.** `marionette/Cargo.toml` gets `[features] gallery = ["dep:linkme"]` and `linkme = { version = "<pin>", optional = true }`. `marionette-macros` stays feature-free — it is a proc-macro crate, always compiled, and simply emits tokens referencing paths that only resolve when the `gallery` feature is on in the consumer. This is standard Rust feature-flag idiom.
- **D-B4: The default build's `registered_demos()` returns an empty iterator.** Because `DEMOS` itself is `#[cfg(feature = "gallery")]`, `registered_demos()` under default build either returns `std::iter::empty()` from a stub or is itself cfg-gated behind `#[cfg(feature = "gallery")]`. Preference: stub that always exists, returning empty under default build. Keeps the public API symmetric (consumers can always call `registered_demos()` without cfg-guards of their own). The `gallery-smoke` crate's test exercises the non-empty path.

### Area C — Registry key + `DemoEntry` shape

- **D-C1: Key defaults from the fn ident; overridable via `#[gallery_demo(key = "...")]`.** Under the default rule, `#[gallery_demo] pub fn button() -> Node` yields `key = "button"`. Under override, `#[gallery_demo(key = "button")] pub fn gallery_demo() -> Node` yields `key = "button"`. **Phase 17 implication:** DEMO-01/02 names every built-in's colocated fn `gallery_demo()`. If Phase 17 keeps that convention verbatim, the default-derived key is `"gallery_demo"` for all ~20 sites — mass collision. Phase 17's sweep MUST therefore use the explicit `key = "..."` override on every annotation (matching each builder's `#[component(type = "…")]` string is the natural choice: `key = "button"`, `key = "text-input"`, etc.). Catalog/exerciser screens (Phases 18/19) may use distinct fn idents and skip the override.
- **D-C2: `DemoEntry` fields = `{ key, render, display_name }`.**
  ```rust
  pub struct DemoEntry {
      pub key: &'static str,         // stable identifier; sorted by this
      pub render: fn() -> Node,      // entry point; pure, no args
      pub display_name: &'static str,// nav-facing label
  }
  ```
  No `component_type` field (would force the macro to read the surrounding struct's `#[component]` attr — fragile, and catalog/exerciser demos have no underlying component). No source-location or crate-name field — Phase 17's nav is keyed purely off `display_name`; future tooling can add metadata without a breaking change.
- **D-C3: `display_name` defaults to title-cased + space-separated `key`; overridable via `#[gallery_demo(name = "...")]`.** Default transform: `"button"` → `"Button"`, `"text-input"` → `"Text Input"`, `"data-table"` → `"Data Table"`. Override when the display name differs from the key's natural title case (e.g., `"Buttons & Actions"`, `"OKLCH Swatches"`). The title-casing algorithm is Claude's discretion (simple split-on-`-_` + capitalize-first is fine; no Unicode heroics required for a closed set of ASCII keys).
- **D-C4: Attribute parsing uses `darling`.** Consistent with `ComponentBuilder`'s existing use of darling in `marionette-macros/Cargo.toml`. `darling`'s error reporting gives clear messages for unknown args, missing args, and wrong types — lower effort than hand-rolling `syn::parse`.

### Area D — Verification approach (FRAME-03 + FRAME-04)

- **D-D1: FRAME-03 is verified by a symbol-table `nm` grep.** A test under `backend/crates/marionette/tests/no_gallery_symbols.rs` (or a Makefile target invoked by CI) shells out to `nm --demangle target/<profile>/libmarionette.rlib`, greps for the `gallery_demo` substring plus the `DEMOS` static, asserts zero matches under default build, and asserts matches DO appear under `--features gallery`. The planner decides whether the test lives as a `cargo test` that shells out to `cargo build` in a subprocess, or as a `make check-no-gallery-symbols` step invoked by CI — both are valid; the shell-out version is self-contained but slower, the Makefile version is simpler but needs CI wiring.
- **D-D2: FRAME-04 uses a new `gallery-smoke` workspace crate.** `backend/crates/gallery-smoke/` is added as a **permanent** workspace member (test-only by convention, but a real crate — not a test file). It depends on `marionette` with `features = ["gallery"]`, registers a single `#[gallery_demo(key = "smoke")] pub fn smoke() -> Node`, and contains a `#[test]` that iterates `marionette::registered_demos()` and asserts the `"smoke"` entry is present with its expected `display_name`. This exercises the **full pipeline including cross-crate distributed-slice wiring** — the thing that a same-crate test inside `marionette/tests/` would miss.
- **D-D3: `gallery-smoke` is retained after Phase 17.** Phase 17's `gallery-demo` binary validates the registry by rendering demos in a browser; `gallery-smoke` validates it by automated `#[test]` assertions. Different purposes, both load-bearing. Phase 16 lands `gallery-smoke` as the 5th workspace crate; Phase 17 lands `gallery-demo` which becomes the 6th (not the "5th" as REQUIREMENTS.md §CRATE-01 currently states). **Flag for Phase 17 planner:** update CRATE-01's wording or accept that gallery-smoke counts as a test-fixture crate.
- **D-D4: The compiler-error path (FRAME-01 clause) is exercised by `trybuild`-style fixtures inside `gallery-smoke`.** A `tests/ui/` folder with `.rs` files that intentionally misuse the macro (wrong signature, non-`pub`, applied to a `struct`, wrong return type) and matching `.stderr` expectations. The proc macro's error messages are therefore version-locked against regression. This is a small addition on top of the smoke test and gives FRAME-01's "clear compiler error that names the violated rule" a concrete CI guardrail.

### Area E — Claude's Discretion

Within Phase 16:

- **Exact macro error messages** for signature/visibility/item-kind violations. The contract is "name the violated rule"; the exact wording is left to implementation (darling + `syn::Error::new` with good spans). Planner preference: mirror the crisp style of the existing `ComponentBuilder` and `action` macros.
- **Macro path re-export strategy.** Whether `marionette-macros` emits `::linkme::distributed_slice(...)` directly or routes through `::marionette::__private::linkme::distributed_slice(...)`. The re-export form insulates consumers from version drift if they also use `linkme` directly; the direct form is simpler. Planner's call — preference is the re-export form for API hygiene, but either is acceptable.
- **Whether `DEMOS`, `DemoEntry`, and `registered_demos()` live at `marionette::gallery::*` (submodule) or `marionette::*` (top-level).** Preference: submodule (`pub mod gallery;`) to keep the framework-facing surface contained — `use marionette::gallery::{DemoEntry, registered_demos}` reads well and matches how `marionette::builders::*` is organized. Top-level is also fine; downstream sweep is mechanical either way.
- **Exact `linkme` version pin.** Latest stable at phase execution time is fine; pin to the major-minor for reproducibility.
- **Whether `registered_demos()` returns `impl Iterator<Item = &'static DemoEntry>` or `&'static [&'static DemoEntry]`.** The roadmap wording uses `impl Iterator`; slice reference is simpler and lets consumers call `.len()` and index. Planner's call.
- **Title-casing algorithm for the default `display_name`.** Split on `-` and `_`, capitalize first letter of each chunk; no Unicode-aware casing.
- **Symbol-test implementation shape.** `cargo test` shelling to `nm` vs `make check-no-gallery-symbols` as a CI step. Either satisfies FRAME-03.
- **Whether the smoke test asserts a `display_name` override** (`#[gallery_demo(key = "smoke", name = "Smoke Check")]`) or relies on the default title-case. Either exercises the pipeline.
- **`OnceLock` vs `OnceCell` vs hand-rolled sync for sort/collision memoization.** `std::sync::OnceLock` (stable since 1.70) is the cleanest; planner's call.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone-level intent

- `.planning/ROADMAP.md` §Phase 16 — goal, depends-on (Phase 15 = v1.1 complete), 4 success criteria (proc macro + clear compile errors; `registered_demos()` with stable order; zero demo symbols under default build; end-to-end smoke test).
- `.planning/REQUIREMENTS.md` §Framework hooks — FRAME-01 (`#[gallery_demo]` attribute macro, misuse compiler errors), FRAME-02 (registry-iteration API via `inventory`/`linkme`, stable order, choice logged), FRAME-03 (`gallery` feature gate, zero demo symbols in default build, verified by artifact inspection).
- `.planning/REQUIREMENTS.md` §v1.3+ Requirements — GALLERY-LINT (CI lint enforcing every built-in has `gallery_demo()`) is **out of scope** for Phase 16; note as deferred.
- `.planning/PROJECT.md` §Key Decisions — "Gallery app as second demo alongside CRM", "Auto-discoverable demos via `#[gallery_demo]` + inventory/linkme", "Pure `fn() -> Node` demo contract". Phase 16 implements these.

### Design foundation

- `.planning/notes/2026-04-21-gallery-demo-architecture.md` — the full architectural rationale. §Auto-discoverable component demos — option C (committed), §Demo contract, §Likely phase decomposition (Phase A = Phase 16). Phase 16 is the realization of this note's "framework hooks" layer.
- `.planning/seeds/gallery-live-token-editor.md` — context only (Phase 20 concern); not in Phase 16 scope but helps explain why the gallery app exists.

### State-level concerns Phase 16 resolves

- `.planning/STATE.md` §Blockers/Concerns — "Registration library selection" (inventory vs linkme, deferred to Phase 16 scoping) → resolved by D-A1 (`linkme`). "Enforcement policy" (whether every new built-in must ship a `gallery_demo()` is CI-enforced or aspirational) → **stays deferred**; Phase 16 enforces signature by proc macro only, not coverage by CI.

### Prior phase context inherited

- `.planning/milestones/v1.1-phases/15-crm-migration-validation/15-CONTEXT.md` — Phase 15 established the pre-deployment posture (no back-compat shims, fix root causes), the Chrome-MCP UAT pattern (not applicable here — no UI in Phase 16), and the "clean break" discipline. Phase 16 inherits the posture.
- `.planning/milestones/v1.1-phases/12-protocol-node-patching-appshell/12-CONTEXT.md` — AppShell as a first-class SDUI component. Phase 17 will add `AppShell::gallery_demo()`; Phase 19 will exercise nested AppShell. Not load-bearing for Phase 16, but worth knowing the component exists.
- `.planning/milestones/v1.1-phases/14-formscreen-enhancements/14-CONTEXT.md` — Field anatomy, FieldSet grouping, validation patch shape. Again: Phase 17+ territory; Phase 16 only needs to know these builders exist in `standard.rs`.

### Code the phase touches

- **`backend/crates/marionette-macros/src/lib.rs`** — append `mod gallery_demo;` and `#[proc_macro_attribute] pub fn gallery_demo(...)` following the existing pattern from `action` (attribute macro) and `ComponentBuilder` (derive). New file: `backend/crates/marionette-macros/src/gallery_demo.rs`.
- **`backend/crates/marionette-macros/Cargo.toml`** — no new dependencies; `darling`, `syn`, `quote`, `proc-macro2` already present and sufficient.
- **`backend/crates/marionette/src/lib.rs`** — add `pub mod gallery;` (behind `#[cfg(feature = "gallery")]` for the internals, with a top-level `pub fn registered_demos()` stub that returns empty iterator when feature is off per D-B4).
- **`backend/crates/marionette/Cargo.toml`** — add `[features] gallery = ["dep:linkme"]` and `linkme = { version = "<pin>", optional = true }`.
- **New file: `backend/crates/marionette/src/gallery.rs`** (or `backend/crates/marionette/src/gallery/mod.rs`) — defines `DemoEntry`, the `#[linkme::distributed_slice] pub static DEMOS`, and `registered_demos()` with the sort + collision-check memoization.
- **`backend/Cargo.toml`** — add `"crates/gallery-smoke"` to `[workspace] members`.
- **New crate: `backend/crates/gallery-smoke/`** — `Cargo.toml` with `marionette = { path = "../marionette", features = ["gallery"] }` + `linkme` (optional, only if the smoke crate itself also registers into the slice, which it does). `src/lib.rs` with the toy `#[gallery_demo(key = "smoke")] pub fn smoke() -> Node`. `tests/registry_roundtrip.rs` with the registered-demos iteration assertion. `tests/ui/` with trybuild fixtures for macro error cases (D-D4).
- **New file: `backend/crates/marionette/tests/no_gallery_symbols.rs`** (or Makefile target) — FRAME-03 verification per D-D1.

### External library docs

- https://docs.rs/linkme/latest/linkme/ — `#[distributed_slice]` usage, cross-crate registration semantics, platform support matrix. Read §Safety and §Limitations before planning D-B1/D-B4.
- https://docs.rs/darling/latest/darling/ — attribute-macro argument parsing; relevant for parsing `key = "..."`, `name = "..."`. Existing `component_builder.rs` is the in-repo reference.
- https://docs.rs/syn/latest/syn/ — `ItemFn`, `Signature`, `Visibility` — the three things the proc macro validates for FRAME-01.
- https://docs.rs/trybuild/latest/trybuild/ — for the macro-error test fixtures in D-D4.

### Codebase intel

- `.planning/codebase/` — if CONVENTIONS.md, STACK.md, or TESTING.md exist, read current state before planning. They may predate the v1.1 shadcn-svelte migration; cross-check against REQUIREMENTS.md §Validated.

### User preferences (global memory)

- `feedback_pre_deployment_no_backcompat.md` — no migration shims, fix root causes → **applies**: Phase 16 commits to `linkme` cleanly, no fallback to `inventory` for "just in case".
- `feedback_options_need_reasoning.md` — every option comes with pros/cons/rationale; framework recipes preferred over hand-rolling → **applied** throughout the discussion (four AskUserQuestion rounds each included pros/cons/rationale and recommended picks).
- `feedback_no_handrolling_ui.md` — framework recipes over custom designs → **N/A for Phase 16** (pure Rust framework work, no UI surface).
- `feedback_use_chrome_for_uat.md` — Chrome-MCP for UAT → **N/A for Phase 16** (no user-facing surface yet; first UAT-able screen lands in Phase 17).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`backend/crates/marionette-macros/src/{lib,action,component_builder,requires}.rs`** — the existing proc-macro crate with three patterns to model the new attribute macro on: `action` (attribute macro with simple string arg), `requires` (attribute macro with multiple arg variants), `ComponentBuilder` (derive macro with darling-based attribute parsing). The new `gallery_demo` attribute macro sits closest to `action`'s shape with `component_builder`'s darling-based argument parsing.
- **`backend/crates/marionette-macros/Cargo.toml`** — already declares `syn = { features = ["full"] }`, `quote`, `proc-macro2`, `darling`. No new deps needed for the macro itself.
- **`backend/crates/marionette/src/builders/standard.rs`** — the ~20 `#[derive(ComponentBuilder)] #[component(type = "…")]` structs that Phase 17 will colocate `gallery_demo()` siblings next to. Phase 16 does not touch this file; it just confirms the macro's error messages compose cleanly when the macro lands on code near these builders.
- **`backend/crates/marionette/src/lib.rs`** — the current public-surface entry point. New `pub mod gallery` export (or top-level re-exports per Area E) lands here.
- **`backend/Cargo.toml`** — the workspace manifest. New member `crates/gallery-smoke` appends to `[workspace] members`.

### Established Patterns

- **Attribute + derive macros authored with darling** — `component_builder.rs` is the canonical in-repo reference for attribute parsing with good error messages. Phase 16's `gallery_demo` macro follows the same style.
- **Feature-gated optional deps via `[features] name = ["dep:xxx"]`** — standard Rust idiom; marionette's current `Cargo.toml` uses straightforward non-feature deps today, so Phase 16 introduces the first optional-dep feature on the crate. No precedent to reuse, but no novelty either.
- **Workspace-member granularity** — `marionette-protocol`, `marionette-macros`, `marionette`, `crm-demo` are the four current members. Each has a single, clear job. `gallery-smoke` follows: one job, tests only.
- **Pure-fn builder composition** — `marionette/src/builders/standard.rs` is already a set of pure `new(...).builder_method(...).build()` functions returning `(NodeId, Component)` pairs. Phase 17's `gallery_demo() -> Node` convention fits cleanly on top; Phase 16 has no composition to implement yet but the pattern is what the macro validates.
- **Edition 2024, rustc-pedantic warnings on** — `marionette-macros/src/lib.rs` opens with `#![warn(clippy::pedantic)]`. Phase 16's new code must clear the same bar.

### Integration Points

- **`backend/crates/marionette-macros/src/lib.rs`** — add `mod gallery_demo;` + `#[proc_macro_attribute] pub fn gallery_demo(...)` export.
- **`backend/crates/marionette-macros/src/gallery_demo.rs`** — new file; darling-parsed `GalleryDemoOpts { key: Option<String>, name: Option<String> }`; signature/visibility validation via `syn`; emit cfg-gated fn + linkme static.
- **`backend/crates/marionette/src/lib.rs`** — add `pub mod gallery;` (and decide module path per Area E).
- **`backend/crates/marionette/src/gallery.rs`** (or `src/gallery/mod.rs`) — new file; `DemoEntry`, `DEMOS`, `registered_demos()` with memoized sort + collision check.
- **`backend/crates/marionette/Cargo.toml`** — add `gallery` feature, optional `linkme` dep.
- **`backend/Cargo.toml`** — register `crates/gallery-smoke` workspace member.
- **`backend/crates/gallery-smoke/`** — new crate; `Cargo.toml`, `src/lib.rs` with toy demo fn, `tests/registry_roundtrip.rs`, `tests/ui/*.rs` + `*.stderr` for trybuild.
- **`backend/crates/marionette/tests/no_gallery_symbols.rs`** (or `Makefile` target) — FRAME-03 verification shell-out to `nm`.
- **CI (`.github/workflows/*.yml`)** — if the FRAME-03 check lands as a Makefile target rather than a `cargo test`, wire it into the CI pipeline. If it lands as a `cargo test`, existing `cargo test --workspace` picks it up automatically.

</code_context>

<specifics>
## Specific Ideas

- **"`linkme` is the library, and we own the stable-order promise ourselves."** The library is picked (D-A1), but its default iteration order is not stable across platforms. We don't delegate FRAME-02's stability promise to `linkme` — we satisfy it by sorting at iteration time (D-A2). This is intentional: the framework controls the contract, the library just provides the slice.
- **"Both fn and registration go behind the feature gate, or FRAME-03 lies."** The Area 2 revisit caught this — an earlier recommendation would have left the fn bodies compiled, which means `nm` would find them, which means FRAME-03 fails. Locked in D-B1. Planner must verify: symbol-test MUST see zero `gallery_demo` matches under default build.
- **"Every Phase 17 annotation will use `key = \"…\"` explicitly, matching the component type string."** The auto-from-ident default is useful for Phase 18/19 where fn idents are distinct, but Phase 17's convention (every built-in has a fn named `gallery_demo()`) forces every annotation to override. This is not a bug — it's a consequence of the "one fn name per builder" convention in DEMO-01. Planner should flag this to Phase 17 explicitly.
- **"`gallery-smoke` is a permanent crate, not a temp scaffold."** It counterbalances the `gallery-demo` binary (Phase 17) by asserting the registry at the `#[test]` level, independent of any browser rendering. It also hosts the `trybuild` fixtures for FRAME-01's error-message stability (D-D4). Retiring it later would leave a regression hole.
- **"`darling` for arg parsing, `syn` for signature validation."** Consistent with the in-repo `component_builder.rs` reference. Nothing novel; good error messages come for free.
- **"No new CI step until after Phase 16 ships (unless the FRAME-03 check lands as a Makefile target)."** `cargo test --workspace` already runs on every push. If the planner picks "test-shells-out-to-nm" for D-D1, no CI change is needed. If they pick the Makefile-target route, CI gets one added step.
- **"Phase 16's architecture note already resolved the big questions."** The gallery-demo architecture note (`.planning/notes/2026-04-21-gallery-demo-architecture.md`) landed before this discussion. It pre-decided: macro name, pure-fn contract, feature-gate concept, registry backbone via distributed slice, gallery-demo as second workspace crate. Phase 16's job is to implement the rails that note described — not to re-debate them.
- **"One plan per area is a reasonable shape."** Planner's call, but the four areas (A: linkme + registry; B: feature gate + `gallery` submodule; C: proc macro in `marionette-macros`; D: `gallery-smoke` + symbol test) map cleanly to ~4 plans. Each plan is atomic and reviewable; D depends on A+B+C but A/B/C are mostly independent (A and B both touch `marionette/src/`, but different files).

</specifics>

<deferred>
## Deferred Ideas

- **GALLERY-LINT** (CI lint enforcing every built-in has a `gallery_demo()`) — v1.3+ per REQUIREMENTS.md. Phase 16 only enforces signature/visibility on annotated items; it does not enforce "every component must be annotated". The DEMO-01 sweep in Phase 17 is manual; a follow-up phase may add the CI lint if the manual sweep shows drift risk.
- **`component_type` field on `DemoEntry`** — rejected during Area C because catalog/exerciser demos have no component_type and because making the macro read the surrounding struct's `#[component]` attr is fragile. If Phase 17 or Phase 18 shows a real need (e.g., auto-grouping demos by component family in nav), add a non-breaking field then.
- **Explicit `order` / `weight` attribute arg** for nav ordering — considered during Area A but rejected as YAGNI for Phase 16. If Phase 17's alphabetical-by-key nav ordering turns out to be UX-inadequate, add an `order` arg then.
- **Enforcement policy for pure-fn contract** (no I/O, no fixtures inside `gallery_demo()` bodies) — stays aspirational per Phase 16's scope. Phase 17 documents the convention in `GALLERY-DEMOS.md`; a future phase may CI-enforce it via a lint or a clippy-custom rule.
- **`marionette-gallery-demo` as a shipped documentation artifact library** — GALLERY-DEMOS-EXPORT, v1.3+ per REQUIREMENTS.md.
- **Macro emitting richer per-demo metadata** (source file path, crate, doc comment extracted) — considered but not needed for Phase 16's success criteria. If a future "generated docs from demos" feature lands, extend the macro then.
- **`inventory` as a fallback/alternative** — explicitly rejected. Pre-deployment posture means no parallel implementations, no "just in case". `linkme` is the choice, period.
- **Protocol-side gallery metadata** (server pushing theme or demo registry to client) — PROJECT.md's "Backend-driven theme tokens" is out of scope for the whole v1.2 milestone; this is a v1.3+ product-direction question.
- **Third-party (non-marionette) crates registering demos** — v1.2 ships demos for built-ins only per REQUIREMENTS.md §Out of Scope. The `linkme` distributed slice would technically allow it (any crate in the workspace could register), but Phase 16 does not guarantee it — the API may harden or soften in response to external consumer demand.

</deferred>

---

*Phase: 16-framework-hooks*
*Context gathered: 2026-04-21*
