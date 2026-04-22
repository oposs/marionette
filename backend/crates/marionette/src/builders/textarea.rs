//! `Textarea` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

/// Multi-line text input primitive (Phase 14 D-E3).
///
/// Renders a shadcn `Textarea` wrapped in the shared `Field.Field` anatomy
/// on the frontend. Used for long-text fields (notes, descriptions, message
/// bodies). `rows` defaults to 4 on the frontend when unset; `full_width=true`
/// makes the wrapper span every column of its parent `FieldSet` grid (D-C4).
#[derive(ComponentBuilder)]
#[component(type = "textarea")]
pub struct Textarea {
    pub label: String,
    #[builder(optional)]
    pub placeholder: Option<String>,
    /// Visible row count for the native `<textarea>` element. Defaults to 4
    /// on the frontend when omitted.
    #[builder(optional)]
    pub rows: Option<u32>,
    #[builder(optional)]
    pub required: Option<bool>,
    #[builder(optional)]
    pub disabled: Option<bool>,
    /// Helper text rendered below the textarea via shadcn `Field.Description`
    /// (Phase 14 D-B3). Hidden while an `/_errors/{bind}` entry is active
    /// (the error replaces the description per the shadcn recipe).
    #[builder(optional)]
    pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column of
    /// its parent `FieldSet` grid (Phase 14 D-C4). Long-text fields typically
    /// take the full row inside a 2-col `FieldSet`.
    #[builder(optional)]
    pub full_width: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textarea_basic_serialization() {
        let (_id, component) = Textarea::new("Notes").build();
        assert_eq!(component.r#type, "textarea");
        let props = component.props.unwrap();
        assert_eq!(props["label"], "Notes");
        // All optional fields omitted when unset:
        assert!(props.get("placeholder").is_none());
        assert!(props.get("rows").is_none());
        assert!(props.get("required").is_none());
        assert!(props.get("disabled").is_none());
        assert!(props.get("description").is_none());
        assert!(props.get("full_width").is_none());
    }

    #[test]
    fn textarea_full_serialization() {
        let (_id, component) = Textarea::new("Notes")
            .placeholder("Type here...")
            .rows(6u32)
            .required(true)
            .disabled(false)
            .description("Max 500 chars.")
            .full_width(true)
            .build();
        assert_eq!(component.r#type, "textarea");
        let props = component.props.unwrap();
        assert_eq!(props["label"], "Notes");
        assert_eq!(props["placeholder"], "Type here...");
        assert_eq!(props["rows"], 6);
        assert_eq!(props["required"], true);
        assert_eq!(props["disabled"], false);
        assert_eq!(props["description"], "Max 500 chars.");
        assert_eq!(props["full_width"], true);
    }

    #[test]
    fn textarea_rows_is_u32() {
        let (_id, component) = Textarea::new("Notes").rows(10u32).build();
        let props = component.props.unwrap();
        assert_eq!(props["rows"], 10);
        // u32 serializes as a JSON number, not a string:
        assert!(props["rows"].is_u64());
    }
}
