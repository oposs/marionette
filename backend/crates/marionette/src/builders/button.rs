//! `Button` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Interactive components --

#[derive(ComponentBuilder)]
#[component(type = "button")]
pub struct Button {
    pub label: String,
    #[builder(optional)]
    pub variant: Option<String>,
    #[builder(optional)]
    pub size: Option<String>,
    #[builder(optional)]
    pub disabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use marionette_protocol::ComponentAction;

    #[test]
    fn button_builder() {
        let (id, component) = Button::new("Save")
            .action(ComponentAction::submit("save"))
            .build();

        assert!(!id.is_empty());
        assert_eq!(component.r#type, "button");
        let props = component.props.unwrap();
        assert_eq!(props["label"], "Save");
        assert!(component.action.is_some());
        let action = component.action.unwrap();
        assert_eq!(action.r#type, "submit");
        assert_eq!(action.name.as_deref(), Some("save"));
    }

    #[test]
    fn optional_fields_omitted() {
        let (_, component) = Button::new("Submit").build();
        let props = component.props.unwrap();
        // Only "label" should be present, no "variant", "size", "disabled"
        assert_eq!(props.as_object().unwrap().len(), 1);
        assert_eq!(props["label"], "Submit");
    }

    #[test]
    fn visibility_binding() {
        let (_, component) = Button::new("Delete")
            .visible("/permissions/canDelete")
            .build();
        assert_eq!(
            component.visible.as_deref(),
            Some("/permissions/canDelete")
        );
    }
}
