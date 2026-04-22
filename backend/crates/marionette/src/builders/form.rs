//! `Form` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Form components --

#[derive(ComponentBuilder)]
#[component(type = "form")]
pub struct Form {
    #[builder(optional)]
    pub submit_label: Option<String>,
}
