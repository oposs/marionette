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

pub mod handlers;
pub mod home;
pub mod state;
