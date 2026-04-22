//! `Spinner` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "spinner")]
pub struct Spinner {
    #[builder(optional)]
    pub size: Option<String>,
}
