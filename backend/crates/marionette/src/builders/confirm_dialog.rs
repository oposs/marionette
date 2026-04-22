//! `ConfirmDialog` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3. Phase 17 Plan 17-05 Task 8
//! extended the struct with `confirm_label` / `cancel_label` /
//! `cancel_action` / `destructive` so the backend can drive the frontend
//! ConfirmDialog.svelte end-to-end (G-04 corrective pass). Field names
//! stay snake_case to match the ComponentBuilder derive macro's
//! identifier-to-key convention, consistent with DataTable's `page_size`
//! precedent. The frontend (ConfirmDialog.svelte) reads snake_case first
//! and falls back to camelCase for legacy call sites.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "confirm-dialog")]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    /// Label rendered on the primary (confirm) button. Defaults to
    /// "Confirm" on the frontend when omitted.
    #[builder(optional)]
    pub confirm_label: Option<String>,
    /// Label rendered on the secondary (cancel) button. Defaults to
    /// "Cancel" on the frontend when omitted.
    #[builder(optional)]
    pub cancel_label: Option<String>,
    /// Action name dispatched when the cancel button is clicked.
    /// Defaults to `close-modal` on the frontend when omitted — useful
    /// when the cancel button should route to a specific handler (e.g.
    /// `gallery-demo/confirm-reject`) so the backend can run
    /// reject-specific side-effects like enqueuing a toast.
    #[builder(optional)]
    pub cancel_action: Option<String>,
    /// When `true`, the confirm button renders with the destructive
    /// (red) shadcn variant. Use for deletion / irreversible actions.
    #[builder(optional)]
    pub destructive: Option<bool>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_dialog_basic_serialization() {
        let (_id, component) = ConfirmDialog::new("Delete?", "Are you sure?").build();
        assert_eq!(component.r#type, "confirm-dialog");
        let props = component.props.unwrap();
        assert_eq!(props["title"], "Delete?");
        assert_eq!(props["message"], "Are you sure?");
        // Optionals omitted when not set
        assert!(props.get("confirm_label").is_none());
        assert!(props.get("cancel_label").is_none());
        assert!(props.get("cancel_action").is_none());
        assert!(props.get("destructive").is_none());
    }

    #[test]
    fn confirm_dialog_with_all_optionals() {
        let (_id, component) = ConfirmDialog::new("Demo confirm", "Choose an option.")
            .confirm_label("Accept")
            .cancel_label("Reject")
            .cancel_action("gallery-demo/confirm-reject")
            .destructive(false)
            .build();
        let props = component.props.unwrap();
        assert_eq!(props["title"], "Demo confirm");
        assert_eq!(props["message"], "Choose an option.");
        assert_eq!(props["confirm_label"], "Accept");
        assert_eq!(props["cancel_label"], "Reject");
        assert_eq!(props["cancel_action"], "gallery-demo/confirm-reject");
        assert_eq!(props["destructive"], false);
    }
}
