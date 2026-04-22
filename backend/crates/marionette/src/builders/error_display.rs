//! `ErrorDisplay` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "error-display")]
pub struct ErrorDisplay {
    pub message: String,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "error-display")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    // ErrorDisplay.svelte reads `errors` ONLY from `bind` (see
    // frontend/src/lib/components/feedback/ErrorDisplay.svelte:26-41).
    // The Rust `message` field on ErrorDisplay (line ~10) is UNUSED by
    // the frontend — Phase 18 polish should either remove it or wire it
    // as a bind-fallback. Flagged in SUMMARY (W-06).
    //
    // The positional `ErrorDisplay::new(message)` arg is required by the
    // builder signature; pass a short label that identifies the demo
    // instance. The visible errors come from the seed_for_key arm in
    // gallery-demo/src/handlers/show.rs.
    let a = ErrorDisplay::new("errors-a")
        .id("demo-error-display-a")
        .bind("/demo/error-display/errors-a")
        .build();
    let b = ErrorDisplay::new("errors-b")
        .id("demo-error-display-b")
        .bind("/demo/error-display/errors-b")
        .build();

    crate::builders::container::Container::new()
        .id("demo-error-display-root")
        .children(vec![a, b])
        .build_with_children()
}
