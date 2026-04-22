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
    let a = ErrorDisplay::new("Example error message").build();
    let b = ErrorDisplay::new("Another error with different content").build();

    crate::builders::container::Container::new()
        .id("demo-error-display-root")
        .children(vec![a, b])
        .build_with_children()
}
