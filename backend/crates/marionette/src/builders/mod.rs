// Every in-scope builder module defines a `gallery_demo` fn (Phase 17 DEMO-01),
// so `pub use <module>::*` creates ambiguous glob re-exports on `gallery_demo`.
// Callers access these fns via their explicit path
// (`crate::builders::<module>::gallery_demo`) — the parent `builders` namespace
// intentionally does not resolve `gallery_demo`. Silence rustc's default warn.
#![allow(ambiguous_glob_reexports)]

pub mod node;
pub mod app_shell;

// Per-component builder modules (Phase 17 D-B3 refactor).
pub mod button;
pub mod text_input;
pub mod select;
pub mod checkbox;
pub mod container;
pub mod grid;
pub mod heading;
pub mod text;
pub mod side_nav;
pub mod nav_item;
pub mod nav_group;
pub mod surface_mount;
pub mod form;
pub mod textarea;
pub mod radio_group;
pub mod switch;
pub mod field_set;
pub mod field_separator;
pub mod data_table;
// `modal` and `toast` modules host ONLY the `gallery_demo()` sibling.
// - `modal`: Phase 17 Plan 17-08 deleted the `Modal` struct after the
//   popups-global architectural refactor.
// - `toast`: the `Toast` struct was deleted in the sonner migration —
//   toasts are now protocol events rendered by the client's sonner
//   chrome (docs/OpenSDUI-CONCEPT.md §"Where the Client Is Smart").
// No `pub use modal::*;` / `pub use toast::*;` re-exports since there
// are no public types in these modules.
pub mod modal;
pub mod toast;
pub mod confirm_dialog;
pub mod spinner;
pub mod error_display;
pub mod composites;

// Preserved re-export shim (Option A from RESEARCH.md §Pattern 5).
pub mod standard;

pub use node::*;
pub use app_shell::*;
pub use button::*;
pub use text_input::*;
pub use select::*;
pub use checkbox::*;
pub use container::*;
pub use grid::*;
pub use heading::*;
pub use text::*;
pub use side_nav::*;
pub use nav_item::*;
pub use nav_group::*;
pub use surface_mount::*;
pub use form::*;
pub use textarea::*;
pub use radio_group::*;
pub use switch::*;
pub use field_set::*;
pub use field_separator::*;
pub use data_table::*;
pub use confirm_dialog::*;
pub use spinner::*;
pub use error_display::*;
pub use composites::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_17_standard_types() {
        // Verify each of the 17 standard component types compiles and builds.
        // Meta-test for the whole builders module — stays here (at the hub)
        // rather than under any single component. Moved from standard.rs
        // during the Phase 17 D-B3 per-component file refactor.
        //
        // History:
        // - Phase 17 Plan 17-08: `"modal"` removed — `ModalSurface.svelte`
        //   became a layout-root singleton mount; the `Modal` builder was
        //   deleted. Popups compose into the `modal` sub-surface directly
        //   (see `GALLERY-DEMOS.md` §Popup composition).
        // - Sonner migration: `"toast"` removed — toasts are protocol events
        //   (`type: "event"`, `name: "toast"`) rendered by the client's
        //   sonner chrome, not persistent SDUI nodes. See docs/OpenSDUI-CONCEPT.md
        //   §"Where the Client Is Smart".
        let types = vec![
            Button::new("x").build().1.r#type,
            TextInput::new("x").build().1.r#type,
            Select::new("x", vec![]).build().1.r#type,
            Checkbox::new("x").build().1.r#type,
            Container::new().build().1.r#type,
            Grid::new().build().1.r#type,
            Heading::new("x").build().1.r#type,
            Text::new("x").build().1.r#type,
            SideNav::new().build().1.r#type,
            NavItem::new("x", "y").build().1.r#type,
            NavGroup::new("x").build().1.r#type,
            SurfaceMount::new("x").build().1.r#type,
            Form::new().build().1.r#type,
            DataTable::new(vec![]).build().1.r#type,
            ConfirmDialog::new("x", "y").build().1.r#type,
            Spinner::new().build().1.r#type,
            ErrorDisplay::new("x").build().1.r#type,
        ];

        let expected = vec![
            "button", "text-input", "select", "checkbox", "container", "grid",
            "heading", "text", "side-nav", "nav-item", "nav-group", "surface-mount",
            "form", "data-table", "confirm-dialog", "spinner", "error-display",
        ];

        assert_eq!(types, expected);
    }
}
