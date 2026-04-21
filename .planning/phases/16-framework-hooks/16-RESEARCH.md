# Phase 16: Framework Hooks — Research

**Researched:** 2026-04-21
**Domain:** Rust proc macros + `linkme` distributed-slice + feature-gated cargo features + cross-crate registry
**Confidence:** HIGH

## Summary

CONTEXT.md has locked the architectural decisions (D-A1 through D-D4). This research answers the
**how-to** questions downstream planning needs: what `linkme` actually requires at the source
level, how feature-gating interacts with macro expansion across crate boundaries, what the
existing `marionette-macros` authoring conventions look like, what shape `gallery-smoke` and the
symbol-table test should take, and what CI / workspace wiring needs to change.

All library-mechanism claims were empirically verified against a minimal workspace built in
`/tmp/linkme-feature-test` with `linkme = "0.3.36"` on `rustc 1.93.1` + Rust edition 2024.

**Primary recommendation:** Implement the four CONTEXT.md areas as four plans:

- **Plan 01 (Area A+B):** `marionette::gallery` module — `DemoEntry`, `DEMOS` distributed slice,
  `registered_demos()` with memoized sort + collision check; `gallery` cargo feature + optional
  `linkme` dep on the `marionette` crate. Pure framework plumbing, no macro involvement.
- **Plan 02 (Area C):** `#[gallery_demo]` attribute macro in `marionette-macros` — darling-based
  attr parsing (`key`, `name`), syn-based signature/visibility validation, emission of
  cfg-gated fn + linkme static. Self-contained unit tests for title-casing helper.
- **Plan 03 (Area D):** `gallery-smoke` crate + FRAME-03 symbol-table assertion. New workspace
  member that registers a toy demo, tests `registered_demos()` round-trip, and contains
  `trybuild` fixtures for FRAME-01 error-message stability. Symbol-table test lives in
  `backend/crates/marionette/tests/no_gallery_symbols.rs` (a cargo test that shells to `nm`).
- **Plan 04 (docs + closure):** Update `PROJECT.md` Key Decisions with linkme rationale
  (FRAME-02 logging requirement); flag Phase 17 conventions (every `#[gallery_demo]` annotation
  must use `key = "..."` because D-C1's default-from-ident collides when all fns are named
  `gallery_demo()`); close STATE.md blocker "Registration library selection".

This decomposition is the smallest atomic breakdown — Plans 01–03 can execute in parallel after
Plan 01's public types exist (the macro only references `marionette::gallery::DemoEntry` by
path), but simplest to serialize them since the total size is ~600 LoC across the whole phase.

<user_constraints>
## User Constraints (from 16-CONTEXT.md)

### Locked Decisions

**Area A — Registration library + iteration semantics**

- **D-A1:** `linkme` backs the distributed-slice registry. Chosen over `inventory` for
  type-safety, zero runtime cost, and explicit mental model. Logged in `PROJECT.md` Key
  Decisions per FRAME-02. Supersedes STATE.md blocker "Registration library selection".
- **D-A2:** Stable iteration order enforced by sorting at iteration time (alphabetical by
  `key`). `linkme`'s native order is linker-defined, not portable. One-time O(n log n) at
  first call; deterministic across Linux/macOS/CI.
- **D-A3:** Duplicate-key collisions: debug-assert panic in debug, `tracing::warn` +
  keep-first in release. `linkme` cannot detect semantic duplicates at compile time.
- **D-A4:** Sort + collision check memoized via `std::sync::OnceLock<Vec<&'static
  DemoEntry>>` (or equivalent). One-time cost per process.

**Area B — Feature-gate strategy**

- **D-B1:** `#[gallery_demo]` gates BOTH the annotated fn body AND the linkme static via
  `#[cfg(feature = "gallery")]`. Under default `cargo build -p marionette`, neither the fn
  symbol nor the registry entry exists in `libmarionette.rlib`.
- **D-B2:** Composite demos are cfg-gated implicitly as a unit. Phase 16 note only —
  nothing to enforce here.
- **D-B3:** `gallery` feature lives on `marionette` only. Pulls `linkme` as an optional dep
  (`linkme = { version = "<pin>", optional = true }`, `[features] gallery = ["dep:linkme"]`).
  `marionette-macros` stays feature-free.
- **D-B4:** `registered_demos()` is ALWAYS compiled (stub returning empty iterator under
  default, real implementation under `--features gallery`). Public API is symmetric —
  consumers never need cfg-guards.

**Area C — Registry key + `DemoEntry` shape**

- **D-C1:** Key defaults from fn ident; overridable via `#[gallery_demo(key = "button")]`.
  Phase 17 implication: every built-in will use explicit `key = "..."` because all fns are
  named `gallery_demo()` per DEMO-01.
- **D-C2:** `DemoEntry = { key: &'static str, render: fn() -> Node, display_name: &'static
  str }`. No `component_type` field, no source-location field.
- **D-C3:** `display_name` default = title-cased + space-separated `key` (e.g. `"text-input"
  → "Text Input"`). Override via `#[gallery_demo(name = "...")]`. Simple ASCII algorithm.
- **D-C4:** Use `darling` for attribute parsing (consistent with `component_builder.rs`).
  Use `syn` for signature/visibility validation.

**Area D — Verification approach (FRAME-03 + FRAME-04)**

- **D-D1:** FRAME-03 verification via `nm` symbol-table grep. Default build: zero
  `gallery_demo` substring matches in `libmarionette.rlib`. Under `--features gallery`:
  matches present. Implementation shape (cargo test shelling to `nm` vs Makefile target)
  is Claude's discretion.
- **D-D2:** FRAME-04 smoke test lives in a NEW workspace crate `backend/crates/gallery-smoke/`.
  Depends on `marionette` with `features = ["gallery"]`. Registers `#[gallery_demo(key =
  "smoke")] pub fn smoke() -> Node`. `#[test]` iterates `registered_demos()`, asserts
  `"smoke"` entry is present with expected `display_name`. Exercises cross-crate
  distributed-slice wiring.
- **D-D3:** `gallery-smoke` is PERMANENT (not retired after Phase 17). Automated counterpart
  to Phase 17's `gallery-demo` binary. Houses `trybuild` fixtures for FRAME-01
  error-message stability.
- **D-D4:** `trybuild` fixtures inside `gallery-smoke/tests/ui/` validate FRAME-01's "clear
  compiler error that names the violated rule" — wrong signature, non-pub, applied to
  struct, wrong return type.

### Claude's Discretion

- Exact macro error wording; preference is to mirror the crisp style of existing
  `ComponentBuilder` + `action` macros.
- `::linkme` vs `marionette::__private::linkme` path re-export; planner's call (this
  research recommends direct `::linkme` — simpler, and `marionette-macros` stays
  feature-free so re-export would add plumbing for marginal benefit).
- `marionette::gallery` submodule vs top-level; preference: submodule.
- Exact `linkme` version pin; **recommendation: `linkme = "0.3.36"` pinned to `0.3`
  compatibility range**.
- `impl Iterator<Item = &'static DemoEntry>` vs `&'static [&'static DemoEntry]` for
  `registered_demos()`; **recommendation: `impl Iterator` (roadmap wording, and the
  memoized `Vec` means `.iter().copied()` yields the iterator cheaply)**.
- `OnceLock` vs `OnceCell` vs hand-rolled; **recommendation: `std::sync::OnceLock`
  (stable since 1.70, zero deps)**.
- Symbol-test implementation shape; **recommendation: `#[test]` in
  `backend/crates/marionette/tests/no_gallery_symbols.rs` that shells to `cargo build` +
  `nm`. Keeps everything `cargo test --workspace` picks up; no CI wiring change**.

### Deferred Ideas (OUT OF SCOPE for Phase 16)

- GALLERY-LINT (CI lint enforcing every built-in has `gallery_demo()`) — v1.3+.
- `component_type` field on `DemoEntry`.
- `order` / `weight` attribute arg for nav ordering.
- Enforcement policy for pure-fn contract — Phase 17 documents; future may CI-enforce.
- `inventory` as a fallback/alternative — EXPLICITLY REJECTED.
- Macro emitting richer per-demo metadata (source file path, crate, doc comment).
- Third-party (non-marionette) crates registering demos — v1.2 ships demos for
  built-ins only; `linkme` would technically allow it.
- `marionette-gallery-demo` as a shipped documentation artifact library.
- Protocol-side gallery metadata.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FRAME-01 | `#[gallery_demo]` attribute macro with signature/visibility validation and clear compiler errors | §2 (macro scaffolding) — darling attr parsing + `syn::ItemFn` validation; §5 (`gallery-smoke`) — `trybuild` fixtures validate stderr output |
| FRAME-02 | `registered_demos()` iteration API backed by `linkme`, stable order, choice logged in PROJECT.md | §1 (linkme usage) — empirically verified cross-crate wiring; §3 (public API) — sort + memoization via `OnceLock`; §8 (PROJECT.md update in Plan 04) |
| FRAME-03 | `gallery` feature gate; default build has zero demo symbols, verified by artifact inspection | §4 (symbol-test) — `nm`-grep on rlib archives; §1 — verified that cfg-gated `distributed_slice` does NOT emit symbols under default build |

FRAME-04 (the smoke test) is not a standalone requirement but a success criterion of Phase 16
(Success Criterion #4 in ROADMAP.md) — the `gallery-smoke` crate discussed in §5 satisfies it.
</phase_requirements>

## Project Constraints (from repo conventions + codebase docs)

No `./CLAUDE.md` at repo root. Constraints inferred from `.planning/codebase/CONVENTIONS.md` and
existing crate code:

1. **Rust edition 2024 workspace-wide.** `backend/rustfmt.toml` pins `edition = "2024"`.
2. **`#![warn(clippy::pedantic)]` on every crate.** `#![allow(clippy::module_name_repetitions)]`
   is the only workspace-wide allow. New code MUST clear pedantic without introducing additional
   allows unless absolutely justified.
3. **Workspace dependency inheritance.** All shared deps (`serde`, `axum`, `syn`, `darling`, etc.)
   live in `backend/Cargo.toml` `[workspace.dependencies]` and crates opt in with `name.workspace
   = true`. New deps (i.e. `linkme`, `trybuild`) MUST follow the same pattern — add to
   `[workspace.dependencies]` in `backend/Cargo.toml`, then `linkme = { workspace = true,
   optional = true }` in `marionette/Cargo.toml`.
4. **Test file placement.** Integration tests under `crate/tests/*.rs` (e.g.
   `marionette/tests/macro_tests.rs`). Inline unit tests via `#[cfg(test)] mod tests` at bottom
   of the source file.
5. **CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build
   --release` — all inside `backend/`**. See `.github/workflows/ci.yml:27-43`. Any new test
   picked up by `cargo test --workspace` needs no CI wiring.
6. **Commit message style (from `git log`):** `type(scope): subject` where scope is the phase
   number (`docs(16)`, `feat(16-02)`, `fix(13-07)`, `test(15-07)`) or a doc-area (`docs:`,
   `chore:`). For this phase, expect `feat(16-01)`, `feat(16-02)`, `feat(16-03)`, etc.
7. **Module-level doc comments use `//!`; public items use `///`.** Existing
   `marionette-macros/src/action.rs` and `component_builder.rs` are templates.

## Architectural Responsibility Map

Phase 16 is pure-Rust backend framework work. No frontend, no protocol changes, no UI. The
tiers below map capability → owning source file.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Attribute proc macro expansion (`#[gallery_demo]`) | `marionette-macros` crate | — | Proc-macro-only crate per its `[lib] proc-macro = true`; all AST manipulation happens here |
| Registry type + distributed slice (`DemoEntry`, `DEMOS`) | `marionette` crate (`gallery` submodule) | — | Public-facing types that consumers import; `marionette-macros` emits tokens that reference these by path |
| Iteration API + sort + dedup memoization (`registered_demos()`) | `marionette` crate (`gallery` submodule) | `std::sync::OnceLock` | Runtime logic that must be always-compiled (D-B4); `OnceLock` gives us thread-safe memoization without new deps |
| Feature-gate plumbing (`gallery = ["dep:linkme"]`) | `marionette/Cargo.toml` | `backend/Cargo.toml` `[workspace.dependencies]` | Standard Rust feature-flag idiom; workspace-level entry for version pinning |
| Toy demo registration + round-trip test | `gallery-smoke` crate | — | New workspace member; exercises cross-crate wiring that same-crate tests would miss |
| Compile-error stability fixtures | `gallery-smoke/tests/ui/` | `trybuild` dev-dep | `trybuild` is the idiomatic tool for `.stderr`-pinned macro tests |
| Symbol-table verification (FRAME-03) | `marionette/tests/no_gallery_symbols.rs` | `nm` (via `std::process::Command`) | `cargo test --workspace` picks it up; no CI wiring change |
| Decision log update | `.planning/PROJECT.md` §Key Decisions | — | FRAME-02 requires the linkme choice be logged |

## 1. `linkme` usage patterns for a cross-crate registry

### Version and platform support

`[VERIFIED: cargo info linkme]` Current version: **`0.3.36`** (as of 2026-04-21). MSRV: **Rust
1.71**. License: MIT OR Apache-2.0. Repository: `https://github.com/dtolnay/linkme`.

`[VERIFIED: docs.rs/linkme + GitHub README]` Platform support matrix covers Linux, macOS,
Windows, FreeBSD, OpenBSD, illumos. WebAssembly (`wasm32`) is **not** on the explicit support
list — `[ASSUMED]` this is not a blocker for Phase 16 because the phase is backend-only and
no `gallery-demo` or `gallery-smoke` target is wasm.

`[VERIFIED: empirical test, /tmp/linkme-check]` `linkme 0.3.36` compiles cleanly on `rustc
1.93.1` with Rust edition 2024, no nightly features required. The `used_linker` crate feature
exists but is NOT needed for stable-Rust usage.

`[CITED: docs.rs/linkme/0.3.35/linkme/struct.DistributedSlice]` "The implementation is based on
`link_section` attributes and platform-specific linker support. It does not involve
life-before-main or any other runtime initialization on any platform. This is a zero-cost safe
abstraction that operates entirely during compilation and linking."

### Public static declaration shape

`[VERIFIED: docs.rs/linkme + empirical test]` The canonical declaration is:

```rust
// In marionette/src/gallery.rs (only when cfg(feature = "gallery"))
#[linkme::distributed_slice]
pub static DEMOS: [DemoEntry] = [..];
```

The `= [..]` initializer is the syntactic marker linkme requires; it is not a real expression.
Missing it causes a parse error.

Note: JetBrains's Rust IDE parses `[..]` as invalid syntax. `[CITED: docs.rs/linkme/0.3.35/
linkme/index_search]` — workaround exists using dummy initializer expressions, but this is not
a blocker (rust-analyzer / VS Code handle it correctly).

### Cross-crate submission

`[VERIFIED: empirical test, /tmp/linkme-feature-test]` A submission in crate `B` against a
distributed slice declared in crate `A` works via the attribute form:

```rust
// In gallery-smoke/src/lib.rs — what the macro emits
#[cfg(feature = "gallery")]
#[::linkme::distributed_slice(::marionette::gallery::DEMOS)]
static __GALLERY_DEMO_smoke: ::marionette::gallery::DemoEntry =
    ::marionette::gallery::DemoEntry {
        key: "smoke",
        render: smoke,
        display_name: "Smoke",
    };
```

At runtime, `marionette::registered_demos()` observes the entry. No special linker flags, no
codegen-units tuning, no LTO concerns. `[VERIFIED]` tested with `dev` profile (default cargo
settings).

### Feature-gate interaction (the subtle bit)

`[VERIFIED: empirical test]` When the target slice is cfg-gated off (e.g.
`marionette` built without `--features gallery` so `marionette::gallery::DEMOS` doesn't exist),
a submission that is NOT cfg-gated in the consumer fails to compile with `E0433: failed to
resolve: use of undeclared crate or module`.

**Therefore the macro MUST emit `#[cfg(feature = "gallery")]` wrapping the `#[distributed_slice]`
static**, so the submission only exists when the consumer crate has its own `gallery` feature
enabled.

`[VERIFIED: empirical test]` But `cfg(feature = "gallery")` resolves in the CONSUMER crate's
feature namespace, not `marionette`'s. Concretely:

- If `gallery-smoke`'s `Cargo.toml` does NOT declare a `gallery` feature, then
  `cfg(feature = "gallery")` in gallery-smoke's compile context is ALWAYS false — the
  macro-emitted static is pruned regardless of how `marionette` was built.
- Therefore `gallery-smoke` (and any other consumer that wants to register demos, including
  `marionette` itself when Phase 17 adds built-in demos) MUST declare its own `gallery` feature
  that propagates: `[features] gallery = ["marionette/gallery"]`.

**Recommended feature plumbing:**

```toml
# backend/crates/marionette/Cargo.toml
[features]
gallery = ["dep:linkme"]

[dependencies]
linkme = { workspace = true, optional = true }

# backend/crates/gallery-smoke/Cargo.toml
[dependencies]
marionette = { path = "../marionette", features = ["gallery"] }

[features]
default = ["gallery"]                  # gallery-smoke is test-only; gallery always on
gallery = ["marionette/gallery"]       # ensures cfg(feature="gallery") resolves true here
```

Under this scheme:
- `cargo build -p marionette` (default): no `gallery` feature anywhere → zero demo symbols in
  `libmarionette.rlib`. ✓ FRAME-03.
- `cargo build -p marionette --features gallery`: internal `#[gallery_demo]` annotations
  (Phase 17) compile in. Symbols present in `libmarionette.rlib`. ✓
- `cargo test -p gallery-smoke`: gallery-smoke's default features include `gallery` →
  `marionette/gallery` is enabled → the macro-emitted static resolves → registry round-trips.
  ✓ FRAME-04.

`[ASSUMED]` This feature-propagation pattern mirrors what most distributed-slice codebases use
(e.g. `inventory`-based frameworks have the same namespace-scoped `cfg` issue). No explicit
citation, but it's a direct consequence of how Cargo features are scoped per-crate.

### Initializer requirements

`[VERIFIED: docs.rs/linkme/0.3.35/linkme/struct.DistributedSlice]` "The initializer is required
to be a const expression." So the macro must emit a `static` with a const-expressible
`DemoEntry { ... }` literal. Since all three fields (`&'static str`, `fn() -> Node`, `&'static
str`) are const-legal, this works without further const tricks.

### Duplicate detection

`[VERIFIED: empirical test, /tmp/dup-test]` `linkme` does NOT detect semantic duplicates at
compile time or link time. Three submissions with the same `key` value (but distinct static
names) all land in the slice. `linkme`'s own `static_slice()` fn has a `DUPCHECK` guard, but
that catches only literal re-declarations of the distributed_slice static itself (not duplicate
entries pointing at the same `key` string).

**Therefore D-A3's runtime collision check is load-bearing** — `registered_demos()` MUST scan
the memoized sorted vec for adjacent duplicates and `debug_assert!` / `tracing::warn!`
accordingly.

### Iteration

`[CITED: docs.rs/linkme]` `DistributedSlice<T>` implements `IntoIterator<Item = &'static T>`
via `static_slice().iter()`. Also `Deref<Target = [T]>`, so indexing, `.len()`, `.iter()`, and
`&DEMOS[1..]` all work. **Order is linker-defined**, not portable — confirming CONTEXT.md D-A2.

### Version pin recommendation

`[VERIFIED]` Pin to `linkme = "0.3"` (Cargo's semver rules will resolve to the latest `0.3.x`
— currently `0.3.36`). Rationale: `linkme` has been `0.3.x` since 2020 without a breaking bump;
pinning to `0.3` gives automatic patch/minor updates within the stable API. Exact pin
`linkme = "0.3.36"` is also acceptable if the reviewer prefers Cargo.lock exactness.

## 2. Attribute proc-macro scaffolding in `marionette-macros`

### Module layout

Following the existing pattern in `marionette-macros/src/lib.rs:5-7`:

- Add `mod gallery_demo;` alongside `mod action;`, `mod component_builder;`, `mod requires;`.
- Add `#[proc_macro_attribute] pub fn gallery_demo(attr: TokenStream, item: TokenStream) ->
  TokenStream` that delegates to `gallery_demo::gallery_demo_impl(attr.into(), item.into())
  .into()`. This is identical to the pattern at `lib.rs:45-48` (`#[action]`) and
  `lib.rs:63-66` (`#[requires]`).
- Create `marionette-macros/src/gallery_demo.rs` as the new implementation file.

### Attribute parsing with `darling`

`[VERIFIED: marionette-macros/src/component_builder.rs:1-22]` The darling convention used in-repo
is `#[derive(FromDeriveInput)]` / `#[derive(FromField)]` for derive macros. For an
attribute-style macro, use `#[derive(FromMeta)]`:

```rust
use darling::FromMeta;
use darling::ast::NestedMeta;

#[derive(FromMeta, Default)]
struct GalleryDemoOpts {
    #[darling(default)]
    key: Option<String>,
    #[darling(default)]
    name: Option<String>,
}

fn parse_opts(attr: TokenStream) -> Result<GalleryDemoOpts, TokenStream> {
    let meta_list = match NestedMeta::parse_meta_list(attr) {
        Ok(v) => v,
        Err(e) => return Err(darling::Error::from(e).write_errors()),
    };
    GalleryDemoOpts::from_list(&meta_list).map_err(|e| e.write_errors())
}
```

`[CITED: docs.rs/darling/0.23.0]` `FromMeta` + `NestedMeta::parse_meta_list` is the idiomatic
path for attribute-arg parsing since darling 0.20.

### Signature/visibility validation with `syn`

`[VERIFIED: marionette-macros/src/action.rs:10-14]` The in-repo pattern is `syn::parse2::
<ItemFn>(item)`. For `#[gallery_demo]`, the validations expand to:

```rust
fn validate_item(func: &ItemFn) -> Result<(), syn::Error> {
    // 1. Must be pub.
    if !matches!(func.vis, syn::Visibility::Public(_)) {
        return Err(syn::Error::new_spanned(
            &func.sig.ident,
            "#[gallery_demo] requires `pub fn` visibility (found private fn)",
        ));
    }
    // 2. No args.
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            format!(
                "#[gallery_demo] fn must be `fn() -> Node` with zero arguments (found {})",
                func.sig.inputs.len()
            ),
        ));
    }
    // 3. No async, no generics, no where-clause.
    if func.sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            func.sig.asyncness,
            "#[gallery_demo] fn must not be async",
        ));
    }
    if !func.sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.generics,
            "#[gallery_demo] fn must not have generic parameters",
        ));
    }
    if func.sig.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &func.sig.generics.where_clause,
            "#[gallery_demo] fn must not have a where-clause",
        ));
    }
    // 4. Return type must be Node (string-match last path segment).
    match &func.sig.output {
        syn::ReturnType::Default => Err(syn::Error::new_spanned(
            &func.sig,
            "#[gallery_demo] fn must return `Node` (found unit return type)",
        )),
        syn::ReturnType::Type(_, ty) => {
            if !return_type_is_node(ty) {
                Err(syn::Error::new_spanned(
                    ty,
                    "#[gallery_demo] fn must return `Node` (an alias for \
                     `(String, marionette_protocol::Component)`)",
                ))
            } else {
                Ok(())
            }
        }
    }
}

fn return_type_is_node(ty: &syn::Type) -> bool {
    // Accept `Node`, `marionette::builders::Node`, `crate::Node`, etc.
    // Match last path segment ident literally.
    if let syn::Type::Path(p) = ty {
        if let Some(last) = p.path.segments.last() {
            return last.ident == "Node";
        }
    }
    false
}
```

**Design note on return-type matching:** The simplest approach is last-segment ident matching.
This accepts both `Node` and fully-qualified paths like `marionette::builders::Node` without
requiring the user to import the alias at a specific location. Misses: if a caller defines their
own `Node` type-alias pointing at something incompatible, the macro accepts it — but the
emitted code will fail to compile when the resulting static's `render: fn() -> Node` field
tries to match the real `DemoEntry::render` signature, producing a clear type-mismatch error at
the use-site. Acceptable per Phase 16's "reasonable error messages" bar.

### Applied-to-struct case

`[VERIFIED: syn docs]` If the user writes `#[gallery_demo] struct Foo;`, `syn::parse2::<ItemFn>`
fails. Catch it explicitly:

```rust
let func: ItemFn = match syn::parse2(item.clone()) {
    Ok(v) => v,
    Err(_) => {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(item),
            "#[gallery_demo] can only be applied to `pub fn` items (not structs, \
             modules, or other items)",
        )
        .to_compile_error();
    }
};
```

### Emission shape

```rust
fn gallery_demo_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let opts = match parse_opts(attr) { Ok(v) => v, Err(e) => return e };
    let func: ItemFn = match syn::parse2(item.clone()) { Ok(v) => v, Err(_) => /* err */ };
    if let Err(e) = validate_item(&func) {
        return e.to_compile_error();
    }

    let fn_ident = &func.sig.ident;
    let key = opts.key.unwrap_or_else(|| fn_ident.to_string());
    let display_name = opts.name.unwrap_or_else(|| title_case(&key));

    // Unique static ident — hash the key + a counter per span, or just use
    // the fn ident since two fns in the same scope can't share an ident.
    let static_ident = syn::Ident::new(
        &format!("__GALLERY_DEMO_{}", fn_ident),
        fn_ident.span(),
    );

    quote! {
        #[cfg(feature = "gallery")]
        #func

        #[cfg(feature = "gallery")]
        #[::linkme::distributed_slice(::marionette::gallery::DEMOS)]
        static #static_ident: ::marionette::gallery::DemoEntry =
            ::marionette::gallery::DemoEntry {
                key: #key,
                render: #fn_ident,
                display_name: #display_name,
            };
    }
}
```

**Unique static ident strategy:** Using `__GALLERY_DEMO_{fn_ident}` is sufficient because two
functions with the same ident cannot coexist in the same module scope. Phase 17's convention
("every built-in has a `gallery_demo()`" per DEMO-01) means each builder file gets its own
`__GALLERY_DEMO_gallery_demo` static — but each is in a different module, so no collision. ✓.

### Title-casing algorithm

`[RECOMMENDED]` Execute at macro expansion time, emitting a `&'static str` literal. Reason:
`display_name: &'static str` can be a const expression, avoids any runtime allocation, and keeps
the `DemoEntry` struct simple. ASCII-only algorithm:

```rust
fn title_case(key: &str) -> String {
    key.split(|c: char| c == '-' || c == '_')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
```

Test cases (add unit tests in `gallery_demo.rs`):
- `"button"` → `"Button"`
- `"text-input"` → `"Text Input"`
- `"data_table"` → `"Data Table"`
- `"OKLCH-swatches"` → `"OKLCH Swatches"` (first-char-upper preserves existing uppercase tail
  characters)
- `""` → `""` (edge case — macro's validation ensures key is non-empty in practice)

### Error span routing

`[VERIFIED: action.rs:11-19, requires.rs:8-13]` The in-repo pattern is `e.to_compile_error()`
for `syn::Error`. For darling errors, the equivalent is `e.write_errors()`. Both produce
`TokenStream` that rustc renders as a normal diagnostic. Use the more specific spanned form
(`syn::Error::new_spanned(thing, "msg")`) whenever a specific AST node caused the failure —
gives the user a red squiggle under the actual violation, not the whole fn.

## 3. Public API shape in `marionette` crate

### File layout

**Recommendation:** Single file `backend/crates/marionette/src/gallery.rs` (not `gallery/mod.rs`
directory), exported from `lib.rs` as `pub mod gallery;`. The content fits in ~80 LoC; a module
directory is overkill.

Add `pub mod gallery;` to `backend/crates/marionette/src/lib.rs` between
`pub mod extractors;` and `pub mod migration;` (alphabetical among existing pub mods at lines
4-13). No `pub use` flattening — consumers write `use marionette::gallery::{DemoEntry,
registered_demos};`. This matches CONTEXT.md Area E's preference for submodule form and is
consistent with the `marionette::builders::*` organization pattern.

### `DemoEntry` (always-compiled)

```rust
// gallery.rs

use marionette_protocol::Component;

/// Alias matching `marionette::builders::Node`. Demo fns return this tuple.
pub type Node = (String, Component);

/// Registry entry for a gallery demo. The macro-expanded static references
/// this type; `registered_demos()` yields `&'static DemoEntry`.
#[derive(Debug)]
pub struct DemoEntry {
    /// Stable, sort key. Derived from fn ident unless `key = "..."` override.
    pub key: &'static str,
    /// Render fn — takes no arguments, returns a `(String, Component)` node tuple.
    pub render: fn() -> Node,
    /// Nav-facing label. Title-cased from `key` unless `name = "..."` override.
    pub display_name: &'static str,
}
```

**Note on `Node` alias location:** `builders::Node` already exists at
`marionette/src/builders/node.rs:6`. To keep the public API clean, re-export it from
`gallery`:

```rust
pub use crate::builders::Node;
```

Or re-declare if circular-dep concerns emerge. The re-export is simpler and matches the
"single meaning for `Node`" principle.

### `DEMOS` distributed slice (feature-gated)

```rust
#[cfg(feature = "gallery")]
#[linkme::distributed_slice]
pub static DEMOS: [DemoEntry] = [..];
```

The `[..]` initializer is a linkme syntactic requirement.

### `registered_demos()` (always-compiled, per D-B4)

`[VERIFIED: std::sync::OnceLock docs]` `OnceLock` is stable since Rust 1.70, zero deps, and
thread-safe. The memoization sits at module level:

```rust
use std::sync::OnceLock;

static SORTED_CACHE: OnceLock<Vec<&'static DemoEntry>> = OnceLock::new();

#[cfg(feature = "gallery")]
fn build_sorted() -> Vec<&'static DemoEntry> {
    let mut v: Vec<&'static DemoEntry> = DEMOS.iter().collect();
    v.sort_by_key(|e| e.key);
    // Duplicate-key check per D-A3.
    for pair in v.windows(2) {
        if pair[0].key == pair[1].key {
            debug_assert!(
                false,
                "duplicate #[gallery_demo] key = {:?} (display_names: {:?}, {:?})",
                pair[0].key, pair[0].display_name, pair[1].display_name,
            );
            tracing::warn!(
                key = pair[0].key,
                first_display_name = pair[0].display_name,
                second_display_name = pair[1].display_name,
                "duplicate #[gallery_demo] key — keeping first, discarding second",
            );
            // Drop is handled by the dedup below.
        }
    }
    v.dedup_by(|a, b| a.key == b.key);  // Keep-first semantics.
    v
}

#[cfg(not(feature = "gallery"))]
fn build_sorted() -> Vec<&'static DemoEntry> { Vec::new() }

/// Return an iterator of all demos registered via `#[gallery_demo]`, in
/// stable alphabetical order by `key`. Under default build (no `gallery`
/// feature), yields an empty iterator.
///
/// First call memoizes the sorted + de-duplicated snapshot; subsequent
/// calls are zero-cost.
#[must_use]
pub fn registered_demos() -> impl Iterator<Item = &'static DemoEntry> {
    SORTED_CACHE.get_or_init(build_sorted).iter().copied()
}
```

**Return type rationale (D-B4 + Area E):** `impl Iterator<Item = &'static DemoEntry>` matches
the roadmap wording ("stable-ordered iterator"). Downside: loses direct `.len()` indexing at
the call-site. For Phase 17's nav iteration (`for demo in registered_demos()`), `impl Iterator`
is perfectly ergonomic. If Phase 17/18 discovers it needs `.len()`, add a companion
`registered_demos_slice() -> &'static [&'static DemoEntry]` then — non-breaking addition.

**Thread-safety note:** `OnceLock::get_or_init` is documented as thread-safe; concurrent first
callers race to produce a value but only one value is used. Since `build_sorted` is a pure
function of immutable linker state, either racer produces the same result. No lock-free
programming concerns.

### Under default build

The `SORTED_CACHE` static is always compiled. Under `cfg(not(feature = "gallery"))`,
`build_sorted` returns empty `Vec`. `DEMOS` is not referenced anywhere in the always-compiled
code path, so it doesn't need to exist under default build. Confirmed by §4's symbol-table
test.

## 4. FRAME-03 symbol-test implementation

### Option comparison

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| A: `#[test]` in `marionette/tests/no_gallery_symbols.rs` that shells to `cargo build` + `nm` | `cargo test --workspace` picks it up; no CI wiring change; self-contained | Slower (spawns subprocess per test); brittle if CARGO_TARGET_DIR not respected | **RECOMMENDED** |
| B: Makefile target `make check-no-gallery-symbols` invoked by CI | Faster (no cargo-test rebuild loop); cleanly separated from unit tests | Requires CI config change; doesn't run under `make test` unless added | Acceptable alternative |
| C: Rely on `registered_demos()` being empty under default | Trivial | Fails FRAME-03's "zero demo SYMBOLS" — symbols can exist even if the slice iterates empty | **REJECTED** |

### Recommended shape (Option A)

```rust
// backend/crates/marionette/tests/no_gallery_symbols.rs
//! FRAME-03 verification: default `cargo build -p marionette` emits zero
//! `gallery_demo` symbols in `libmarionette.rlib`; enabling `--features
//! gallery` brings them in.

use std::process::Command;

fn target_dir() -> std::path::PathBuf {
    // Respect CARGO_TARGET_DIR if set; else use the workspace default.
    std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop(); // backend/crates/marionette -> backend/crates
            p.pop(); // -> backend
            p.push("target");
            p
        })
}

fn build_and_dump_symbols(features: &[&str]) -> String {
    let mut cmd = Command::new(env!("CARGO"));
    cmd.arg("build").arg("-p").arg("marionette");
    for f in features {
        cmd.arg("--features").arg(f);
    }
    let build = cmd.output().expect("cargo build should succeed");
    assert!(
        build.status.success(),
        "cargo build failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );
    let rlib = target_dir().join("debug").join("libmarionette.rlib");
    assert!(rlib.exists(), "expected rlib at {:?}", rlib);
    let nm = Command::new("nm")
        .arg("--demangle")
        .arg(&rlib)
        .output()
        .expect("nm should be available on PATH");
    String::from_utf8_lossy(&nm.stdout).into_owned()
}

#[test]
fn default_build_has_zero_gallery_symbols() {
    let symbols = build_and_dump_symbols(&[]);
    // Look for either DEMOS slice or __GALLERY_DEMO_ static names.
    let hits: Vec<&str> = symbols
        .lines()
        .filter(|l| l.contains("DEMOS") || l.contains("__GALLERY_DEMO_"))
        .collect();
    assert!(
        hits.is_empty(),
        "default build should have ZERO gallery symbols in libmarionette.rlib, \
         found {}:\n{}",
        hits.len(),
        hits.join("\n"),
    );
}

#[test]
fn gallery_feature_build_has_gallery_symbols() {
    let symbols = build_and_dump_symbols(&["gallery"]);
    // Under --features gallery, the DEMOS slice static exists but it may be
    // empty of entries until Phase 17 adds built-in demos. The DEMOS-linked
    // linker markers are the robust signal:
    let has_demos = symbols
        .lines()
        .any(|l| l.contains("DEMOS::LINKME_PLEASE") || l.contains("DEMOS::DUPCHECK"));
    assert!(
        has_demos,
        "gallery feature build should emit DEMOS symbols in libmarionette.rlib:\n{}",
        &symbols[..symbols.len().min(2000)],
    );
}
```

### Caveats and platform notes

`[VERIFIED]` `nm` is present on Linux and macOS CI images. Windows CI would need `dumpbin` or
LLVM `llvm-nm` — `[ASSUMED]` not a concern since the workflow runs on `ubuntu-latest` per
`.github/workflows/ci.yml:11`.

`[VERIFIED]` `rlib` files are `ar`-archives; `nm` on Linux GNU binutils handles them. Per the
empirical test `/tmp/linkme-feature-test`, `nm --demangle libmylib.rlib` produced the expected
output.

`[VERIFIED: empirical test]` Under default build, `libmarionette.rlib` will contain
`marionette::registered_demos` (always compiled) but NOT any `DEMOS` or `__GALLERY_DEMO_`
symbols — the test's filter must be specific enough to avoid matching `registered_demos`.
Filtering on `DEMOS` (upper-case) OR `__GALLERY_DEMO_` (upper-case prefix) avoids collision.

**Target-directory robustness:** `CARGO_TARGET_DIR` is honored. If a developer runs
`CARGO_TARGET_DIR=/tmp/mytarget cargo test -p marionette`, the test's `target_dir()` helper
picks up the env var. The default workspace `backend/target` is derived from
`env!("CARGO_MANIFEST_DIR")` at compile time.

**Potential brittleness: cargo's rlib cache.** If `cargo build -p marionette --features gallery`
was run most recently, a subsequent `cargo test -p marionette` might reuse the cached gallery
rlib. `[VERIFIED: empirical test]` Cargo uses feature-aware fingerprinting — running `cargo
build -p marionette` without `--features gallery` rebuilds a distinct rlib. No manual cache
invalidation needed. However, running the two tests **in parallel** (cargo-test default
behavior) could cause cargo to thrash between the two builds. **Mitigation:** add
`#[ignore]` + `#[test]` or single-thread-the-tests via a `CARGO_BUILD_JOBS=1` env, OR use
distinct `--target-dir` per sub-test:

```rust
cmd.arg("--target-dir").arg(format!("{}/no-gallery-check", target_dir().display()));
```

Using per-test target-dirs is the simplest robust fix. Document this in the test file header.

## 5. `gallery-smoke` crate layout

### Cargo.toml

```toml
[package]
name = "gallery-smoke"
version = "0.1.0"
edition.workspace = true
license.workspace = true
publish = false   # Workspace-internal test-only crate.

[dependencies]
marionette = { path = "../marionette", features = ["gallery"] }
marionette-protocol = { path = "../marionette-protocol" }

[features]
default = ["gallery"]
gallery = ["marionette/gallery"]    # Propagates so cfg(feature="gallery") is true here.

[dev-dependencies]
trybuild = { workspace = true }
```

**Note:** `linkme` is NOT a direct dep of `gallery-smoke`. The `#[gallery_demo]` macro emits
`::linkme::distributed_slice(...)` using the absolute-path form. For that path to resolve,
`linkme` must be visible. Options:

- **Re-export `linkme` from `marionette`:** `marionette/src/gallery.rs` adds
  `#[cfg(feature = "gallery")] pub use ::linkme;` under a not-too-public name like
  `marionette::gallery::__linkme` or `marionette::__private::linkme`. Then the macro emits
  `#[::marionette::gallery::__linkme::distributed_slice(::marionette::gallery::DEMOS)]`.
- **Have each consumer crate add `linkme` as its own dep:**
  `gallery-smoke/Cargo.toml` adds `linkme = { workspace = true }` (no optional).

**Recommendation: re-export from `marionette`.** Downsides to asking every consumer to
declare `linkme`: Phase 17's `gallery-demo` would need it too; any future third-party crate
that wants to register demos would need it; `linkme` version drift becomes possible between
`marionette` and consumers. The re-export route is the clean one — it's the pattern `serde`
uses for `#[derive(Serialize)]` via `serde::__private::Serialize`.

Add `[workspace.dependencies]` entry in `backend/Cargo.toml`:
```toml
linkme = "0.3"
trybuild = "1"
```

### src/lib.rs

```rust
//! Permanent test-fixture crate for the Phase 16 gallery-demo framework.
//!
//! Exercises the end-to-end pipeline:
//!   - The `#[gallery_demo]` proc macro in `marionette-macros`,
//!   - The `linkme`-backed `DEMOS` distributed slice in `marionette::gallery`,
//!   - Cross-crate submission (this crate submits, `marionette` aggregates),
//!   - The `#[gallery_demo(key = "...", name = "...")]` attribute parsing.
//!
//! This crate is NOT retired after Phase 17 — it is the automated counterpart
//! to the `gallery-demo` binary (which validates the registry by rendering
//! demos in a browser). See CONTEXT.md §D-D3.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use marionette::builders::standard::Text;
use marionette::gallery::Node;
use marionette_macros::gallery_demo;

/// Toy demo registered against the Phase 16 registry. The `#[test]` in
/// `tests/registry_roundtrip.rs` asserts this key + display_name appear.
#[gallery_demo(key = "smoke", name = "Smoke Check")]
pub fn smoke() -> Node {
    Text::new("gallery-smoke").build()
}
```

Minimal fn body — `Text::new(s).build()` is the simplest public builder.

### tests/registry_roundtrip.rs

```rust
//! FRAME-04 verification: the `#[gallery_demo]` macro + linkme slice +
//! cross-crate submission all wire up end-to-end.

use marionette::gallery::registered_demos;

#[test]
fn smoke_demo_is_registered() {
    let entries: Vec<_> = registered_demos().collect();
    let smoke = entries
        .iter()
        .find(|e| e.key == "smoke")
        .expect("expected `smoke` entry from gallery-smoke's #[gallery_demo]");
    assert_eq!(smoke.display_name, "Smoke Check");
}

#[test]
fn registry_is_alphabetically_ordered() {
    let keys: Vec<&'static str> = registered_demos().map(|e| e.key).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted, "registered_demos() must yield keys in sorted order");
}

#[test]
fn registry_iteration_is_idempotent() {
    let first: Vec<&str> = registered_demos().map(|e| e.key).collect();
    let second: Vec<&str> = registered_demos().map(|e| e.key).collect();
    assert_eq!(first, second, "memoized iteration must be deterministic");
}
```

### tests/ui/*.rs + *.stderr (trybuild fixtures)

Four fixtures per D-D4. Filenames use `fail_*.rs` convention so the trybuild glob picks them
up.

```rust
// tests/ui/fail_not_pub.rs
use marionette::gallery::Node;
use marionette::builders::standard::Text;
use marionette_macros::gallery_demo;

#[gallery_demo]
fn private_demo() -> Node {
    Text::new("nope").build()
}

fn main() {}
```

```rust
// tests/ui/fail_wrong_signature.rs
use marionette::gallery::Node;
use marionette::builders::standard::Text;
use marionette_macros::gallery_demo;

#[gallery_demo]
pub fn has_args(arg: u32) -> Node {
    Text::new("nope").build()
}

fn main() {}
```

```rust
// tests/ui/fail_wrong_return.rs
use marionette_macros::gallery_demo;

#[gallery_demo]
pub fn wrong_return() -> Vec<u32> {
    vec![]
}

fn main() {}
```

```rust
// tests/ui/fail_applied_to_struct.rs
use marionette_macros::gallery_demo;

#[gallery_demo]
pub struct NotAFn;

fn main() {}
```

### tests/compile_errors.rs

```rust
#[test]
fn compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_*.rs");
}
```

### `.stderr` rustc-version sensitivity

`[CITED: docs.rs/trybuild/1.0]` `trybuild` compares actual stderr against `.stderr` files.
Rust compiler diagnostic format occasionally changes between versions — stderr mismatch is not
a test failure per se, but a "different output" event that `TRYBUILD=overwrite` can refresh.

**Recommendation:**
1. Pin the toolchain via a `rust-toolchain.toml` at repo root, OR
2. Commit `.stderr` files generated against the CI-used `stable` toolchain, and accept that
   a new stable release may require `TRYBUILD=overwrite` to regenerate.

Given the repo already uses `dtolnay/rust-toolchain@stable` (CI line 35) without pinning, path
(2) is the pragmatic choice. Add a short README in `tests/ui/` explaining the regeneration
procedure:

```
# Regenerating trybuild .stderr files after rustc version bump:
cd backend
TRYBUILD=overwrite cargo test -p gallery-smoke --test compile_errors
git diff backend/crates/gallery-smoke/tests/ui/*.stderr  # review, commit if reasonable
```

### trybuild tests and `cargo test --workspace`

`[CITED: docs.rs/trybuild]` trybuild tests are normal `#[test]` fns. They compile the `.rs`
files in `tests/ui/` as internal processes; the test harness asserts those compilations fail
with the expected output. `cargo test --workspace` picks up `tests/compile_errors.rs` like any
other test file — no special wiring.

## 6. Workspace integration

### Edits to `backend/Cargo.toml`

```toml
[workspace]
resolver = "3"
members = [
    "crates/marionette-protocol",
    "crates/marionette-macros",
    "crates/marionette",
    "crates/crm-demo",
    "crates/gallery-smoke",   # NEW
]

# ... existing [workspace.package] ...

[workspace.dependencies]
# ... existing entries ...
linkme = "0.3"        # NEW (used optionally by marionette; directly by gallery-smoke if
                      #  re-export path is rejected — but re-export is recommended)
trybuild = "1"        # NEW (dev-dep in gallery-smoke)
```

### Edits to `backend/crates/marionette/Cargo.toml`

```toml
[dependencies]
# ... existing ...
linkme = { workspace = true, optional = true }   # NEW
tracing = { workspace = true }                   # existing, noted because build_sorted uses tracing::warn!

[features]
gallery = ["dep:linkme"]                         # NEW
```

`dep:linkme` syntax disables the implicit feature name that would otherwise be created;
`cargo`-recommended since 1.60.

### CI considerations

`[VERIFIED: .github/workflows/ci.yml:29-43]` CI runs `cargo test` in `backend/`. This picks up
`gallery-smoke` tests automatically including trybuild. No CI YAML change required.

**However:** `cargo test --workspace` default runs tests in `gallery-smoke` with
gallery-smoke's default features (which include `gallery`). That's the correct behavior for
FRAME-04. But it means the FRAME-03 symbol-test (in `marionette/tests/no_gallery_symbols.rs`)
will be running alongside builds that may have already cached a `--features gallery` variant.
See §4 mitigation ("use distinct `--target-dir` per sub-test"); this is the simplest fix.

**Phase 17 forward-looking note:** When Phase 17 adds `gallery-demo` as a 6th workspace member
with its own `gallery` feature, the CI run will build and test it by default — `cargo run -p
gallery-demo` for manual smoke stays a developer concern. No CI change at that phase either.

## 7. Validation Architecture (Nyquist Dimension 8)

### Test framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (built into toolchain) |
| Config file | None; configured via `Cargo.toml` `[dev-dependencies]` + `tests/` convention |
| Quick run command | `cargo test -p gallery-smoke` |
| Full suite command | `cargo test --workspace` |

**New test types introduced this phase:**
- `trybuild` for compile-error fixture verification (first use in the repo).
- Subprocess-based symbol-table assertion (first use in the repo — previous tests all run in
  a single `rustc` process).

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| FRAME-01 | `#[gallery_demo]` on valid `pub fn name() -> Node` expands without error | unit | `cargo test -p marionette-macros` (inline tests in `gallery_demo.rs`) | Wave 0 |
| FRAME-01 | Misuse: private fn → clear error naming "visibility" | compile-fail (trybuild) | `cargo test -p gallery-smoke --test compile_errors` | Wave 0 |
| FRAME-01 | Misuse: fn with args → clear error naming "signature"/"arguments" | compile-fail (trybuild) | `cargo test -p gallery-smoke --test compile_errors` | Wave 0 |
| FRAME-01 | Misuse: fn returning `Vec<Node>` (wrong ret type) → clear error naming "return type" | compile-fail (trybuild) | `cargo test -p gallery-smoke --test compile_errors` | Wave 0 |
| FRAME-01 | Misuse: applied to `struct` → clear error naming "item kind" | compile-fail (trybuild) | `cargo test -p gallery-smoke --test compile_errors` | Wave 0 |
| FRAME-02 | `registered_demos()` returns stable-ordered iterator | integration | `cargo test -p gallery-smoke --test registry_roundtrip` | Wave 0 |
| FRAME-02 | Sort + collision memoization runs exactly once | unit | `cargo test -p marionette gallery::` (inline tests in `gallery.rs`) | Wave 0 |
| FRAME-02 | Default (no-gallery) build yields empty iterator | unit | `cargo test -p marionette gallery::tests::registered_demos_empty_under_default` | Wave 0 |
| FRAME-02 | Duplicate-key collision: debug panics / release warns | unit (both branches) | `cargo test -p marionette` (debug); `cargo test -p marionette --release` (release) | Wave 0 |
| FRAME-03 | Default `cargo build -p marionette` produces zero `gallery_demo`/`DEMOS` symbols | integration | `cargo test -p marionette --test no_gallery_symbols default_build_has_zero_gallery_symbols` | Wave 0 |
| FRAME-03 | `cargo build -p marionette --features gallery` produces `DEMOS` symbols | integration | `cargo test -p marionette --test no_gallery_symbols gallery_feature_build_has_gallery_symbols` | Wave 0 |
| FRAME-04 (roadmap SC #4) | Smoke test registers toy demo, iterates, asserts key present | integration | `cargo test -p gallery-smoke --test registry_roundtrip smoke_demo_is_registered` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p <touched-crate>` — scoped to the affected crate.
- **Per wave merge:** `cargo test --workspace` — picks up all new tests (marionette-macros
  inline, marionette inline + integration, gallery-smoke integration + trybuild).
- **Phase gate:** `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test
  --workspace && cargo build --release -p marionette` green before `/gsd-verify-work`.

### Boundary conditions covered

- **Empty registry** (default build): `registered_demos()` yields empty iter — covered by
  a unit test in `gallery.rs::tests`.
- **Single entry**: gallery-smoke registers exactly one demo — FRAME-04 test covers trivial-
  sort case.
- **Duplicate keys**: unit test registers two demos with the same `key` in a child mod, asserts
  debug-panic or release-warn. **Implementation note:** because the collision check is inside
  `registered_demos()`'s memoization, the test needs to inject duplicates via
  `#[gallery_demo(key = "dup")]` twice in the same crate OR use a test-only helper that
  constructs a `Vec<&'static DemoEntry>` and passes it through the same sort/dedup logic.
  Refactoring `build_sorted` to take a `&[&'static DemoEntry]` parameter (pure function) enables
  direct unit testing without global state.

### Error paths

- Four macro-misuse cases covered by trybuild fixtures (D-D4).
- Wrong return type (e.g. unit return, wrong named type, generic wrapper) — three variants
  are sufficient; we're not exhaustively testing syn's matcher.

### Concurrency

`OnceLock::get_or_init` is thread-safe by the stdlib contract. A straightforward correctness
test would spawn N threads that all call `registered_demos()` concurrently and assert they
all yield identical sorted sequences — easy addition, low payoff, include only if Plan 01's
wave budget permits.

### Performance

Registry size is ≤~50 entries (Phase 17 adds ~20 built-ins; Phase 18-20 add screens and
exercisers). Sort is O(n log n) on n≤50 — microseconds. Memoized after first call.
No perf test necessary; document the complexity in `registered_demos()`'s docstring.

### Test coverage goal

Every macro validation branch has a trybuild fixture; every public API fn (`DemoEntry`,
`registered_demos()`) has at least one `#[test]` asserting behavior. FRAME-03's symbol-table
test is the artifact-level cross-check that catches regressions where a well-meaning
refactor accidentally leaks demo code into the default-feature rlib.

### Wave 0 Gaps

- [ ] `backend/crates/gallery-smoke/` — entire new crate including `Cargo.toml`, `src/lib.rs`,
      `tests/registry_roundtrip.rs`, `tests/compile_errors.rs`, `tests/ui/fail_*.rs` +
      `.stderr` pairs.
- [ ] `backend/crates/marionette/tests/no_gallery_symbols.rs` — new integration test file.
- [ ] `backend/crates/marionette-macros/src/gallery_demo.rs` — new module file.
- [ ] Inline unit tests in `backend/crates/marionette/src/gallery.rs` — sort-order,
      duplicate-key panic/warn, empty-under-default, memoization-idempotence.
- [ ] `[workspace.dependencies]` entries for `linkme` and `trybuild` in `backend/Cargo.toml`.
- [ ] `[features] gallery = ["dep:linkme"]` stanza in `backend/crates/marionette/Cargo.toml`.

All test infrastructure is pre-existing (`cargo test` + inline + integration tests); Phase 16
adds the new test files but no new framework machinery.

## 8. Project-specific conventions to follow

### From `.planning/codebase/CONVENTIONS.md` + existing crate code

- **Rust formatting:** `rustfmt` with `edition = "2024"` in `backend/rustfmt.toml`. CI runs
  `cargo fmt --check`.
- **Clippy:** `#![warn(clippy::pedantic)]` on every crate; only `clippy::module_name_repetitions`
  allowed project-wide. New code MUST pass `cargo clippy -- -D warnings` without additional
  allows. `marionette-macros/src/lib.rs:3` shows the one additional
  `clippy::needless_continue` allow for darling-generated code; fine to replicate if needed.
- **Doc comments:** `//!` for modules, `///` for items. `action.rs:6-10` and
  `component_builder.rs:17-29` are in-repo templates.
- **Imports:** `std::` first, external crates (alphabetical), internal crates, local modules.
- **`#[must_use]`** on builder methods and all fns that return a value consumers should act on.
  `component_builder.rs:91` and onward are templates.
- **Test organization:** inline `#[cfg(test)] mod tests` for unit tests; `tests/*.rs` for
  integration. `marionette/tests/macro_tests.rs` is the template for integration-style macro
  tests.
- **Commit message style** (from recent `git log`):
  - Docs: `docs(16): capture phase context ...`
  - Feat: `feat(16-01): register DemoEntry type + DEMOS slice`
  - Test: `test(16-03): trybuild fixtures for macro misuse`
  - Use the phase-plan prefix (`16-01`, `16-02`, etc.) in the scope slot.

### From the discuss-phase constraint list (CONTEXT.md Canonical References)

- `feedback_pre_deployment_no_backcompat.md`: **no migration shims, fix root causes.** Phase 16
  commits to `linkme` cleanly; no `inventory` fallback path, no `#[cfg]` toggles between the
  two libraries.
- `feedback_options_need_reasoning.md`: every option comes with pros/cons/rationale. This
  research file follows that pattern — see §4 (symbol-test options), §5 (linkme re-export
  vs consumer dep), §6 (target-dir robustness).
- `feedback_no_handrolling_ui.md`: N/A for Phase 16 (no UI).
- `feedback_use_chrome_for_uat.md`: N/A for Phase 16 (no user-facing surface).
- `project_protocol_node_patching_gap.md`: N/A — Phase 16 touches no protocol.

## Runtime State Inventory

Phase 16 is pure code + config. No stored data, no live service config, no OS-registered state,
no secrets/env vars, no pre-existing build artifacts to migrate.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — no databases or datastores touched | None |
| Live service config | None — no external services have state | None |
| OS-registered state | None — no OS-level registrations | None |
| Secrets / env vars | None — no new secrets; `TRYBUILD=overwrite` is developer-only | None |
| Build artifacts | None on initial implementation. After Plan 01 lands, `target/debug/libmarionette.rlib` will change shape under `--features gallery` (new symbols). `cargo clean -p marionette` suffices if a developer hits cache confusion. | None as part of the phase |

**Nothing found in any category — verified by reading the phase boundary (CONTEXT.md `<domain>`
section: "NOT a protocol change", "NOT any built-in demos", "NOT the gallery-demo binary") and
by the pure-code nature of the changes (new types, new proc-macro, new feature flag).**

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| `cargo` / `rustc` | All plans | ✓ | 1.93.1 (from cargo info) | — |
| `linkme` crate | Plan 01 (registry) | ✓ (via crates.io) | 0.3.36 | — (D-A1 rejects inventory fallback) |
| `trybuild` crate | Plan 03 (compile-fail fixtures) | ✓ (via crates.io) | 1.0.116 | — |
| `darling` crate | Plan 02 (macro attr parsing) | ✓ (already in marionette-macros/Cargo.toml) | 0.23.0 | — |
| `syn` crate | Plan 02 (macro AST) | ✓ (already in marionette-macros/Cargo.toml) | 2.x with "full" features | — |
| `nm` (binutils) | Plan 03 (symbol-table assertion) | ✓ (standard on ubuntu-latest, macOS) | GNU binutils 2.x | `llvm-nm` (installed via rustup component) |
| `tracing` crate | Plan 01 (release-mode collision warn) | ✓ (already in marionette/Cargo.toml) | 0.1 | — |

**Missing dependencies with no fallback:** None.
**Missing dependencies with fallback:** None.

## Security Domain

Phase 16 is internal framework plumbing with no auth, no I/O, no serialization boundary, no
user input, no network surface. Most ASVS categories do not apply.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A — no auth surface |
| V3 Session Management | no | N/A — no sessions |
| V4 Access Control | no | N/A — no access-controlled resources |
| V5 Input Validation | partially | Macro attribute parsing via `darling` — library-grade input validation, not user-trust-boundary |
| V6 Cryptography | no | N/A — no crypto |
| V7 Error Handling & Logging | yes | `tracing::warn!` on duplicate-key collisions; `syn::Error::new_spanned` for macro misuse — neither reveals sensitive data because neither handles user data |
| V10 Malicious Code | no | N/A — proc-macros execute at compile time only, against developer-trusted source |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Macro injects unsanitized tokens that compile into unintended code | Tampering | darling + syn perform structural AST parsing, not string interpolation — injected code paths can't bypass the validation |
| Proc-macro supply chain (compromised `linkme` crate) | T / E | Standard Cargo supply-chain trust; `linkme` is authored by dtolnay (same author as `syn`, `quote`, `serde`) — de facto trusted in the Rust ecosystem |
| Panics in macro expansion leak build-time internal paths | Information disclosure | `syn::Error` spans are the std diagnostic path — no sensitive data in play |

No further security controls needed for this phase. Phase 17+ that exposes the gallery HTTP
surface will inherit the Phase 6 auth posture (no-auth in gallery per PROJECT.md § gallery
scope).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | wasm32 platform support is not needed for Phase 16 — all new crates build to native only | §1 | Low — gallery-smoke and gallery-demo (Phase 17) run on dev machines and CI, not in browsers |
| A2 | Feature-namespace scoping of `cfg(feature = "gallery")` in consumer crates (vs. marionette's namespace) is a direct consequence of Cargo's per-crate feature resolution | §1, §6 | Medium — if wrong, the recommended feature-propagation pattern fails. Mitigation: Plan 01 includes a smoke build of `gallery-smoke` with default features to confirm behavior before locking the pattern in Phase 17. (The /tmp experiment already confirms it.) |
| A3 | rust-analyzer / VS Code (the dominant IDEs among marionette contributors) handle the `[..]` linkme initializer syntax without issue; only JetBrains has the documented limitation | §1 | Low — no known JetBrains users on the project |
| A4 | `nm --demangle` is present on `ubuntu-latest` CI images and on dev macOS machines | §4 | Low — binutils/llvm-nm are standard toolchain components |
| A5 | CI's `dtolnay/rust-toolchain@stable` does not roll fast enough that trybuild `.stderr` files regress between merges | §5 | Medium — if a new stable changes diagnostic wording, CI breaks on a PR unrelated to macros. Mitigation: documented `TRYBUILD=overwrite` recovery procedure in `tests/ui/README.md` |
| A6 | Phase 17's DEMO-01 convention ("every built-in has a `gallery_demo()`") maps 1:1 to explicit `key = "..."` overrides (no ambiguity in the mapping) | CONTEXT.md D-C1 | Low — mechanical sweep during Phase 17 |

## Open Questions for Planner (RESOLVED)

These questions were resolved by the planner during Phase 16 plan creation (commit `3a4bdbe`).
Resolutions are recorded inline so downstream agents see the final decisions without re-reading
the plans.

1. **Plan granularity — 4 plans (Areas A/B/C/D) vs. 3 plans (A+B combined + C + D)?** CONTEXT.md
   §specifics suggests "one plan per area" is reasonable but ultimately the planner's call.
   This research recommends 4 plans with Plan 01 (A+B combined because both touch
   `marionette/`) + Plan 02 (C) + Plan 03 (D) + Plan 04 (docs/closure). A 3-plan variant
   dropping Plan 04 into Plan 03 is acceptable.
   **RESOLVED:** 4 plans — 16-01 (gallery module + feature), 16-02 (gallery_demo macro),
   16-03 (gallery-smoke + symbol test), 16-04 (docs closure). Wave 1: 01+02 parallel;
   Wave 2: 03; Wave 3: 04. See `16-01-PLAN.md` through `16-04-PLAN.md`.

2. **Re-export `linkme` from `marionette` vs. have consumers declare it?** §5 recommends
   re-export for API hygiene. The planner decides whether Plan 01 adds the re-export, or
   Plan 03 adds `linkme` to `gallery-smoke`'s Cargo.toml directly. Either works; re-export is
   cleaner but requires a public path decision (`marionette::gallery::__linkme` vs
   `marionette::__private::linkme`).
   **RESOLVED:** Re-export `linkme` from `marionette::gallery::__linkme` (the module's
   hidden-but-accessible API surface). Macro-emitted paths reference `::marionette::gallery::__linkme`
   so consumers do not need to declare `linkme` themselves. Locked in 16-01-PLAN.md.

3. **Whether `Node` is re-exported from `marionette::gallery` or stays only at
   `marionette::builders::Node`.** §3 recommends re-exporting so `use marionette::gallery::
   {Node, DemoEntry, registered_demos}` composes cleanly for Phase 17. Either direction
   works; the alternative is `use marionette::builders::Node; use marionette::gallery::
   {DemoEntry, registered_demos};`.
   **RESOLVED:** Re-export `Node` from `marionette::gallery`. Phase 17 composite demos
   import everything from one path: `use marionette::gallery::{Node, DemoEntry, registered_demos};`.
   Locked in 16-01-PLAN.md.

4. **Parallel-test target-dir isolation in the FRAME-03 symbol-table test.** §4 describes the
   cargo-cache-thrash risk when the symbol test and the gallery-smoke test run in parallel.
   Recommended mitigation is per-test `--target-dir`. The planner picks: (a) implement the
   target-dir isolation; (b) add `#[ignore]` and run only under a specific `cargo test
   --ignored` pass; (c) collapse both FRAME-03 sub-tests into a single `#[test]` that runs
   both builds sequentially.
   **RESOLVED:** Option (a) — per-test `--target-dir`. The FRAME-03 symbol test in
   `backend/crates/marionette/tests/no_gallery_symbols.rs` spawns two subprocess `cargo build`
   invocations, each with a dedicated `--target-dir` under `target/no-gallery-symbols-test-<profile>/`.
   No `#[ignore]` required; the test runs under the normal `cargo test --workspace` pass.
   Locked in 16-03-PLAN.md Task 4.

5. **Trybuild toolchain pinning — pin via `rust-toolchain.toml` or commit current-stable
   `.stderr` and document `TRYBUILD=overwrite`?** §5 recommends the latter. If the project
   later decides to pin, a single-line `rust-toolchain.toml` at repo root does it.
   **RESOLVED:** No toolchain pin. Commit current-stable `.stderr` snapshots alongside the
   trybuild `.rs` fixtures. `backend/crates/gallery-smoke/tests/ui/README.md` documents the
   `TRYBUILD=overwrite cargo test -p gallery-smoke --test ui_errors` recovery procedure for
   post-rustc-upgrade diagnostic-wording churn. Locked in 16-03-PLAN.md Task 3.

6. **Sort/dedup implementation tested as global-state integration vs. pure-function unit?**
   §7 boundary conditions notes that testing the collision-check against global linker state
   is fragile; refactoring `build_sorted` to a pure function `fn sort_entries(entries: &[&'static
   DemoEntry]) -> Vec<&'static DemoEntry>` enables direct unit tests. Planner decides:
   refactor for testability (recommended) vs. accept global-state test difficulty.
   **RESOLVED:** Refactor for testability. `backend/crates/marionette/src/gallery.rs`
   exposes a pure `fn sort_entries(entries: &[&'static DemoEntry]) -> Vec<&'static DemoEntry>`
   under a private scope plus `#[cfg(test)]` `pub(crate)` export. Unit tests in
   `gallery::tests` exercise empty-input, single-entry, pre-sorted, reverse-sorted, and
   duplicate-key paths without touching the global `DEMOS` slice. Locked in 16-01-PLAN.md Task 2.

## Recommended Plan Decomposition

**4 plans; sequential with optional parallel opportunities between Plan 02 and Plan 03.**

```
16-01-gallery-module-and-feature.md     Area A + Area B
        │
        ├─ 16-02-gallery-demo-proc-macro.md      Area C (depends on DemoEntry path from 01)
        │
        └─ 16-03-gallery-smoke-and-symbol-test.md  Area D (depends on 01 + 02)
              │
              └─ 16-04-phase-closure.md           PROJECT.md update, STATE.md blocker close,
                                                  Phase 17 hand-off notes, retrospective
```

### Plan 01 — `gallery` module + feature flag (Area A + Area B)
**Touch:** `backend/crates/marionette/src/lib.rs`, new `backend/crates/marionette/src/gallery.rs`,
`backend/crates/marionette/Cargo.toml`, `backend/Cargo.toml` (workspace.dependencies).
**Delivers:** `DemoEntry`, `DEMOS` distributed slice (cfg-gated), `registered_demos()` with
memoized sort + collision check, `gallery` cargo feature with optional `linkme` dep, optional
`marionette::gallery::__linkme` re-export, inline unit tests for sort/dedup/idempotence.
**Size:** ~150 LoC including tests.
**Depends on:** nothing — this is the foundation.

### Plan 02 — `#[gallery_demo]` proc macro (Area C)
**Touch:** `backend/crates/marionette-macros/src/lib.rs`, new
`backend/crates/marionette-macros/src/gallery_demo.rs`.
**Delivers:** `#[gallery_demo]` attribute macro with darling-based attr parsing,
syn-based signature/visibility validation, emission of cfg-gated fn + linkme static, title-case
helper with inline unit tests.
**Size:** ~200 LoC including tests.
**Depends on:** Plan 01's public path (`marionette::gallery::DemoEntry`, `marionette::gallery::
DEMOS`) — the macro emits token references to these paths; they must exist at consumer compile
time. Compiles fine without Plan 01 in isolation (the emitted tokens only resolve when a
consumer tries to use the macro).

### Plan 03 — `gallery-smoke` crate + symbol-table test (Area D)
**Touch:** `backend/Cargo.toml` (workspace members + trybuild dep), new
`backend/crates/gallery-smoke/` (entire crate), new
`backend/crates/marionette/tests/no_gallery_symbols.rs`.
**Delivers:** gallery-smoke crate with toy demo, registry round-trip integration test, trybuild
compile-error fixtures with 4 misuse cases + `.stderr` files, symbol-table test with default-
build and gallery-feature-build assertions.
**Size:** ~250 LoC including fixtures and `.stderr` files.
**Depends on:** Plan 01 + Plan 02.

### Plan 04 — Phase closure (docs)
**Touch:** `.planning/PROJECT.md` (Key Decisions table — add linkme rationale row),
`.planning/STATE.md` (close "Registration library selection" blocker), new Phase 17 hand-off
note inside the 16-phase dir.
**Delivers:** FRAME-02's required "choice logged in Key Decisions" satisfied; Phase 17 planner
gets an explicit pointer to CONTEXT.md D-C1's "every annotation must use `key = "..."`"
implication (since Phase 17's `gallery_demo()` convention would otherwise mass-collide);
updated progress counters.
**Size:** ~30 LoC of docs.
**Depends on:** Plan 01 + Plan 02 + Plan 03.

### Alternate 3-plan variant
If the planner prefers fewer plans and wider waves: merge Plan 04 into Plan 03 (docs close
alongside the gallery-smoke crate landing). Keeps Plan 01 and 02 as separate atomic units.

## Code Examples

### Macro expansion example (what the user writes vs. what the macro emits)

User source:
```rust
// Some consumer crate, e.g. gallery-smoke/src/lib.rs
use marionette::gallery::Node;
use marionette::builders::standard::Text;
use marionette_macros::gallery_demo;

#[gallery_demo(key = "smoke", name = "Smoke Check")]
pub fn smoke() -> Node {
    Text::new("smoke").build()
}
```

Macro-expanded output (conceptual):
```rust
#[cfg(feature = "gallery")]
pub fn smoke() -> Node {
    Text::new("smoke").build()
}

#[cfg(feature = "gallery")]
#[::linkme::distributed_slice(::marionette::gallery::DEMOS)]
static __GALLERY_DEMO_smoke: ::marionette::gallery::DemoEntry =
    ::marionette::gallery::DemoEntry {
        key: "smoke",
        render: smoke,
        display_name: "Smoke Check",
    };
```

### Registry iteration consumer example (what Phase 17 will write)

```rust
use marionette::gallery::registered_demos;

fn build_gallery_nav() -> Vec<NavEntry> {
    registered_demos()
        .map(|demo| NavEntry {
            label: demo.display_name.to_string(),
            route: format!("/demo/{}", demo.key),
            render_fn: demo.render,
        })
        .collect()
}
```

### PROJECT.md Key Decisions entry (Plan 04 writes this)

```markdown
| linkme over inventory for gallery-demo registry | Type-safe distributed_slice attribute
binds element type to static slice declaration; zero runtime cost (no ctor-style global
initializers); explicit mental model consistent with marionette's clean-framework posture.
Stable-order promise is owned by marionette (sort at iteration time), not delegated to
linkme. | → v1.2 Phase 16 |
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-maintained `static REGISTRY: &[(Key, Fn)] = &[...]` with manual entries | Auto-discovery via `#[gallery_demo]` + distributed slice | Phase 16 landing | Adding a new built-in component no longer requires touching a central registry file |
| `inventory` crate (historical default for cross-crate registry) | `linkme` crate (zero-cost, type-safe) | CONTEXT.md D-A1 decision | No ctor overhead, compile-time type binding |

**Deprecated/outdated in CONTEXT.md:**
- `inventory` as fallback — EXPLICITLY REJECTED per D-A1 + pre-deployment posture.

## Sources

### Primary (HIGH confidence)

- **Context7 `/websites/rs_linkme_0_3_35`** — `distributed_slice` usage, cross-crate
  registration semantics, IntoIterator, `static_slice()` internals, Windows/UEFI black_box
  caveat.
- **`cargo info linkme`** — version 0.3.36, MSRV 1.71, license, repo URL.
- **`cargo info darling`** — version 0.23.0, MSRV 1.88.0.
- **`cargo info trybuild`** — version 1.0.116, MSRV 1.76.
- **Empirical test `/tmp/linkme-feature-test`** — feature-gate interaction across
  crates, namespace-scoped `cfg(feature = "gallery")`, rlib symbol inspection.
- **Empirical test `/tmp/dup-test`** — linkme does not detect semantic duplicates.
- **In-repo source (HIGH confidence)**:
  - `backend/crates/marionette-macros/src/lib.rs` — proc-macro crate setup.
  - `backend/crates/marionette-macros/src/action.rs` — attribute-macro template.
  - `backend/crates/marionette-macros/src/component_builder.rs` — darling template.
  - `backend/crates/marionette-macros/src/requires.rs` — multi-shape attribute parsing.
  - `backend/crates/marionette/src/builders/node.rs` — `Node` type-alias definition.
  - `backend/crates/marionette/tests/macro_tests.rs` — integration-test template.
  - `.planning/codebase/CONVENTIONS.md` — rustc/clippy/fmt conventions.
  - `.planning/codebase/TESTING.md` — test-framework conventions.

### Secondary (MEDIUM confidence)

- **`docs.rs/linkme/0.3.36/linkme/` (WebFetch)** — platform support matrix, MSRV (not
  documented on page; fetched via `cargo info`).
- **`github.com/dtolnay/linkme` (WebFetch)** — general README content; some questions returned
  unanswered by the page (duplicate detection, conditional compilation). Cross-verified by
  empirical testing.

### Tertiary (LOW confidence)

- Assumption A1 (wasm32 not needed) — no authoritative source consulted; based on phase
  boundary.
- Assumption A5 (trybuild CI stability) — probabilistic, based on historical observation
  that stable-Rust diagnostic wording is relatively stable.

## Metadata

**Confidence breakdown:**
- Standard stack (linkme, darling, syn, trybuild versions + usage): **HIGH** — all versions
  verified via `cargo info`, all usage patterns verified empirically or via Context7.
- Feature-gate interaction + cfg-namespace subtlety: **HIGH** — verified by executing a
  minimal workspace reproduction.
- Architecture (module layout, public API, sort/dedup memoization): **HIGH** — grounded in
  existing in-repo patterns (`component_builder.rs`, `action.rs`, `marionette/src/lib.rs`).
- Symbol-test (nm on rlib, target-dir robustness): **HIGH** — verified empirically against
  the same linkme setup.
- Trybuild fixtures (stderr stability, toolchain pinning): **MEDIUM** — standard trybuild
  practice; `.stderr` churn is real but tractable.
- Cross-crate distributed slice (gallery-smoke → marionette::gallery::DEMOS): **HIGH** —
  verified empirically.

**Research date:** 2026-04-21
**Valid until:** 2026-05-21 (30 days — the stack is stable and well-understood; `linkme`
has not had a breaking release since 2020).
