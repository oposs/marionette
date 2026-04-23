//! CAT-03 — Data Table catalog screen.
//!
//! Composes one `DataTable` demonstrating filter bar + virtualized infinite
//! scroll + column visibility toggle + per-`ColumnKind` rendering against the
//! shared `fixtures::synthetic_rows(500)` generator from Plan 18-03.
//!
//! Per CONTEXT.md §D-2-B, does NOT invoke the leaf `data_table::gallery_demo()`
//! — the catalog screen is a fresh composition. Locked strings (title, intro,
//! filter ids/labels, column keys/order) are mirrored from UI-SPEC §CAT-03.
//! The DataTable's `.source("catalog-synthetic-rows")` / `.bind("/demo/catalog-data-table/rows")`
//! contract is shared with Plan 18-03's `fetch-rows` handler arm.

use marionette::builders::data_table::{ColumnKind, DataTable, Filter, TableColumn};
use marionette::builders::select::SelectOption;
use marionette::builders::{Container, Heading, Text};
use marionette::gallery::Node;

/// Locked CSS class string (UI-SPEC §Spacing Scale).
const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-data-table", name = "Catalog: Data Table")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // --- Columns (UI-SPEC §CAT-03 lines 472-482) ---
    // 7 columns exercising every ColumnKind variant:
    //   Number (id, score), Text default (name, email), Badge (status),
    //   Date (joined_at), Actions (actions). `status` + `actions` are
    //   hidden_default so the column-visibility dropdown has something to
    //   toggle on visibly on first paint.
    let columns = vec![
        TableColumn::new("id", "ID").kind(ColumnKind::Number),
        TableColumn::new("name", "Name"), // default Text
        TableColumn::new("email", "Email"),
        TableColumn::new("status", "Status")
            .kind(ColumnKind::Badge)
            .hidden_default(true),
        TableColumn::new("score", "Score").kind(ColumnKind::Number),
        TableColumn::new("joined_at", "Joined").kind(ColumnKind::Date),
        TableColumn::new("actions", "")
            .kind(ColumnKind::Actions)
            .hidden_default(true),
    ];

    // --- Filters (UI-SPEC §CAT-03 lines 465-469) ---
    let status_options = vec![
        SelectOption {
            value: "active".into(),
            label: "Active".into(),
        },
        SelectOption {
            value: "inactive".into(),
            label: "Inactive".into(),
        },
        SelectOption {
            value: "pending".into(),
            label: "Pending".into(),
        },
    ];

    // --- DataTable (leaf component; no descendants) ---
    // Contract (Plan 18-03 SUMMARY + PATTERNS.md §CAT-03):
    //   .source("catalog-synthetic-rows") → matches fetch-rows arm (paginated).
    //   .bind("/demo/catalog-data-table/rows") → MANDATORY or DataTable.svelte
    //     reads `{}` and renders zero rows (G-03 lesson).
    //   .row_id_key("id") → matches Row.id on fixtures::Row.
    //   .total_rows(500) → frontend stops fetching once rows.length >= 500.
    //   .page_size(50) → IntersectionObserver sentinel sends limit=50.
    let table = DataTable::new(columns)
        .id("catalog-data-table-root")
        .source("catalog-synthetic-rows")
        .bind("/demo/catalog-data-table/rows")
        .row_id_key("id")
        .page_size(50u32)
        .total_rows(500u64)
        .filter(
            Filter::text("name-search")
                .label("Name")
                .placeholder("Filter by name…"),
        )
        .filter(Filter::select("status-filter", status_options).label("Status"))
        .filter(Filter::date_range("joined-range").label("Joined"))
        .build();

    // --- Title + intro (locked copy from UI-SPEC §Copywriting Contract) ---
    let title = Heading::new("Data Table")
        .id("catalog-data-table-title")
        .level(1)
        .build();
    let intro = Text::new(
        "Filter bar, virtualized infinite scroll, column visibility, and \
         per-ColumnKind rendering against 500 synthetic rows. Scroll to the \
         bottom to trigger fetch-rows pagination; toggle columns from the \
         header dropdown.",
    )
    .id("catalog-data-table-intro")
    .build();

    // --- Outer root Container: title + intro + table ---
    // DataTable is a leaf (no descendants to flatten); Container's
    // `build_with_children` emits [container-root, title, intro, table] in
    // the exact order passed to `.children(...)`.
    Container::new()
        .id("catalog-data-table-container")
        .class(OUTER_CLASS)
        .children(vec![title, intro, table])
        .build_with_children()
}

#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;

    /// Locate the `DataTable` component tuple in the rendered tree and return
    /// its serialized JSON for field-level assertions.
    fn find_table(v: &[Node]) -> serde_json::Value {
        let (_id, comp) = v
            .iter()
            .find(|(id, _)| id == "catalog-data-table-root")
            .expect("data-table root node present");
        serde_json::to_value(comp).expect("serialize DataTable")
    }

    #[test]
    fn root_id_is_catalog_data_table_container() {
        let v = gallery_demo();
        assert_eq!(
            v[0].0, "catalog-data-table-container",
            "first entry is the outer container root"
        );
    }

    #[test]
    fn datatable_source_and_bind_match_lock() {
        let v = gallery_demo();
        let t = find_table(&v);
        assert_eq!(t["type"], "data-table");
        assert_eq!(t["props"]["source"], "catalog-synthetic-rows");
        assert_eq!(t["bind"], "/demo/catalog-data-table/rows");
    }

    #[test]
    fn datatable_has_seven_columns_in_lockorder() {
        let v = gallery_demo();
        let t = find_table(&v);
        let cols = t["props"]["columns"].as_array().expect("columns array");
        assert_eq!(cols.len(), 7);
        let keys: Vec<&str> = cols
            .iter()
            .map(|c| c["key"].as_str().unwrap_or(""))
            .collect();
        assert_eq!(
            keys,
            vec!["id", "name", "email", "status", "score", "joined_at", "actions"]
        );
    }

    #[test]
    fn columnkinds_match_lock() {
        let v = gallery_demo();
        let t = find_table(&v);
        let cols = t["props"]["columns"].as_array().unwrap();
        // id + score → Number; status → Badge; joined_at → Date; actions → Actions.
        // name + email default (Text) — `kind` field is omitted on the JSON
        // wire (TableColumn serializes None-kind as absent field per
        // `#[serde(skip_serializing_if = "Option::is_none")]`).
        let by_key = |k: &str| cols.iter().find(|c| c["key"] == k).cloned().unwrap();
        assert_eq!(by_key("id")["kind"], "number");
        assert_eq!(by_key("score")["kind"], "number");
        assert_eq!(by_key("status")["kind"], "badge");
        assert_eq!(by_key("joined_at")["kind"], "date");
        assert_eq!(by_key("actions")["kind"], "actions");
        // Text-default columns: kind is absent on the wire.
        assert!(by_key("name")
            .get("kind")
            .is_none_or(serde_json::Value::is_null));
        assert!(by_key("email")
            .get("kind")
            .is_none_or(serde_json::Value::is_null));
        // hidden_default flags
        assert_eq!(by_key("status")["hidden_default"], true);
        assert_eq!(by_key("actions")["hidden_default"], true);
        // Non-hidden columns: hidden_default is absent on the wire.
        assert!(by_key("name")
            .get("hidden_default")
            .is_none_or(serde_json::Value::is_null));
    }

    #[test]
    fn three_filters_with_lockshape() {
        let v = gallery_demo();
        let t = find_table(&v);
        let filters = t["props"]["filters"].as_array().expect("filters array");
        assert_eq!(filters.len(), 3);
        // Filter 1: Text — name-search
        assert_eq!(filters[0]["kind"], "text");
        assert_eq!(filters[0]["id"], "name-search");
        assert_eq!(filters[0]["label"], "Name");
        assert_eq!(filters[0]["placeholder"], "Filter by name…");
        // Filter 2: Select — status-filter with 3 options
        assert_eq!(filters[1]["kind"], "select");
        assert_eq!(filters[1]["id"], "status-filter");
        assert_eq!(filters[1]["label"], "Status");
        let opts = filters[1]["options"].as_array().unwrap();
        assert_eq!(opts.len(), 3);
        assert_eq!(opts[0]["value"], "active");
        assert_eq!(opts[0]["label"], "Active");
        assert_eq!(opts[1]["value"], "inactive");
        assert_eq!(opts[1]["label"], "Inactive");
        assert_eq!(opts[2]["value"], "pending");
        assert_eq!(opts[2]["label"], "Pending");
        // Filter 3: DateRange — joined-range (kebab kind)
        assert_eq!(filters[2]["kind"], "date-range");
        assert_eq!(filters[2]["id"], "joined-range");
        assert_eq!(filters[2]["label"], "Joined");
    }

    #[test]
    fn total_rows_500_and_page_size_50() {
        let v = gallery_demo();
        let t = find_table(&v);
        assert_eq!(t["props"]["total_rows"], 500);
        assert_eq!(t["props"]["page_size"], 50);
        assert_eq!(t["props"]["row_id_key"], "id");
    }

    #[test]
    fn registered_demos_includes_catalog_data_table() {
        let e = registered_demos()
            .find(|e| e.key == "catalog-data-table")
            .expect("catalog-data-table must be registered via linkme");
        assert_eq!(e.display_name, "Catalog: Data Table");
        let rendered = (e.render)();
        assert_eq!(rendered[0].0, "catalog-data-table-container");
    }

    #[test]
    fn tree_contains_title_and_intro_children() {
        // Outer container's `build_with_children` produces the flat
        // [container-root, title, intro, table] sequence.
        let v = gallery_demo();
        assert_eq!(v.len(), 4, "container + title + intro + table = 4 nodes");
        let ids: Vec<&str> = v.iter().map(|(id, _)| id.as_str()).collect();
        assert!(ids.contains(&"catalog-data-table-title"));
        assert!(ids.contains(&"catalog-data-table-intro"));
        assert!(ids.contains(&"catalog-data-table-root"));
    }
}
