//! `Spinner` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "spinner")]
pub struct Spinner {
    #[builder(optional)]
    pub size: Option<String>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "spinner")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    let sm = Spinner::new().size("sm").build();
    let md = Spinner::new().size("md").build();
    let lg = Spinner::new().size("lg").build();

    crate::builders::container::Container::new()
        .id("demo-spinner-root")
        .children(vec![sm, md, lg])
        .build_with_children()
}
