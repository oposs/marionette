//! Re-export shim for pre-Phase-17 import paths.
//!
//! `marionette::builders::standard::Button` etc. continue to resolve here
//! after the D-B3 per-component file refactor (Option A — preserved
//! import surface for the 10 external callers enumerated in RESEARCH.md).

// Every in-scope builder module defines a `gallery_demo` fn (Phase 17 DEMO-01),
// so the glob re-exports collide on that ident. Callers reach the demos via
// their explicit module path; the shim intentionally does not resolve
// `gallery_demo` at this namespace.
#![allow(ambiguous_glob_reexports)]

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
