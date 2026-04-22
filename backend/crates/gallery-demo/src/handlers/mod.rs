//! Gallery-demo action handlers.
//!
//! One file per handler family (per RESEARCH.md §Architecture Patterns recommended
//! project structure). `register_gallery_actions()` is the single entry point main.rs
//! calls to wire every handler into the ActionRouter.

use marionette::router::{ActionRouter, box_handler};
use marionette_protocol::common::AuthRequirement;

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
}
