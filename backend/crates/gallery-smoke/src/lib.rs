//! Permanent test-fixture crate for the Phase 16 gallery-demo framework.
//!
//! Exercises the end-to-end pipeline:
//! - The `#[gallery_demo]` proc macro in `marionette-macros`,
//! - The `linkme`-backed `DEMOS` distributed slice in `marionette::gallery`,
//! - Cross-crate submission (this crate submits, `marionette` aggregates),
//! - The `#[gallery_demo(key = "...", name = "...")]` attribute parsing.
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
/// `tests/registry_roundtrip.rs` asserts this key + `display_name` appear
/// when `registered_demos()` is invoked.
#[gallery_demo(key = "smoke", name = "Smoke Check")]
#[must_use]
pub fn smoke() -> Vec<Node> {
    vec![Text::new("gallery-smoke").build()]
}
