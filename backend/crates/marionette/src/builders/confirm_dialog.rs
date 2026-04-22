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
