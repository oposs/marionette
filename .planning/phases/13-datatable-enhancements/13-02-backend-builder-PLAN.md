---
phase: 13
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - backend/crates/marionette/src/builders/standard.rs
autonomous: true
requirements: [TABLE-01, TABLE-02, TABLE-03]
must_haves:
  truths:
    - "`Filter::text(id)`, `Filter::select(id, options)`, `Filter::date_range(id)` constructors exist and serialize to tagged JSON matching the `{id, kind, label?, placeholder?, options?}` shape the frontend expects"
    - "`ColumnKind` enum with variants `Text`, `Badge`, `Actions`, `Date`, `Number` exists and serializes as lowercase kebab-ish strings matching `'text' | 'badge' | 'actions' | 'date' | 'number'`"
    - "`TableColumn` struct has new optional fields `kind: Option<ColumnKind>` and `hidden_default: Option<bool>`"
    - "`DataTable` struct has new optional fields `total_rows: Option<u64>`, `filters: Option<Vec<Filter>>`, `row_id_key: Option<String>`"
    - "`DataTableBuilder` has a hand-written `.filter(Filter) -> Self` method that APPENDS to the filters vec (NOT a replace-setter)"
    - "`DataTableBuilder` has `.total_rows(u64)` and `.row_id_key(impl Into<String>)` setters (either auto-derived or hand-written)"
    - "`TableColumn` has ergonomic constructors or builder methods for setting `kind` and `hidden_default` (e.g., `.kind(ColumnKind::Actions)`, `.hidden_default(true)`)"
    - "All new types serialize/deserialize losslessly via `serde_json`"
  artifacts:
    - path: "backend/crates/marionette/src/builders/standard.rs"
      provides: "Extended DataTable builder, Filter enum, ColumnKind enum"
      contains: "pub enum Filter"
    - path: "backend/crates/marionette/src/builders/standard.rs"
      provides: "ColumnKind enum"
      contains: "pub enum ColumnKind"
  key_links:
    - from: "CRM handlers (Plan 06)"
      to: "DataTable::new(...).filter(Filter::text(\"search\")).total_rows(n)"
      via: "fluent builder API"
      pattern: "DataTable::new.*\\.filter\\("
    - from: "frontend DataTable.svelte (Plan 05)"
      to: "serialized JSON shape of filters[] and column.kind"
      via: "`props.filters` and `props.columns[].kind`"
      pattern: "\"kind\":"
---

<objective>
Extend the Rust backend component builder so CRM handlers can declare the new DataTable capabilities (filter bar, column kinds, total_rows, row_id_key, hidden_default columns) with the same fluent `.method()` ergonomics as every other builder in the crate.

Purpose: Every CRM list handler migrated in Plan 06 needs this API. The frontend DataTable rewrite in Plan 05 reads the resulting JSON shape. Without this plan's Rust types, Plan 06 cannot compile and Plan 05 cannot render real data.

Output: One extended `standard.rs` file with `Filter` and `ColumnKind` enums, extended `TableColumn` and `DataTable` structs, hand-written `impl DataTableBuilder { pub fn filter }` method, and inline `#[cfg(test)]` tests proving the JSON shape.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/13-datatable-enhancements/13-CONTEXT.md
@.planning/phases/13-datatable-enhancements/13-RESEARCH.md
@.planning/codebase/CONVENTIONS.md
@backend/crates/marionette/src/builders/standard.rs
@backend/crates/marionette-macros/src/component_builder.rs

<interfaces>
<!-- Executor needs these BEFORE touching standard.rs. Extracted from current code. -->

Current `TableColumn` struct (backend/crates/marionette/src/builders/standard.rs:140-146):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
}
```

Current `DataTable` struct (lines 148-154):
```rust
#[derive(ComponentBuilder)]
#[component(type = "data-table")]
pub struct DataTable {
    pub columns: Vec<TableColumn>,
    #[builder(optional)]
    pub page_size: Option<u32>,
}
```

**Builder macro behavior (verified by reading marionette-macros):**

The `#[derive(ComponentBuilder)]` macro generates a `DataTableBuilder` struct + `DataTable::new(columns)` constructor + fluent setters for each field. For `#[builder(optional)]` fields of type `Option<T>`, it generates a SET-style setter `.page_size(val)` that assigns. **For `Option<Vec<T>>`, the auto-generated setter is also a replace-setter, NOT an append-setter.** Phase 12's `AppShellBuilder` uses the same pattern (hand-written `impl AppShellBuilder { pub fn slot(self, ...) -> Self { ... } }`) to work around this.

Phase 13 must hand-write `impl DataTableBuilder { pub fn filter(self, f: Filter) -> Self }` that appends to the inner `filters` field, so handlers can chain `.filter(...).filter(...).filter(...)`.

**JSON shape downstream frontend expects (per D-B2 and D-F1):**

```json
{
  "type": "data-table",
  "props": {
    "columns": [
      { "key": "name", "label": "Name", "sortable": true, "kind": "text" },
      { "key": "actions", "label": "", "kind": "actions", "hidden_default": false }
    ],
    "filters": [
      { "id": "search", "kind": "text", "label": "Search", "placeholder": "Filter..." },
      { "id": "company", "kind": "select", "label": "Company",
        "options": [{"value":"1","label":"Acme"}] },
      { "id": "created", "kind": "date-range", "label": "Created" }
    ],
    "total_rows": 237,
    "row_id_key": "id",
    "page_size": 50
  }
}
```

Field names in camelCase vs snake_case: the Marionette convention is **snake_case** Rust field names serialized **as-is** (no `rename_all = "camelCase"`). Confirmed by reading existing `TableColumn` (`sortable` stays `sortable`). So `total_rows`, `row_id_key`, `hidden_default`, `page_size` all serialize as snake_case. The frontend DataTable.svelte reads them in snake_case. (Current DataTable.svelte reads `props.totalRows` at line 31 — that's the EXISTING camelCase reader and it's WRONG; Plan 05 will switch it to `total_rows` when rewriting.)

**Existing test pattern (inline #[cfg(test)] mod tests { ... } at end of file, lines 196-355).**
</interfaces>

<research_references>
- 13-RESEARCH.md §Standard Stack §Architecture Patterns Pattern 3 — per-kind cell renderer signatures that dictate the JSON-side `kind` enum values
- 13-CONTEXT.md §D-G1 — exact shape of the extended struct
- 13-CONTEXT.md §D-F1 — list of ColumnKind variants
- 13-CONTEXT.md §D-H1 — downstream generic `fetch_rows` handler (Plan 03) reads back the `row_id_key` indirectly via CRM handler code
</research_references>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend TableColumn + DataTable + add Filter/ColumnKind enums with inline tests</name>
  <files>backend/crates/marionette/src/builders/standard.rs</files>
  <read_first>
    - backend/crates/marionette/src/builders/standard.rs (entire file — you're extending it)
    - backend/crates/marionette-macros/src/component_builder.rs (confirms builder macro behavior for `Option<Vec<T>>` fields)
    - Phase 12 hand-written builder: grep for "impl AppShellBuilder" in standard.rs or adjacent files to see the convention for hand-written append-style setters
    - .planning/phases/13-datatable-enhancements/13-CONTEXT.md §D-G1 (exact field list)
    - .planning/phases/13-datatable-enhancements/13-RESEARCH.md §Standard Stack (serialization expectations)
  </read_first>
  <behavior>
    Inline tests MUST prove:
    - `Filter::text("search").label("Search").placeholder("Find...")` serializes to `{"id":"search","kind":"text","label":"Search","placeholder":"Find..."}`
    - `Filter::select("company", vec![SelectOption{...}])` serializes to `{"id":"company","kind":"select","label":null-or-missing,"options":[...]}`
    - `Filter::date_range("created")` serializes to `{"id":"created","kind":"date-range"}`
    - `ColumnKind::Actions` serializes as `"actions"`, `ColumnKind::DateRange` N/A (not a column kind), `ColumnKind::Text` as `"text"`, `ColumnKind::Badge` as `"badge"`, `ColumnKind::Date` as `"date"`, `ColumnKind::Number` as `"number"`
    - `DataTable::new(vec![...]).filter(Filter::text("a")).filter(Filter::select("b", vec![])).filter(Filter::date_range("c")).build()` produces a component whose `props.filters` has exactly 3 entries in declared order
    - `DataTable::new(vec![]).total_rows(237).row_id_key("id").build()` sets `props.total_rows == 237` and `props.row_id_key == "id"`
    - `TableColumn` extended with `kind: Some(ColumnKind::Badge)` serializes `"kind": "badge"`
    - `TableColumn` with `hidden_default: Some(true)` serializes `"hidden_default": true`
    - Omitted optional fields do NOT appear in the serialized JSON (serde `skip_serializing_if = "Option::is_none"`)
  </behavior>
  <action>
    Edit `backend/crates/marionette/src/builders/standard.rs`. Make these EXACT changes in place:

    **A. Extend `TableColumn` (lines 139-146).** Replace the current struct with:

    ```rust
    /// Column definition for a `DataTable` component.
    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Per-column cell rendering kind. Maps 1:1 to the Svelte DataTable's
    /// per-kind snippet lookup. Introduced in Phase 13 (D-F1).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum ColumnKind {
        /// Default — renders `String(value)`.
        Text,
        /// Renders a shadcn `Badge` component.
        Badge,
        /// Renders a `DropdownMenu` of `{label, action}` items. Resolves the
        /// latent "actions column renders [object Object]" bug (research §D-F1).
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
    ```

    **B. Extend the `DataTable` struct (lines 148-154).** Replace with:

    ```rust
    #[derive(ComponentBuilder)]
    #[component(type = "data-table")]
    pub struct DataTable {
        pub columns: Vec<TableColumn>,
        #[builder(optional)]
        pub page_size: Option<u32>,
        /// Total row count known server-side (Phase 13 D-D3). When set, the
        /// frontend DataTable stops fetching once `rows.length >= total_rows`.
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
    }

    // Hand-written append-style setter for `filters`. The derived macro
    // generates a replace-setter for `Option<Vec<T>>` fields, which doesn't
    // compose when a handler wants to chain `.filter(...).filter(...)`.
    // Same pattern as Phase 12's AppShellBuilder slot helpers.
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
    ```

    If `DataTableBuilder` is named differently by the macro (inspect `marionette-macros/src/component_builder.rs` — it's likely `<StructName>Builder`), adjust the `impl` block target accordingly. If the macro exposes the `filters` field under a different name (e.g., inside an inner options struct), match whatever shape the macro produces.

    **C. Add new inline tests** to the existing `#[cfg(test)] mod tests { ... }` block near the bottom of the file, right before the closing `}` of the module:

    ```rust
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
        let f = Filter::select("company", vec![
            SelectOption { value: "1".into(), label: "Acme".into() },
            SelectOption { value: "2".into(), label: "Globex".into() },
        ]).label("Company");
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["kind"], "select");
        assert_eq!(json["id"], "company");
        assert_eq!(json["label"], "Company");
        assert_eq!(json["options"][0]["value"], "1");
        assert_eq!(json["options"][1]["label"], "Globex");
    }

    #[test]
    fn filter_date_range_serializes_with_kebab_kind() {
        let f = Filter::date_range("created").label("Created");
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["kind"], "date-range");
        assert_eq!(json["id"], "created");
    }

    #[test]
    fn column_kind_serializes_lowercase() {
        assert_eq!(serde_json::to_value(ColumnKind::Text).unwrap(), serde_json::json!("text"));
        assert_eq!(serde_json::to_value(ColumnKind::Badge).unwrap(), serde_json::json!("badge"));
        assert_eq!(serde_json::to_value(ColumnKind::Actions).unwrap(), serde_json::json!("actions"));
        assert_eq!(serde_json::to_value(ColumnKind::Date).unwrap(), serde_json::json!("date"));
        assert_eq!(serde_json::to_value(ColumnKind::Number).unwrap(), serde_json::json!("number"));
    }

    #[test]
    fn table_column_kind_and_hidden_default_serialize() {
        let col = TableColumn::new("actions", "Actions")
            .kind(ColumnKind::Actions)
            .hidden_default(true);
        let json = serde_json::to_value(&col).unwrap();
        assert_eq!(json["kind"], "actions");
        assert_eq!(json["hidden_default"], true);
        // Omitted sortable should not appear
        assert!(json.get("sortable").is_none() || json["sortable"].is_null());
    }

    #[test]
    fn data_table_fluent_filters_accumulate() {
        let (_id, component) = DataTable::new(vec![TableColumn::new("n", "Name")])
            .filter(Filter::text("search").label("Search"))
            .filter(Filter::select("company", vec![SelectOption { value: "1".into(), label: "Acme".into() }]))
            .filter(Filter::date_range("created"))
            .total_rows(237)
            .row_id_key("id")
            .page_size(50)
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
        // Only columns should be present
        assert!(props.get("total_rows").is_none() || props["total_rows"].is_null());
        assert!(props.get("filters").is_none() || props["filters"].is_null());
        assert!(props.get("row_id_key").is_none() || props["row_id_key"].is_null());
    }
    ```

    If `DataTableBuilder` does not expose `total_rows`, `row_id_key`, or `page_size` as fluent setters after the macro expansion, inspect the macro source and match whatever naming convention it uses. The core requirement is: the test cases above must compile and pass.

    **Do NOT** rename existing `TableColumn` callers that use the struct-literal shape `TableColumn { key, label, sortable }`. Those will break because the new struct has extra fields — the existing callers in `audit.rs`, `contact.rs`, `company.rs`, `user.rs` would need `..Default::default()` or migration to `TableColumn::new(...)`. They will be migrated in Plan 06.

    To keep the build green without touching those callers yet, either:
    - (a) Implement `Default for TableColumn` and update the four existing call sites in this task to use `..Default::default()` spread. **DO THIS.**
    - OR (b) Delay the struct-field addition and instead add a separate type. **DO NOT — this doubles the API.**

    Chosen: (a). Implement `Default`:

    ```rust
    impl Default for TableColumn {
        fn default() -> Self {
            Self {
                key: String::new(),
                label: String::new(),
                sortable: None,
                kind: None,
                hidden_default: None,
            }
        }
    }
    ```

    Then run `cargo build -p marionette -p crm-demo` and update any struct-literal callers in `audit.rs`, `company.rs`, `contact.rs`, `user.rs` that fail to compile by adding `..Default::default()` at the end of each `TableColumn { ... }` literal. These are minimal mechanical edits to keep the build green; full migration of these handlers happens in Plan 06.
  </action>
  <verify>
    <automated>cd backend && cargo test -p marionette --lib builders::standard::tests::filter_text_serializes_with_kind_tag builders::standard::tests::filter_select_serializes_with_options builders::standard::tests::filter_date_range_serializes_with_kebab_kind builders::standard::tests::column_kind_serializes_lowercase builders::standard::tests::table_column_kind_and_hidden_default_serialize builders::standard::tests::data_table_fluent_filters_accumulate builders::standard::tests::data_table_omits_new_optional_fields_when_unset 2>&1 | tee /tmp/phase13-02-test.log && cargo build -p crm-demo</automated>
  </verify>
  <acceptance_criteria>
    - `cd backend && cargo test -p marionette --lib builders::standard::tests` runs and 7 new tests pass (plus existing tests still pass)
    - `cd backend && cargo build -p crm-demo` succeeds with ZERO compilation errors (existing CRM handlers still compile with `..Default::default()` spreads added where needed)
    - `grep -c "pub enum Filter" backend/crates/marionette/src/builders/standard.rs` == 1
    - `grep -c "pub enum ColumnKind" backend/crates/marionette/src/builders/standard.rs` == 1
    - `grep -c "impl DataTableBuilder" backend/crates/marionette/src/builders/standard.rs` == 1
    - `grep -c "pub fn filter(mut self, f: Filter)" backend/crates/marionette/src/builders/standard.rs` == 1
    - `grep -c 'rename_all = "kebab-case"' backend/crates/marionette/src/builders/standard.rs` >= 1 (Filter enum)
    - `grep -c 'rename_all = "lowercase"' backend/crates/marionette/src/builders/standard.rs` >= 1 (ColumnKind enum)
    - `cd backend && cargo clippy -p marionette -- -D warnings` exits 0 (no new clippy pedantic warnings)
  </acceptance_criteria>
  <done>All new types + fluent API in place, inline tests green, CRM crate still compiles.</done>
</task>

<task type="auto">
  <name>Task 2: Update the "all 19 standard types" inventory test to exercise the new DataTable fluent API</name>
  <files>backend/crates/marionette/src/builders/standard.rs</files>
  <read_first>
    - backend/crates/marionette/src/builders/standard.rs §`fn all_19_standard_types` (around line 303) — the existing inventory test uses `TableColumn { key, label, sortable }` struct literals that will have been updated in Task 1
    - Task 1 inline tests you just added
  </read_first>
  <action>
    Verify that the existing `all_19_standard_types` inventory test (around line 303) still compiles and passes after Task 1's struct extension. If it uses `TableColumn { key: "x".into(), label: "x".into(), sortable: None }` struct-literal syntax, update to either `TableColumn::new("x", "x")` or add `..Default::default()`.

    Also add ONE new inventory-level test that proves the frontend-facing JSON shape for a realistic DataTable:

    ```rust
    #[test]
    fn data_table_phase13_example_serializes_correctly() {
        let (_id, component) = DataTable::new(vec![
            TableColumn::new("name", "Name").sortable(),
            TableColumn::new("email", "Email"),
            TableColumn::new("created", "Created").kind(ColumnKind::Date).sortable(),
            TableColumn::new("actions", "").kind(ColumnKind::Actions),
            TableColumn::new("internal_id", "ID").hidden_default(true),
        ])
        .filter(Filter::text("search").label("Search").placeholder("Filter contacts..."))
        .filter(Filter::select("company", vec![
            SelectOption { value: "".into(), label: "All companies".into() },
            SelectOption { value: "1".into(), label: "Acme".into() },
        ]).label("Company"))
        .filter(Filter::date_range("created").label("Created date"))
        .total_rows(237)
        .row_id_key("id")
        .page_size(50)
        .build();

        let props = component.props.unwrap();
        // Columns
        let cols = props["columns"].as_array().unwrap();
        assert_eq!(cols.len(), 5);
        assert_eq!(cols[0]["key"], "name");
        assert_eq!(cols[0]["sortable"], true);
        assert_eq!(cols[2]["kind"], "date");
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
        // Type
        assert_eq!(component.r#type, "data-table");
    }
    ```

    Do not touch any other builder or test. Keep the diff focused.
  </action>
  <verify>
    <automated>cd backend && cargo test -p marionette --lib builders::standard::tests::data_table_phase13_example_serializes_correctly builders::standard::tests::all_19_standard_types && cargo test -p marionette --lib</automated>
  </verify>
  <acceptance_criteria>
    - `data_table_phase13_example_serializes_correctly` test passes
    - `all_19_standard_types` test still passes (proof of no regressions on other builders)
    - `cd backend && cargo test -p marionette --lib` passes every builder test (no red)
    - `cd backend && cargo clippy -p marionette --lib -- -D warnings` exits 0
  </acceptance_criteria>
  <done>Comprehensive end-to-end inventory test green; no regressions in other builders.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Rust builder → JSON wire | The `Filter`/`ColumnKind` serialization defines the contract the frontend parses. Breakage here = invalid props on the wire. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-13-02-01 | Tampering | `Filter` tagged enum serialization drift | mitigate | Inline tests assert the exact JSON shape (`kind` tag value, field presence). Any refactor that changes the shape must update the tests, making drift loud. |
| T-13-02-02 | Tampering | Hand-written `DataTableBuilder::filter` breaks when the derive macro changes | mitigate | Test `data_table_fluent_filters_accumulate` proves chained `.filter(...).filter(...)` accumulates. If the macro ever introduces a conflicting method, rustc fails loudly. |

No HIGH severity threats. This plan touches only Rust types; no user input is processed here.
</threat_model>

<verification>
```bash
cd backend
cargo test -p marionette --lib builders::standard::tests
cargo clippy -p marionette --lib -- -D warnings
cargo build -p crm-demo   # existing CRM handlers must still compile
```

All three MUST exit 0.
</verification>

<success_criteria>
- `Filter`, `ColumnKind` enums with exact-shape serialization tests pass
- `DataTableBuilder::filter` append-style helper exists and chains
- `TableColumn::new / .sortable / .kind / .hidden_default` ergonomic builder methods exist
- `DataTable::new(..).filter(..).total_rows(..).row_id_key(..)` end-to-end test (`data_table_phase13_example_serializes_correctly`) green
- `backend/crates/crm-demo/` still compiles (existing CRM handlers patched minimally if struct-literal `TableColumn` breaks)
- Zero clippy pedantic warnings in `marionette/builders/standard.rs`
</success_criteria>

<output>
After completion, create `.planning/phases/13-datatable-enhancements/13-02-backend-builder-SUMMARY.md` recording:
- The exact `DataTableBuilder` method names the macro generated for `total_rows`, `row_id_key`, `page_size`, `filters` (confirm they match the test expectations)
- Any CRM call site in `audit.rs/company.rs/contact.rs/user.rs` that had to be tweaked (expected: struct-literal `TableColumn { ... }` spots)
- The serialized JSON shape of a realistic DataTable with all five kinds, all three filter types, `total_rows`, and `row_id_key` — as a code block Plan 05 can reference when writing the frontend reader
</output>
