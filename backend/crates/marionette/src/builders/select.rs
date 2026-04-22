//! `Select` component builder (with colocated `SelectOption`).
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;
use serde::{Deserialize, Serialize};

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
    /// Backend-authoritative placeholder text rendered inside the trigger
    /// when no value is selected. Phase 14 Plan 03: the Svelte component
    /// already reads `props.placeholder` — this field surfaces it in the
    /// typed Rust builder for parity.
    #[builder(optional)]
    pub placeholder: Option<String>,
    /// Disabled state passthrough. Phase 14 Plan 03: the Svelte component
    /// already reads `props.disabled` — this field surfaces it in the
    /// typed Rust builder for parity.
    #[builder(optional)]
    pub disabled: Option<bool>,
    /// Helper text rendered below the trigger via shadcn `Field.Description`
    /// (Phase 14 D-B3). Hidden while an `/_errors/{bind}` entry is active
    /// (the error replaces the description per the shadcn recipe).
    #[builder(optional)]
    pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column of
    /// its parent `FieldSet` grid (Phase 14 D-C4). Used for Select fields
    /// that should take the full `FieldSet` row.
    #[builder(optional)]
    pub full_width: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_serializes_description_and_full_width() {
        let options = vec![SelectOption {
            value: "a".into(),
            label: "A".into(),
        }];
        let (_id, component) = Select::new("X", options)
            .description("pick one")
            .full_width(true)
            .build();
        assert_eq!(component.r#type, "select");
        let props = component.props.unwrap();
        assert_eq!(props["description"], "pick one");
        assert_eq!(props["full_width"], true);
    }

    #[test]
    fn select_serializes_placeholder_and_disabled() {
        let options = vec![SelectOption {
            value: "a".into(),
            label: "A".into(),
        }];
        let (_id, component) = Select::new("X", options)
            .placeholder("Select...")
            .disabled(true)
            .build();
        let props = component.props.unwrap();
        assert_eq!(props["placeholder"], "Select...");
        assert_eq!(props["disabled"], true);
    }

    #[test]
    fn select_omits_new_optionals_when_not_set() {
        let options = vec![SelectOption {
            value: "a".into(),
            label: "A".into(),
        }];
        let (_id, component) = Select::new("X", options).build();
        let props = component.props.unwrap();
        assert!(
            props.get("description").is_none(),
            "description should be omitted"
        );
        assert!(
            props.get("full_width").is_none(),
            "full_width should be omitted"
        );
        assert!(
            props.get("placeholder").is_none(),
            "placeholder should be omitted"
        );
        assert!(
            props.get("disabled").is_none(),
            "disabled should be omitted"
        );
        // Existing optional `required` must also remain omitted.
        assert!(
            props.get("required").is_none(),
            "required should be omitted"
        );
    }
}
