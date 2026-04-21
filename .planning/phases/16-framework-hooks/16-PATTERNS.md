# Phase 16: Framework Hooks — Pattern Map

**Mapped:** 2026-04-21
**Files analyzed:** 10 (4 NEW + 4 MODIFIED + 1 new-crate cluster counted as a group)
**Analogs found:** 9 / 10 (one "no in-repo analog" case: trybuild fixtures)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| **NEW** `backend/crates/marionette-macros/src/gallery_demo.rs` | proc-macro (attribute) | transform (TokenStream → TokenStream) | `marionette-macros/src/action.rs` (attribute macro) + `marionette-macros/src/component_builder.rs` (darling attr parsing) | exact (role + flow) |
| **MOD** `backend/crates/marionette-macros/src/lib.rs` | proc-macro crate root | module registration + re-export | `marionette-macros/src/lib.rs:5-7` + `lib.rs:45-48` (existing `action` export) | exact — amend in place |
| **NEW** `backend/crates/marionette/src/gallery.rs` | public-API submodule (types + registry + iterator fn) | in-memory lookup (memoized) | `marionette/src/builders/node.rs` (tiny submodule defining public `Node` alias + helper) | role-match (tiny public module) |
| **MOD** `backend/crates/marionette/src/lib.rs` | crate root (module list + re-exports) | module registration | `marionette/src/lib.rs:4-13` itself (existing `pub mod` block, alphabetical) | exact — amend in place |
| **MOD** `backend/crates/marionette/Cargo.toml` | manifest (add feature + optional dep) | cargo feature declaration | `marionette/Cargo.toml` (no existing optional-dep feature in repo — first of its kind) | partial (idiom-only) |
| **NEW** `backend/crates/marionette/tests/no_gallery_symbols.rs` | integration test (subprocess + artifact inspection) | file-I/O + process spawn | `marionette/tests/macro_tests.rs` (integration test file layout) | role-match; data-flow is novel (subprocess + nm) |
| **MOD** `backend/Cargo.toml` | workspace manifest | add member + workspace.dependencies entries | `backend/Cargo.toml:3-8` (members list) + `:15-37` (workspace.dependencies) | exact — amend in place |
| **NEW** `backend/crates/gallery-smoke/Cargo.toml` | workspace-member manifest (minimal) | manifest declaration | `marionette-protocol/Cargo.toml` (3-field minimal manifest) | exact (minimalism) |
| **NEW** `backend/crates/gallery-smoke/src/lib.rs` | library crate entry (registers one demo) | compile-time registration | `marionette/src/builders/node.rs` (tiny pub module) + `marionette/tests/macro_tests.rs` (demonstrates macro at crate boundary) | role-match |
| **NEW** `backend/crates/gallery-smoke/tests/registry_roundtrip.rs` | integration test (`#[test]` fns) | assertion-based | `marionette/tests/macro_tests.rs` (integration-style macro test) | exact |
| **NEW** `backend/crates/gallery-smoke/tests/ui/*.rs` + `.stderr` | trybuild compile-fail fixtures | compile-error snapshot | **NO IN-REPO ANALOG** — trybuild is first use in the repo; follow external trybuild idiom per RESEARCH §5 | none (use external pattern) |
| **NEW** `backend/crates/gallery-smoke/tests/ui_errors.rs` | trybuild driver (`#[test]`) | test-harness invocation | `marionette/tests/macro_tests.rs` (skeletal test file) | role-match |

## Pattern Assignments

### `backend/crates/marionette-macros/src/gallery_demo.rs` (proc-macro attribute)

**Primary analog:** `marionette-macros/src/action.rs` (attribute macro, `ItemFn` validation, `quote!` emission)
**Secondary analog:** `marionette-macros/src/component_builder.rs` (darling-based attr parsing via `FromDeriveInput` — adapt to `FromMeta` + `NestedMeta::parse_meta_list` for attribute form per RESEARCH §2)

**File-header pattern** (mirror `action.rs:1-3` — imports ordering: `proc-macro2`, `quote`, `syn`):

```rust
// action.rs:1-3 — COPY shape for imports
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Ident};
```

**Attribute macro entry-point shape** (`action.rs:10-14` — parse `ItemFn`, early-return `to_compile_error()` on failure):

```rust
// action.rs:10-14
pub fn action_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemFn = match syn::parse2(item) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };
    ...
}
```

Apply this shape to `gallery_demo_impl(attr, item)`, but catch the parse failure explicitly so the "applied to struct" case can produce a targeted error (RESEARCH §2 "Applied-to-struct case").

**Darling attribute-arg parsing** (`component_builder.rs:1-13` uses `FromDeriveInput`; gallery_demo uses `FromMeta` — same library, attribute form):

```rust
// component_builder.rs:1-13 — REFERENCE for darling usage style
use darling::{FromDeriveInput, FromField};
...
#[derive(FromDeriveInput)]
#[darling(attributes(component))]
struct ComponentOpts {
    ident: Ident,
    data: darling::ast::Data<(), FieldOpts>,
    #[darling(rename = "type")]
    component_type: String,
}
```

```rust
// component_builder.rs:26-29 — COPY error-emission pattern
let opts = match ComponentOpts::from_derive_input(input) {
    Ok(v) => v,
    Err(e) => return e.write_errors(),
};
```

Gallery-demo translation (per RESEARCH §2 "Attribute parsing with darling"):

```rust
use darling::FromMeta;
use darling::ast::NestedMeta;

#[derive(FromMeta, Default)]
struct GalleryDemoOpts {
    #[darling(default)] key: Option<String>,
    #[darling(default)] name: Option<String>,
}
```

**Signature/visibility validation pattern** (new — RESEARCH §2 "Signature/visibility validation with syn"). No in-repo analog validates `Visibility`/`inputs.is_empty()`/`asyncness`/`generics`/`ReturnType`; follow RESEARCH §2's block. Emit errors via `syn::Error::new_spanned(target, msg)` (same shape as `requires.rs:51-54`):

```rust
// requires.rs:51-54 — COPY error-construction idiom
return Err(syn::Error::new_spanned(
    ident,
    "expected `authenticated` or `role = \"...\"`",
));
```

**`quote!` emission + static ident synthesis** (`action.rs:22-30` — `Ident::new` + `quote!` block mixing original fn with generated const):

```rust
// action.rs:22-30 — COPY for static ident + quote block
let const_ident = Ident::new(&const_name_str, proc_macro2::Span::call_site());
quote! {
    /// Action name constant generated by `#[action]`.
    pub const #const_ident: &str = #name_value;
    #input
}
```

Gallery-demo adaptation: emit two `#[cfg(feature = "gallery")]`-gated items (the fn itself + the linkme static). Static ident follows `__GALLERY_DEMO_{fn_ident}` per RESEARCH §2 "Unique static ident strategy". Use `::linkme::distributed_slice(::marionette::gallery::DEMOS)` absolute path (RESEARCH §5 recommends `linkme` re-export from `marionette::gallery::__linkme` — planner's call, but prefer the re-export form).

**Doc-comment convention** (`action.rs:5-9` is the template — `///` on the impl fn with a short one-liner + example):

```rust
// action.rs:5-9
/// Parse `#[action(name = "save-contact")]` and generate a constant.
///
/// Given `#[action(name = "save-contact")] fn save_contact(...)`,
/// generates `pub const SAVE_CONTACT: &str = "save-contact";`
/// alongside the original function.
```

---

### `backend/crates/marionette-macros/src/lib.rs` (crate root — amend)

**Analog:** itself (`lib.rs:5-7` + `:45-48`).

**Module declaration block** (`lib.rs:5-7` — alphabetical-ish, one `mod` per line):

```rust
// lib.rs:5-7 — EXTEND with `mod gallery_demo;` (alphabetical insertion after `mod component_builder;`)
mod action;
mod component_builder;
mod requires;
```

**Proc-macro-attribute export pattern** (`lib.rs:36-48` — `///` doc with `# Example`, then `#[proc_macro_attribute] pub fn ... { action::action_impl(attr.into(), item.into()).into() }`):

```rust
// lib.rs:36-48 — COPY shape verbatim (swap names)
/// Generate an action name constant from a handler function.
///
/// # Example
///
/// ```ignore
/// #[action(name = "save-contact")]
/// async fn save_contact() { }
/// // Generates: pub const SAVE_CONTACT: &str = "save-contact";
/// ```
#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    action::action_impl(attr.into(), item.into()).into()
}
```

Gallery-demo translation: `#[proc_macro_attribute] pub fn gallery_demo(attr, item) -> TokenStream { gallery_demo::gallery_demo_impl(attr.into(), item.into()).into() }` with a `# Example` doc block showing `#[gallery_demo(key = "button", name = "Button")] pub fn gallery_demo() -> Node { ... }`.

---

### `backend/crates/marionette/src/gallery.rs` (public API submodule)

**Closest analog:** `marionette/src/builders/node.rs` (tiny pub submodule defining one alias + one helper). The gallery.rs module is larger (~80 LoC) but shares the "small, focused, public types + helpers" shape.

**Module-header doc + imports** (`builders/node.rs:1-3` — `//!` module doc, then import):

```rust
// builders/node.rs:1-3 — COPY shape
//! Node type alias and helper functions for component builders.

use marionette_protocol::Component;
```

Gallery.rs adaptation: `//! Gallery demo registry — `DemoEntry`, the `linkme`-backed `DEMOS` distributed slice, and `registered_demos()` iteration API.` Then imports per RESEARCH §3 ordering (`std::` first, then external `linkme`, then internal).

**Public type + `#[must_use]` helper fn** (`builders/node.rs:5-14` — two public items with `///` docs):

```rust
// builders/node.rs:5-14 — COPY for DemoEntry / registered_demos shape
/// A node is a `(node_id, Component)` pair for insertion into a surface's nodes map.
pub type Node = (String, Component);

/// Generate a unique node ID with the given prefix.
///
/// Format: `"{prefix}-{uuid_v4}"`.
#[must_use]
pub fn node_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}
```

Gallery.rs translation (RESEARCH §3):
- `pub struct DemoEntry { key: &'static str, render: fn() -> Node, display_name: &'static str }` with `#[derive(Debug)]` and `///` doc per field.
- `#[cfg(feature = "gallery")] #[linkme::distributed_slice] pub static DEMOS: [DemoEntry] = [..];` (the `[..]` is a linkme syntactic requirement — RESEARCH §1).
- `pub use crate::builders::Node;` to reuse the existing alias.
- `static SORTED_CACHE: OnceLock<Vec<&'static DemoEntry>> = OnceLock::new();` with `cfg`-split `build_sorted()` (real impl under `gallery`, empty `Vec::new()` under default).
- `#[must_use] pub fn registered_demos() -> impl Iterator<Item = &'static DemoEntry>` delegating to `SORTED_CACHE.get_or_init(build_sorted).iter().copied()`.

**Duplicate-key detection + tracing::warn pattern** — no in-repo analog for `OnceLock`-memoized collision scan. Follow RESEARCH §3's `build_sorted()` block verbatim; uses `tracing::warn!` (already workspace dep in `marionette/Cargo.toml:15`) and `debug_assert!`.

**Inline unit tests** (follow `builders/*.rs` convention — none have inline tests today; use the RESEARCH §7 Wave 0 Gaps list: sort-order, duplicate-key panic/warn, empty-under-default, memoization-idempotence. Use `#[cfg(test)] mod tests { ... }` at the bottom of the file per RESEARCH §8 "Test organization").

---

### `backend/crates/marionette/src/lib.rs` (crate root — amend)

**Analog:** itself (`lib.rs:4-13`).

**`pub mod` block** (alphabetical):

```rust
// lib.rs:4-13 — INSERT `pub mod gallery;` alphabetically between `pub mod extractors;` and `pub mod migration;`
pub mod auth;
pub mod builders;
pub mod db;
pub mod error;
pub mod extractors;
pub mod migration;
pub mod router;
pub mod session;
pub mod validation;
pub mod ws;
```

Per RESEARCH §3: no flattening `pub use` — consumers write `use marionette::gallery::{DemoEntry, registered_demos};`.

**NOTE on the existing re-export block** (`lib.rs:15-22`): do NOT add `pub use gallery::*;`. The `lib.rs:18` line (`pub use marionette_macros::*;`) already re-exports the `gallery_demo` proc macro transparently — consumers can write `use marionette::gallery_demo` or `use marionette_macros::gallery_demo` interchangeably. No `lib.rs` re-export change needed for the macro.

---

### `backend/crates/marionette/Cargo.toml` (amend — add feature + optional dep)

**Analog (partial):** no existing optional-dep feature in the repo — first of its kind. Anchor on idiom + RESEARCH §1 "Feature-gate interaction" and RESEARCH §6 "Edits to marionette/Cargo.toml".

**Existing manifest shape to preserve** (`marionette/Cargo.toml:1-26`):

```toml
# marionette/Cargo.toml:1-26 — existing; ADD below [dependencies] + new [features]
[package]
name = "marionette"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
marionette-protocol = { path = "../marionette-protocol" }
marionette-macros = { path = "../marionette-macros" }
serde.workspace = true
# ... (existing workspace deps unchanged) ...

[dev-dependencies]
sea-orm = { workspace = true, features = ["mock"] }
tokio-tungstenite.workspace = true
```

**Additions per RESEARCH §6:**

```toml
[dependencies]
# ... existing ...
linkme = { workspace = true, optional = true }   # NEW

[features]
gallery = ["dep:linkme"]                         # NEW — `dep:linkme` suppresses implicit feature
```

The `dep:linkme` prefix is the post-1.60 idiom (RESEARCH §6) that prevents an implicit `linkme` feature name from being created.

---

### `backend/Cargo.toml` (workspace manifest — amend)

**Analog:** itself (`backend/Cargo.toml:3-8` members list + `:15-37` workspace.dependencies).

**Members block** (`backend/Cargo.toml:3-8` — preserve ordering):

```toml
# backend/Cargo.toml:3-8 — APPEND `crates/gallery-smoke`
[workspace]
resolver = "3"
members = [
    "crates/marionette-protocol",
    "crates/marionette-macros",
    "crates/marionette",
    "crates/crm-demo",
    "crates/gallery-smoke",   # NEW
]
```

**workspace.dependencies block** (`backend/Cargo.toml:15-37` — entries are alphabetically loose; append two):

```toml
# backend/Cargo.toml:15-37 — ADD two entries following existing workspace.dependencies idiom
[workspace.dependencies]
# ... existing: serde, serde_json, tokio, axum, ... ...
darling = "0.23"        # existing (reference for style)
# ...
linkme = "0.3"          # NEW — 0.3.x stable API since 2020 per RESEARCH §1
trybuild = "1"          # NEW — dev-dep for gallery-smoke tests/ui/ fixtures per RESEARCH §5
```

Pin to major-minor per RESEARCH §1 "Version pin recommendation" (e.g. `"0.3"` lets Cargo.lock pick the latest 0.3.x; `"0.3.36"` pins exactly — planner's call, prefer `"0.3"`).

---

### `backend/crates/gallery-smoke/Cargo.toml` (NEW — minimal workspace member)

**Analog:** `marionette-protocol/Cargo.toml` (minimalist 3-dep manifest). gallery-smoke is small like marionette-protocol, not large like crm-demo.

**Shape to model** (`marionette-protocol/Cargo.toml:1-11`):

```toml
# marionette-protocol/Cargo.toml — COPY the minimalism
[package]
name = "marionette-protocol"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
```

**gallery-smoke adaptation** (per RESEARCH §5 "Cargo.toml"):

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
gallery = ["marionette/gallery"]    # Propagates so cfg(feature="gallery") is true in this crate's context

[dev-dependencies]
trybuild = { workspace = true }
```

The feature propagation stanza is load-bearing per RESEARCH §1 "Feature-gate interaction": without the local `gallery` feature re-declared here, `cfg(feature = "gallery")` in the macro-emitted `static` is always false inside gallery-smoke and the registration is pruned.

---

### `backend/crates/gallery-smoke/src/lib.rs` (NEW — registers one toy demo)

**Analogs:**
- `marionette/src/builders/node.rs` (tiny public-module shape + header doc).
- `marionette/tests/macro_tests.rs` (demonstrates the `#[action]` / `#[requires]` macros crossing crate boundaries — closest to what `#[gallery_demo]` does here).

**Crate-root doc header + pedantic allow** (mirror `marionette-macros/src/lib.rs:1-3`):

```rust
// marionette-macros/src/lib.rs:1-3 — COPY for crate-level lint attrs
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
```

**Macro usage across crate boundary** (`macro_tests.rs:3-7` — the template for "this crate imports macro + proto types + uses them"):

```rust
// macro_tests.rs:1-8 — COPY shape for imports + one annotated fn
//! Integration tests for action and requires attribute macros.

use marionette_macros::{action, requires};
use marionette_protocol::AuthRequirement;

#[action(name = "save-contact")]
async fn save_contact() {}
```

**gallery-smoke adaptation** (per RESEARCH §5 "src/lib.rs"):

```rust
//! Permanent test-fixture crate for the Phase 16 gallery-demo framework.
//! ... (full header per RESEARCH §5) ...

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use marionette::builders::standard::Text;
use marionette::gallery::Node;
use marionette_macros::gallery_demo;

#[gallery_demo(key = "smoke", name = "Smoke Check")]
pub fn smoke() -> Node {
    Text::new("gallery-smoke").build()
}
```

The macro-usage shape mirrors `macro_tests.rs:6` (`#[action(name = "save-contact")]` → `#[gallery_demo(key = "smoke", name = "Smoke Check")]`).

---

### `backend/crates/gallery-smoke/tests/registry_roundtrip.rs` (NEW — integration test)

**Analog:** `marionette/tests/macro_tests.rs` (integration tests that exercise macros by assertion).

**File structure** (`macro_tests.rs:1-28` — doc header, imports, `#[test]` fns with `assert_eq!`):

```rust
// macro_tests.rs:1-11 — COPY shape for doc + imports + first test
//! Integration tests for action and requires attribute macros.

use marionette_macros::{action, requires};
use marionette_protocol::AuthRequirement;

#[action(name = "save-contact")]
async fn save_contact() {}

#[test]
fn action_macro_constant() {
    assert_eq!(SAVE_CONTACT, "save-contact");
}
```

**registry_roundtrip.rs adaptation** (per RESEARCH §5 "tests/registry_roundtrip.rs"):
- `use marionette::gallery::registered_demos;`
- three `#[test]` fns: `smoke_demo_is_registered`, `registry_is_alphabetically_ordered`, `registry_iteration_is_idempotent`.
- assertions are plain `assert_eq!` / `.find(...).expect(...)` — exactly the style of `macro_tests.rs:10-12, 18-19, 26-27`.

---

### `backend/crates/gallery-smoke/tests/ui/*.rs` + `.stderr` (NEW — trybuild fixtures)

**Analog:** **NONE IN REPO.** trybuild is first use in this workspace.

**Action for planner:** follow the external trybuild idiom as described in RESEARCH §5 "tests/ui/*.rs + *.stderr". Four fixtures per D-D4:

| Fixture | Misuse | Expected error token |
|---------|--------|----------------------|
| `fail_not_pub.rs` | `#[gallery_demo] fn private_demo() -> Node` | "visibility" / "pub fn" |
| `fail_wrong_signature.rs` | `#[gallery_demo] pub fn has_args(arg: u32) -> Node` | "arguments" / "signature" |
| `fail_wrong_return.rs` | `#[gallery_demo] pub fn wrong_return() -> Vec<u32>` | "return type" / "Node" |
| `fail_applied_to_struct.rs` | `#[gallery_demo] pub struct NotAFn;` | "item kind" / "pub fn items" |

Each `.rs` fixture imports `marionette::gallery::Node` + `marionette_macros::gallery_demo` and declares a stub `fn main() {}`. The paired `.stderr` file is generated via `TRYBUILD=overwrite cargo test -p gallery-smoke --test compile_errors` and committed. Regeneration procedure documented in a `tests/ui/README.md` per RESEARCH §5.

**No copy-from-codebase pattern** — use RESEARCH §5's four code blocks as the templates verbatim.

---

### `backend/crates/gallery-smoke/tests/compile_errors.rs` (NEW — trybuild driver)

**Analog:** `marionette/tests/macro_tests.rs` (skeletal test file with `#[test]` fns).

**Shape** (one-line test body per RESEARCH §5 "tests/compile_errors.rs"):

```rust
#[test]
fn compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_*.rs");
}
```

File convention matches `macro_tests.rs` — single-purpose integration test under `tests/`.

---

### `backend/crates/marionette/tests/no_gallery_symbols.rs` (NEW — subprocess + nm)

**Analog:** `marionette/tests/macro_tests.rs` (integration-test placement).
**Data-flow novelty:** shells to `cargo build` + `nm` — first use of `std::process::Command` in a test in this repo; follow RESEARCH §4 "Recommended shape (Option A)" verbatim.

**Test-file layout pattern** (`macro_tests.rs:1` — `//!` doc header, then test fns):

```rust
// macro_tests.rs:1 — COPY shape for module doc
//! Integration tests for action and requires attribute macros.
```

no_gallery_symbols.rs adaptation:

```rust
//! FRAME-03 verification: default `cargo build -p marionette` emits zero
//! `gallery_demo` symbols in `libmarionette.rlib`; enabling `--features
//! gallery` brings them in.
```

**Body pattern** — RESEARCH §4 is the only template. Copy its `target_dir()`, `build_and_dump_symbols()`, and the two `#[test]` fns (`default_build_has_zero_gallery_symbols`, `gallery_feature_build_has_gallery_symbols`). Use per-test `--target-dir` flag to avoid cargo cache thrash (RESEARCH §4 "Mitigation").

---

## Shared Patterns

### Pedantic-clippy crate-level lint attrs

**Source:** `marionette-macros/src/lib.rs:1-3`
**Apply to:** every new Rust file that is a crate root (gallery-smoke/src/lib.rs). For non-root source files inside existing crates (e.g. gallery_demo.rs, gallery.rs, no_gallery_symbols.rs), inherit the crate-root's attrs — no per-file duplication.

```rust
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_continue)] // darling-generated code triggers this
```

The third allow is needed only in crates that use `darling` (so: `marionette-macros/src/lib.rs` already has it; `gallery-smoke/src/lib.rs` does not need it since gallery-smoke doesn't use darling).

### Doc-comment convention

**Source:** `action.rs:5-9` (item docs) + `builders/node.rs:1` (module docs)
**Apply to:** every public item across all new files.

```rust
//! Module-level doc comment (mod-header).

/// Item-level doc comment — one-line summary, then blank line, then details/example.
///
/// # Example
///
/// ```ignore
/// // short usage example
/// ```
```

`#[must_use]` on every public fn that returns a value (`builders/node.rs:11`, `component_builder.rs:91+`). Applied to `registered_demos()` per RESEARCH §3.

### Error emission in proc macros

**Source:** `action.rs:11-14`, `:16-19` (early-return `to_compile_error()`); `component_builder.rs:26-29` (darling `write_errors()`); `requires.rs:51-54` (spanned error).
**Apply to:** `gallery_demo.rs` for all three error-source forms (syn parse failure, darling attr parse failure, explicit signature/visibility validation).

```rust
// action.rs:10-14
let input: ItemFn = match syn::parse2(item) {
    Ok(v) => v,
    Err(e) => return e.to_compile_error(),
};
```

```rust
// component_builder.rs:26-29
let opts = match ComponentOpts::from_derive_input(input) {
    Ok(v) => v,
    Err(e) => return e.write_errors(),
};
```

```rust
// requires.rs:51-54
return Err(syn::Error::new_spanned(
    ident,
    "expected `authenticated` or `role = \"...\"`",
));
```

### Workspace dependency inheritance

**Source:** every crate's `Cargo.toml` uses `foo.workspace = true` or `foo = { workspace = true, <extras> }`.
**Apply to:** every new dep in gallery-smoke + marionette (`linkme`, `trybuild`). Add to `backend/Cargo.toml [workspace.dependencies]` first, then reference via `workspace = true` in the consuming crate.

```toml
# marionette/Cargo.toml:10 (reference)
tracing.workspace = true

# marionette/Cargo.toml — NEW (matches optional pattern)
linkme = { workspace = true, optional = true }

# gallery-smoke/Cargo.toml — NEW (workspace dev-dep)
[dev-dependencies]
trybuild = { workspace = true }
```

### Integration-test placement

**Source:** `marionette/tests/macro_tests.rs`, `ws_integration.rs`, `db_integration.rs`.
**Apply to:** all new `tests/*.rs` files (`no_gallery_symbols.rs`, `registry_roundtrip.rs`, `compile_errors.rs`). Integration tests live under `tests/` directly (no nested dirs); one `#[test]` fn per behavior; module-level `//!` doc header states the requirement being verified.

### Commit message scope

**Source:** recent `git log` (per RESEARCH §8 "Commit message style").
**Apply to:** every Phase 16 commit.

```
feat(16-01): add DemoEntry + DEMOS distributed slice
feat(16-02): implement #[gallery_demo] attribute macro
feat(16-03): add gallery-smoke crate + trybuild fixtures
test(16-03): wire no_gallery_symbols FRAME-03 check
docs(16): close STATE.md linkme-selection blocker
```

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `backend/crates/gallery-smoke/tests/ui/fail_*.rs` + `.stderr` pairs | trybuild compile-fail fixtures | compile-error snapshot | First use of `trybuild` in the workspace. No in-repo precedent; follow RESEARCH §5's four fixture templates verbatim. |
| Feature-gated optional-dep pattern in `marionette/Cargo.toml` | cargo feature declaration | manifest | First optional-dep feature in this repo. No in-repo precedent for `[features] name = ["dep:xxx"]` idiom; follow RESEARCH §6 "Edits to marionette/Cargo.toml" — this is standard Cargo idiom since 1.60. |
| `OnceLock`-memoized sort + collision scan in `gallery.rs::build_sorted()` | runtime memoization | in-memory cached computation | No in-repo use of `std::sync::OnceLock`. Follow RESEARCH §3 "`registered_demos()`" block verbatim — stdlib, thread-safe, zero deps. |
| Subprocess `nm`-grep symbol-table test in `no_gallery_symbols.rs` | integration test (file-I/O + process) | subprocess + artifact inspection | No in-repo use of `std::process::Command` inside a `#[test]`. Follow RESEARCH §4 "Recommended shape (Option A)" verbatim; use per-test `--target-dir` to avoid cache thrash. |

## Metadata

**Analog search scope:**
- `backend/crates/marionette-macros/src/*.rs` (all 4 files: lib.rs, action.rs, requires.rs, component_builder.rs)
- `backend/crates/marionette/src/lib.rs`, `src/builders/{mod.rs,node.rs,standard.rs}`
- `backend/crates/marionette/tests/*.rs` (3 files)
- `backend/crates/{marionette-protocol,marionette,marionette-macros,crm-demo}/Cargo.toml`
- `backend/Cargo.toml`

**Files scanned:** 14 Rust sources + 5 Cargo manifests
**Pattern extraction date:** 2026-04-21
**Ready for planning:** yes — per RESEARCH §Primary Recommendation, this maps cleanly to 4 plans (Areas A+B together; C alone; D with symbol-test; Plan 04 = docs + STATE closure).

## PATTERN MAPPING COMPLETE
