//! `RadioGroup` component builder (with colocated `RadioOption`).
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;
use serde::{Deserialize, Serialize};

/// Option entry for a `RadioGroup` component (Phase 14 D-E4).
///
/// Mirrors `SelectOption` but adds an optional per-option `description`
/// rendered as 12px muted text beneath the option label (14-UI-SPEC.md
/// §Component Visual Contracts — `RadioGroup`). When `description` is
/// `None`, serde omits the key entirely via `skip_serializing_if`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Radio group primitive (Phase 14 D-E4).
///
/// Renders a shadcn `RadioGroup` wrapped in the shared `Field.Field` anatomy
/// on the frontend. Use for single-choice selection among N options when the
/// choices should be visible at-a-glance (contrast with `Select`, which hides
/// options behind a trigger). Each `RadioOption` can optionally carry a
/// per-option `description` rendered as muted 12px text beneath the label.
#[derive(ComponentBuilder)]
#[component(type = "radio-group")]
pub struct RadioGroup {
    pub label: String,
    pub options: Vec<RadioOption>,
    #[builder(optional)]
    pub required: Option<bool>,
    #[builder(optional)]
    pub disabled: Option<bool>,
    /// Helper text rendered below the group via shadcn `Field.Description`
    /// (Phase 14 D-B3). Hidden while an `/_errors/{bind}` entry is active
    /// (the error replaces the description per the shadcn recipe).
    #[builder(optional)]
    pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column of
    /// its parent `FieldSet` grid (Phase 14 D-C4).
    #[builder(optional)]
    pub full_width: Option<bool>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "radio-group")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    let options = vec![
        RadioOption {
            value: "alpha".into(),
            label: "Alpha".into(),
            description: None,
        },
        RadioOption {
            value: "beta".into(),
            label: "Beta".into(),
            description: Some("Second option with a description line.".into()),
        },
        RadioOption {
            value: "gamma".into(),
            label: "Gamma".into(),
            description: None,
        },
    ];
    let group = RadioGroup::new("Pick one", options)
        .bind("/demo/radio-group/value")
        .build();

    crate::builders::container::Container::new()
        .id("demo-radio-group-root")
        .children(vec![group])
        .build_with_children()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_group_serializes_options_and_optionals() {
        let options = vec![
            RadioOption {
                value: "a".into(),
                label: "Apple".into(),
                description: Some("Red fruit".into()),
            },
            RadioOption {
                value: "b".into(),
                label: "Banana".into(),
                description: None,
            },
        ];
        let (_id, component) = RadioGroup::new("Pick one", options)
            .description("Choose a fruit.")
            .full_width(true)
            .build();
        let props = component.props.unwrap();
        assert_eq!(component.r#type, "radio-group");
        assert_eq!(props["label"], "Pick one");
        assert_eq!(props["options"][0]["value"], "a");
        assert_eq!(props["options"][0]["label"], "Apple");
        assert_eq!(props["options"][0]["description"], "Red fruit");
        // Banana has no description — serde omits the key via skip_serializing_if:
        assert!(props["options"][1].get("description").is_none());
        assert_eq!(props["description"], "Choose a fruit.");
        assert_eq!(props["full_width"], true);
    }

    #[test]
    fn radio_group_basic_serialization() {
        let options = vec![RadioOption {
            value: "x".into(),
            label: "X".into(),
            description: None,
        }];
        let (_id, component) = RadioGroup::new("Choice", options).build();
        let props = component.props.unwrap();
        assert_eq!(component.r#type, "radio-group");
        assert!(props.get("description").is_none());
        assert!(props.get("full_width").is_none());
        assert!(props.get("required").is_none());
        assert!(props.get("disabled").is_none());
    }
}
