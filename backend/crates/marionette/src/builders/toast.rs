//! `Toast` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "toast")]
pub struct Toast {
    pub message: String,
    #[builder(optional)]
    pub variant: Option<String>,
    #[builder(optional)]
    pub duration: Option<u32>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01 composite) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "toast")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    use marionette_protocol::ComponentAction;

    // D-D4: Fire-toast Button + a Heading label so the content surface
    // has something visible even when no toasts are dispatched.
    let label = crate::builders::heading::Heading::new("Example toast demo")
        .id("demo-toast-label")
        .build();
    let fire = crate::builders::button::Button::new("Fire toast")
        .id("demo-toast-fire")
        .action(ComponentAction::click("gallery-demo/toast-fire"))
        .build();

    crate::builders::container::Container::new()
        .id("demo-toast-root")
        .children(vec![label, fire])
        .build_with_children()
}
