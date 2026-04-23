//! `Container` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Layout components --

#[derive(ComponentBuilder)]
#[component(type = "container")]
pub struct Container {
    #[builder(optional)]
    pub class: Option<String>,
    /// Optional kebab-case Lucide icon name (display-only).
    ///
    /// When set, the Svelte `Container` component renders the icon (via
    /// `getIcon`) before any children. Paired with a sibling text label
    /// in CAT-05 catalog cells — the icon itself is `aria-hidden`.
    #[builder(optional)]
    pub icon: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::heading::Heading;
    use super::super::button::Button;
    use super::super::text_input::TextInput;

    #[test]
    fn container_icon_prop_serialised() {
        let (_id, comp) = Container::new().icon("plus").build();
        let v = serde_json::to_value(&comp).unwrap();
        assert_eq!(v["props"]["icon"], "plus");
    }

    #[test]
    fn container_icon_absent_when_not_set() {
        let (_id, comp) = Container::new().build();
        let v = serde_json::to_value(&comp).unwrap();
        // Either `icon` is missing from the props object or serialised as null.
        // Both shapes satisfy "not set" for the frontend's `{#if props.icon}` guard.
        let icon = &v["props"]["icon"];
        assert!(icon.is_null() || v["props"].as_object().is_none_or(|o| !o.contains_key("icon")));
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
}
