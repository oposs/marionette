//! FRAME-04 verification: the `#[gallery_demo]` macro + linkme slice +
//! cross-crate submission all wire up end-to-end.
//!
//! Phase 16 Success Criterion #4: "A smoke test in the workspace registers a
//! toy demo fn via `#[gallery_demo]`, enables the `gallery` feature, iterates
//! the registry, and asserts the toy key is present in stable order."

use marionette::gallery::registered_demos;

// Force the linker to pull in `gallery_smoke` (otherwise the test binary
// would never reference any of its symbols and dead-code elimination would
// drop the `#[gallery_demo] fn smoke` registration, leaving DEMOS empty).
// `use gallery_smoke::smoke` alone is not enough under some toolchain
// configurations; a live reference in a test fn is the safe belt.
use gallery_smoke::smoke;

#[test]
fn force_link_smoke_demo() {
    // Reference the fn so the linker keeps gallery_smoke's object file,
    // which in turn drags in the `__GALLERY_DEMO_smoke` static that
    // linkme uses to populate `DEMOS`.
    let _render: fn() -> marionette::gallery::Node = smoke;
}

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
    assert_eq!(
        keys, sorted,
        "registered_demos() must yield keys in sorted order"
    );
}

#[test]
fn registry_iteration_is_idempotent() {
    let first: Vec<&'static str> = registered_demos().map(|e| e.key).collect();
    let second: Vec<&'static str> = registered_demos().map(|e| e.key).collect();
    assert_eq!(first, second, "memoized iteration must be deterministic");
}
