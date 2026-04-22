//! `TextInput` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "text-input")]
pub struct TextInput {
    pub label: String,
    #[builder(optional)]
    pub placeholder: Option<String>,
    #[builder(optional)]
    pub required: Option<bool>,
    #[builder(optional)]
    pub input_type: Option<String>,
    #[builder(optional)]
    pub disabled: Option<bool>,
    /// Helper text rendered below the input via shadcn `Field.Description`
    /// (Phase 14 D-B3). Replaces the retired `helperText` prop — pre-deployment
    /// posture, no back-compat alias. Hidden while an `/_errors/{bind}` entry
    /// is active (the error replaces the description per the shadcn recipe).
    #[builder(optional)]
    pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column of
    /// its parent `FieldSet` grid (Phase 14 D-C4). Used for long-text or
    /// full-width fields inside a 2-col `FieldSet`.
    #[builder(optional)]
    pub full_width: Option<bool>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "text-input")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    let a = TextInput::new("Label")
        .bind("/demo/text-input/value")
        .build();
    let b = TextInput::new("Disabled")
        .disabled(true)
        .bind("/demo/text-input/value-disabled")
        .build();
    let c = TextInput::new("With description")
        .description("Helper text rendered below via Field.Description.")
        .bind("/demo/text-input/value-desc")
        .build();

    crate::builders::container::Container::new()
        .id("demo-text-input-root")
        .children(vec![a, b, c])
        .build_with_children()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_input_builder() {
        let (id, component) = TextInput::new("Name")
            .placeholder("Enter name")
            .bind("/contact/name")
            .build();

        assert!(id.starts_with("text-input-"));
        assert_eq!(component.r#type, "text-input");
        assert_eq!(component.bind.as_deref(), Some("/contact/name"));
        let props = component.props.unwrap();
        assert_eq!(props["label"], "Name");
        assert_eq!(props["placeholder"], "Enter name");
    }

    // -- Phase 14 Plan 02 TextInput extensions (D-B3, D-C4) --

    #[test]
    fn text_input_serializes_description() {
        let (_id, component) = TextInput::new("Name")
            .description("We keep this private.")
            .build();
        assert_eq!(component.r#type, "text-input");
        let props = component.props.unwrap();
        assert_eq!(props["description"], "We keep this private.");
    }

    #[test]
    fn text_input_serializes_full_width() {
        let (_id, component) = TextInput::new("Bio").full_width(true).build();
        let props = component.props.unwrap();
        assert_eq!(props["full_width"], true);
    }

    #[test]
    fn text_input_omits_description_when_not_set() {
        let (_id, component) = TextInput::new("Name").build();
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
    fn custom_id() {
        let (id, _) = TextInput::new("X").id("my-input").build();
        assert_eq!(id, "my-input");
    }
}
