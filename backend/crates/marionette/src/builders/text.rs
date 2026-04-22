//! `Text` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "text")]
pub struct Text {
    pub text: String,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "text")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    let short = Text::new("Short text block.").build();
    let paragraph = Text::new(
        "Longer paragraph text demonstrating how the Text builder renders \
         multiple sentences. This is roughly twenty words of placeholder \
         copy for visual texture.",
    )
    .build();
    let technical = Text::new("Sample /api/path reference: /demo/text/value").build();

    crate::builders::container::Container::new()
        .id("demo-text-root")
        .children(vec![short, paragraph, technical])
        .build_with_children()
}
