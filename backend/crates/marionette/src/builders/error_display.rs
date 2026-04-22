//! `ErrorDisplay` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "error-display")]
pub struct ErrorDisplay {
    pub message: String,
}
