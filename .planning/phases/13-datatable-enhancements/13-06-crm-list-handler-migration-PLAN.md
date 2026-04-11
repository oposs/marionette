---
phase: 13
plan: 06
type: execute
wave: 4
depends_on: [13-02, 13-03, 13-05]
files_modified:
  - backend/crates/crm-demo/src/handlers/audit.rs
  - backend/crates/crm-demo/src/handlers/contact.rs
  - backend/crates/crm-demo/src/handlers/company.rs
  - backend/crates/crm-demo/src/handlers/user.rs
  - backend/crates/marionette/src/builders/standard.rs
  - spec/PROTOCOL.md
  - spec/schemas/data.yaml
autonomous: true
requirements: [TABLE-01, TABLE-02, TABLE-03]
must_haves:
  truths:
    - "All four CRM list handlers (audit, contact, company, user) use the new DataTable shape: filters declared inline via `Filter::text/select/date_range(...)`, `total_rows` set from a COUNT(*) query, `source` set to the handler name, `row_id_key` set to `id`"
    - "The old TableScreen-adjacent filter UI pattern is gone: each handler no longer builds a separate filter-form Container with `Text/Select/Button` children; instead those live inside `DataTable.props.filters`"
    - "Each handler sets `.source(<handler_name>)` so DataTable can dispatch `sendAction('fetch-rows', { source })` to Plan 03's generic handler"
    - "Each handler runs a `.count()` query with the SAME filter WHERE clauses as the page query to populate `total_rows` (D-H2)"
    - "Actions-column latent bug is fixed: `contact.rs`, `company.rs`, `user.rs` rows that already ship `actions: [{label, action}]` arrays now have `TableColumn::new(\"actions\", \"\").kind(ColumnKind::Actions)` declared so the frontend renders a DropdownMenu instead of `[object Object]`"
    - "CRM list handlers keep their existing action-router registrations (names: `contact_list`, `company_list`, `user_list`, `audit_list`) — only the rendered component tree changes"
    - "The `source` field is added to the backend `DataTable` struct as `#[builder(optional)] pub source: Option<String>` in standard.rs, with a corresponding inline test"
    - "`spec/PROTOCOL.md` data-table example at line 373 is updated to reflect the new props (filters[], total_rows, source, row_id_key, columns[].kind, columns[].hidden_default)"
    - "Existing tests (`cd backend && cargo test -p crm-demo`) still pass"
  artifacts:
    - path: "backend/crates/marionette/src/builders/standard.rs"
      provides: "source field on DataTable"
      contains: "pub source: Option<String>"
    - path: "backend/crates/crm-demo/src/handlers/audit.rs"
      provides: "Migrated audit_list handler with inline filters + total_rows + source"
    - path: "backend/crates/crm-demo/src/handlers/contact.rs"
      provides: "Migrated contact_list handler"
    - path: "backend/crates/crm-demo/src/handlers/company.rs"
      provides: "Migrated company_list handler"
    - path: "backend/crates/crm-demo/src/handlers/user.rs"
      provides: "Migrated user_list handler"
    - path: "spec/PROTOCOL.md"
      provides: "Updated data-table example"
    - path: "spec/schemas/data.yaml"
      provides: "New DataTable schema section documenting filters / total_rows / row_id_key / source / columns[].kind / columns[].hidden_default (D-B2, D-G1)"
  key_links:
    - from: "Migrated handlers"
      to: "DataTable::new(...).filter(...).total_rows(...).source(...).row_id_key(...).build()"
      via: "fluent builder"
      pattern: "\\.total_rows\\("
    - from: "Frontend DataTable.svelte (Plan 05)"
      to: "handler-set `source` prop"
      via: "props.source → sendAction('fetch-rows', {source})"
      pattern: "props.source"
---

<objective>
Migrate the four CRM list handlers to use the new Phase 13 DataTable shape end-to-end. Each handler drops its own separate filter-form `Container` and moves those filter declarations INSIDE the DataTable via `.filter(...)`. Each handler adds a `.count()` query to populate `total_rows` (D-H2). Each handler sets `.source("<handler_name>")` so DataTable's sentinel can reach Plan 03's `fetch-rows` handler. The three handlers that ship `actions` arrays in their rows (contact, company, user) get a new `TableColumn::new("actions", "").kind(ColumnKind::Actions)` entry so the frontend renders the latent bug.

Purpose: Without this plan, the migrated DataTable renders against empty/wrong-shaped handler output. This is where the new frontend meets the real CRM data. It's also where the latent `[object Object]` actions bug finally resolves in production, and where the spec's data-table example gets updated to the new shape.

Output: Four migrated handler files, a new `source` field on the backend DataTable struct (with inline test), and an updated `spec/PROTOCOL.md` data-table example.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/13-datatable-enhancements/13-CONTEXT.md
@.planning/phases/13-datatable-enhancements/13-RESEARCH.md
@.planning/phases/13-datatable-enhancements/13-02-backend-builder-PLAN.md
@.planning/phases/13-datatable-enhancements/13-03-fetch-rows-handler-PLAN.md
@.planning/phases/13-datatable-enhancements/13-05-datatable-rewrite-PLAN.md
@.planning/codebase/CONVENTIONS.md
@backend/crates/marionette/src/builders/standard.rs
@backend/crates/crm-demo/src/handlers/audit.rs
@backend/crates/crm-demo/src/handlers/contact.rs
@backend/crates/crm-demo/src/handlers/company.rs
@backend/crates/crm-demo/src/handlers/user.rs
@spec/PROTOCOL.md

<interfaces>
<!-- Executor MUST read each handler in full before migrating it. -->

Current `handle_audit_list` structure (backend/crates/crm-demo/src/handlers/audit.rs:25-217):
- Accepts `Payload::<AuditFilterPayload>::from_context(&ctx)` with fields `user_id`, `table`, `date_from`, `date_to`
- Builds query with conditional `.filter(...)` calls
- Hand-builds a filter-form Container with 4 Select/TextInput children + a "Filter" button
- Builds a DataTable with 6 columns
- Composes them into `audit-root` Container
- Returns Render + nav_active_patch
- **Migration: drop `user_select`, `table_input`, `date_from_input`, `date_to_input`, `filter_button`, `filter_container_child/descendants`. Move all four filters inline via `Filter::select(...)` and `Filter::text(...)` and `Filter::date_range(...)` on the DataTable builder. Set `total_rows` from a COUNT(*) query with matching WHERE clauses. Set `source` to `"audit_list"`. Set `row_id_key` to `"id"`. Existing row data fields stay unchanged.**

Current `handle_contact_list` structure (backend/crates/crm-demo/src/handlers/contact.rs:380-452):
- Builds filter-form with search, company_filter, date_from, date_to, tag_filter_text
- Rows include `actions` array at line 423-426 (four fields: id, name, email, phone, company, tags, sync_status, created, actions)
- Composes `contact-list-root` Container
- **Migration: same pattern — drop filter-form children, inline filters on DataTable. Add `TableColumn::new("actions", "").kind(ColumnKind::Actions)` as the last column so the actions array renders as a DropdownMenu. Set `source: "contact_list"`. Add `total_rows` via COUNT(*).**

Current `handle_company_list` (backend/crates/crm-demo/src/handlers/company.rs:148+):
- Similar shape; also ships per-row actions at line 126-129 (the latent bug)
- **Migration: same pattern + actions column kind.**

Current `handle_user_list` (backend/crates/crm-demo/src/handlers/user.rs:110+):
- Similar shape; ships per-row actions at line 87-90
- **Migration: same pattern + actions column kind.**

Plan 02 added `DataTable` builder fields: `total_rows: Option<u64>`, `filters: Option<Vec<Filter>>`, `row_id_key: Option<String>`, `page_size: Option<u32>`, plus hand-written `.filter(Filter) -> Self` append method.

**MISSING FIELD:** `source: Option<String>`. Plan 05's DataTable.svelte reads `props.source` and passes it to `sendAction('fetch-rows', { source })`. This plan (06) adds the field to the Rust `DataTable` struct as Task 1 before migrating the handlers.

**SeaORM count pattern** (used already in existing handlers like `seed.rs:20` via `user::Entity::find().count(db)`):
```rust
use sea_orm::PaginatorTrait;
let total: u64 = audit_log::Entity::find()
    .filter(/* same WHERE as page query */)
    .count(&*db.0)
    .await
    .map_err(|e| ActionError::Internal(e.to_string()))?;
```

PROTOCOL.md data-table example (lines 373-384):
```yaml
contact-list:
  type: data-table
  props:
    columns:
      - { key: "name", label: "Name", sortable: true }
      - { key: "email", label: "Email" }
      - { key: "phone", label: "Phone" }
    keyField: "id"
  bind: "/contacts"
```

Needs updating to show the new props (`filters`, `total_rows`, `row_id_key`, `source`, per-column `kind` and `hidden_default`).
</interfaces>

<research_references>
- 13-RESEARCH.md Summary paragraph 1 — the `[object Object]` bug is at `contact.rs:423`, `company.rs:126`, `user.rs:87`
- 13-CONTEXT.md §D-A2 — CRM handlers compose `Container([Heading, …toolbar Buttons, DataTable])` directly, dropping TableScreen entirely
- 13-CONTEXT.md §D-H1, D-H2 — source field + total_rows for all four handlers
- 13-CONTEXT.md §D-B2 — filters[] props shape
- 13-CONTEXT.md §D-F1 — column.kind enum
</research_references>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add `source` field to backend DataTable struct with inline test</name>
  <files>backend/crates/marionette/src/builders/standard.rs</files>
  <read_first>
    - backend/crates/marionette/src/builders/standard.rs (Plan 02 extended this — the `DataTable` struct at around line 148)
    - .planning/phases/13-datatable-enhancements/13-02-backend-builder-SUMMARY.md (for exact field ordering and macro-generated setter names)
    - Plan 02's inline test `data_table_phase13_example_serializes_correctly` — pattern to extend
  </read_first>
  <behavior>
    - `DataTable::new(cols).source("contact_list").build()` produces `component.props["source"] == "contact_list"`
    - Omitted `source` does not appear in the serialized JSON (serde `skip_serializing_if` via `#[builder(optional)]`)
  </behavior>
  <action>
    Add a `source` field to the `DataTable` struct in `backend/crates/marionette/src/builders/standard.rs`:

    ```rust
    #[derive(ComponentBuilder)]
    #[component(type = "data-table")]
    pub struct DataTable {
        pub columns: Vec<TableColumn>,
        #[builder(optional)]
        pub page_size: Option<u32>,
        #[builder(optional)]
        pub total_rows: Option<u64>,
        #[builder(optional)]
        pub filters: Option<Vec<Filter>>,
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
    ```

    Extend the existing `data_table_phase13_example_serializes_correctly` test (added in Plan 02) to also set `.source("contact_list")` and assert `props["source"] == "contact_list"`.

    Add a new focused test:

    ```rust
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
    ```

    Run the tests.
  </action>
  <verify>
    <automated>cd backend && cargo test -p marionette --lib builders::standard::tests::data_table_source_field_serializes builders::standard::tests::data_table_source_omitted_when_unset builders::standard::tests::data_table_phase13_example_serializes_correctly</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "pub source: Option<String>" backend/crates/marionette/src/builders/standard.rs` == 1
    - Both new tests (`data_table_source_field_serializes`, `data_table_source_omitted_when_unset`) pass
    - The extended `data_table_phase13_example_serializes_correctly` test still passes with the new assertion
    - `cd backend && cargo clippy -p marionette -- -D warnings` exits 0
  </acceptance_criteria>
  <done>DataTable struct has `source` field; tests prove round-trip; clippy clean.</done>
</task>

<task type="auto">
  <name>Task 2: Migrate all four CRM list handlers to the new DataTable shape</name>
  <files>
    backend/crates/crm-demo/src/handlers/audit.rs,
    backend/crates/crm-demo/src/handlers/contact.rs,
    backend/crates/crm-demo/src/handlers/company.rs,
    backend/crates/crm-demo/src/handlers/user.rs
  </files>
  <read_first>
    - backend/crates/crm-demo/src/handlers/audit.rs (entire file — you're rewriting the handler body)
    - backend/crates/crm-demo/src/handlers/contact.rs:380-452 (the `render_contact_list` function — the most complex migration target, has 5 filters + actions column)
    - backend/crates/crm-demo/src/handlers/company.rs:70-146 (list handler with actions column)
    - backend/crates/crm-demo/src/handlers/user.rs:36-108 (list handler with actions column)
    - backend/crates/marionette/src/builders/standard.rs §`DataTable`, `Filter`, `ColumnKind`, `TableColumn::new`, `impl DataTableBuilder` (Plan 02's additions, confirmed by Task 1)
    - backend/crates/crm-demo/src/handlers/fetch_rows.rs (Plan 03 — confirm the source dispatch table expects `"contact_list"`, `"audit_list"`, `"company_list"`, `"user_list"` — must match exactly)
    - backend/crates/crm-demo/src/entities/ (entity column names for building `.filter(...).count()` chains)
  </read_first>
  <action>
    Migrate each handler with the same 6-step pattern. Do them one at a time, committing between each so if one breaks the others aren't in a half-migrated state.

    **Step 1 — Migrate `handle_audit_list` (audit.rs).**

    Delete lines 84-122 of `audit.rs` (the hand-built filter-form Container — `user_select`, `table_input`, `date_from_input`, `date_to_input`, `filter_button`, `filter_container_child/descendants`).

    Replace the DataTable construction (lines 124-158) with:

    ```rust
    // Compute total_rows with the same WHERE clauses as the page query (D-H2).
    // Note: we have to rebuild the filter chain because SeaORM's query is consumed.
    let mut count_query = audit_log::Entity::find();
    if let Some(uid) = filter.user_id {
        count_query = count_query.filter(audit_log::Column::AuditLogUser.eq(uid));
    }
    if let Some(ref tbl) = filter.table {
        if !tbl.is_empty() {
            count_query = count_query.filter(audit_log::Column::AuditLogTable.eq(tbl.as_str()));
        }
    }
    if let Some(ref date_from) = filter.date_from {
        if !date_from.is_empty() {
            count_query = count_query.filter(audit_log::Column::AuditLogTimestamp.gte(date_from.as_str()));
        }
    }
    if let Some(ref date_to) = filter.date_to {
        if !date_to.is_empty() {
            count_query = count_query.filter(audit_log::Column::AuditLogTimestamp.lte(date_to.as_str()));
        }
    }
    let total_rows: u64 = count_query
        .count(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    let table = DataTable::new(vec![
        TableColumn::new("timestamp", "When").sortable().kind(ColumnKind::Date),
        TableColumn::new("user", "Who").sortable(),
        TableColumn::new("table", "Table").sortable(),
        TableColumn::new("recordId", "Record"),
        TableColumn::new("action", "Action").sortable(),
        TableColumn::new("changes", "Changes").hidden_default(true),
    ])
    .filter(Filter::select("user_id", user_options).label("User"))
    .filter(Filter::text("table").label("Table").placeholder("e.g. user"))
    .filter(Filter::date_range("date").label("Date range"))
    .total_rows(total_rows)
    .source("audit_list")
    .row_id_key("id")
    .page_size(50)
    .id("audit-table")
    .bind("/auditEntries")
    .build();
    ```

    **NOTE on filter id mapping:** The old `AuditFilterPayload` uses `user_id`, `table`, `date_from`, `date_to`. With the new date-range filter, the frontend will dispatch `{ date: { from, to } }` — a SINGLE field containing both endpoints. You need to update `AuditFilterPayload` accordingly:

    ```rust
    #[derive(Deserialize, Default)]
    pub struct AuditFilterPayload {
        user_id: Option<String>,  // now a string because select values are strings; parse to i32 inside handler
        table: Option<String>,
        /// Combined date range from the frontend's `date-range` filter kind.
        date: Option<DateRange>,
    }

    #[derive(Deserialize, Default)]
    pub struct DateRange {
        #[serde(default)]
        from: Option<String>,
        #[serde(default)]
        to: Option<String>,
    }
    ```

    Update the query construction to parse `user_id` to `i32` with `.parse().ok()` and to read the date range from `filter.date.as_ref()`:

    ```rust
    if let Some(ref s) = filter.user_id {
        if let Ok(uid) = s.parse::<i32>() {
            query = query.filter(audit_log::Column::AuditLogUser.eq(uid));
        }
    }
    if let Some(ref dr) = filter.date {
        if let Some(ref from) = dr.from {
            if !from.is_empty() {
                query = query.filter(audit_log::Column::AuditLogTimestamp.gte(from.as_str()));
            }
        }
        if let Some(ref to) = dr.to {
            if !to.is_empty() {
                query = query.filter(audit_log::Column::AuditLogTimestamp.lte(to.as_str()));
            }
        }
    }
    ```

    Mirror the same filter shape in the `count_query` block.

    After the DataTable construction, delete the old `filter_container_descendants` handling at lines 189-195. The new container composition is:

    ```rust
    let all_children = vec![heading, table];

    let container_nodes = Container::new()
        .id("audit-root")
        .children(all_children)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }
    ```

    The `data` payload stays mostly the same but replaces `auditFilter` with the new filter shape:

    ```rust
    let data = serde_json::json!({
        "auditEntries": rows,
    });
    ```

    (DataTable owns filter state locally per D-C4 — the backend no longer pre-populates initial filter values. If handlers want to pre-seed filter values after a filter action round-trips, they can send `props.filter_defaults` — but that's deferred.)

    **Step 2 — Migrate `handle_contact_list` (contact.rs).**

    Same pattern. The 5 existing filters become:
    ```rust
    .filter(Filter::text("search").label("Search").placeholder("Filter contacts..."))
    .filter(Filter::select("company_filter", company_options).label("Company"))
    .filter(Filter::text("tag_filter_text").label("Tag").placeholder("e.g. VIP"))
    .filter(Filter::date_range("date").label("Created date"))
    ```

    (Dropping `date_from`/`date_to` as separate fields, collapsing to one `date` date-range.)

    Add `TableColumn::new("actions", "").kind(ColumnKind::Actions)` as the LAST column. Existing row data at line 423 already ships the `actions` array — the frontend now renders it via `DataTableActions` (Plan 04).

    Set `.source("contact_list").total_rows(contact_count).row_id_key("id")`.

    Update `ContactFilterPayload` (the struct name may differ — grep for it) to the new shape: `search: Option<String>`, `company_filter: Option<String>`, `tag_filter_text: Option<String>`, `date: Option<DateRange>`.

    Delete the `filter_form_child` / `filter_form_descendants` construction entirely. The Container assembly at line 385 becomes:

    ```rust
    let all_children = vec![heading, new_button, sync_all_button, table];
    ```

    **Step 3 — Migrate `handle_company_list` (company.rs).**

    Same pattern. Read the current filters at the top of the handler, inline them via `Filter::*`. Add actions column kind. Set source/total_rows/row_id_key. Delete the filter-form Container.

    **Step 4 — Migrate `handle_user_list` (user.rs).**

    Same pattern. Add actions column kind. Set source/total_rows/row_id_key. Delete filter-form.

    **Step 5 — Build and test.**

    ```bash
    cd backend && cargo build -p crm-demo && cargo test -p crm-demo
    ```

    Fix any compilation errors that surface — typical issues:
    - `TableColumn { ... }` struct-literal callers that Plan 02 patched with `..Default::default()` can now be fully migrated to `TableColumn::new(...).sortable().kind(...)` style; or leave the spreads in place if the migration is mechanically done
    - `Filter` / `ColumnKind` imports need adding to the `use` line at the top of each handler file
    - `PaginatorTrait` needs importing for `.count()` calls
    - The frontend type for date-range filter expects `{from, to}` with exact keys — make sure Rust's `DateRange` struct matches

    **Step 6 — Ensure existing integration/unit tests still pass.** Grep for any CRM test that asserts the old filter-form Container structure:
    ```bash
    grep -rn "filter_container\|filter-form\|filter_form_descendants" backend/crates/crm-demo/tests/ 2>/dev/null
    ```
    Update or delete stale assertions. The migration intentionally breaks tests that asserted the old structure — that's expected, and those assertions should be replaced with assertions about the new DataTable `filters` prop shape.

    **Step 7 — Add `FilterParams` validation + round-trip unit tests (V-07 per 13-VALIDATION.md row 7).**

    Add a `#[cfg(test)] mod tests { ... }` block to `backend/crates/crm-demo/src/handlers/contact.rs` (or extend the existing one if present). Include BOTH a malformed-date rejection test and a full-payload round-trip test:

    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;

        #[test]
        fn contact_filter_params_rejects_bad_date() {
            // V5 Input Validation: malformed date strings inside the date-range
            // filter must either (a) fail to deserialize with a serde error or
            // (b) deserialize to Some(DateRange { from: Some("garbage"), ... })
            // which the SeaORM `.gte(...)` will then push into SQLite as a
            // parameter that compares always-false. Either way the handler
            // must not panic and must not produce SQL injection.
            //
            // This test asserts the strict-deserialize path: an obviously
            // structurally-invalid payload (date is a number, not an object)
            // returns Err.
            let bad_shape = json!({
                "search": "Alice",
                "date": 42  // should be { from, to } object
            });
            let r = serde_json::from_value::<ContactFilterParams>(bad_shape);
            assert!(r.is_err(), "expected deserialize error for malformed date-range shape");

            // Structurally-valid but semantically-bad date strings are accepted
            // at deserialize time (serde doesn't parse the date); they're
            // filtered to parameterized SQL comparisons which never inject.
            // This is the documented behavior per T-13-06-02.
            let bad_date_string = json!({
                "search": "Alice",
                "date": { "from": "not-a-date", "to": "2026-13-01" }
            });
            let parsed: ContactFilterParams = serde_json::from_value(bad_date_string)
                .expect("strings-as-dates should deserialize; SeaORM handles bad values at query time");
            assert!(parsed.date.is_some());
        }

        #[test]
        fn contact_filter_params_deserializes_full_payload() {
            // Round-trip proof that the new struct shape accepts every field
            // the frontend can legitimately send. Covers search, company filter,
            // tag filter, and the collapsed `date` date-range.
            let json = json!({
                "search": "Alice",
                "company_filter": "acme-inc",
                "tag_filter_text": "vip,priority",
                "date": { "from": "2026-01-01", "to": "2026-04-01" }
            });
            let parsed: ContactFilterParams = serde_json::from_value(json)
                .expect("full payload should deserialize");
            assert_eq!(parsed.search.as_deref(), Some("Alice"));
            assert!(parsed.date.is_some());
            let dr = parsed.date.unwrap();
            assert_eq!(dr.from.as_deref(), Some("2026-01-01"));
            assert_eq!(dr.to.as_deref(), Some("2026-04-01"));
        }
    }
    ```

    **IMPORTANT:** The exact struct name (`ContactFilterParams`) and field names must match the real struct you defined earlier in this task. If you named it `ContactFilterPayload` or similar, adapt the test. The test name `contact_filter_params_rejects_bad_date` is REQUIRED (it's the V-07 target per 13-VALIDATION.md row 7) — do not rename it.

    Run the tests:

    ```bash
    cd backend && cargo test -p crm-demo contact_filter_params_rejects_bad_date contact_filter_params_deserializes_full_payload
    ```

    Both MUST pass.
  </action>
  <verify>
    <automated>cd backend && cargo build -p crm-demo && cargo test -p crm-demo 2>&1 | tail -40</automated>
  </verify>
  <acceptance_criteria>
    - `cd backend && cargo build -p crm-demo` exits 0
    - `cd backend && cargo test -p crm-demo` passes all tests
    - `grep -c "\\.source(" backend/crates/crm-demo/src/handlers/audit.rs` == 1
    - `grep -c "\\.source(" backend/crates/crm-demo/src/handlers/contact.rs` >= 1
    - `grep -c "\\.source(" backend/crates/crm-demo/src/handlers/company.rs` >= 1
    - `grep -c "\\.source(" backend/crates/crm-demo/src/handlers/user.rs` >= 1
    - `grep -c "\\.total_rows(" backend/crates/crm-demo/src/handlers/audit.rs` == 1
    - `grep -c "\\.total_rows(" backend/crates/crm-demo/src/handlers/contact.rs` >= 1
    - `grep -c "\\.total_rows(" backend/crates/crm-demo/src/handlers/company.rs` >= 1
    - `grep -c "\\.total_rows(" backend/crates/crm-demo/src/handlers/user.rs` >= 1
    - `grep -c "ColumnKind::Actions" backend/crates/crm-demo/src/handlers/contact.rs` == 1
    - `grep -c "ColumnKind::Actions" backend/crates/crm-demo/src/handlers/company.rs` == 1
    - `grep -c "ColumnKind::Actions" backend/crates/crm-demo/src/handlers/user.rs` == 1
    - `grep -c "filter_form_descendants\|filter_container_descendants" backend/crates/crm-demo/src/handlers/ -r` == 0
    - `grep -c "Filter::text\|Filter::select\|Filter::date_range" backend/crates/crm-demo/src/handlers/ -r` >= 12 (roughly 3 filters avg × 4 handlers)
    - Test `contact_filter_params_rejects_bad_date` exists in `backend/crates/crm-demo/src/handlers/contact.rs` and passes (`cd backend && cargo test -p crm-demo contact_filter_params_rejects_bad_date` exits 0) — satisfies 13-VALIDATION.md row 7 (V5 Input Validation)
    - Test `contact_filter_params_deserializes_full_payload` exists and passes (`cd backend && cargo test -p crm-demo contact_filter_params_deserializes_full_payload` exits 0) — round-trip proof of the new `ContactFilterParams` struct shape
    - `cd backend && cargo clippy -p crm-demo -- -D warnings` exits 0
  </acceptance_criteria>
  <done>All four CRM list handlers migrated; compile green; tests green; actions latent bug fixed via ColumnKind::Actions.</done>
</task>

<task type="auto">
  <name>Task 3: Update spec/PROTOCOL.md data-table example AND add DataTable section to spec/schemas/data.yaml</name>
  <files>spec/PROTOCOL.md, spec/schemas/data.yaml</files>
  <read_first>
    - spec/PROTOCOL.md lines 360-390 (current data-table example and surrounding context)
    - spec/schemas/data.yaml (entire file — 145 lines; understand the existing top-level structure to know where to add the DataTable section)
    - spec/schemas/component.yaml (to confirm `props.additionalProperties: true` already permits the new fields at runtime — this edit is for documentation + future IDE tooling, NOT a runtime fix)
    - Plan 02's example JSON (from the `data_table_phase13_example_serializes_correctly` test) — reference shape
    - .planning/phases/13-datatable-enhancements/13-CONTEXT.md §canonical_refs (D-B2 and D-G1 both say schema additions live in `spec/schemas/data.yaml`)
  </read_first>
  <action>
    Edit `spec/PROTOCOL.md` around line 373. Replace the current minimal `contact-list` data-table example:

    ```yaml
      contact-list:
        type: data-table
        props:
          columns:
            - { key: "name", label: "Name", sortable: true }
            - { key: "email", label: "Email" }
            - { key: "phone", label: "Phone" }
          keyField: "id"
        bind: "/contacts"
        action:
          type: navigate
          idPath: "/id"
    ```

    with an expanded Phase 13 example:

    ```yaml
      contact-list:
        type: data-table
        props:
          columns:
            - { key: "name",    label: "Name",    sortable: true, kind: "text" }
            - { key: "email",   label: "Email",   kind: "text" }
            - { key: "phone",   label: "Phone",   kind: "text" }
            - { key: "created", label: "Created", kind: "date", sortable: true }
            - { key: "actions", label: "",        kind: "actions" }
            - { key: "internal_id", label: "ID",  hidden_default: true }
          filters:
            - { id: "search",         kind: "text",       label: "Search", placeholder: "Filter contacts..." }
            - { id: "company_filter", kind: "select",     label: "Company", options: [{ value: "", label: "All" }, { value: "1", label: "Acme" }] }
            - { id: "date",           kind: "date-range", label: "Created date" }
          total_rows: 237
          row_id_key: "id"
          source: "contact_list"
          page_size: 50
        bind: "/contacts"
        action:
          type: navigate
          idPath: "/id"
    ```

    Add a short prose section immediately after the code block explaining the new fields:

    ```markdown
    **Phase 13 `data-table` props:**

    - `columns[].kind` (optional, default `"text"`): cell render kind. One of `"text" | "badge" | "actions" | "date" | "number"`. The `"actions"` kind expects `row[col.key]` to be an array of `{label, action}` objects and renders a DropdownMenu. Other kinds render via per-kind formatters (`Intl.DateTimeFormat`, `Intl.NumberFormat`, shadcn `Badge`).
    - `columns[].hidden_default` (optional): if `true`, the column starts hidden. Users can toggle it visible via the DataTable's "Columns" dropdown. Per-mount state only — NOT persisted across reloads.
    - `filters[]` (optional): structured filter bar declarations. Each entry is one of `{id, kind: "text", label, placeholder?}`, `{id, kind: "select", label, options}`, or `{id, kind: "date-range", label}`. Filter values are local to the DataTable component (not bound via `/bind`); on change (debounced 300ms for text, immediate for selects), DataTable dispatches `sendAction("filter", { filter_id: value, ... })` with empty values stripped.
    - `total_rows` (optional): total server-side row count. If set, the infinite-scroll sentinel idles once `rows.length >= total_rows`. If unset, the sentinel idles once a `fetch-rows` response returns fewer rows than the requested `limit`.
    - `row_id_key` (optional, default `"id"`): the field on each row object that DataTable uses as the stable row identifier for TanStack's `getRowId`.
    - `source` (optional): identifier passed to the `fetch-rows` action dispatch (`sendAction("fetch-rows", { source, offset, limit })`). The backend's generic `fetch-rows` handler maps this string to a per-screen fetcher (per D-H1).
    ```

    Keep the existing "This flat structure is easy to patch..." paragraph below.

    **Part B — Add a DataTable schema section to `spec/schemas/data.yaml`.**

    CONTEXT.md D-B2 says "New schema dimension lives in `spec/schemas/data.yaml`" and D-G1 says "Schema additions in `spec/schemas/data.yaml` mirror the Rust types." `spec/schemas/component.yaml` line 13 already has `props.additionalProperties: true`, so the runtime protocol validator accepts the new props without any schema edit — but the user's locked decision requires them to be DOCUMENTED in data.yaml for future IDE tooling and for the spec-as-source-of-truth invariant.

    Read `spec/schemas/data.yaml` first. The current file documents patch operations, `KeyedCollection`, and `ValidationError` as top-level definitions. Add a new top-level `DataTable` section at the bottom of the file (before EOF), documenting the Phase 13 prop shape:

    ```yaml
    DataTable:
      description: >-
        Phase 13 `data-table` component props shape. This section documents the
        structured props that DataTable accepts; the runtime validator treats
        `component.props` as `additionalProperties: true` (see component.yaml),
        so these fields are permissive at the protocol layer — this schema is
        the canonical reference for tooling and generated types.
      type: object
      properties:
        columns:
          type: array
          items:
            $ref: "#/DataTableColumn"
          description: Column definitions (required on every DataTable).
        filters:
          type: array
          items:
            $ref: "#/DataTableFilter"
          description: >-
            Optional filter-bar declarations (Phase 13 D-B2). Each entry
            produces a shadcn primitive at render time.
        total_rows:
          type: integer
          minimum: 0
          description: >-
            Optional total row count known server-side (Phase 13 D-D3). When
            set, the frontend sentinel idles once `rows.length >= total_rows`.
        row_id_key:
          type: string
          description: >-
            Optional field name on each row object that DataTable uses as the
            stable row identifier. Defaults to `"id"` on the frontend.
        source:
          type: string
          description: >-
            Optional identifier passed to the `fetch-rows` action dispatch
            (Phase 13 D-H1). The backend's generic `fetch-rows` handler maps
            this string to a per-screen fetcher.
        page_size:
          type: integer
          minimum: 1
          description: Default page size for initial render and pagination chunks.
      required: [columns]
      additionalProperties: false

    DataTableColumn:
      description: >-
        Column definition for a DataTable. Phase 13 adds the `kind` and
        `hidden_default` fields (D-F1, D-E2).
      type: object
      required: [key, label]
      properties:
        key:
          type: string
          description: Row-object field name this column reads.
        label:
          type: string
          description: Visible column header text.
        sortable:
          type: boolean
          description: If true, header click fires a `sort` action.
        kind:
          type: string
          enum: [text, badge, actions, date, number]
          description: >-
            Cell render kind (Phase 13 D-F1). Defaults to `text`. The `actions`
            kind expects `row[col.key]` to be an array of `{label, action}`
            objects and renders a DropdownMenu.
        hidden_default:
          type: boolean
          description: >-
            If true, the column starts hidden on mount. User can toggle it via
            the DataTable's "Columns" dropdown. Per-mount state only, not
            persisted across reloads (D-E1).
      additionalProperties: false

    DataTableFilter:
      description: >-
        Phase 13 filter-bar entry (D-B2). Tagged union keyed by `kind`.
      oneOf:
        - $ref: "#/DataTableFilterText"
        - $ref: "#/DataTableFilterSelect"
        - $ref: "#/DataTableFilterDateRange"
      discriminator:
        propertyName: kind
        mapping:
          text: "#/DataTableFilterText"
          select: "#/DataTableFilterSelect"
          date-range: "#/DataTableFilterDateRange"

    DataTableFilterText:
      type: object
      required: [id, kind]
      properties:
        id:
          type: string
        kind:
          type: string
          const: text
        label:
          type: string
        placeholder:
          type: string
        span:
          type: integer
          minimum: 1
      additionalProperties: false

    DataTableFilterSelect:
      type: object
      required: [id, kind, options]
      properties:
        id:
          type: string
        kind:
          type: string
          const: select
        label:
          type: string
        options:
          type: array
          items:
            type: object
            required: [value, label]
            properties:
              value:
                type: string
              label:
                type: string
            additionalProperties: false
        span:
          type: integer
          minimum: 1
      additionalProperties: false

    DataTableFilterDateRange:
      type: object
      required: [id, kind]
      properties:
        id:
          type: string
        kind:
          type: string
          const: date-range
        label:
          type: string
        span:
          type: integer
          minimum: 1
      additionalProperties: false
    ```

    Append these sections to the END of `spec/schemas/data.yaml`. Do NOT reorder or modify the existing `PatchOperation*`, `KeyedCollection`, or `ValidationError` sections.

    If the existing file already uses a different structure (e.g., all definitions wrapped in a top-level `definitions:` key), adapt the indentation to match — but keep every field name and description exactly as written above.
  </action>
  <verify>
    <automated>grep -q "kind: \"actions\"" spec/PROTOCOL.md && grep -q "total_rows:" spec/PROTOCOL.md && grep -q "source: \"contact_list\"" spec/PROTOCOL.md && grep -q "DataTable:" spec/schemas/data.yaml && grep -q "total_rows" spec/schemas/data.yaml && grep -q "hidden_default" spec/schemas/data.yaml && grep -q "row_id_key" spec/schemas/data.yaml && grep -q "DataTableFilter" spec/schemas/data.yaml</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "kind: \"actions\"" spec/PROTOCOL.md` >= 1
    - `grep -c "total_rows:" spec/PROTOCOL.md` >= 1
    - `grep -c "source: \"contact_list\"" spec/PROTOCOL.md` >= 1
    - `grep -c "row_id_key:" spec/PROTOCOL.md` >= 1
    - `grep -c "filters:" spec/PROTOCOL.md` >= 1
    - `grep -c "Phase 13" spec/PROTOCOL.md` >= 1
    - Existing surrounding paragraphs ("Component types are an open set", "Strict envelope, open props") are NOT removed
    - `grep -q "DataTable:" spec/schemas/data.yaml` (new top-level DataTable section exists — satisfies D-B2 and D-G1)
    - `grep -q "total_rows" spec/schemas/data.yaml`
    - `grep -q "hidden_default" spec/schemas/data.yaml`
    - `grep -q "row_id_key" spec/schemas/data.yaml`
    - `grep -q "DataTableColumn" spec/schemas/data.yaml` (column schema documented)
    - `grep -q "DataTableFilter" spec/schemas/data.yaml` (filter schema documented)
    - `grep -q "date-range" spec/schemas/data.yaml` (filter kind enum documented)
    - Existing `PatchOperation*`, `KeyedCollection`, and `ValidationError` top-level sections are UNCHANGED (edit is purely additive)
  </acceptance_criteria>
  <done>PROTOCOL.md data-table example reflects the new Phase 13 shape with explanatory prose; `spec/schemas/data.yaml` has a new top-level DataTable / DataTableColumn / DataTableFilter schema section per D-B2 and D-G1.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| WebSocket client → list handler filter payload | Untrusted. Payload fields drive SQL WHERE clauses. |
| Filter payload → SeaORM query | Must use parameterized queries. No raw SQL concatenation. |
| `Payload::<FilterPayload>::from_context` | Strongly typed deserialize; malformed payloads produce `ActionError::BadPayload`. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-13-06-01 | Tampering (SQL injection) | Filter `user_id`, `table`, `date_from`, `date_to`, `search`, `company_filter`, `tag_filter_text` fields applied via SeaORM `.filter(col.eq(v))` / `.filter(col.gte(v))` | mitigate | SeaORM generates parameterized queries. No `.raw_sql(...)` anywhere. The `user_id: String` + `.parse::<i32>().ok()` pattern means malformed integers are silently ignored, not appended to SQL. |
| T-13-06-02 | Tampering (Malformed date injection) | `date.from` / `date.to` strings passed to `.gte(string)` | mitigate | SeaORM passes the string as a parameterized query arg. SQLite rejects malformed date strings at comparison time with a comparison-always-false semantic — no injection possible. For defense in depth, a future phase may add `chrono::NaiveDate::parse` validation. Accept for now. |
| T-13-06-03 | Access control bypass via handler-side filter | Non-admin calling `audit_list` directly | accept | Router-level `AuthRequirement::Role("admin")` already gates `audit_list`. `user_list` likewise. `contact_list` and `company_list` are `Authenticated`-only per existing wiring. No changes needed; migration preserves the auth wiring. |
| T-13-06-04 | I (Information disclosure via hidden_default columns) | Hidden columns still transmit data to the client | accept | `hidden_default` is UX, not access control. Documented in the spec (Plan 03 notes this). If a column must not be visible to certain users, exclude it from the row JSON server-side; don't rely on `hidden_default`. |
| T-13-06-05 | Tampering (Unknown filter id silent-drop) | Frontend sending `{ unknown_filter: "x" }` | accept | Rust's `#[derive(Deserialize)]` on `FilterPayload` uses `#[serde(deny_unknown_fields)]`? Check: if NOT set, unknown fields are silently ignored (serde default). Decide per handler — defaulting to accept unknown fields is more permissive but safer for forward-compat. Accept the default. |

No HIGH severity threats. Existing auth + parameterized-query conventions carry over.
</threat_model>

<verification>
```bash
cd backend
cargo test -p marionette --lib builders::standard
cargo build -p crm-demo
cargo test -p crm-demo
cargo clippy -p marionette -p crm-demo -- -D warnings
```

All MUST exit 0.
</verification>

<success_criteria>
- `DataTable` struct has `source: Option<String>` field; inline tests prove serialization
- All four CRM list handlers migrated: inline filters via `Filter::*`, `total_rows` via `.count()`, `source` via `.source()`, `row_id_key("id")`
- `contact.rs`, `company.rs`, `user.rs` have a new `ColumnKind::Actions` column that consumes the existing `actions` row arrays
- No `filter_form_descendants` / `filter_container_descendants` references remain in any handler
- `spec/PROTOCOL.md` data-table example shows the new props with explanatory prose
- `cargo build -p crm-demo` + `cargo test -p crm-demo` green
- Clippy clean on all touched crates
</success_criteria>

<output>
After completion, create `.planning/phases/13-datatable-enhancements/13-06-crm-list-handler-migration-SUMMARY.md` recording:
- The final filter-id → entity column mapping for each handler (so Plan 07's E2E specs can send the right payloads)
- Any CRM test that had to be rewritten due to the filter-form Container going away
- Handler-specific notes (e.g., if contact.rs had a 5th filter that needed special handling, or if company.rs had a nested DataTable for interactions that wasn't migrated because it's not a list screen)
- The `total_rows` values observed during development (anecdote — Wave 4 E2E specs will want > page_size for infinite scroll)
</output>
