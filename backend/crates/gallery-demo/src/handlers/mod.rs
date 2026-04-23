//! Gallery-demo action handlers.
//!
//! One file per handler family (per RESEARCH.md §Architecture Patterns recommended
//! project structure). `register_gallery_actions()` is the single entry point main.rs
//! calls to wire every handler into the ActionRouter.

use marionette::router::{ActionRouter, box_handler};
use marionette_protocol::common::AuthRequirement;

pub mod catalog_forms;
pub mod confirm;
pub mod fetch_rows;
pub mod modal;
pub mod navigate;
pub mod noop;
pub mod show;
pub mod toast;

/// Register every gallery-demo action handler on the given router, in a single
/// helper for readability. Action names match CONTEXT.md §D-C4 with the
/// frontend-hardcoded `close-modal` + `dismiss-toast` names included per
/// RESEARCH.md §Pitfall 3.
#[must_use]
pub fn register_gallery_actions(router: ActionRouter) -> ActionRouter {
    // Force-link external demo crates (currently just gallery-smoke) so the
    // linkme-backed DEMOS slice is populated. Without this, integration tests
    // and the production binary would see an empty registry despite the
    // Cargo.toml dep on gallery-smoke. See `lib.rs::__force_link_gallery_smoke`.
    crate::ensure_demos_linked();
    router
        .action("navigate", box_handler(navigate::handle_navigate), AuthRequirement::None)
        .action("gallery-show", box_handler(show::handle_gallery_show), AuthRequirement::None)
        .action("gallery-demo/noop", box_handler(noop::handle_noop), AuthRequirement::None)
        .action("gallery-demo/modal-open", box_handler(modal::handle_modal_open), AuthRequirement::None)
        .action("close-modal", box_handler(modal::handle_modal_close), AuthRequirement::None)
        .action("gallery-demo/confirm-open", box_handler(confirm::handle_confirm_open), AuthRequirement::None)
        .action("gallery-demo/confirm-accept", box_handler(confirm::handle_confirm_accept), AuthRequirement::None)
        .action("gallery-demo/confirm-reject", box_handler(confirm::handle_confirm_reject), AuthRequirement::None)
        .action("gallery-demo/toast-fire", box_handler(toast::handle_toast_fire), AuthRequirement::None)
        .action("dismiss-toast", box_handler(toast::handle_dismiss_toast), AuthRequirement::None)
        .action("fetch-rows", box_handler(fetch_rows::handle_demo_fetch_rows), AuthRequirement::None)
        // --- CAT-02 blur-validate handlers (Phase 18 Plan 18-05) ---
        // Six validators demonstrate Phase 12 node-tree ops (set-node /
        // set-children / delete-node) rotated across every input type.
        .action(
            "gallery-demo/catalog-forms/validate-text-input",
            box_handler(catalog_forms::validate_text_input),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-select",
            box_handler(catalog_forms::validate_select),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-checkbox",
            box_handler(catalog_forms::validate_checkbox),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-switch",
            box_handler(catalog_forms::validate_switch),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-radio",
            box_handler(catalog_forms::validate_radio),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-textarea",
            box_handler(catalog_forms::validate_textarea),
            AuthRequirement::None,
        )
}
