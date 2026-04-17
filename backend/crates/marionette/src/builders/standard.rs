//! Standard component builders for all 19 protocol component types.
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
    /// Helper text rendered below the input via shadcn `Field.Description`
    /// (Phase 14 D-B3). Replaces the retired `helperText` prop — pre-deployment
    /// posture, no back-compat alias. Hidden while an `/_errors/{bind}` entry
    /// is active (the error replaces the description per the shadcn recipe).
    #[builder(optional)]
    pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column of
    /// its parent `FieldSet` grid (Phase 14 D-C4). Used for long-text or
    /// full-width fields inside a 2-col FieldSet.
    #[builder(optional)]
    pub full_width: Option<bool>,
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
    /// that should take the full FieldSet row.
    #[builder(optional)]
    pub full_width: Option<bool>,
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

// -- Sub-surface mounting --

/// Mount a named sub-surface at this position in the tree.
///
/// `SurfaceMount` is a leaf node that renders `<Surface name={props.name}/>`
/// on the frontend, recursively mounting another surface's tree. Used by
/// `AppShell` (and future tabs, split-panes, etc.) to compose content surfaces
/// into a shell surface.
#[derive(ComponentBuilder)]
#[component(type = "surface-mount")]
pub struct SurfaceMount {
    pub name: String,
}

// -- Form components --

#[derive(ComponentBuilder)]
#[component(type = "form")]
pub struct Form {
    #[builder(optional)]
    pub submit_label: Option<String>,
}

/// Column definition for a `DataTable` component.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
    /// Cell rendering kind. Frontend maps each variant to a per-kind Svelte
    /// snippet. Default (None / omitted) renders as plain text. See
    /// `ColumnKind` for variants. Introduced in Phase 13 (D-F1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ColumnKind>,
    /// If `Some(true)`, the column starts hidden on mount; user can toggle
    /// it visible via the "Columns" dropdown. Introduced in Phase 13 (D-E2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_default: Option<bool>,
}

impl TableColumn {
    /// Ergonomic constructor for a plain text column.
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            sortable: None,
            kind: None,
            hidden_default: None,
        }
    }

    /// Mark this column sortable.
    #[must_use]
    pub fn sortable(mut self) -> Self {
        self.sortable = Some(true);
        self
    }

    /// Set the cell-rendering kind (default is plain text).
    #[must_use]
    pub fn kind(mut self, kind: ColumnKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Mark this column hidden by default on mount.
    #[must_use]
    pub fn hidden_default(mut self, hidden: bool) -> Self {
        self.hidden_default = Some(hidden);
        self
    }
}

/// Per-column cell rendering kind. Maps 1:1 to the Svelte `DataTable`'s
/// per-kind snippet lookup. Introduced in Phase 13 (D-F1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnKind {
    /// Default — renders `String(value)`.
    Text,
    /// Renders a shadcn `Badge` component.
    Badge,
    /// Renders a `DropdownMenu` of `{label, action}` items. Resolves the
    /// latent "actions column renders `[object Object]`" bug (see 13-RESEARCH §D-F1).
    Actions,
    /// Formats an ISO-8601 date via `Intl.DateTimeFormat`.
    Date,
    /// Right-aligns and formats via `Intl.NumberFormat`.
    Number,
}

/// Filter-bar entry for a `DataTable`. Each variant produces a different
/// shadcn primitive at render time (text input, select, or date-range
/// pair). Introduced in Phase 13 (D-B2, D-B3). Serialized as a tagged
/// union keyed by the `kind` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Filter {
    Text {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<u8>,
    },
    Select {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        options: Vec<SelectOption>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<u8>,
    },
    DateRange {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<u8>,
    },
}

impl Filter {
    /// Construct a text input filter.
    #[must_use]
    pub fn text(id: impl Into<String>) -> Self {
        Filter::Text {
            id: id.into(),
            label: None,
            placeholder: None,
            span: None,
        }
    }

    /// Construct a select-dropdown filter.
    #[must_use]
    pub fn select(id: impl Into<String>, options: Vec<SelectOption>) -> Self {
        Filter::Select {
            id: id.into(),
            label: None,
            options,
            span: None,
        }
    }

    /// Construct a date-range filter (two date inputs).
    #[must_use]
    pub fn date_range(id: impl Into<String>) -> Self {
        Filter::DateRange {
            id: id.into(),
            label: None,
            span: None,
        }
    }

    /// Set the visible label for this filter.
    #[must_use]
    pub fn label(mut self, l: impl Into<String>) -> Self {
        match &mut self {
            Filter::Text { label, .. }
            | Filter::Select { label, .. }
            | Filter::DateRange { label, .. } => *label = Some(l.into()),
        }
        self
    }

    /// Set a placeholder (text filters only — no-op on other variants).
    #[must_use]
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        if let Filter::Text { placeholder, .. } = &mut self {
            *placeholder = Some(p.into());
        }
        self
    }

    /// Set a grid-column span hint.
    #[must_use]
    pub fn span(mut self, s: u8) -> Self {
        match &mut self {
            Filter::Text { span, .. }
            | Filter::Select { span, .. }
            | Filter::DateRange { span, .. } => *span = Some(s),
        }
        self
    }
}

#[derive(ComponentBuilder)]
#[component(type = "data-table")]
pub struct DataTable {
    pub columns: Vec<TableColumn>,
    #[builder(optional)]
    pub page_size: Option<u32>,
    /// Total row count known server-side (Phase 13 D-D3). When set, the
    /// frontend `DataTable` stops fetching once `rows.length >= total_rows`.
    #[builder(optional)]
    pub total_rows: Option<u64>,
    /// Structured filter definitions consumed by the frontend's internal
    /// filter bar (Phase 13 D-B2). The derived setter replaces the vec;
    /// use the hand-written `.filter()` helper below to APPEND incrementally.
    #[builder(optional)]
    pub filters: Option<Vec<Filter>>,
    /// Key on each row object that the frontend treats as the stable row
    /// identifier (Phase 13 D-G1). Defaults to `"id"` on the frontend if
    /// absent.
    #[builder(optional)]
    pub row_id_key: Option<String>,
    /// Source identifier used by the frontend's fetch-rows sentinel
    /// (Phase 13 D-H1). Must match one of the whitelisted source strings
    /// in `crm-demo/src/handlers/fetch_rows.rs::required_role_for`.
    /// Typically matches the list handler's action name
    /// (e.g., `"contact_list"`, `"audit_list"`).
    #[builder(optional)]
    pub source: Option<String>,
}

// Hand-written append-style setter for `filters`. The derived macro
// generates a replace-setter for `Option<Vec<T>>` fields, which doesn't
// compose when a handler wants to chain `.filter(...).filter(...)`.
// Same pattern as Phase 12's `AppShellBuilder` slot helpers.
impl DataTableBuilder {
    /// Append a single filter to the filter bar. Chainable — call multiple
    /// times to add multiple filters in order.
    #[must_use]
    pub fn filter(mut self, f: Filter) -> Self {
        match self.filters.as_mut() {
            Some(existing) => existing.push(f),
            None => self.filters = Some(vec![f]),
        }
        self
    }
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

    // -- Phase 14 Plan 03 Select extensions (D-B3, D-C4, parity) --

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
    fn surface_mount_builder() {
        let (id, component) = SurfaceMount::new("content").build();
        assert!(!id.is_empty());
        assert!(id.starts_with("surface-mount-"));
        assert_eq!(component.r#type, "surface-mount");
        let props = component.props.as_ref().unwrap();
        assert_eq!(props["name"], "content");
        assert!(component.children.is_none());
        assert!(component.bind.is_none());
        assert!(component.action.is_none());
    }

    #[test]
    fn surface_mount_builder_custom_id() {
        let (id, _) = SurfaceMount::new("modal").id("shell-modal-mount").build();
        assert_eq!(id, "shell-modal-mount");
    }

    #[test]
    fn all_19_standard_types() {
        // Verify each of the 19 standard component types compiles and builds
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
            SurfaceMount::new("x").build().1.r#type,
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
            "heading", "text", "side-nav", "nav-item", "nav-group", "surface-mount",
            "form", "data-table", "modal", "toast", "confirm-dialog", "spinner", "error-display",
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

    // -- Phase 13 DataTable extension tests --

    #[test]
    fn filter_text_serializes_with_kind_tag() {
        let f = Filter::text("search")
            .label("Search")
            .placeholder("Filter by name...");
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["kind"], "text");
        assert_eq!(json["id"], "search");
        assert_eq!(json["label"], "Search");
        assert_eq!(json["placeholder"], "Filter by name...");
    }

    #[test]
    fn filter_select_serializes_with_options() {
        let f = Filter::select(
            "company",
            vec![
                SelectOption {
                    value: "1".into(),
                    label: "Acme".into(),
                },
                SelectOption {
                    value: "2".into(),
                    label: "Globex".into(),
                },
            ],
        )
        .label("Company");
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["kind"], "select");
        assert_eq!(json["id"], "company");
        assert_eq!(json["label"], "Company");
        assert_eq!(json["options"][0]["value"], "1");
        assert_eq!(json["options"][1]["label"], "Globex");
        // `label` omitted when not set → should not exist as a key
        let f2 = Filter::select("x", vec![]);
        let json2 = serde_json::to_value(&f2).unwrap();
        assert!(json2.get("label").is_none());
    }

    #[test]
    fn filter_date_range_serializes_with_kebab_kind() {
        let f = Filter::date_range("created").label("Created");
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["kind"], "date-range");
        assert_eq!(json["id"], "created");
        assert_eq!(json["label"], "Created");
    }

    #[test]
    fn column_kind_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(ColumnKind::Text).unwrap(),
            serde_json::json!("text")
        );
        assert_eq!(
            serde_json::to_value(ColumnKind::Badge).unwrap(),
            serde_json::json!("badge")
        );
        assert_eq!(
            serde_json::to_value(ColumnKind::Actions).unwrap(),
            serde_json::json!("actions")
        );
        assert_eq!(
            serde_json::to_value(ColumnKind::Date).unwrap(),
            serde_json::json!("date")
        );
        assert_eq!(
            serde_json::to_value(ColumnKind::Number).unwrap(),
            serde_json::json!("number")
        );
    }

    #[test]
    fn table_column_kind_and_hidden_default_serialize() {
        let col = TableColumn::new("actions", "Actions")
            .kind(ColumnKind::Actions)
            .hidden_default(true);
        let json = serde_json::to_value(&col).unwrap();
        assert_eq!(json["kind"], "actions");
        assert_eq!(json["hidden_default"], true);
        // Omitted `sortable` should not appear in the serialized map.
        assert!(json.get("sortable").is_none());
    }

    #[test]
    fn table_column_omits_new_optionals_when_unset() {
        let col = TableColumn::new("name", "Name");
        let json = serde_json::to_value(&col).unwrap();
        assert!(json.get("kind").is_none());
        assert!(json.get("hidden_default").is_none());
        assert!(json.get("sortable").is_none());
        assert_eq!(json["key"], "name");
        assert_eq!(json["label"], "Name");
    }

    #[test]
    fn data_table_fluent_filters_accumulate() {
        let (_id, component) = DataTable::new(vec![TableColumn::new("n", "Name")])
            .filter(Filter::text("search").label("Search"))
            .filter(Filter::select(
                "company",
                vec![SelectOption {
                    value: "1".into(),
                    label: "Acme".into(),
                }],
            ))
            .filter(Filter::date_range("created"))
            .total_rows(237u64)
            .row_id_key("id")
            .page_size(50u32)
            .build();
        let props = component.props.unwrap();
        let filters = props["filters"].as_array().unwrap();
        assert_eq!(filters.len(), 3);
        assert_eq!(filters[0]["kind"], "text");
        assert_eq!(filters[1]["kind"], "select");
        assert_eq!(filters[2]["kind"], "date-range");
        assert_eq!(props["total_rows"], 237);
        assert_eq!(props["row_id_key"], "id");
        assert_eq!(props["page_size"], 50);
    }

    #[test]
    fn data_table_omits_new_optional_fields_when_unset() {
        let (_id, component) = DataTable::new(vec![TableColumn::new("n", "Name")]).build();
        let props = component.props.unwrap();
        // Only `columns` should be present.
        assert!(props.get("total_rows").is_none());
        assert!(props.get("filters").is_none());
        assert!(props.get("row_id_key").is_none());
        assert!(props.get("page_size").is_none());
        assert!(props["columns"].is_array());
    }

    #[test]
    fn data_table_phase13_example_serializes_correctly() {
        let (_id, component) = DataTable::new(vec![
            TableColumn::new("name", "Name").sortable(),
            TableColumn::new("email", "Email"),
            TableColumn::new("created", "Created")
                .kind(ColumnKind::Date)
                .sortable(),
            TableColumn::new("actions", "").kind(ColumnKind::Actions),
            TableColumn::new("internal_id", "ID").hidden_default(true),
        ])
        .filter(
            Filter::text("search")
                .label("Search")
                .placeholder("Filter contacts..."),
        )
        .filter(
            Filter::select(
                "company",
                vec![
                    SelectOption {
                        value: String::new(),
                        label: "All companies".into(),
                    },
                    SelectOption {
                        value: "1".into(),
                        label: "Acme".into(),
                    },
                ],
            )
            .label("Company"),
        )
        .filter(Filter::date_range("created").label("Created date"))
        .total_rows(237u64)
        .row_id_key("id")
        .page_size(50u32)
        .source("contact_list")
        .build();

        let props = component.props.unwrap();
        // Columns
        let cols = props["columns"].as_array().unwrap();
        assert_eq!(cols.len(), 5);
        assert_eq!(cols[0]["key"], "name");
        assert_eq!(cols[0]["sortable"], true);
        // Columns that did NOT set `kind` should omit the field entirely.
        assert!(cols[0].get("kind").is_none());
        assert!(cols[1].get("kind").is_none());
        assert_eq!(cols[2]["kind"], "date");
        assert_eq!(cols[2]["sortable"], true);
        assert_eq!(cols[3]["kind"], "actions");
        assert_eq!(cols[4]["hidden_default"], true);
        // Filters
        let filters = props["filters"].as_array().unwrap();
        assert_eq!(filters.len(), 3);
        assert_eq!(filters[0]["kind"], "text");
        assert_eq!(filters[0]["placeholder"], "Filter contacts...");
        assert_eq!(filters[1]["kind"], "select");
        assert_eq!(filters[1]["options"].as_array().unwrap().len(), 2);
        assert_eq!(filters[2]["kind"], "date-range");
        // Other props
        assert_eq!(props["total_rows"], 237);
        assert_eq!(props["row_id_key"], "id");
        assert_eq!(props["page_size"], 50);
        assert_eq!(props["source"], "contact_list");
        // Type
        assert_eq!(component.r#type, "data-table");
    }

    #[test]
    fn data_table_source_field_serializes() {
        let (_id, component) = DataTable::new(vec![TableColumn::new("n", "Name")])
            .source("audit_list")
            .build();
        let props = component.props.unwrap();
        assert_eq!(props["source"], "audit_list");
    }

    #[test]
    fn data_table_source_omitted_when_unset() {
        let (_id, component) = DataTable::new(vec![TableColumn::new("n", "Name")]).build();
        let props = component.props.unwrap();
        assert!(props.get("source").is_none() || props["source"].is_null());
    }
}
