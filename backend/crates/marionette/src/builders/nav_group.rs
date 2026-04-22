//! `NavGroup` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "nav-group")]
pub struct NavGroup {
    pub label: String,
}
