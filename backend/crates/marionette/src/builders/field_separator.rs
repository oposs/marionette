//! `FieldSeparator` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

/// Explicit sibling-divider node rendered between consecutive
/// `FieldSet` components inside a `Form` (D-C2, preferred explicit-node
/// path). Renders a thin `<Field.Separator />` line in the current
/// `--border` token colour.
#[derive(ComponentBuilder)]
#[component(type = "field-separator")]
pub struct FieldSeparator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_separator_serializes_with_no_props() {
        let (_id, component) = FieldSeparator::new().build();
        assert_eq!(component.r#type, "field-separator");
        // Unit struct: the ComponentBuilder derive serialises no prop
        // fields. The macro may produce either an empty props object or
        // omit the field entirely — accept both shapes.
        match component.props.as_ref() {
            None => {}
            Some(v) => {
                let obj = v.as_object().expect("props should be a JSON object when present");
                assert!(
                    obj.is_empty(),
                    "FieldSeparator should serialise with no own prop keys, got: {obj:?}"
                );
            }
        }
    }
}
