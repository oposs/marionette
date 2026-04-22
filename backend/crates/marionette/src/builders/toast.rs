//! `Toast` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "toast")]
pub struct Toast {
    pub message: String,
    #[builder(optional)]
    pub variant: Option<String>,
    #[builder(optional)]
    pub duration: Option<u32>,
}
