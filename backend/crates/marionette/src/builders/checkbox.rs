//! `Checkbox` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "checkbox")]
pub struct Checkbox {
    pub label: String,
    #[builder(optional)]
    pub disabled: Option<bool>,
    /// Helper text rendered below the checkbox row via shadcn
    /// `Field.Description` (Phase 14 D-B3). Hidden while an
    /// `/_errors/{bind}` entry is active (the error replaces the
    /// description per the shadcn recipe).
    #[builder(optional)]
    pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column
    /// of its parent `FieldSet` grid (Phase 14 D-C4). Used for consent
    /// checkboxes that should take the full `FieldSet` row.
    #[builder(optional)]
    pub full_width: Option<bool>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "checkbox")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    let a = Checkbox::new("Unchecked").bind("/demo/checkbox/a").build();
    let b = Checkbox::new("With description")
        .description("Helper text from Field.Description.")
        .bind("/demo/checkbox/b")
        .build();
    let c = Checkbox::new("Disabled")
        .disabled(true)
        .bind("/demo/checkbox/c")
        .build();

    crate::builders::container::Container::new()
        .id("demo-checkbox-root")
        .children(vec![a, b, c])
        .build_with_children()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkbox_serializes_description_and_full_width() {
        let (_id, component) = Checkbox::new("Agree")
            .description("Read the terms first.")
            .full_width(true)
            .build();
        assert_eq!(component.r#type, "checkbox");
        let props = component.props.unwrap();
        assert_eq!(props["description"], "Read the terms first.");
        assert_eq!(props["full_width"], true);
    }

    #[test]
    fn checkbox_omits_new_optionals_when_not_set() {
        let (_id, component) = Checkbox::new("Agree").build();
        let props = component.props.unwrap();
        assert!(
            props.get("description").is_none(),
            "description should be omitted"
        );
        assert!(
            props.get("full_width").is_none(),
            "full_width should be omitted"
        );
    }

    #[test]
    fn checkbox_preserves_existing_disabled_field() {
        let (_id, component) = Checkbox::new("Agree").disabled(true).build();
        let props = component.props.unwrap();
        assert_eq!(props["disabled"], true);
    }
}
