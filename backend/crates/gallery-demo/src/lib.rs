//! Gallery-demo library crate — exposes handlers/home/state modules so that
//! integration tests in `tests/*.rs` can `use gallery_demo::handlers::...`.
//!
//! Per Plan 17-03 Task 3 Step 1, the gallery-demo crate is split into a library
//! (`src/lib.rs`) + binary (`src/main.rs`) pair. The binary owns the full Axum
//! and ActionRouter boot code (so acceptance-criterion greps hit `main.rs`);
//! the library owns the shared modules so tests can import them.
//!
//! `main.rs` declares its own `mod handlers;` etc. referring to the same source
//! files — Rust resolves each via the binary's own module tree, independent of
//! the library's. The library's `pub mod` declarations make the modules
//! reachable under the `gallery_demo::` crate path for test code.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Handlers return `ActionResult` (which is Result<Vec<ProtocolMessage>, ActionError>).
// Every handler's only error path is "propagate ActionError from a ? or early
// return"; per-handler `# Errors` docs would be boilerplate. Allow crate-wide.
#![allow(clippy::missing_errors_doc)]
// Doc-comment references to handler fn names and type names are plentiful;
// enforcing backticks on every one turns pedantic into noise without improving
// clarity. This is opt-out for the gallery-demo prose layer only.
#![allow(clippy::doc_markdown)]

pub mod fixtures;
pub mod handlers;
pub mod home;
pub mod state;

/// Force-link gallery-smoke's `smoke` demo so the linkme `DEMOS` slice is
/// populated at runtime (and in integration tests).
///
/// Without an explicit reference, the linker dead-strips `gallery_smoke`'s
/// object file because nothing in gallery-demo calls it directly — the
/// `#[gallery_demo]`-emitted `static` registration is link-time side-effect
/// only. The same belt-and-suspenders pattern lives in
/// `gallery-smoke/tests/registry_roundtrip.rs::force_link_smoke_demo`.
///
/// Plan 17-04 will add 19 more `#[gallery_demo]` fns inside the `marionette`
/// crate — gallery-demo already depends on `marionette`, so those don't need
/// a force-link here. This one is specific to gallery-smoke, the external
/// fixture crate.
pub fn ensure_demos_linked() {
    // `black_box` takes the fn pointer and makes the compiler treat it as a
    // live observation that cannot be optimized away, which in turn forces
    // the linker to keep `gallery_smoke`'s object file and its
    // `#[gallery_demo]`-emitted static registration.
    let smoke_ref: fn() -> Vec<marionette::gallery::Node> = gallery_smoke::smoke;
    std::hint::black_box(smoke_ref);
}
