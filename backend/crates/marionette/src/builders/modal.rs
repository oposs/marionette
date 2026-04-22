//! `Modal` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Dialog / feedback components --

#[derive(ComponentBuilder)]
#[component(type = "modal")]
pub struct Modal {
    pub title: String,
    #[builder(optional)]
    pub size: Option<String>,
}
