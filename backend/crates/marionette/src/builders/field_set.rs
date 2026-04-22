//! `FieldSet` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Field structural components (Phase 14 — D-C1, D-C2) --

/// Structural SDUI container that wraps form fields in a shadcn
/// `<Field.Set>` with an optional legend + description and an
/// auto-responsive grid (D-C1, D-C3). The frontend renders children
/// via `NodeRenderer` inside a `<Field.Group>`.
///
/// Without `cols`, the grid defaults to `grid-cols-1 md:grid-cols-2`
/// (stacked on mobile, two columns from the `md:` breakpoint). Setting
/// `cols` to `Some(N)` switches to a fixed N-column grid at all viewport
/// widths via inline `grid-template-columns: repeat(N, minmax(0, 1fr))`
/// — Tailwind v4's JIT cannot resolve dynamic `grid-cols-{N}` class
/// names, so the inline style is the required workaround (Pitfall #1).
#[derive(ComponentBuilder)]
#[component(type = "field-set")]
pub struct FieldSet {
    /// Visible group title rendered as `<Field.Legend>` (screen-reader
    /// announced when focus enters any child field).
    #[builder(optional)]
    pub legend: Option<String>,
    /// Optional group-level explanation rendered below the legend via
    /// `<Field.Description>`.
    #[builder(optional)]
    pub description: Option<String>,
    /// Column count override. `None` → auto-responsive (1 column on
    /// mobile, 2 columns from `md:` up, per D-C3). `Some(N)` → fixed
    /// N-column grid at all viewport widths (D-C4). `0` is not a valid
    /// column count.
    #[builder(optional)]
    pub cols: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_set_basic_serialization() {
        let (_id, component) = FieldSet::new().build();
        assert_eq!(component.r#type, "field-set");
        // When every optional is unset, the ComponentBuilder derive may
        // either produce an empty props object or omit `props` entirely.
        // Both are valid "no props emitted" shapes — assert either.
        match component.props.as_ref() {
            None => {}
            Some(v) => {
                assert!(v.get("legend").is_none(), "legend should be omitted");
                assert!(v.get("description").is_none(), "description should be omitted");
                assert!(v.get("cols").is_none(), "cols should be omitted");
            }
        }
    }

    #[test]
    fn field_set_full_serialization() {
        let (_id, component) = FieldSet::new()
            .legend("Contact Info")
            .description("Primary contact details.")
            .cols(3u8)
            .build();
        assert_eq!(component.r#type, "field-set");
        let props = component.props.unwrap();
        assert_eq!(props["legend"], "Contact Info");
        assert_eq!(props["description"], "Primary contact details.");
        assert_eq!(props["cols"], 3);
    }
}
