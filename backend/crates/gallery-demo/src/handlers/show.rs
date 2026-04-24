//! `gallery-show` handler — single entry point for every nav click.
//!
//! Payload `{ "key": "<demo-key>" }`. Handler looks up the DemoEntry,
//! invokes `(entry.render)()` returning `Vec<Node>` (post-D-Z1), packs
//! into a HashMap, seeds any needed /demo/{key}/... state into the
//! Render's data field, and emits a Render to the `content` sub-surface.

use std::collections::HashMap;

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette::gallery::registered_demos;
use marionette_protocol::messages::RenderMessage;
use marionette_protocol::{Component, ProtocolMessage};

#[allow(clippy::unused_async)]
pub async fn handle_gallery_show(ctx: HandlerContext) -> ActionResult {
    let key = ctx
        .action
        .payload
        .as_ref()
        .and_then(|p| p.get("key"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ActionError::BadPayload("gallery-show requires `key` field".into()))?
        .to_string();

    let entry = registered_demos()
        .find(|e| e.key == key)
        .ok_or_else(|| ActionError::BadPayload(format!("unknown gallery demo '{key}'")))?;

    let nodes_vec = (entry.render)();
    if nodes_vec.is_empty() {
        return Err(ActionError::BadPayload(format!(
            "gallery demo '{key}' returned empty Vec<Node>"
        )));
    }
    let root_id = nodes_vec[0].0.clone();
    let nodes_map: HashMap<String, Component> = nodes_vec.into_iter().collect();

    let data = seed_for_key(&key);

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        root: root_id,
        nodes: nodes_map,
        data,
    })])
}

/// Per-demo state seeds (CONTEXT.md §D-D1 + §D-D2).
///
/// Paths under `/demo/{key}/...` seed the data store so bindings in the
/// demo's Component tree resolve to sensible initial values. Unknown keys
/// yield an empty seed — fine for pure-visual leaves (Heading, Text, Spinner).
///
/// Clippy: the function is a registry-style dispatch with short parallel
/// arms; splitting it would obscure the per-key seed correspondence.
#[allow(clippy::too_many_lines)]
fn seed_for_key(key: &str) -> serde_json::Value {
    // Explicit `catalog-*` arms that return empty JSON are deliberate
    // documentation — the wildcard also returns empty, but naming the
    // known zero-state catalog keys prevents accidental seed drift as
    // future catalog plans land (18-05..18-08). Allow the lint locally.
    #[allow(clippy::match_same_arms)]
    match key {
        "text-input" => serde_json::json!({ "demo": { "text-input": { "value": "" } } }),
        "select" => serde_json::json!({ "demo": { "select":     { "value": "" } } }),
        "checkbox" => serde_json::json!({ "demo": { "checkbox":   { "checked": false } } }),
        // switch demo binds /demo/switch/checked-1 + /demo/switch/checked-2 (see
        // marionette/src/builders/switch.rs::gallery_demo). Pre-17-06 seed wrote
        // only `/demo/switch/checked` (G-05 path mismatch). Seed both now, with
        // Wifi on and Bluetooth off for a visually distinct initial state.
        "switch" => serde_json::json!({
            "demo": { "switch": { "checked-1": true, "checked-2": false } }
        }),
        "radio-group" => serde_json::json!({ "demo": { "radio-group":{ "value": "" } } }),
        // textarea demo binds /demo/textarea/value + /demo/textarea/value-desc
        // (see marionette/src/builders/textarea.rs::gallery_demo). Pre-17-06
        // seed only wrote `/demo/textarea/value` (G-05 path mismatch on the
        // second textarea). Seed both empty so the Field.Label stays bound.
        "textarea" => serde_json::json!({
            "demo": { "textarea": { "value": "", "value-desc": "" } }
        }),
        // error-display demo binds /demo/error-display/errors-a +
        // /demo/error-display/errors-b (added in 17-06 to fix G-05: the
        // pre-17-06 demo omitted .bind(...) entirely, so the frontend's
        // `{#if errors.length > 0}` guard failed and nothing rendered).
        // ErrorEntry = { path?: string, message: string } per frontend
        // ErrorDisplay.svelte:26-41.
        "error-display" => serde_json::json!({
            "demo": {
                "error-display": {
                    "errors-a": [
                        { "path": "/contact/email", "message": "Email is required" },
                        { "path": "/contact/phone", "message": "Phone number is invalid" },
                    ],
                    "errors-b": [
                        { "message": "A system-level error (no path)" },
                    ],
                }
            }
        }),
        "form" => serde_json::json!({ "demo": { "form":       { "email": "", "name": "" } } }),
        "field-set" => serde_json::json!({ "demo": { "field-set":  { "a": "", "b": "" } } }),
        "data-table" => serde_json::json!({ "demo": { "data-table": { "rows": seed_table_rows() } } }),
        // Phase 18 Plan 04 (CAT-01): pure-visual screen; no bind paths are read
        // by catalog/buttons.rs (the matrix fires `gallery-demo/noop` on click
        // but reads no surface data). Empty seed is the correct zero-state.
        "catalog-buttons" => serde_json::json!({}),
        // Phase 18 Plan 05 (CAT-02): seed table locked in UI-SPEC §CAT-02
        // lines 384-429. 36 value paths under /demo/catalog-forms/ + 6
        // pre-seeded errors under /_errors/demo/catalog-forms/ for the
        // "With error" state-demo fields. Hard contract per G-05 lesson:
        // every .bind(...) path in catalog/forms.rs MUST have a matching
        // entry in the `demo.catalog-forms` object below; the integration
        // test `catalog_forms_seed_covers_every_bind_path_in_the_demo`
        // asserts this at build time.
        // Phase 18 Plan 06 (CAT-03): seed the first 50 rows of the shared
        // synthetic generator under `/demo/catalog-data-table/rows` as an
        // object-map keyed by stringified id (the frontend DataTable.svelte
        // contract — `Object.entries(rawData)`; array shape silently renders
        // zero rows). Actions array is injected per-row to mirror the
        // fetch-rows handler (Plan 18-03) so initial render and paginated
        // pages share the identical shape — no visual seam at the 50-row
        // boundary when scroll triggers fetch-rows for rows 51-500.
        "catalog-data-table" => serde_json::json!({
            "demo": { "catalog-data-table": {
                "rows": catalog_rows_initial_object_map(),
            }},
        }),
        "catalog-forms" => serde_json::json!({
            "demo": { "catalog-forms": {
                // TextInput Card
                "text-normal": "",
                "text-disabled": "Cannot edit",
                "text-with-error": "bad-input",
                "text-focused": "",
                "text-desc": "",
                "text-value": "",
                // Select Card
                "select-normal": "",
                "select-disabled": "USA",
                "select-with-error": "",
                "select-focused": "",
                "select-desc": "",
                "select-value": "",
                // Checkbox Card
                "checkbox-normal": false,
                "checkbox-checked": true,
                "checkbox-disabled": false,
                "checkbox-with-error": false,
                "checkbox-desc": false,
                "checkbox-value": false,
                // Switch Card
                "switch-off": false,
                "switch-on": true,
                "switch-disabled": false,
                "switch-with-error": false,
                "switch-desc": false,
                "switch-value": false,
                // Radio Card
                "radio-normal": "",
                "radio-selected": "pro",
                "radio-disabled": "",
                "radio-with-error": "",
                "radio-desc": "",
                "radio-value": "",
                // Textarea Card
                "textarea-normal": "",
                "textarea-disabled": "Cannot edit content.",
                "textarea-with-error": "too short",
                "textarea-focused": "",
                "textarea-desc": "",
                "textarea-value": "",
            }},
            "_errors": { "demo": { "catalog-forms": {
                "text-with-error": "Enter a valid email address.",
                "select-with-error": "Please make a selection.",
                "checkbox-with-error": "You must agree to continue.",
                "switch-with-error": "Notifications must be enabled.",
                "radio-with-error": "Please pick one option.",
                "textarea-with-error": "Bio must be at least 20 characters.",
            }}},
        }),
        // Phase 18 Plan 07 (CAT-04): Feedback catalog screen's error-state
        // placeholder mini-Card renders via an `ErrorDisplay` bound to
        // `/demo/catalog-feedback/errors`. Pre-seed a single synthetic entry
        // so the component lights up on first paint without requiring a
        // round-trip. ErrorEntry = { path?: string, message: string } per
        // frontend `ErrorDisplay.svelte:26-41` (same contract Phase 17 G-05
        // canonicalized; a path of `null` is omitted on the frontend).
        //
        // Bind-alignment contract (G-05 regression guard): the sibling test
        // `catalog_feedback_error_bind_aligns_with_demo_tree` asserts the
        // demo tree's ErrorDisplay node binds to this same path.
        "catalog-feedback" => serde_json::json!({
            "demo": { "catalog-feedback": {
                "errors": [
                    {
                        "message": "Sample error: failed to load resource. Retry or check your connection.",
                        "path": null,
                    },
                ],
            }},
        }),
        // Phase 18 Plan 08 (CAT-05): pure static typography + icon + swatch
        // catalog screen. No bind paths; the demo tree is purely visual.
        // Empty seed is correct — named explicitly (rather than falling through
        // the wildcard) to prevent accidental seed drift if CAT-05 ever gains
        // bindable state in a later plan.
        "catalog-typography" => serde_json::json!({}),
        // Phase 19 Plan 19-01 (EXER-01): observation matrix of 4 dimensions
        // (provider-context, mobile-sheet, keyboard-shortcuts, sidebar-tokens)
        // seeded with the Phase 17 G-02 findings verbatim from 19-PATTERNS.md.
        // Wave 2 Plan 19-02 may update copy as the investigation deepens.
        "exer-01" => serde_json::json!({
            "demo": { "exer-01": {
                "matrix": {
                    "provider-context": {
                        "state": "FAIL",
                        "details": "shadcn <Sidebar.Provider> is not scoped: the inner provider re-mounts with the same viewport anchors as the outer, visually replacing the outer 20-entry nav. Observed 2026-04-22 in Phase 17 G-02."
                    },
                    "mobile-sheet": {
                        "state": "FAIL",
                        "details": "Inner AppShell on narrow viewport opens a Sheet that covers the outer Sheet; dismissing either closes both. Expected scoping by surface name is absent."
                    },
                    "keyboard-shortcuts": {
                        "state": "FAIL",
                        "details": "Sidebar toggle shortcut (Ctrl+B by default) triggers both providers. Last-registered wins; which shell responds is implementation-detail, not contract."
                    },
                    "sidebar-tokens": {
                        "state": "WARN",
                        "details": "CSS custom-property inheritance cascades naturally: inner shell inherits whatever the outer sets. Not a bug per se — scoped tokens would need :where(.surface-name) or a style-isolation mechanism (e.g. shadow DOM, @scope)."
                    }
                }
            }}
        }),
        // Phase 19 Plan 19-01 (EXER-02): rapid-patching dashboard seed.
        // Four invariants start as PENDING; cadence defaults to 500ms (aligned
        // with GalleryState::default().exer02_cadence_ms); elapsed resets to 0.
        "exer-02" => serde_json::json!({
            "demo": { "exer-02": {
                "focused-value": "",
                "cadence-ms": 500,
                "invariants": {
                    "focus":  { "state": "PENDING" },
                    "cursor": { "state": "PENDING" },
                    "typed":  { "state": "PENDING" },
                    "ime":    { "state": "PENDING" }
                },
                "elapsed-s": 0
            }}
        }),
        // Phase 19 Plan 19-01 (EXER-03): pathological-scale seed.
        // rows is EMPTY (fetch-rows paginates 10_000 rows — 19-RESEARCH.md
        // §Pitfall 7); 4 form groups (personal-info, contact, preferences,
        // advanced) total 80 fields per 19-UI-SPEC §EXER-03 legends.
        "exer-03" => seed_exer_03(),
        _ => serde_json::json!({}),
    }
}

/// Phase 19 Plan 19-01: EXER-03 pathological-scale seed.
///
/// Per 19-RESEARCH.md §Pitfall 7, rows are seeded EMPTY — the 10 000 rows
/// load 50 at a time via fetch-rows (source="exer-03-synthetic"); never
/// seed the full generator into the initial Render data payload.
///
/// The 80-field defaults across 4 groups per 19-UI-SPEC §EXER-03 legends
/// (lines 294-299). Bind-path template: /demo/exer-03/<group-slug>/<field-name>
/// with group-slug ∈ {personal-info, contact, preferences, advanced}.
/// All TextInput/Textarea/Select/RadioGroup default to empty string;
/// Switch/Checkbox default to false.
fn seed_exer_03() -> serde_json::Value {
    // Group 1: Personal info — 15 TextInput + 2 Select + 2 RadioGroup + 1 Textarea = 20
    let personal = serde_json::json!({
        "first-name": "", "last-name": "", "middle-name": "", "preferred-name": "",
        "nickname": "", "birthdate-text": "", "gender-text": "", "pronouns": "",
        "nationality": "", "languages": "", "ethnicity": "", "religion": "",
        "marital-status": "", "dependents-count": "", "emergency-contact": "",
        "salutation": "", "title": "",
        "primary-language": "", "secondary-language": "",
        "bio": ""
    });

    // Group 2: Contact — 12 TextInput + 2 Select + 4 Checkbox + 2 Textarea = 20
    let contact = serde_json::json!({
        "email-primary": "", "email-secondary": "", "phone-mobile": "", "phone-home": "",
        "phone-work": "", "address-line-1": "", "address-line-2": "", "city": "",
        "state-region": "", "postal-code": "", "country-text": "", "fax": "",
        "preferred-method": "", "timezone": "",
        "consent-email": false, "consent-sms": false, "consent-phone": false, "consent-mail": false,
        "address-notes": "", "communication-notes": ""
    });

    // Group 3: Preferences — 5 Select + 8 Switch + 4 RadioGroup + 3 Checkbox = 20
    let preferences = serde_json::json!({
        "theme": "", "density": "", "language": "", "timezone-pref": "", "currency": "",
        "notif-email": false, "notif-sms": false, "notif-push": false, "notif-weekly": false,
        "notif-monthly": false, "notif-marketing": false, "notif-security": false, "notif-updates": false,
        "frequency": "", "privacy": "", "sharing": "", "visibility": "",
        "terms": false, "privacy-policy": false, "marketing": false
    });

    // Group 4: Advanced — 10 TextInput + 4 Textarea + 3 Select + 2 Switch + 1 Checkbox = 20
    let advanced = serde_json::json!({
        "api-key-alias-1": "", "api-key-alias-2": "", "api-key-alias-3": "", "api-key-alias-4": "",
        "api-key-alias-5": "", "api-key-alias-6": "", "api-key-alias-7": "", "api-key-alias-8": "",
        "api-key-alias-9": "", "api-key-alias-10": "",
        "notes-1": "", "notes-2": "", "notes-3": "", "notes-4": "",
        "access-level": "", "rotation-policy": "", "audit-mode": "",
        "mfa-enabled": false, "biometric-enabled": false,
        "danger-ack": false
    });

    serde_json::json!({
        "demo": { "exer-03": {
            "rows": {},
            "perf": {
                "ttfp_ms": null, "fps": null, "memory_mb": null,
                "latency_p95_ms": null, "remeasure-tick": 0
            },
            "personal-info": personal,
            "contact": contact,
            "preferences": preferences,
            "advanced": advanced
        }}
    })
}

/// Initial 50-row object-map for the CAT-03 Data Table catalog screen.
///
/// Returns a `serde_json::Value::Object` keyed by stringified row id
/// (`"1"` through `"50"`) so the frontend `DataTable.svelte`'s
/// `Object.entries(rawData)` iteration (DataTable.svelte:113-119) lights up
/// on first paint without waiting for a fetch-rows round-trip.
///
/// Each row mirrors the shape emitted by `handlers::fetch_rows.rs`'s
/// `catalog-synthetic-rows` arm (Plan 18-03): the `Row` struct serialized
/// via serde, with an `actions` array (Edit / Delete / Duplicate, each
/// firing `gallery-demo/noop`) injected so the initial 50 rows and the
/// paginated rows 51-500 use the identical column-kind `Actions`
/// rendering without a visual seam at the page boundary.
///
/// Do NOT unify with [`seed_table_rows`] (D-4-C locks that helper untouched
/// — the `data-table` leaf demo in Phase 17 depends on its exact 5-row
/// fixture at `/demo/data-table/rows`).
fn catalog_rows_initial_object_map() -> serde_json::Value {
    let rows = crate::fixtures::synthetic_rows(50);
    let mut map = serde_json::Map::new();
    for row in rows {
        let id_key = row.id.to_string();
        let mut v = serde_json::to_value(&row).expect("Row serializes");
        // Mirror actions injection from fetch_rows.rs so initial render
        // shows the Actions column exactly as subsequent pages will.
        v["actions"] = serde_json::json!([
            { "label": "Edit",      "action": { "type": "click", "name": "gallery-demo/noop" } },
            { "label": "Delete",    "action": { "type": "click", "name": "gallery-demo/noop" } },
            { "label": "Duplicate", "action": { "type": "click", "name": "gallery-demo/noop" } },
        ]);
        map.insert(id_key, v);
    }
    serde_json::Value::Object(map)
}

fn seed_table_rows() -> serde_json::Value {
    // Object-map keyed by stringified id — matches the frontend contract in
    // DataTable.svelte:113-119 (`Object.entries(rawData)` iteration) AND the
    // CRM per-row Set pattern in crm-demo/src/handlers/fetch_rows.rs:136-149.
    // (G-03 fix, Phase 17 Plan 17-05 Task 3.) Phase 18 CAT-03 will extract a
    // shared generator (≥500 rows); Phase 19 EXER-03 pushes to ≥10 000.
    serde_json::json!({
        "1": {"id": 1, "name": "Alice Baker", "email": "alice@example.com", "created": "2026-01-05"},
        "2": {"id": 2, "name": "Bob Chen",    "email": "bob@example.com",   "created": "2026-01-08"},
        "3": {"id": 3, "name": "Carol Davis", "email": "carol@example.com", "created": "2026-01-12"},
        "4": {"id": 4, "name": "Dan Evans",   "email": "dan@example.com",   "created": "2026-01-15"},
        "5": {"id": 5, "name": "Eva Frost",   "email": "eva@example.com",   "created": "2026-01-20"},
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_for_unknown_key_is_empty() {
        assert_eq!(seed_for_key("bogus"), serde_json::json!({}));
    }

    #[test]
    fn seed_for_form_has_email_and_name() {
        let seed = seed_for_key("form");
        assert_eq!(seed["demo"]["form"]["email"], "");
        assert_eq!(seed["demo"]["form"]["name"], "");
    }

    #[test]
    fn seed_table_rows_has_five_rows() {
        let rows = seed_table_rows();
        // Object-map keyed by stringified id (matches frontend Object.entries
        // contract in DataTable.svelte:113). See seed_table_rows comment.
        assert_eq!(rows.as_object().unwrap().len(), 5);
        assert_eq!(rows["1"]["name"], "Alice Baker");
        assert_eq!(rows["5"]["name"], "Eva Frost");
    }

    #[test]
    fn catalog_data_table_seed_matches_row_shape_and_action_injection() {
        let seed = seed_for_key("catalog-data-table");
        let rows = seed["demo"]["catalog-data-table"]["rows"]
            .as_object()
            .expect("rows is object-map keyed by stringified id");
        assert_eq!(rows.len(), 50, "first page seeds exactly 50 rows");
        let r1 = rows.get("1").expect("row id=1 must be seeded");
        assert_eq!(r1["id"], 1);
        assert!(r1["name"].is_string());
        assert!(r1["email"].is_string());
        assert!(r1["status"].is_string());
        assert!(r1["score"].is_number());
        assert!(
            r1["joined_at"].is_string(),
            "joined_at serializes as ISO YYYY-MM-DD string"
        );
        let actions = r1["actions"]
            .as_array()
            .expect("actions present on every seeded row");
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0]["label"], "Edit");
        assert_eq!(actions[1]["label"], "Delete");
        assert_eq!(actions[2]["label"], "Duplicate");
        // Every action fires the gallery-demo/noop click handler — same as
        // handlers::fetch_rows.rs's catalog-synthetic-rows arm.
        assert_eq!(actions[0]["action"]["type"], "click");
        assert_eq!(actions[0]["action"]["name"], "gallery-demo/noop");
    }

    #[test]
    fn catalog_data_table_seed_aligns_with_generator() {
        // The seeded row id=1 must share its fields with the first row of
        // `fixtures::synthetic_rows(n)` for any n — proving the seed is
        // backed by the same deterministic generator that fetch_rows will
        // slice for rows 51-500. Generator is deterministic (LCG seeded at
        // 0x1234_5678_9ABC_DEF0) — same n → same Row.
        let seed = seed_for_key("catalog-data-table");
        let r1 = &seed["demo"]["catalog-data-table"]["rows"]["1"];
        let gen_row = &crate::fixtures::synthetic_rows(1)[0];
        assert_eq!(r1["id"], gen_row.id);
        assert_eq!(r1["name"], gen_row.name);
        assert_eq!(r1["email"], gen_row.email);
        // score + joined_at also match bit-for-bit
        assert_eq!(r1["score"], gen_row.score);
    }

    #[test]
    fn catalog_data_table_seed_spans_full_first_page() {
        // Boundaries: ids "1" and "50" both present; "51" absent (paginated).
        let seed = seed_for_key("catalog-data-table");
        let rows = seed["demo"]["catalog-data-table"]["rows"]
            .as_object()
            .expect("rows is object-map");
        assert!(rows.contains_key("1"));
        assert!(rows.contains_key("50"));
        assert!(
            !rows.contains_key("51"),
            "row 51 is paginated via fetch-rows, not seeded"
        );
        // Every seeded row carries a non-empty actions array (actions
        // injection runs unconditionally inside catalog_rows_initial_object_map).
        for (_id, row) in rows {
            let actions = row["actions"]
                .as_array()
                .expect("every row has actions array");
            assert_eq!(actions.len(), 3);
        }
    }

    #[test]
    fn catalog_forms_seed_covers_every_bind_path_in_the_demo() {
        // Hard contract (Phase 17 G-05 lesson): every `.bind(...)` path used
        // by `catalog::forms::gallery_demo()` MUST have a matching key in
        // `seed_for_key("catalog-forms")` so the frontend's `getData()`
        // returns a seeded value on first visit. Missing entries would
        // render silently empty — the pre-17-06 bug class this test guards
        // against.
        use marionette::gallery::registered_demos;

        let forms_entry = registered_demos()
            .find(|e| e.key == "catalog-forms")
            .expect("catalog-forms demo must be registered");
        let tree = (forms_entry.render)();

        // Collect every unique /demo/catalog-forms/<suffix> bind path used
        // in the tree, stripped to its suffix so we can match against the
        // seed's demo.catalog-forms object keys.
        let binds: std::collections::HashSet<String> = tree
            .iter()
            .filter_map(|(_id, c)| {
                let v = serde_json::to_value(c).ok()?;
                v["bind"]
                    .as_str()?
                    .strip_prefix("/demo/catalog-forms/")
                    .map(str::to_string)
            })
            .collect();

        assert!(
            binds.len() >= 30,
            "expected >=30 bind paths on catalog-forms tree, got {}",
            binds.len()
        );

        let seed = seed_for_key("catalog-forms");
        let seeded_paths: std::collections::HashSet<String> = seed["demo"]["catalog-forms"]
            .as_object()
            .expect("seed.demo.catalog-forms must be an object")
            .keys()
            .cloned()
            .collect();

        for b in &binds {
            assert!(
                seeded_paths.contains(b),
                "bind path /demo/catalog-forms/{b} has no matching seed entry \
                 — add it to the `catalog-forms` arm of seed_for_key"
            );
        }
    }

    #[test]
    fn catalog_forms_seed_preseeds_error_messages_for_with_error_demo_fields() {
        // The six "With error" demo fields show a red-border state on first
        // render. Achieved by pre-seeding the /_errors/demo/catalog-forms/<...>
        // sub-tree at the same bind suffix as the value field. UI-SPEC
        // §CAT-02 lines 384-429 locks the messages.
        let seed = seed_for_key("catalog-forms");
        let errs = seed["_errors"]["demo"]["catalog-forms"]
            .as_object()
            .expect("_errors.demo.catalog-forms must be an object");
        assert_eq!(errs["text-with-error"], "Enter a valid email address.");
        assert_eq!(errs["select-with-error"], "Please make a selection.");
        assert_eq!(errs["checkbox-with-error"], "You must agree to continue.");
        assert_eq!(errs["switch-with-error"], "Notifications must be enabled.");
        assert_eq!(errs["radio-with-error"], "Please pick one option.");
        assert_eq!(errs["textarea-with-error"], "Bio must be at least 20 characters.");
    }

    #[test]
    fn catalog_feedback_seed_has_one_sample_error() {
        // Plan 18-07 Task 2: single synthetic entry so the CAT-04 error-state
        // placeholder mini-Card renders on first paint. Locked copy from
        // UI-SPEC §Copywriting Contract §CAT-04 "Sample error:" prefix.
        let seed = seed_for_key("catalog-feedback");
        let errs = seed["demo"]["catalog-feedback"]["errors"]
            .as_array()
            .expect("errors is array");
        assert_eq!(errs.len(), 1, "exactly one sample error is seeded");
        let m = errs[0]["message"].as_str().expect("message is string");
        assert!(
            m.contains("Sample error"),
            "message copy drifted: {m}"
        );
        assert!(errs[0]["path"].is_null(), "path is null for system-level sample error");
    }

    #[test]
    fn seed_exer_01_has_four_dimensions_with_fail_and_warn() {
        // Phase 19 Plan 19-01: verify the observation-matrix seed shape.
        let s = seed_for_key("exer-01");
        let matrix = &s["demo"]["exer-01"]["matrix"];
        assert_eq!(matrix["provider-context"]["state"], "FAIL");
        assert_eq!(matrix["mobile-sheet"]["state"], "FAIL");
        assert_eq!(matrix["keyboard-shortcuts"]["state"], "FAIL");
        assert_eq!(matrix["sidebar-tokens"]["state"], "WARN");
    }

    #[test]
    fn seed_exer_02_pending_invariants_and_default_cadence() {
        // Phase 19 Plan 19-01: verify 4 invariants seeded PENDING + 500ms cadence.
        let s = seed_for_key("exer-02");
        let e2 = &s["demo"]["exer-02"];
        assert_eq!(e2["cadence-ms"], 500);
        assert_eq!(e2["focused-value"], "");
        for inv in ["focus", "cursor", "typed", "ime"] {
            assert_eq!(
                e2["invariants"][inv]["state"],
                "PENDING",
                "invariant {inv} should start PENDING"
            );
        }
    }

    #[test]
    fn seed_exer_03_has_four_groups_and_empty_rows() {
        // Phase 19 Plan 19-01: rows MUST be empty (fetch-rows paginates per
        // 19-RESEARCH.md §Pitfall 7); 4 form groups all objects.
        let s = seed_for_key("exer-03");
        let e3 = &s["demo"]["exer-03"];
        assert_eq!(e3["rows"], serde_json::json!({}));
        for group in ["personal-info", "contact", "preferences", "advanced"] {
            assert!(e3[group].is_object(), "group {group} must be an object");
        }
        // Spot-check one field per group confirms defaults.
        assert_eq!(e3["personal-info"]["first-name"], "");
        assert_eq!(e3["contact"]["consent-email"], false);
        assert_eq!(e3["preferences"]["notif-email"], false);
        assert_eq!(e3["advanced"]["danger-ack"], false);
    }

    #[test]
    fn seed_exer_03_field_count_is_80() {
        // UI-SPEC §EXER-03 locks 80 total fields across 4 groups of 20 each.
        let s = seed_for_key("exer-03");
        let e3 = &s["demo"]["exer-03"];
        let count = ["personal-info", "contact", "preferences", "advanced"]
            .iter()
            .map(|g| e3[*g].as_object().expect("group is object").len())
            .sum::<usize>();
        assert_eq!(
            count, 80,
            "UI-SPEC §EXER-03 locks 80 total fields across 4 groups"
        );
    }

    #[test]
    fn catalog_feedback_error_bind_aligns_with_demo_tree() {
        // Hard contract (Phase 17 G-05 regression guard): the CAT-04 demo
        // tree's ErrorDisplay node MUST bind to the same path that the
        // catalog-feedback arm of seed_for_key writes. A drift between these
        // two sites was the G-05 bug class (bind paths silently mismatched
        // seed paths, causing the frontend's `{#if errors.length > 0}` guard
        // to hide the component without error). Covering it here means any
        // future refactor of either side triggers a red test.
        use marionette::gallery::registered_demos;
        let entry = registered_demos()
            .find(|e| e.key == "catalog-feedback")
            .expect("catalog-feedback must be registered");
        let tree = (entry.render)();
        let error_bind = tree
            .iter()
            .find_map(|(id, c)| {
                if id == "catalog-feedback-error" {
                    let v = serde_json::to_value(c).ok()?;
                    v["bind"].as_str().map(String::from)
                } else {
                    None
                }
            })
            .expect("catalog-feedback-error node with a bind");
        assert_eq!(error_bind, "/demo/catalog-feedback/errors");
        // Seed path matches (and is a non-empty array with the sample entry).
        let seed = seed_for_key("catalog-feedback");
        let seeded_errors = seed["demo"]["catalog-feedback"]["errors"]
            .as_array()
            .expect("seed writes errors as an array at /demo/catalog-feedback/errors");
        assert!(
            !seeded_errors.is_empty(),
            "seed must be non-empty or the placeholder renders silently hidden"
        );
    }
}
