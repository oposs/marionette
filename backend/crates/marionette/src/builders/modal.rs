//! `Modal` primitive removed — popups are composed, not wrapped.
//!
//! Phase 17 Plan 17-05 moved `ModalSurface.svelte` to a layout-root singleton
//! mount (`frontend/src/routes/+layout.svelte`). It reads the `modal`
//! sub-surface tree and wraps whatever is there in `<Dialog.Root>` /
//! `<Dialog.Content>` automatically. Handler authors just render an SDUI
//! tree (Container, Form, Buttons, …) to the `modal` sub-surface — no
//! dedicated `Modal::new(...)` wrapper needed.
//!
//! Plan 17-08 deleted the `Modal` struct (dead code after 17-05's dispatch
//! unregistration). `ConfirmDialog` (see `confirm_dialog.rs`) remains as the
//! structured accept-cancel variant.
//!
//! This file survives only to host the `gallery_demo()` sibling for the
//! `modal` nav entry — the demo itself never referenced `Modal::new(...)`;
//! it builds a trigger Button + explainer Text in a Container and dispatches
//! `gallery-demo/modal-open`, which the gallery-demo crate's handler renders
//! into the `modal` sub-surface out-of-band from this demo's content.
//!
//! See `backend/crates/marionette/GALLERY-DEMOS.md` §Popup composition for
//! the canonical "form in popup" recipe.

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "modal")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    use marionette_protocol::ComponentAction;

    // D-A4: Modal demo renders a trigger Button + static explainer Text.
    // Clicking the Button dispatches gallery-demo/modal-open, which the
    // gallery-demo crate's handler renders into the `modal` sub-surface
    // (out-of-band from this demo's content). There is NO Modal::new(...)
    // wrapper — popups are compositional (see GALLERY-DEMOS.md §Popup
    // composition).
    let trigger = crate::builders::button::Button::new("Open modal")
        .id("demo-modal-trigger")
        .action(ComponentAction::click("gallery-demo/modal-open"))
        .build();
    let explainer = crate::builders::text::Text::new(
        "Clicking the button opens a popup in the `modal` sub-surface. \
         The X or backdrop dismisses via close-modal (frontend hardcode). \
         ModalSurface.svelte (layout-root singleton, Plan 17-05) supplies \
         the Dialog.Root / Dialog.Content chrome automatically.",
    )
    .id("demo-modal-explainer")
    .build();

    crate::builders::container::Container::new()
        .id("demo-modal-root")
        .children(vec![trigger, explainer])
        .build_with_children()
}
