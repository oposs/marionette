//! `Switch` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

/// Toggle switch primitive (Phase 14 D-E4).
///
/// Renders a shadcn `Switch` wrapped in a horizontally-oriented `Field.Field`
/// (label on the left, control on the right). Semantically distinct from
/// `Checkbox`: use `Switch` for on/off state that takes effect immediately
/// (e.g., "Dark mode", "Notifications"); use `Checkbox` for agreement or
/// list-item selection. Boolean-typed bind.
#[derive(ComponentBuilder)]
#[component(type = "switch")]
pub struct Switch {
    pub label: String,
    #[builder(optional)]
    pub disabled: Option<bool>,
    /// Helper text rendered below the switch row via shadcn
    /// `Field.Description` (Phase 14 D-B3). Hidden while an
    /// `/_errors/{bind}` entry is active.
    #[builder(optional)]
    pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column of
    /// its parent `FieldSet` grid (Phase 14 D-C4).
    #[builder(optional)]
    pub full_width: Option<bool>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "switch")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    let a = Switch::new("Wifi").bind("/demo/switch/checked-1").build();
    let b = Switch::new("Bluetooth")
        .description("With a helper line below via Field.Description.")
        .bind("/demo/switch/checked-2")
        .build();

    crate::builders::container::Container::new()
        .id("demo-switch-root")
        .children(vec![a, b])
        .build_with_children()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_basic_serialization() {
        let (_id, component) = Switch::new("Notifications").build();
        let props = component.props.unwrap();
        assert_eq!(component.r#type, "switch");
        assert_eq!(props["label"], "Notifications");
        assert!(props.get("description").is_none());
        assert!(props.get("full_width").is_none());
        assert!(props.get("disabled").is_none());
    }

    #[test]
    fn switch_full_serialization() {
        let (_id, component) = Switch::new("Dark mode")
            .disabled(false)
            .description("Switch to dark theme.")
            .full_width(true)
            .build();
        let props = component.props.unwrap();
        assert_eq!(component.r#type, "switch");
        assert_eq!(props["label"], "Dark mode");
        assert_eq!(props["description"], "Switch to dark theme.");
        assert_eq!(props["full_width"], true);
        assert_eq!(props["disabled"], false);
    }
}
