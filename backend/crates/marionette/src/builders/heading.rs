//! `Heading` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Content components --

#[derive(ComponentBuilder)]
#[component(type = "heading")]
pub struct Heading {
    pub text: String,
    #[builder(optional)]
    pub level: Option<u8>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "heading")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    let h1 = Heading::new("Heading level 1").level(1).build();
    let h2 = Heading::new("Heading level 2").level(2).build();
    let h3 = Heading::new("Heading level 3").level(3).build();

    crate::builders::container::Container::new()
        .id("demo-heading-root")
        .children(vec![h1, h2, h3])
        .build_with_children()
}
