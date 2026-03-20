//! Standard component builders for all 18 protocol component types.
//!
//! Each component type is defined as a struct with `#[derive(ComponentBuilder)]`,
//! which generates a fluent builder API.

use marionette_macros::ComponentBuilder;
use serde::{Deserialize, Serialize};

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
}

/// Option entry for a Select component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(ComponentBuilder)]
#[component(type = "select")]
pub struct Select {
    pub label: String,
    pub options: Vec<SelectOption>,
    #[builder(optional)]
    pub required: Option<bool>,
}

#[derive(ComponentBuilder)]
#[component(type = "checkbox")]
pub struct Checkbox {
    pub label: String,
    #[builder(optional)]
    pub disabled: Option<bool>,
}

// -- Layout components --

#[derive(ComponentBuilder)]
#[component(type = "container")]
pub struct Container {
    #[builder(optional)]
    pub class: Option<String>,
}

#[derive(ComponentBuilder)]
#[component(type = "grid")]
pub struct Grid {
    #[builder(optional)]
    pub cols: Option<u8>,
    #[builder(optional)]
    pub gap: Option<String>,
}

// -- Content components --

#[derive(ComponentBuilder)]
#[component(type = "heading")]
pub struct Heading {
    pub text: String,
    #[builder(optional)]
    pub level: Option<u8>,
}

#[derive(ComponentBuilder)]
#[component(type = "text")]
pub struct Text {
    pub text: String,
}

// -- Navigation components --

#[derive(ComponentBuilder)]
#[component(type = "side-nav")]
pub struct SideNav {}

#[derive(ComponentBuilder)]
#[component(type = "nav-item")]
pub struct NavItem {
    pub label: String,
    pub path: String,
    #[builder(optional)]
    pub icon: Option<String>,
}

#[derive(ComponentBuilder)]
#[component(type = "nav-group")]
pub struct NavGroup {
    pub label: String,
}

// -- Form components --

#[derive(ComponentBuilder)]
#[component(type = "form")]
pub struct Form {
    #[builder(optional)]
    pub submit_label: Option<String>,
}

/// Column definition for a `DataTable` component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
}

#[derive(ComponentBuilder)]
#[component(type = "data-table")]
pub struct DataTable {
    pub columns: Vec<TableColumn>,
    #[builder(optional)]
    pub page_size: Option<u32>,
}

// -- Dialog / feedback components --

#[derive(ComponentBuilder)]
#[component(type = "modal")]
pub struct Modal {
    pub title: String,
    #[builder(optional)]
    pub size: Option<String>,
}

#[derive(ComponentBuilder)]
#[component(type = "toast")]
pub struct Toast {
    pub message: String,
    #[builder(optional)]
    pub variant: Option<String>,
    #[builder(optional)]
    pub duration: Option<u32>,
}

#[derive(ComponentBuilder)]
#[component(type = "confirm-dialog")]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
}

#[derive(ComponentBuilder)]
#[component(type = "spinner")]
pub struct Spinner {
    #[builder(optional)]
    pub size: Option<String>,
}

#[derive(ComponentBuilder)]
#[component(type = "error-display")]
pub struct ErrorDisplay {
    pub message: String,
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

    #[test]
    fn container_builder_with_children() {
        let heading = Heading::new("Title").id("heading-1").build();
        let nodes = Container::new()
            .child(heading)
            .build_with_children();

        // Should contain container + heading
        assert_eq!(nodes.len(), 2);
        let (container_id, container) = &nodes[0];
        assert!(!container_id.is_empty());
        assert_eq!(container.r#type, "container");
        assert_eq!(
            container.children.as_ref().unwrap(),
            &["heading-1".to_string()]
        );

        let (heading_id, heading) = &nodes[1];
        assert_eq!(heading_id, "heading-1");
        assert_eq!(heading.r#type, "heading");
    }

    #[test]
    fn children_method() {
        let btn = Button::new("OK").id("btn-1").build();
        let input = TextInput::new("Email").id("input-1").build();

        let nodes = Container::new()
            .children(vec![btn, input])
            .build_with_children();

        assert_eq!(nodes.len(), 3);
        let (_, container) = &nodes[0];
        let child_ids = container.children.as_ref().unwrap();
        assert_eq!(child_ids, &["btn-1", "input-1"]);
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
    fn custom_id() {
        let (id, _) = TextInput::new("X").id("my-input").build();
        assert_eq!(id, "my-input");
    }

    #[test]
    fn all_18_standard_types() {
        // Verify each of the 18 standard component types compiles and builds
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
            Form::new().build().1.r#type,
            DataTable::new(vec![]).build().1.r#type,
            Modal::new("x").build().1.r#type,
            Toast::new("x").build().1.r#type,
            ConfirmDialog::new("x", "y").build().1.r#type,
            Spinner::new().build().1.r#type,
            ErrorDisplay::new("x").build().1.r#type,
        ];

        let expected = vec![
            "button", "text-input", "select", "checkbox", "container", "grid",
            "heading", "text", "side-nav", "nav-item", "nav-group", "form",
            "data-table", "modal", "toast", "confirm-dialog", "spinner", "error-display",
        ];

        assert_eq!(types, expected);
    }

    #[test]
    fn grid_with_optional_fields() {
        let (_, component) = Grid::new().cols(3).gap("1rem").build();
        let props = component.props.unwrap();
        assert_eq!(props["cols"], 3);
        assert_eq!(props["gap"], "1rem");
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
