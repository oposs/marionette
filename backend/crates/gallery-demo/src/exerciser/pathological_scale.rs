//! EXER-03 — Pathological Scale exerciser (Plan 19-04).
//!
//! 3 Cards: Perf readouts (4 signals), Pathological DataTable (10 000 rows),
//! Pathological FormScreen (80 fields in 4 FieldSet groups).
//!
//! Per 19-CONTEXT.md §D-3: signals are captured as advisory baselines — missing
//! a threshold is a finding, not a verification failure.
//!
//! # Column-set fidelity
//! The 10 000-row DataTable mirrors the column list from
//! `backend/crates/gallery-demo/src/catalog/data_table.rs` verbatim so the
//! shared `fixtures::synthetic_rows` generator stays compatible. Only `source`,
//! `bind`, `id`, and `total_rows` differ from CAT-03.
//!
//! # Plan-to-reality deviations (Rule 3 blocking fixes)
//!
//! - The plan's action script referenced `Badge::new(...)` for the perf
//!   status pill. No `Badge` builder exists in `marionette::builders` — Badge
//!   is only a DataTable ColumnKind variant. Substituted with a `Text`
//!   component whose value is bound to `/demo/exer-03/perf/{slug}/badge`.
//!   The backend handler (Plan 19-04 Task 2) writes `"WITHIN TARGET"` /
//!   `"OVER TARGET"` strings to that path — rendering as plain text satisfies
//!   the UI-SPEC contract without inventing a new component.
//! - The plan's action script called `FieldSet::new("Personal info")` as a
//!   positional constructor. The actual derive-generated `FieldSet::new()`
//!   takes no arguments; the legend is set via `.legend(...)`. Adapted here.

use marionette::builders::data_table::{ColumnKind, DataTable, Filter, TableColumn};
use marionette::builders::radio_group::RadioOption;
use marionette::builders::select::SelectOption;
use marionette::builders::{
    Button, Checkbox, Container, FieldSeparator, FieldSet, Heading, RadioGroup, Select, Switch,
    Text, TextInput, Textarea,
};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

// -- Locked CSS class strings (UI-SPEC §Spacing Scale / §EXER-03) ----------

const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
const PERF_GRID_CLASS: &str = "grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3";
const PERF_CELL_CLASS: &str =
    "rounded-md border border-border bg-card p-3 flex flex-col gap-2 items-start";
const CTA_ROW_CLASS: &str = "flex justify-end";

// -- Top-level demo fn -----------------------------------------------------

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "exer-03", name = "Exerciser: Pathological Scale")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    let title = Heading::new("Pathological Scale")
        .id("exer-03-title")
        .level(1)
        .build();
    let intro = Text::new(
        "A single page mounting a DataTable with 10 000 synthetic rows AND a \
         FormScreen with 80 synthetic fields. Captures four advisory \
         performance signals — time-to-first-paint, scroll FPS, JS heap size, \
         patch-apply latency — as baselines. Targets are advisory, not \
         gating: missing a threshold is a finding, not a verification failure.",
    )
    .id("exer-03-intro")
    .build();

    let (c1_root, c1_desc) = build_perf_readouts_card();
    let (c2_root, c2_desc) = build_pathological_data_table_card();
    let (c3_root, c3_desc) = build_pathological_form_screen_card();

    let (outer, outer_direct) = Container::new()
        .id("exer-03-root")
        .class(OUTER_CLASS)
        .children(vec![title, intro, c1_root, c2_root, c3_root])
        .build_tree();

    let mut out = Vec::with_capacity(
        1 + outer_direct.len() + c1_desc.len() + c2_desc.len() + c3_desc.len(),
    );
    out.push(outer);
    out.extend(outer_direct);
    out.extend(c1_desc);
    out.extend(c2_desc);
    out.extend(c3_desc);
    out
}

// -- Card 1: 4-cell perf banner + Remeasure CTA ----------------------------

fn build_perf_readouts_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Perf readouts")
        .id("exer-03-c1-h2")
        .level(2)
        .build();

    let remeasure = Button::new("Remeasure")
        .id("exer-03-remeasure")
        .icon("rotate-ccw")
        .action(ComponentAction::click("gallery-demo/exer-03/remeasure"))
        .build();
    let (cta_row, cta_row_desc) = Container::new()
        .id("exer-03-remeasure-row")
        .class(CTA_ROW_CLASS)
        .children(vec![remeasure])
        .build_tree();

    let cell_ttfp = build_perf_cell(
        "exer-03-perf-ttfp",
        "TTFP",
        "timer",
        "ms",
        "≤ 3000 ms",
        "ttfp_ms",
    );
    let cell_fps = build_perf_cell(
        "exer-03-perf-fps",
        "Scroll FPS",
        "gauge",
        "fps",
        "≥ 30 fps",
        "fps",
    );
    let cell_memory = build_perf_cell(
        "exer-03-perf-memory",
        "Memory growth",
        "cpu",
        "MB",
        "≤ +50 MB after 30 s scroll",
        "memory_mb",
    );
    let cell_latency = build_perf_cell(
        "exer-03-perf-latency",
        "Patch latency p95",
        "zap",
        "ms",
        "p95 ≤ 50 ms",
        "latency_p95_ms",
    );

    let (grid_root, grid_desc) = Container::new()
        .id("exer-03-perf-grid")
        .class(PERF_GRID_CLASS)
        .children(vec![
            cell_ttfp.0,
            cell_fps.0,
            cell_memory.0,
            cell_latency.0,
        ])
        .build_tree();

    let mut cells_desc = Vec::new();
    cells_desc.extend(cell_ttfp.1);
    cells_desc.extend(cell_fps.1);
    cells_desc.extend(cell_memory.1);
    cells_desc.extend(cell_latency.1);

    let (card_root, card_direct) = Container::new()
        .id("exer-03-c1-perf-readouts")
        .class(CARD_CLASS)
        .children(vec![heading, cta_row, grid_root])
        .build_tree();

    let mut desc = Vec::with_capacity(
        card_direct.len() + cta_row_desc.len() + grid_desc.len() + cells_desc.len(),
    );
    desc.extend(card_direct);
    desc.extend(cta_row_desc);
    desc.extend(grid_desc);
    desc.extend(cells_desc);
    (card_root, desc)
}

/// Build one perf readout cell.
///
/// The cell root `id` is the stable anchor that tests assert against
/// (`exer-03-perf-{ttfp,fps,memory,latency}`). Inside, a small stack of:
/// icon container, label heading, large value Text (bound to
/// `/demo/exer-03/perf/{slug}/value`), unit Text, target-copy Text, and a
/// status-pill Text bound to `/demo/exer-03/perf/{slug}/badge` (renders
/// "WITHIN TARGET" / "OVER TARGET" / "PENDING").
fn build_perf_cell(
    id: &str,
    label: &str,
    icon: &str,
    unit: &str,
    target_copy: &str,
    bind_slug: &str,
) -> (Node, Vec<Node>) {
    let icon_node = Container::new()
        .id(format!("{id}-icon"))
        .icon(icon)
        .build();
    let h4 = Heading::new(label)
        .id(format!("{id}-label"))
        .level(4)
        .build();

    // Large numeric — bound to /demo/exer-03/perf/{bind_slug}/value.
    let value = Text::new("—")
        .id(format!("{id}-value"))
        .bind(format!("/demo/exer-03/perf/{bind_slug}/value"))
        .build();

    let unit_text = Text::new(unit).id(format!("{id}-unit")).build();
    let target_text = Text::new(target_copy).id(format!("{id}-target")).build();

    // Status pill — Text bound to the backend-written badge string. The
    // handler writes "WITHIN TARGET" / "OVER TARGET"; pre-report state
    // is seeded to null in Plan 19-01's seed_exer_03.
    let badge = Text::new("PENDING")
        .id(format!("{id}-badge"))
        .bind(format!("/demo/exer-03/perf/{bind_slug}/badge"))
        .build();

    Container::new()
        .id(id)
        .class(PERF_CELL_CLASS)
        .children(vec![icon_node, h4, value, unit_text, target_text, badge])
        .build_tree()
}

// -- Card 2: 10 000-row Pathological DataTable -----------------------------

fn build_pathological_data_table_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Pathological DataTable")
        .id("exer-03-c2-h2")
        .level(2)
        .build();
    let intro = Text::new(
        "10 000 synthetic rows. Virtualization must keep scroll responsive; \
         the filter must stay performant.",
    )
    .id("exer-03-c2-intro")
    .build();

    // Column set mirrored verbatim from catalog/data_table.rs — the
    // `synthetic_rows` fixture emits these exact keys and kinds, so
    // pagination via fetch-rows (source="exer-03-synthetic") works without
    // row-shape adaptation.
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

    let table = DataTable::new(columns)
        .id("exer-03-data-table")
        .source("exer-03-synthetic")
        .bind("/demo/exer-03/rows")
        .row_id_key("id")
        .page_size(50u32)
        .total_rows(10_000u64)
        .filter(
            Filter::text("name-search")
                .label("Name")
                .placeholder("Filter by name…"),
        )
        .filter(Filter::select("status-filter", status_options).label("Status"))
        .filter(Filter::date_range("joined-range").label("Joined"))
        .build();

    Container::new()
        .id("exer-03-c2-data-table")
        .class(CARD_CLASS)
        .children(vec![heading, intro, table])
        .build_tree()
}

// -- Card 3: 80-field FormScreen across 4 FieldSet groups ------------------

fn build_pathological_form_screen_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Pathological FormScreen")
        .id("exer-03-c3-h2")
        .level(2)
        .build();
    let intro = Text::new(
        "80 synthetic fields in 4 FieldSet groups. Rendering, focus-flow, and \
         scroll-anchoring all under pressure.",
    )
    .id("exer-03-c3-intro")
    .build();

    let (fs1_root, fs1_desc) = build_personal_info_group();
    let sep1 = FieldSeparator::new().id("exer-03-sep-1").build();
    let (fs2_root, fs2_desc) = build_contact_group();
    let sep2 = FieldSeparator::new().id("exer-03-sep-2").build();
    let (fs3_root, fs3_desc) = build_preferences_group();
    let sep3 = FieldSeparator::new().id("exer-03-sep-3").build();
    let (fs4_root, fs4_desc) = build_advanced_group();

    let (card_root, card_direct) = Container::new()
        .id("exer-03-c3-form-screen")
        .class(CARD_CLASS)
        .children(vec![
            heading, intro, fs1_root, sep1, fs2_root, sep2, fs3_root, sep3, fs4_root,
        ])
        .build_tree();

    let mut desc = Vec::with_capacity(
        card_direct.len() + fs1_desc.len() + fs2_desc.len() + fs3_desc.len() + fs4_desc.len(),
    );
    desc.extend(card_direct);
    desc.extend(fs1_desc);
    desc.extend(fs2_desc);
    desc.extend(fs3_desc);
    desc.extend(fs4_desc);
    (card_root, desc)
}

fn build_personal_info_group() -> (Node, Vec<Node>) {
    // 15 TextInput + 2 Select + 2 RadioGroup + 1 Textarea = 20.
    // Field names mirror seed_exer_03() in handlers/show.rs (Plan 19-01).
    let text_input_names = [
        "first-name",
        "last-name",
        "middle-name",
        "preferred-name",
        "nickname",
        "birthdate-text",
        "gender-text",
        "pronouns",
        "nationality",
        "languages",
        "ethnicity",
        "religion",
        "marital-status",
        "dependents-count",
        "emergency-contact",
    ];
    let select_names = ["salutation", "title"];
    let radio_names = ["primary-language", "secondary-language"];
    let textarea_names = ["bio"];

    let mut fields: Vec<Node> = Vec::with_capacity(20);
    for name in text_input_names {
        fields.push(
            TextInput::new(labelize(name))
                .id(format!("exer-03-personal-info-{name}"))
                .placeholder(format!("Placeholder for {}", labelize(name)))
                .bind(format!("/demo/exer-03/personal-info/{name}"))
                .build(),
        );
    }
    for name in select_names {
        fields.push(
            Select::new(labelize(name), sample_select_options())
                .id(format!("exer-03-personal-info-{name}"))
                .bind(format!("/demo/exer-03/personal-info/{name}"))
                .build(),
        );
    }
    for name in radio_names {
        fields.push(
            RadioGroup::new(labelize(name), sample_radio_options())
                .id(format!("exer-03-personal-info-{name}"))
                .bind(format!("/demo/exer-03/personal-info/{name}"))
                .build(),
        );
    }
    for name in textarea_names {
        fields.push(
            Textarea::new(labelize(name))
                .id(format!("exer-03-personal-info-{name}"))
                .bind(format!("/demo/exer-03/personal-info/{name}"))
                .build(),
        );
    }

    FieldSet::new()
        .id("exer-03-fieldset-personal-info")
        .legend("Personal info")
        .cols(2u8)
        .children(fields)
        .build_tree()
}

fn build_contact_group() -> (Node, Vec<Node>) {
    // 12 TextInput + 2 Select + 4 Checkbox + 2 Textarea = 20.
    let text_input_names = [
        "email-primary",
        "email-secondary",
        "phone-mobile",
        "phone-home",
        "phone-work",
        "address-line-1",
        "address-line-2",
        "city",
        "state-region",
        "postal-code",
        "country-text",
        "fax",
    ];
    let select_names = ["preferred-method", "timezone"];
    let checkbox_names = [
        "consent-email",
        "consent-sms",
        "consent-phone",
        "consent-mail",
    ];
    let textarea_names = ["address-notes", "communication-notes"];

    let mut fields: Vec<Node> = Vec::with_capacity(20);
    for name in text_input_names {
        fields.push(
            TextInput::new(labelize(name))
                .id(format!("exer-03-contact-{name}"))
                .placeholder(format!("Placeholder for {}", labelize(name)))
                .bind(format!("/demo/exer-03/contact/{name}"))
                .build(),
        );
    }
    for name in select_names {
        fields.push(
            Select::new(labelize(name), sample_select_options())
                .id(format!("exer-03-contact-{name}"))
                .bind(format!("/demo/exer-03/contact/{name}"))
                .build(),
        );
    }
    for name in checkbox_names {
        fields.push(
            Checkbox::new(labelize(name))
                .id(format!("exer-03-contact-{name}"))
                .bind(format!("/demo/exer-03/contact/{name}"))
                .build(),
        );
    }
    for name in textarea_names {
        fields.push(
            Textarea::new(labelize(name))
                .id(format!("exer-03-contact-{name}"))
                .bind(format!("/demo/exer-03/contact/{name}"))
                .build(),
        );
    }

    FieldSet::new()
        .id("exer-03-fieldset-contact")
        .legend("Contact")
        .cols(2u8)
        .children(fields)
        .build_tree()
}

fn build_preferences_group() -> (Node, Vec<Node>) {
    // 5 Select + 8 Switch + 4 RadioGroup + 3 Checkbox = 20.
    let select_names = ["theme", "density", "language", "timezone-pref", "currency"];
    let switch_names = [
        "notif-email",
        "notif-sms",
        "notif-push",
        "notif-weekly",
        "notif-monthly",
        "notif-marketing",
        "notif-security",
        "notif-updates",
    ];
    let radio_names = ["frequency", "privacy", "sharing", "visibility"];
    let checkbox_names = ["terms", "privacy-policy", "marketing"];

    let mut fields: Vec<Node> = Vec::with_capacity(20);
    for name in select_names {
        fields.push(
            Select::new(labelize(name), sample_select_options())
                .id(format!("exer-03-preferences-{name}"))
                .bind(format!("/demo/exer-03/preferences/{name}"))
                .build(),
        );
    }
    for name in switch_names {
        fields.push(
            Switch::new(labelize(name))
                .id(format!("exer-03-preferences-{name}"))
                .bind(format!("/demo/exer-03/preferences/{name}"))
                .build(),
        );
    }
    for name in radio_names {
        fields.push(
            RadioGroup::new(labelize(name), sample_radio_options())
                .id(format!("exer-03-preferences-{name}"))
                .bind(format!("/demo/exer-03/preferences/{name}"))
                .build(),
        );
    }
    for name in checkbox_names {
        fields.push(
            Checkbox::new(labelize(name))
                .id(format!("exer-03-preferences-{name}"))
                .bind(format!("/demo/exer-03/preferences/{name}"))
                .build(),
        );
    }

    FieldSet::new()
        .id("exer-03-fieldset-preferences")
        .legend("Preferences")
        .cols(2u8)
        .children(fields)
        .build_tree()
}

fn build_advanced_group() -> (Node, Vec<Node>) {
    // 10 TextInput + 4 Textarea + 3 Select + 2 Switch + 1 Checkbox = 20.
    let text_inputs: Vec<String> = (1..=10).map(|n| format!("api-key-alias-{n}")).collect();
    let textareas = ["notes-1", "notes-2", "notes-3", "notes-4"];
    let selects = ["access-level", "rotation-policy", "audit-mode"];
    let switches = ["mfa-enabled", "biometric-enabled"];
    let checkboxes = ["danger-ack"];

    let mut fields: Vec<Node> = Vec::with_capacity(20);
    for name in &text_inputs {
        fields.push(
            TextInput::new(labelize(name))
                .id(format!("exer-03-advanced-{name}"))
                .placeholder(format!("Placeholder for {}", labelize(name)))
                .bind(format!("/demo/exer-03/advanced/{name}"))
                .build(),
        );
    }
    for name in textareas {
        fields.push(
            Textarea::new(labelize(name))
                .id(format!("exer-03-advanced-{name}"))
                .bind(format!("/demo/exer-03/advanced/{name}"))
                .build(),
        );
    }
    for name in selects {
        fields.push(
            Select::new(labelize(name), sample_select_options())
                .id(format!("exer-03-advanced-{name}"))
                .bind(format!("/demo/exer-03/advanced/{name}"))
                .build(),
        );
    }
    for name in switches {
        fields.push(
            Switch::new(labelize(name))
                .id(format!("exer-03-advanced-{name}"))
                .bind(format!("/demo/exer-03/advanced/{name}"))
                .build(),
        );
    }
    for name in checkboxes {
        fields.push(
            Checkbox::new(labelize(name))
                .id(format!("exer-03-advanced-{name}"))
                .bind(format!("/demo/exer-03/advanced/{name}"))
                .build(),
        );
    }

    FieldSet::new()
        .id("exer-03-fieldset-advanced")
        .legend("Advanced")
        .cols(2u8)
        .children(fields)
        .build_tree()
}

// -- Helpers ---------------------------------------------------------------

/// Slug-to-label: `"first-name"` → `"First name"`.
fn labelize(slug: &str) -> String {
    let spaced = slug.replace('-', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn sample_select_options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "option-a".into(),
            label: "Option A".into(),
        },
        SelectOption {
            value: "option-b".into(),
            label: "Option B".into(),
        },
        SelectOption {
            value: "option-c".into(),
            label: "Option C".into(),
        },
    ]
}

fn sample_radio_options() -> Vec<RadioOption> {
    vec![
        RadioOption {
            value: "yes".into(),
            label: "Yes".into(),
            description: None,
        },
        RadioOption {
            value: "no".into(),
            label: "No".into(),
            description: None,
        },
        RadioOption {
            value: "maybe".into(),
            label: "Maybe".into(),
            description: None,
        },
    ]
}

// -- Tests -----------------------------------------------------------------

#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;
    use std::collections::HashSet;

    #[test]
    fn root_id_is_exer_03_root() {
        let v = gallery_demo();
        assert_eq!(v[0].0, "exer-03-root");
    }

    #[test]
    fn data_table_has_10k_rows_and_correct_source() {
        let v = gallery_demo();
        let (_, table_comp) = v
            .iter()
            .find(|(id, _)| id == "exer-03-data-table")
            .expect("data table node present");
        let json = serde_json::to_value(table_comp).expect("serialize");
        assert_eq!(json["props"]["source"], "exer-03-synthetic");
        assert_eq!(json["props"]["total_rows"], 10_000);
        assert_eq!(json["bind"], "/demo/exer-03/rows");
        assert_eq!(json["props"]["row_id_key"], "id");
        assert_eq!(json["props"]["page_size"], 50);
    }

    #[test]
    fn four_perf_readout_cells_present() {
        let v = gallery_demo();
        for id in [
            "exer-03-perf-ttfp",
            "exer-03-perf-fps",
            "exer-03-perf-memory",
            "exer-03-perf-latency",
        ] {
            assert!(
                v.iter().any(|(nid, _)| nid == id),
                "missing perf cell {id}"
            );
        }
    }

    #[test]
    fn four_field_set_groups_present() {
        let v = gallery_demo();
        for id in [
            "exer-03-fieldset-personal-info",
            "exer-03-fieldset-contact",
            "exer-03-fieldset-preferences",
            "exer-03-fieldset-advanced",
        ] {
            assert!(
                v.iter().any(|(nid, _)| nid == id),
                "missing FieldSet group {id}"
            );
        }
    }

    #[test]
    fn eighty_form_fields_with_unique_binds() {
        let v = gallery_demo();
        let mut binds: Vec<String> = Vec::new();
        for (_, comp) in &v {
            let json = serde_json::to_value(comp).expect("serialize");
            if let Some(b) = json["bind"].as_str()
                && b.starts_with("/demo/exer-03/")
                && !b.starts_with("/demo/exer-03/perf/")
                && b != "/demo/exer-03/rows"
            {
                binds.push(b.to_string());
            }
        }
        assert_eq!(
            binds.len(),
            80,
            "expected 80 form-field binds, got {} — check FieldSet breakdown",
            binds.len()
        );
        let unique: HashSet<&String> = binds.iter().collect();
        assert_eq!(
            unique.len(),
            80,
            "bind paths must be unique across FieldSets"
        );
    }

    #[test]
    fn remeasure_cta_has_correct_action() {
        let v = gallery_demo();
        let (_, comp) = v
            .iter()
            .find(|(id, _)| id == "exer-03-remeasure")
            .expect("remeasure button");
        let json = serde_json::to_value(comp).expect("serialize");
        assert_eq!(json["type"], "button");
        assert_eq!(json["action"]["type"], "click");
        assert_eq!(json["action"]["name"], "gallery-demo/exer-03/remeasure");
    }

    #[test]
    fn registered_demos_includes_exer_03() {
        let e = registered_demos()
            .find(|e| e.key == "exer-03")
            .expect("exer-03 registered via linkme");
        assert_eq!(e.display_name, "Exerciser: Pathological Scale");
        let rendered = (e.render)();
        assert_eq!(rendered[0].0, "exer-03-root");
    }

    #[test]
    fn perf_cell_value_binds_to_expected_paths() {
        // Each cell embeds a Text whose bind path is the canonical source for
        // the backend's report-perf handler to write into.
        let v = gallery_demo();
        let expected_binds = [
            "/demo/exer-03/perf/ttfp_ms/value",
            "/demo/exer-03/perf/fps/value",
            "/demo/exer-03/perf/memory_mb/value",
            "/demo/exer-03/perf/latency_p95_ms/value",
        ];
        for bind in expected_binds {
            let found = v.iter().any(|(_, comp)| {
                let json = serde_json::to_value(comp).expect("serialize");
                json["bind"].as_str() == Some(bind)
            });
            assert!(found, "no component bound to {bind}");
        }
    }

    #[test]
    fn field_set_groups_use_two_columns() {
        let v = gallery_demo();
        for id in [
            "exer-03-fieldset-personal-info",
            "exer-03-fieldset-contact",
            "exer-03-fieldset-preferences",
            "exer-03-fieldset-advanced",
        ] {
            let (_, comp) = v
                .iter()
                .find(|(nid, _)| nid == id)
                .unwrap_or_else(|| panic!("{id} present"));
            let json = serde_json::to_value(comp).expect("serialize");
            assert_eq!(json["props"]["cols"], 2, "{id} should use cols=2");
        }
    }
}
