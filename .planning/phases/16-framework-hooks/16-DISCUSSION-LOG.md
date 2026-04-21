# Phase 16: Framework Hooks - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `16-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 16-framework-hooks
**Areas discussed:** Registration library, Feature-gate strategy, Registry key + DemoEntry shape, Zero-symbols verification + smoke test

---

## Gray-area selection

| Option | Description | Selected |
|--------|-------------|----------|
| Registration library | inventory vs linkme — the STATE-flagged deferred decision; drives macro codegen + API | ✓ |
| Feature-gate strategy | How `#[gallery_demo]` sites compile when `gallery=off`; macro no-ops, call-site cfg, or module cfg | ✓ |
| Registry key + DemoEntry shape | Key derivation + `DemoEntry` fields; consumed by Phase 17 nav | ✓ |
| Zero-symbols verification + smoke test | Verification style for FRAME-03 and FRAME-04 | ✓ |

**User's choice:** all four areas discussed.

---

## Area 1 — Registration library

### Q1.1: Which distributed-slice library should back the gallery demo registry?

| Option | Description | Selected |
|--------|-------------|----------|
| linkme (Recommended) | Type-safe `#[distributed_slice]`; zero runtime cost (link-time, no ctors); proc-macro expansion = single attribute on a static. Cons: less battle-tested on exotic targets; historical Windows/macOS/wasm caveats (largely resolved). | ✓ |
| inventory | Very widely used (typetag et al); simple `inventory::submit!` model; ctor-style global initializers; excellent cross-platform story. Cons: ctor-before-main complicates cold-start reasoning; typed per-T requires `inventory::collect!(DemoEntry)`. | |

**User's choice:** linkme (Recommended)
**Notes:** Locked as D-A1. Tiebreaker was the "clean framework" posture — linkme's no-ctor, type-safe model aligns with marionette's minimal-surface preference.

### Q1.2: How should stable iteration order be enforced?

| Option | Description | Selected |
|--------|-------------|----------|
| Sort by key at iteration time (Recommended) | `registered_demos()` sorts alphabetically by key before yielding. Truly portable stable order; O(n log n) one-time; nav displays alphabetically by default. | ✓ |
| Trust linker order + document | Zero runtime cost, but FRAME-02's "stable order" becomes platform-dependent; fragile. | |
| Explicit weight/order attr arg | `#[gallery_demo(order = 10)]` + sort by (order, key). Adds attribute surface Phase 16 doesn't need. | |

**User's choice:** Sort by key at iteration time (Recommended)
**Notes:** Locked as D-A2. Memoization via `OnceLock` (D-A4).

### Q1.3: What happens on duplicate-key collision?

| Option | Description | Selected |
|--------|-------------|----------|
| Runtime debug-assert + document (Recommended) | Panic in debug; log-and-keep-first in release. Cheap; catches mistakes fast; doesn't brick production if a third-party demo collides. | ✓ |
| Panic unconditionally | Always panic on duplicate, debug and release. Strictest but overly aggressive for production. | |
| Silently keep first + log warn | Never break; silent bugs possible; conflicts with "clean framework" posture. | |

**User's choice:** Runtime debug-assert + document (Recommended)
**Notes:** Locked as D-A3.

---

## Area 2 — Feature-gate strategy

### Q2.1 (initial): How should `#[gallery_demo]` sites compile when `gallery=off`?

| Option | Description | Selected |
|--------|-------------|----------|
| Macro emits cfg-gated registration (initial Recommended) | `#[gallery_demo]` expands to: fn always + cfg-gated linkme static. Pros: minimal call-site noise. Cons: fn body always compiled — marginal size cost. | ✓ (later revised) |
| Macro + fn both gated | Whole fn + registration under `#[cfg(feature = "gallery")]`. Strictest; initially flagged as adding cfg noise across composite demos (incorrectly — the noise claim was overblown). | |
| Caller cfg-guards, macro is feature-free | Each annotation site writes its own `#[cfg]`. Most explicit but worst ergonomics. | |

**User's choice (initial):** Macro emits cfg-gated registration
**Notes:** Revisited — see Q2.3 below.

### Q2.2: Where should the `gallery` cargo feature be declared?

| Option | Description | Selected |
|--------|-------------|----------|
| `gallery` on marionette only, pulls linkme (Recommended) | Single feature surface; `linkme` is optional dep; matches standard Rust feature-gating. | ✓ |
| `gallery` on both marionette and marionette-macros | Bi-crate feature; proc macros don't get features from consumer in the obvious way; extra complexity without benefit. | |
| `gallery` on marionette + separate `demos-in-workspace` flag | Decouples registry API from built-in demos. YAGNI for v1.2. | |

**User's choice:** `gallery` on marionette only, pulls linkme (Recommended)
**Notes:** Locked as D-B3.

### Q2.3 (revisit): Given FRAME-03's "zero demo symbols in default build" — which feature-gate strategy do we lock in?

Raised after recognizing that Option 1 from Q2.1 would compile the fn bodies into `libmarionette.rlib` even when `gallery=off`, which a `nm` symbol-grep would find — directly contradicting FRAME-03.

| Option | Description | Selected |
|--------|-------------|----------|
| Macro gates both fn + registration (Recommended — satisfies FRAME-03) | Under default build: fn body + registration both absent. Composite demos (Phase 18) are themselves `#[gallery_demo]` so their cfg is implicit; nested calls compile normally under the feature. No call-site cfg noise. | ✓ |
| Stick with Q2.1 Option 1, relax FRAME-03 to "zero registry entries" | Reinterpret requirement to "zero registry bytes" rather than "zero fn symbols"; ~20 unused fn bodies would add bytes to default builds. | |

**User's choice:** Macro gates both fn + registration (Recommended)
**Notes:** Locked as D-B1 and D-B2. This supersedes the initial Q2.1 choice.

---

## Area 3 — Registry key + DemoEntry shape

### Q3.1: How should the registry key be derived?

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit attr arg, required (initial Recommended) | `#[gallery_demo(key = "button")]` mandatory. Zero ambiguity; matches existing `#[component(type = "button")]` pattern. | |
| Derive from enclosing struct/fn context | Macro inspects surrounding impl/module. Fragile; proc macros don't see enclosing impl reliably. | |
| Derive from fn module path + crate | Key becomes `marionette::builders::standard::button`. UI-unfriendly; refactoring breaks keys. | |
| Auto from fn ident, with override arg | Default: stringified fn ident. Override via `#[gallery_demo(key = "...")]`. | ✓ |

**User's choice:** Auto from fn ident, with override arg
**Notes:** Locked as D-C1. Phase 17 implication flagged: because every built-in will name its fn `gallery_demo()` per DEMO-01/02, every Phase 17 annotation MUST use the explicit `key = "..."` override — otherwise all ~20 keys collide. Catalog/exerciser screens (Phases 18/19) with distinct fn idents can skip the override.

### Q3.2: What fields does `DemoEntry` carry?

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: key + render fn (Recommended) | `{ key, render }`. Phase 17 derives nav labels from key with Title Case or a separate map. Smallest surface. | |
| Key + render + display_name | `{ key, render, display_name }`. Display name default = title-cased key; override via `#[gallery_demo(name = "...")]`. Nav labels are framework-locked. | ✓ |
| Key + render + component_type | Adds `component_type` matching `#[component(type = "...")]`. Fragile (macro would parse surrounding struct); empty for catalog/exerciser demos. | |

**User's choice:** Key + render + display_name
**Notes:** Locked as D-C2 and D-C3. `display_name` default via title-casing algorithm (Claude's discretion per Area E).

---

## Area 4 — Zero-symbols verification + smoke test

### Q4.1: How should FRAME-03 "zero demo symbols in default cargo build -p marionette" be verified?

| Option | Description | Selected |
|--------|-------------|----------|
| Symbol-table grep via nm/objdump (Recommended) | Cargo test shells out to `nm --demangle target/<profile>/libmarionette.rlib`; greps for `gallery_demo`; asserts zero matches default / matches appear under feature. Directly proves FRAME-03's letter. | ✓ |
| Registry-API assertion test | Call `registered_demos()` and assert empty. Proves zero registry entries only; fn bodies could still exist. Weaker. | |
| Artifact-size threshold | Stat rlib sizes under default vs feature; assert delta. Brittle thresholds; noisy. | |
| Compile-fail via trybuild | Make `registered_demos()` itself cfg-gated. Overkill; breaks symmetric public API. | |

**User's choice:** Symbol-table grep via nm/objdump (Recommended)
**Notes:** Locked as D-D1. Implementation detail (shell-out from cargo test vs Makefile target invoked by CI) left to planner.

### Q4.2: Where should FRAME-04's end-to-end smoke test live?

| Option | Description | Selected |
|--------|-------------|----------|
| New workspace test crate `gallery-smoke` (Recommended) | `backend/crates/gallery-smoke/`: depends on marionette with `features = ["gallery"]`; registers toy demo; `#[test]` asserts registry round-trip. Mirrors intended downstream shape; exercises cross-crate distributed-slice wiring. | ✓ |
| Inside marionette/tests/ | Put smoke test in marionette's own tests/. Doesn't exercise cross-crate wiring — false sense of security. | |
| Inside marionette-macros/tests/ | Not viable — marionette-macros can't depend on marionette (cyclic) so can't invoke linkme end-to-end. | |

**User's choice:** New workspace test crate `gallery-smoke` (Recommended)
**Notes:** Locked as D-D2. Retained permanently (D-D3) as the automated counterpart to Phase 17's `gallery-demo` binary. Houses `trybuild` fixtures for FRAME-01 error-message stability (D-D4). Phase 17 note: `gallery-smoke` as 5th workspace crate means `gallery-demo` becomes the 6th, not the 5th as REQUIREMENTS §CRATE-01 currently states — flag for Phase 17 planner.

---

## Closing question

### Q5: Ready for context, or explore more gray areas?

| Option | Description | Selected |
|--------|-------------|----------|
| I'm ready for context | Write CONTEXT.md now. Remaining items (compiler-error UX wording, macro path re-export strategy, `marionette::gallery` submodule vs top-level) go under Claude's Discretion. | ✓ |
| Explore compiler-error UX | Pin down exact macro rejection rules + error message wording. | |
| Explore macro path re-export | `::linkme` direct vs `marionette::__private::linkme` indirection. | |
| Explore gallery module layout | Submodule vs top-level for `DemoEntry`/`DEMOS`/`registered_demos()`. | |

**User's choice:** I'm ready for context
**Notes:** All three deferred items captured in CONTEXT.md §Area E (Claude's Discretion) with preferred directions noted for the planner.

---

## Claude's Discretion

Areas explicitly left to the planner / implementer:

- Exact macro error messages for signature/visibility/item-kind violations (style: mirror existing `ComponentBuilder`/`action` crispness).
- Macro path re-export strategy (`::linkme` direct vs `marionette::__private::linkme` — preference: re-export for API hygiene).
- `marionette::gallery::*` submodule vs `marionette::*` top-level for public API (preference: submodule).
- Exact `linkme` version pin (latest stable at execution time, pinned major-minor).
- `registered_demos()` return type (`impl Iterator` vs `&'static [&'static DemoEntry]`).
- Title-casing algorithm for default `display_name` (ASCII split-on-`-_` + capitalize-first is fine).
- Symbol-test shape (cargo test shelling out to `nm` vs `make` target invoked by CI).
- Whether the smoke test asserts a `display_name` override or the default title-case.
- Synchronization primitive for sort/collision memoization (`OnceLock` preferred).

---

## Deferred Ideas

See `16-CONTEXT.md` §deferred. Summary:

- GALLERY-LINT (CI lint for "every built-in has `gallery_demo()`") — v1.3+.
- `component_type` field on `DemoEntry` — add non-breakingly if Phase 17/18 shows need.
- `order`/`weight` attribute arg for nav ordering — add if alphabetical-by-key UX is inadequate in Phase 17.
- Pure-fn contract enforcement (no I/O, no fixtures) — aspirational per Phase 16; future lint may enforce.
- `marionette-gallery-demo` as shipped doc artifact — GALLERY-DEMOS-EXPORT, v1.3+.
- Richer per-demo metadata (source path, doc comment) — add when a "generated docs" feature needs it.
- `inventory` as fallback — rejected; pre-deployment posture means no parallel implementations.
- Third-party crates registering demos — v1.2 built-ins only.
