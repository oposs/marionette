//! `Toast` component builder removed — toasts are protocol events, not
//! persistent nodes.
//!
//! Toasts are dispatched via `type: "event"` with `name: "toast"` and a
//! structured hint (message / severity / duration / action / component).
//! The client (svelte-sonner in the reference frontend) owns the overlay
//! chrome — stacking, fade, countdown, dismissal. See CONCEPT.md
//! §"Where the Client Is Smart" for the protocol-vs-client boundary
//! this reflects.
//!
//! This file survives only to host the `gallery_demo()` sibling for the
//! `toast` nav entry — the demo itself never referenced `Toast::new(...)`;
//! it builds a trigger Button + label in a Container and dispatches
//! `gallery-demo/toast-fire`, which the gallery-demo crate's handler
//! responds to with the `toast` event that the client renders through
//! sonner.

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

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
