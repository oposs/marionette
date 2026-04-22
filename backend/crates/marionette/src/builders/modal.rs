//! `Modal` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Dialog / feedback components --

#[derive(ComponentBuilder)]
#[component(type = "modal")]
pub struct Modal {
    pub title: String,
    #[builder(optional)]
    pub size: Option<String>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01 composite) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "modal")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    use marionette_protocol::ComponentAction;

    // D-A4: Modal demo renders a trigger Button + static explainer Text.
    // Clicking the Button dispatches gallery-demo/modal-open, which the
    // gallery-demo crate's handler renders into the `modal` sub-surface
    // (out-of-band from this demo's content).
    let trigger = crate::builders::button::Button::new("Open modal")
        .id("demo-modal-trigger")
        .action(ComponentAction::click("gallery-demo/modal-open"))
        .build();
    let explainer = crate::builders::text::Text::new(
        "Clicking the button opens a Modal in the popup sub-surface. \
         The X or backdrop dismisses via close-modal (frontend hardcode).",
    )
    .id("demo-modal-explainer")
    .build();

    crate::builders::container::Container::new()
        .id("demo-modal-root")
        .children(vec![trigger, explainer])
        .build_with_children()
}
