//! Re-export shim for pre-Phase-17 import paths.
//!
//! `marionette::builders::standard::Button` etc. continue to resolve here
//! after the D-B3 per-component file refactor (Option A — preserved
//! import surface for the 10 external callers enumerated in RESEARCH.md).

pub use super::{
    button::*, text_input::*, select::*, checkbox::*,
    container::*, grid::*, heading::*, text::*,
    side_nav::*, nav_item::*, nav_group::*, surface_mount::*,
    form::*, textarea::*, radio_group::*, switch::*,
    field_set::*, field_separator::*, data_table::*,
    modal::*, toast::*, confirm_dialog::*,
    spinner::*, error_display::*,
    composites::*,
};
