//! CAT-02 — Forms catalog screen.
//!
//! Composes a per-input Card for each of the 6 form input types (TextInput,
//! Select, Checkbox, Switch, RadioGroup, Textarea). Each Card contains:
//!   1. H2 heading with the input type name.
//!   2. State-matrix Container (responsive grid, 5 demo fields).
//!   3. FieldSeparator (visual divider between state-matrix and live row).
//!   4. Interactive field (bind + blur-action fires a Phase 12 patch).
//!   5. Pre-mounted empty error-slot Container — stable id for
//!      `set-node` / `set-children` / `delete-node` to target.
//!
//! Per CONTEXT.md §D-2-B, does NOT invoke leaf `gallery_demo()` fns.
//! Locked classes + copy come from UI-SPEC §CAT-02. `ComponentAction { type:
//! "blur" }` is hand-constructed because no `.on_blur()` builder helper
//! exists today (RESEARCH.md §Q1) — inline construction keeps the blur shape
//! consistent with what Plan 18-02 wired into SelectInput / Checkbox /
//! Switch / RadioGroup.

#![allow(clippy::too_many_lines)]

use marionette::builders::radio_group::RadioOption;
use marionette::builders::select::SelectOption;
use marionette::builders::{
    Checkbox, Container, FieldSeparator, Heading, RadioGroup, Select, Switch, Text, TextInput,
    Textarea,
};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

// ---------- Locked CSS class strings (UI-SPEC §Spacing Scale / §CAT-02) ----------

const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
const STATE_GRID_CLASS: &str = "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3";

// ---------- Hand-rolled ComponentAction with type="blur" ----------
//
// No `ComponentAction::blur()` constructor exists yet (RESEARCH.md §Q1).
// Plan 18-02 wired blur dispatch into SelectInput / Checkbox / Switch /
// RadioGroup on the frontend — this is the shape they expect.
fn blur_action(name: &str) -> ComponentAction {
    ComponentAction {
        r#type: "blur".into(),
        name: Some(name.into()),
        target: None,
        id_path: None,
        extra: serde_json::Map::new(),
    }
}

// ---------- Top-level demo fn ----------

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-forms", name = "Catalog: Forms")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // --- Title + intro row (locked copy, UI-SPEC §Copywriting) ---
    let title = Heading::new("Forms")
        .id("catalog-forms-title")
        .level(1)
        .build();
    let intro = Text::new(
        "Every input × every state, plus a live validation round-trip via Phase 12 \
         node-tree patches. Type into any interactive field, then tab out — the error \
         appears (or clears) via a server-driven node patch.",
    )
    .id("catalog-forms-intro")
    .build();

    // --- Build all 6 Cards. Each helper returns:
    //       (card_root_tuple, descendants)
    //     where descendants is a flat Vec<Node> of EVERY sub-tree node under
    //     the card root (heading, state-grid root + its demo fields, separator,
    //     interactive field, pre-mounted error-slot). ---
    let (text_root, text_desc) = build_text_card();
    let (select_root, select_desc) = build_select_card();
    let (checkbox_root, checkbox_desc) = build_checkbox_card();
    let (switch_root, switch_desc) = build_switch_card();
    let (radio_root, radio_desc) = build_radio_card();
    let (textarea_root, textarea_desc) = build_textarea_card();

    // --- Outer root: title + intro + 6 card-root tuples. ---
    let root_children: Vec<Node> = vec![
        title,
        intro,
        text_root,
        select_root,
        checkbox_root,
        switch_root,
        radio_root,
        textarea_root,
    ];

    let (outer_root, outer_direct) = Container::new()
        .id("catalog-forms-root")
        .class(OUTER_CLASS)
        .children(root_children)
        .build_tree();

    // --- Flatten: outer_root + outer_direct (title + intro + 6 card roots)
    //     + all card descendants (5×6 state fields + 6 separators + 6 interactive
    //     + 6 error slots + 6 state-grid roots + 6 heading roots). ---
    let mut result: Vec<Node> = Vec::with_capacity(
        1 + outer_direct.len()
            + text_desc.len()
            + select_desc.len()
            + checkbox_desc.len()
            + switch_desc.len()
            + radio_desc.len()
            + textarea_desc.len(),
    );
    result.push(outer_root);
    result.extend(outer_direct);
    result.extend(text_desc);
    result.extend(select_desc);
    result.extend(checkbox_desc);
    result.extend(switch_desc);
    result.extend(radio_desc);
    result.extend(textarea_desc);
    result
}

// ---------- Shared Card-composition helper ----------
//
// Given already-built pieces, compose the Card root + flatten its descendants.
// `interactive` + `error_slot` are passed as tuples — caller already built
// them with the right id / bind / action / etc. `state_fields` is the
// fully-built vec of 5 state-demo fields (each a Node).
#[cfg(feature = "gallery")]
fn assemble_card(
    input_stem: &str,
    state_fields: Vec<Node>,
    interactive: Node,
    error_slot: Node,
) -> (Node, Vec<Node>) {
    let heading_id = format!("catalog-forms-{input_stem}-heading");
    let sep_id = format!("catalog-forms-{input_stem}-sep");
    let grid_id = format!("catalog-forms-{input_stem}-state-grid");
    let card_id = format!("catalog-forms-{input_stem}-card");

    let heading = Heading::new(input_display_label(input_stem))
        .id(&heading_id)
        .level(2)
        .build();

    let (grid_root, grid_desc) = Container::new()
        .id(&grid_id)
        .class(STATE_GRID_CLASS)
        .children(state_fields)
        .build_tree();

    let sep = FieldSeparator::new().id(&sep_id).build();

    // Direct children of the Card (in locked order):
    //   heading, state-grid root, separator, interactive, error slot.
    let card_kids: Vec<Node> = vec![
        heading,
        grid_root,
        sep,
        interactive,
        error_slot,
    ];

    let (card_root, card_direct) = Container::new()
        .id(&card_id)
        .class(CARD_CLASS)
        .children(card_kids)
        .build_tree();

    // Flatten: card's direct children + the state-grid's descendants
    // (the 5 state-demo fields).
    let mut descendants: Vec<Node> = Vec::with_capacity(card_direct.len() + grid_desc.len());
    descendants.extend(card_direct);
    descendants.extend(grid_desc);
    (card_root, descendants)
}

#[cfg(feature = "gallery")]
fn input_display_label(stem: &str) -> &'static str {
    match stem {
        "text" => "TextInput",
        "select" => "Select",
        "checkbox" => "Checkbox",
        "switch" => "Switch",
        "radio" => "Radio Group",
        "textarea" => "Textarea",
        _ => "Unknown",
    }
}

// ---------- TextInput Card ----------

#[cfg(feature = "gallery")]
fn build_text_card() -> (Node, Vec<Node>) {
    let state_fields: Vec<Node> = vec![
        TextInput::new("Normal")
            .id("catalog-forms-text-normal")
            .bind("/demo/catalog-forms/text-normal")
            .build(),
        TextInput::new("Disabled")
            .id("catalog-forms-text-disabled")
            .bind("/demo/catalog-forms/text-disabled")
            .disabled(true)
            .build(),
        TextInput::new("With error")
            .id("catalog-forms-text-with-error")
            .bind("/demo/catalog-forms/text-with-error")
            .build(),
        TextInput::new("Focused (click me)")
            .id("catalog-forms-text-focused")
            .bind("/demo/catalog-forms/text-focused")
            .build(),
        TextInput::new("With description")
            .id("catalog-forms-text-desc")
            .bind("/demo/catalog-forms/text-desc")
            .description("Helper text rendered below via Field.Description.")
            .build(),
    ];

    let interactive = TextInput::new("Email (type then tab out)")
        .id("catalog-forms-text-interactive")
        .bind("/demo/catalog-forms/text-value")
        .description(
            "Invalid → red border on blur. Correct → error clears via set-children + \
             delete-node patch.",
        )
        .action(blur_action("gallery-demo/catalog-forms/validate-text-input"))
        .build();

    let error_slot = Container::new()
        .id("catalog-forms-text-error-slot")
        .build();

    assemble_card("text", state_fields, interactive, error_slot)
}

// ---------- Select Card ----------

#[cfg(feature = "gallery")]
fn select_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "USA".into(), label: "USA".into() },
        SelectOption { value: "Canada".into(), label: "Canada".into() },
        SelectOption { value: "Mexico".into(), label: "Mexico".into() },
        SelectOption { value: "Switzerland".into(), label: "Switzerland".into() },
        SelectOption { value: "Germany".into(), label: "Germany".into() },
    ]
}

#[cfg(feature = "gallery")]
fn build_select_card() -> (Node, Vec<Node>) {
    let state_fields: Vec<Node> = vec![
        Select::new("Normal", select_options())
            .id("catalog-forms-select-normal")
            .bind("/demo/catalog-forms/select-normal")
            .build(),
        Select::new("Disabled", select_options())
            .id("catalog-forms-select-disabled")
            .bind("/demo/catalog-forms/select-disabled")
            .disabled(true)
            .build(),
        Select::new("With error", select_options())
            .id("catalog-forms-select-with-error")
            .bind("/demo/catalog-forms/select-with-error")
            .build(),
        Select::new("Open (click me)", select_options())
            .id("catalog-forms-select-focused")
            .bind("/demo/catalog-forms/select-focused")
            .build(),
        Select::new("With description", select_options())
            .id("catalog-forms-select-desc")
            .bind("/demo/catalog-forms/select-desc")
            .description("Helper text rendered below via Field.Description.")
            .build(),
    ];

    let interactive = Select::new("Country (required — pick one then tab out)", select_options())
        .id("catalog-forms-select-interactive")
        .bind("/demo/catalog-forms/select-value")
        .required(true)
        .description(
            "Empty selection → error appears via delete-node sibling. Pick a value → \
             the error node is deleted from the tree.",
        )
        .action(blur_action("gallery-demo/catalog-forms/validate-select"))
        .build();

    let error_slot = Container::new()
        .id("catalog-forms-select-error-slot")
        .build();

    assemble_card("select", state_fields, interactive, error_slot)
}

// ---------- Checkbox Card ----------

#[cfg(feature = "gallery")]
fn build_checkbox_card() -> (Node, Vec<Node>) {
    let state_fields: Vec<Node> = vec![
        Checkbox::new("Normal")
            .id("catalog-forms-checkbox-normal")
            .bind("/demo/catalog-forms/checkbox-normal")
            .build(),
        Checkbox::new("Checked")
            .id("catalog-forms-checkbox-checked")
            .bind("/demo/catalog-forms/checkbox-checked")
            .build(),
        Checkbox::new("Disabled")
            .id("catalog-forms-checkbox-disabled")
            .bind("/demo/catalog-forms/checkbox-disabled")
            .disabled(true)
            .build(),
        Checkbox::new("With error")
            .id("catalog-forms-checkbox-with-error")
            .bind("/demo/catalog-forms/checkbox-with-error")
            .build(),
        Checkbox::new("With description")
            .id("catalog-forms-checkbox-desc")
            .bind("/demo/catalog-forms/checkbox-desc")
            .description("Helper text rendered below via Field.Description.")
            .build(),
    ];

    let interactive = Checkbox::new("I agree to the terms")
        .id("catalog-forms-checkbox-interactive")
        .bind("/demo/catalog-forms/checkbox-value")
        .description(
            "Required. Unchecked → error appears via set-node into a pre-mounted slot. \
             Checked → set-node swaps the slot back to empty Container.",
        )
        .action(blur_action("gallery-demo/catalog-forms/validate-checkbox"))
        .build();

    let error_slot = Container::new()
        .id("catalog-forms-checkbox-error-slot")
        .build();

    assemble_card("checkbox", state_fields, interactive, error_slot)
}

// ---------- Switch Card ----------

#[cfg(feature = "gallery")]
fn build_switch_card() -> (Node, Vec<Node>) {
    let state_fields: Vec<Node> = vec![
        Switch::new("Off")
            .id("catalog-forms-switch-off")
            .bind("/demo/catalog-forms/switch-off")
            .build(),
        Switch::new("On")
            .id("catalog-forms-switch-on")
            .bind("/demo/catalog-forms/switch-on")
            .build(),
        Switch::new("Disabled")
            .id("catalog-forms-switch-disabled")
            .bind("/demo/catalog-forms/switch-disabled")
            .disabled(true)
            .build(),
        Switch::new("With error")
            .id("catalog-forms-switch-with-error")
            .bind("/demo/catalog-forms/switch-with-error")
            .build(),
        Switch::new("With description")
            .id("catalog-forms-switch-desc")
            .bind("/demo/catalog-forms/switch-desc")
            .description("Helper text rendered below via Field.Description.")
            .build(),
    ];

    let interactive = Switch::new("Enable notifications")
        .id("catalog-forms-switch-interactive")
        .bind("/demo/catalog-forms/switch-value")
        .description(
            "Required. Off → error appears via set-node. On → set-node swaps the slot \
             back to empty Container.",
        )
        .action(blur_action("gallery-demo/catalog-forms/validate-switch"))
        .build();

    let error_slot = Container::new()
        .id("catalog-forms-switch-error-slot")
        .build();

    assemble_card("switch", state_fields, interactive, error_slot)
}

// ---------- Radio Group Card ----------

#[cfg(feature = "gallery")]
fn radio_options() -> Vec<RadioOption> {
    vec![
        RadioOption {
            value: "free".into(),
            label: "Free".into(),
            description: None,
        },
        RadioOption {
            value: "pro".into(),
            label: "Pro".into(),
            description: None,
        },
        RadioOption {
            value: "enterprise".into(),
            label: "Enterprise".into(),
            description: None,
        },
    ]
}

#[cfg(feature = "gallery")]
fn build_radio_card() -> (Node, Vec<Node>) {
    let state_fields: Vec<Node> = vec![
        RadioGroup::new("Normal", radio_options())
            .id("catalog-forms-radio-normal")
            .bind("/demo/catalog-forms/radio-normal")
            .build(),
        RadioGroup::new("Selected", radio_options())
            .id("catalog-forms-radio-selected")
            .bind("/demo/catalog-forms/radio-selected")
            .build(),
        RadioGroup::new("Disabled", radio_options())
            .id("catalog-forms-radio-disabled")
            .bind("/demo/catalog-forms/radio-disabled")
            .disabled(true)
            .build(),
        RadioGroup::new("With error", radio_options())
            .id("catalog-forms-radio-with-error")
            .bind("/demo/catalog-forms/radio-with-error")
            .build(),
        RadioGroup::new("With description", radio_options())
            .id("catalog-forms-radio-desc")
            .bind("/demo/catalog-forms/radio-desc")
            .description("Helper text rendered below via Field.Description.")
            .build(),
    ];

    let interactive = RadioGroup::new("Plan (pick one)", radio_options())
        .id("catalog-forms-radio-interactive")
        .bind("/demo/catalog-forms/radio-value")
        .required(true)
        .description(
            "Required. Empty → error appears via set-children. Choose any plan → \
             set-children swaps back to no error.",
        )
        .action(blur_action("gallery-demo/catalog-forms/validate-radio"))
        .build();

    let error_slot = Container::new()
        .id("catalog-forms-radio-error-slot")
        .build();

    assemble_card("radio", state_fields, interactive, error_slot)
}

// ---------- Textarea Card ----------

#[cfg(feature = "gallery")]
fn build_textarea_card() -> (Node, Vec<Node>) {
    let state_fields: Vec<Node> = vec![
        Textarea::new("Normal")
            .id("catalog-forms-textarea-normal")
            .bind("/demo/catalog-forms/textarea-normal")
            .build(),
        Textarea::new("Disabled")
            .id("catalog-forms-textarea-disabled")
            .bind("/demo/catalog-forms/textarea-disabled")
            .disabled(true)
            .build(),
        Textarea::new("With error")
            .id("catalog-forms-textarea-with-error")
            .bind("/demo/catalog-forms/textarea-with-error")
            .build(),
        Textarea::new("Focused")
            .id("catalog-forms-textarea-focused")
            .bind("/demo/catalog-forms/textarea-focused")
            .build(),
        Textarea::new("With description")
            .id("catalog-forms-textarea-desc")
            .bind("/demo/catalog-forms/textarea-desc")
            .description("Helper text rendered below via Field.Description.")
            .build(),
    ];

    let interactive = Textarea::new("Bio (min. 20 characters)")
        .id("catalog-forms-textarea-interactive")
        .bind("/demo/catalog-forms/textarea-value")
        .description(
            "Type at least 20 chars. Short → error appears via delete-node-removed \
             sibling. Long enough → delete-node removes the error.",
        )
        .action(blur_action("gallery-demo/catalog-forms/validate-textarea"))
        .build();

    let error_slot = Container::new()
        .id("catalog-forms-textarea-error-slot")
        .build();

    assemble_card("textarea", state_fields, interactive, error_slot)
}

// ---------- Tests ----------

#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;

    #[test]
    fn root_id_is_catalog_forms_root() {
        let v = gallery_demo();
        assert_eq!(v[0].0, "catalog-forms-root");
    }

    #[test]
    fn six_cards_with_locked_ids() {
        let v = gallery_demo();
        let expected = [
            "catalog-forms-text-card",
            "catalog-forms-select-card",
            "catalog-forms-checkbox-card",
            "catalog-forms-switch-card",
            "catalog-forms-radio-card",
            "catalog-forms-textarea-card",
        ];
        for id in expected {
            assert!(
                v.iter().any(|(i, _)| i == id),
                "missing card root node id {id}"
            );
        }
    }

    #[test]
    fn six_error_slots_pre_mounted() {
        let v = gallery_demo();
        let expected_slots = [
            "catalog-forms-text-error-slot",
            "catalog-forms-select-error-slot",
            "catalog-forms-checkbox-error-slot",
            "catalog-forms-switch-error-slot",
            "catalog-forms-radio-error-slot",
            "catalog-forms-textarea-error-slot",
        ];
        for id in expected_slots {
            let node = v
                .iter()
                .find(|(i, _)| i == id)
                .unwrap_or_else(|| panic!("missing error slot node {id}"));
            // Slot must be a Container at render time so SetNode / SetChildren
            // targets an EXISTING node (RESEARCH.md §Pitfall 4).
            let v = serde_json::to_value(&node.1).expect("serialize");
            assert_eq!(v["type"], "container", "{id} must be a container");
        }
    }

    #[test]
    fn six_interactive_fields_have_blur_actions() {
        let v = gallery_demo();
        let interactive_ids = [
            "catalog-forms-text-interactive",
            "catalog-forms-select-interactive",
            "catalog-forms-checkbox-interactive",
            "catalog-forms-switch-interactive",
            "catalog-forms-radio-interactive",
            "catalog-forms-textarea-interactive",
        ];
        for id in interactive_ids {
            let node = v
                .iter()
                .find(|(i, _)| i == id)
                .unwrap_or_else(|| panic!("missing interactive field {id}"));
            let val = serde_json::to_value(&node.1).expect("serialize");
            let action_type = val["action"]["type"].as_str().unwrap_or("");
            assert_eq!(action_type, "blur", "{id} action.type must be 'blur'");
            let action_name = val["action"]["name"].as_str().unwrap_or("");
            assert!(
                action_name.starts_with("gallery-demo/catalog-forms/validate-"),
                "{id} action.name={action_name}"
            );
        }
    }

    #[test]
    fn registered_demos_includes_catalog_forms() {
        let found = registered_demos().find(|e| e.key == "catalog-forms");
        assert!(found.is_some(), "catalog-forms must register via linkme");
        let entry = found.expect("entry");
        assert_eq!(entry.display_name, "Catalog: Forms");
        let rendered = (entry.render)();
        assert_eq!(rendered[0].0, "catalog-forms-root");
    }

    #[test]
    fn every_bind_under_catalog_forms_namespace_with_ge_30_paths() {
        // Structural invariant: every .bind(...) path used by a CAT-02 field
        // lives under /demo/catalog-forms/ and there are at least 30 distinct
        // paths (5 state demos × 6 inputs + 6 interactive = 36). Task 3's
        // seed_for_key test asserts each path has a matching seed entry.
        let v = gallery_demo();
        let mut binds: Vec<String> = v
            .iter()
            .filter_map(|(_id, c)| {
                let s = serde_json::to_value(c).ok()?;
                s["bind"].as_str().map(String::from)
            })
            .collect();
        binds.sort();
        binds.dedup();
        assert!(
            binds.len() >= 30,
            "expected >=30 bind paths, got {} ({binds:?})",
            binds.len()
        );
        for b in &binds {
            assert!(
                b.starts_with("/demo/catalog-forms/"),
                "all catalog-forms binds must live under /demo/catalog-forms/, got {b}"
            );
        }
    }
}
