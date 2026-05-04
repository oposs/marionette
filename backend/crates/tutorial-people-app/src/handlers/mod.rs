//! Tutorial action handlers.
//!
//! `register_app_actions` is the single entry point `main.rs` calls to wire
//! every handler into the [`marionette::router::ActionRouter`]. The same
//! function is used by integration tests so the route table stays in sync
//! between the binary and the test harness.

use marionette::router::{ActionRouter, box_handler};
use marionette_protocol::common::AuthRequirement;

pub mod navigate;
pub mod people;

#[must_use]
pub fn register_app_actions(router: ActionRouter) -> ActionRouter {
    router
        .action(
            "navigate",
            box_handler(navigate::handle_navigate),
            AuthRequirement::None,
        )
        .action(
            people::APP_ADD_PERSON,
            box_handler(people::handle_add_person),
            AuthRequirement::None,
        )
}
