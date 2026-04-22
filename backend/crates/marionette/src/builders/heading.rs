//! `Heading` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Content components --

#[derive(ComponentBuilder)]
#[component(type = "heading")]
pub struct Heading {
    pub text: String,
    #[builder(optional)]
    pub level: Option<u8>,
}
