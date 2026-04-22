//! `SideNav` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Navigation components --

#[derive(ComponentBuilder)]
#[component(type = "side-nav")]
pub struct SideNav {}
