//! `NavItem` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "nav-item")]
pub struct NavItem {
    pub label: String,
    pub path: String,
    #[builder(optional)]
    pub icon: Option<String>,
}
