//! `ConfirmDialog` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "confirm-dialog")]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01 composite) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "confirm-dialog")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    use marionette_protocol::ComponentAction;

    // D-A4: trigger Button opens the ConfirmDialog via gallery-demo/confirm-open.
    // gallery-demo's handler renders the dialog into the `modal` sub-surface;
    // accept/reject clear it + enqueue a toast.
    let trigger = crate::builders::button::Button::new("Open confirm")
        .id("demo-confirm-trigger")
        .action(ComponentAction::click("gallery-demo/confirm-open"))
        .build();
    let explainer = crate::builders::text::Text::new(
        "Accept/Reject buttons in the dialog fire \
         gallery-demo/confirm-accept / gallery-demo/confirm-reject.",
    )
    .id("demo-confirm-explainer")
    .build();

    crate::builders::container::Container::new()
        .id("demo-confirm-dialog-root")
        .children(vec![trigger, explainer])
        .build_with_children()
}
