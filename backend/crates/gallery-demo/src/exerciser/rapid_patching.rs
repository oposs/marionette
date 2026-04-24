//! EXER-02 — Rapid Patching exerciser (Plan 19-03).
//!
//! 4 Cards: Focused input, Cadence control, Invariant dashboard, Patch log.
//! Per 19-CONTEXT.md §D-2, exercises all 4 invariants: focus retention,
//! cursor position, typed-input integrity, IME composition.
//!
//! A1 resolution (from 19-01-SUMMARY): patch loop is driven by client-initiated
//! ticks — the frontend sends `gallery-demo/exer-02/tick` every cadence_ms;
//! the backend responds with a real PatchMessage per tick. Backend-owned task
//! state (running flag, elapsed-start) lives in the once-cell GalleryState.
//!
//! ## Node layout
//!
//! ```text
//! exer-02-root (Container, flex-col gap-6 p-6)
//! ├─ exer-02-title        (Heading H1 "Rapid Patching")
//! ├─ exer-02-intro        (Text intro)
//! ├─ exer-02-c1-focused-input         (Card — Focused input)
//! │   ├─ exer-02-c1-h2
//! │   └─ exer-02-focused-input        (TextInput, bind /demo/exer-02/focused-value)
//! ├─ exer-02-c2-cadence-control       (Card — Cadence control)
//! │   ├─ exer-02-c2-h2
//! │   ├─ exer-02-cadence              (RadioGroup, bind /demo/exer-02/cadence-ms)
//! │   └─ exer-02-cta-row              (Container — 3 Buttons)
//! │       ├─ exer-02-start            (Button, gallery-demo/exer-02/start)
//! │       ├─ exer-02-pause            (Button, gallery-demo/exer-02/pause)
//! │       └─ exer-02-reset            (Button, gallery-demo/exer-02/reset)
//! ├─ exer-02-c3-invariant-dashboard   (Card — Invariant dashboard)
//! │   ├─ exer-02-c3-h2
//! │   ├─ exer-02-elapsed              (Text, bind /demo/exer-02/elapsed-display)
//! │   └─ exer-02-dashboard-grid       (Container grid)
//! │       ├─ exer-02-invariant-focus
//! │       ├─ exer-02-invariant-cursor
//! │       ├─ exer-02-invariant-typed
//! │       └─ exer-02-invariant-ime
//! └─ exer-02-c4-patch-log             (Card — Patch log)
//!     ├─ exer-02-c4-h2
//!     └─ exer-02-log-container        (Container — empty; tick appends rows)
//!         └─ exer-02-log-empty        (Text "No patches yet…")
//! ```

use marionette::builders::radio_group::RadioOption;
use marionette::builders::{Button, Container, Heading, RadioGroup, Text, TextInput};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

// UI-SPEC §Spacing Scale locks (19-UI-SPEC.md §EXER-02):
const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
// Per 19-UI-SPEC.md §EXER-02 Invariant dashboard: "grid-cols-2 lg:grid-cols-4 — never collapse to 1 col".
const DASHBOARD_GRID_CLASS: &str = "grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3";
const INVARIANT_CELL_CLASS: &str =
    "rounded-md border border-border bg-card p-3 flex flex-col gap-2 items-start";
const BUTTON_ROW_CLASS: &str = "flex gap-2 flex-wrap";
const LOG_CLASS: &str =
    "max-h-64 overflow-y-auto font-mono text-xs bg-muted/50 rounded border p-3 flex flex-col gap-1";

/// Status-pill class (Badge substitute — no Badge builder exists in marionette
/// as of v1.2). Bound Text with this class acts as the PASS/FAIL/PENDING badge.
const BADGE_CLASS: &str =
    "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-semibold";

// ---------- Top-level demo fn ----------

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "exer-02", name = "Exerciser: Rapid Patching")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    let title = Heading::new("Rapid Patching")
        .id("exer-02-title")
        .level(1)
        .build();
    let intro = Text::new(
        "Node patches fire at the chosen cadence while the input below retains \
         focus. Type, paste, or compose via IME — the four invariants (focus \
         retention, cursor position, typed-input integrity, IME composition) \
         report PASS/FAIL in real time. Target: 60 seconds of sustained \
         mutation pressure without a single FAIL.",
    )
    .id("exer-02-intro")
    .build();

    let (c1_root, c1_desc) = build_focused_input_card();
    let (c2_root, c2_desc) = build_cadence_control_card();
    let (c3_root, c3_desc) = build_invariant_dashboard_card();
    let (c4_root, c4_desc) = build_patch_log_card();

    let (outer, outer_direct) = Container::new()
        .id("exer-02-root")
        .class(OUTER_CLASS)
        .children(vec![title, intro, c1_root, c2_root, c3_root, c4_root])
        .build_tree();

    let mut out = Vec::with_capacity(
        1 + outer_direct.len() + c1_desc.len() + c2_desc.len() + c3_desc.len() + c4_desc.len(),
    );
    out.push(outer);
    out.extend(outer_direct);
    out.extend(c1_desc);
    out.extend(c2_desc);
    out.extend(c3_desc);
    out.extend(c4_desc);
    out
}

// ---------- Card 1: Focused input ----------

#[cfg(feature = "gallery")]
fn build_focused_input_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Focused input")
        .id("exer-02-c1-h2")
        .level(2)
        .build();

    // UI-SPEC §EXER-02 Focused input copy (lines 197-200 verbatim).
    let focused = TextInput::new("Type here — focus must not leak")
        .id("exer-02-focused-input")
        .placeholder("Start typing… paste fast… compose CJK via IME…")
        .description(
            "This input is the sole target of the focus-retention test. \
             Patches fire against sibling nodes at the cadence chosen \
             below; your input must remain focused and responsive.",
        )
        .bind("/demo/exer-02/focused-value")
        .build();

    Container::new()
        .id("exer-02-c1-focused-input")
        .class(CARD_CLASS)
        .children(vec![heading, focused])
        .build_tree()
}

// ---------- Card 2: Cadence control ----------

#[cfg(feature = "gallery")]
fn build_cadence_control_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Cadence control")
        .id("exer-02-c2-h2")
        .level(2)
        .build();

    // UI-SPEC §EXER-02 Cadence control copy (lines 204-213 verbatim — 4 locked
    // values, 4 locked labels, 4 locked descriptions).
    let cadence = RadioGroup::new(
        "Patch cadence",
        vec![
            RadioOption {
                value: "250".into(),
                label: "Aggressive (250 ms)".into(),
                description: Some(
                    "~4 patches per second. Stress-tests the frontend store update path.".into(),
                ),
            },
            RadioOption {
                value: "500".into(),
                label: "Default (500 ms)".into(),
                description: Some(
                    "~2 patches per second. The locked EXER-02 baseline; PATCH-02 invariant target."
                        .into(),
                ),
            },
            RadioOption {
                value: "1000".into(),
                label: "Relaxed (1000 ms)".into(),
                description: Some(
                    "~1 patch per second. Matches typical real-world server push rate.".into(),
                ),
            },
            RadioOption {
                value: "2000".into(),
                label: "Slow (2000 ms)".into(),
                description: Some(
                    "~0.5 patches per second. Sanity check; should never fail.".into(),
                ),
            },
        ],
    )
    .id("exer-02-cadence")
    .bind("/demo/exer-02/cadence-ms")
    .build();

    // UI-SPEC §EXER-02 Control row CTAs (lines 219-223 verbatim).
    let start_btn = Button::new("Start patching")
        .id("exer-02-start")
        .icon("play")
        .action(ComponentAction::click("gallery-demo/exer-02/start"))
        .build();
    let pause_btn = Button::new("Pause patching")
        .id("exer-02-pause")
        .icon("pause")
        .variant("outline")
        .action(ComponentAction::click("gallery-demo/exer-02/pause"))
        .build();
    let reset_btn = Button::new("Reset counters")
        .id("exer-02-reset")
        .icon("rotate-ccw")
        .variant("ghost")
        .action(ComponentAction::click("gallery-demo/exer-02/reset"))
        .build();

    let (btn_row, btn_row_desc) = Container::new()
        .id("exer-02-cta-row")
        .class(BUTTON_ROW_CLASS)
        .children(vec![start_btn, pause_btn, reset_btn])
        .build_tree();

    let (card_root, card_direct) = Container::new()
        .id("exer-02-c2-cadence-control")
        .class(CARD_CLASS)
        .children(vec![heading, cadence, btn_row])
        .build_tree();

    let mut desc = Vec::with_capacity(card_direct.len() + btn_row_desc.len());
    desc.extend(card_direct);
    desc.extend(btn_row_desc);
    (card_root, desc)
}

// ---------- Card 3: Invariant dashboard ----------

#[cfg(feature = "gallery")]
fn build_invariant_dashboard_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Invariant dashboard")
        .id("exer-02-c3-h2")
        .level(2)
        .build();

    // Elapsed display (UI-SPEC line 244) — bound text; hidden while empty.
    let elapsed_text = Text::new("")
        .id("exer-02-elapsed")
        .bind("/demo/exer-02/elapsed-display")
        .build();

    // UI-SPEC §EXER-02 Invariant dashboard copy (table at lines 235-240):
    //   Focus retention → icon `focus`        → bind slug `focus`
    //   Cursor position → icon `move-horizontal` → bind slug `cursor`
    //   Typed input     → icon `type`         → bind slug `typed`
    //   IME composition → icon `languages`    → bind slug `ime`
    let cell_focus = build_invariant_cell(
        "exer-02-invariant-focus",
        "Focus retention",
        "focus",
        "focus",
    );
    let cell_cursor = build_invariant_cell(
        "exer-02-invariant-cursor",
        "Cursor position",
        "move-horizontal",
        "cursor",
    );
    let cell_typed = build_invariant_cell(
        "exer-02-invariant-typed",
        "Typed input",
        "type",
        "typed",
    );
    let cell_ime = build_invariant_cell(
        "exer-02-invariant-ime",
        "IME composition",
        "languages",
        "ime",
    );

    let (grid_root, grid_desc) = Container::new()
        .id("exer-02-dashboard-grid")
        .class(DASHBOARD_GRID_CLASS)
        .children(vec![cell_focus.0, cell_cursor.0, cell_typed.0, cell_ime.0])
        .build_tree();

    let mut cells_desc = Vec::new();
    cells_desc.extend(cell_focus.1);
    cells_desc.extend(cell_cursor.1);
    cells_desc.extend(cell_typed.1);
    cells_desc.extend(cell_ime.1);

    let (card_root, card_direct) = Container::new()
        .id("exer-02-c3-invariant-dashboard")
        .class(CARD_CLASS)
        .children(vec![heading, elapsed_text, grid_root])
        .build_tree();

    let mut desc = Vec::with_capacity(card_direct.len() + grid_desc.len() + cells_desc.len());
    desc.extend(card_direct);
    desc.extend(grid_desc);
    desc.extend(cells_desc);
    (card_root, desc)
}

/// One invariant cell.
///
/// `bind_slug` is the short path suffix — the bound Text below the label
/// reads the current state string at `/demo/exer-02/invariants/{slug}/state`;
/// details Text reads `/demo/exer-02/invariants/{slug}/details`.
///
/// Note: no Badge builder exists in marionette as of v1.2 (see
/// `backend/crates/marionette/src/builders/` — only data-table's ColumnKind
/// references a Badge cell renderer). We render a Badge-styled Text node
/// bound to the state string instead. The frontend Text component renders
/// the raw string; CSS class `BADGE_CLASS` gives the visual pill look.
#[cfg(feature = "gallery")]
fn build_invariant_cell(
    id: &str,
    label: &str,
    icon: &str,
    bind_slug: &str,
) -> (Node, Vec<Node>) {
    // Icon as Container with icon prop (per CAT-05 pattern).
    let icon_node = Container::new()
        .id(format!("{id}-icon"))
        .icon(icon)
        .build();

    let label_heading = Heading::new(label)
        .id(format!("{id}-label"))
        .level(4)
        .build();

    // Badge-styled bound Text. Initial text "PENDING" is the seed value —
    // seed_for_key("exer-02") sets /demo/exer-02/invariants/{slug}/state to
    // "PENDING". The bound value replaces the literal at render time.
    //
    // Text has no .class() builder (only `text` prop) — wrap the bound Text
    // in a Badge-styled Container so the pill styling is applied. The bound
    // child Text renders the state string (PASS/FAIL/PENDING).
    let badge_text = Text::new("PENDING")
        .id(format!("{id}-badge-text"))
        .bind(format!("/demo/exer-02/invariants/{bind_slug}/state"))
        .build();
    let (badge, badge_desc) = Container::new()
        .id(format!("{id}-badge"))
        .class(BADGE_CLASS)
        .children(vec![badge_text])
        .build_tree();

    // Small details Text bound to the per-invariant details string.
    // Wrap in a Container for muted-text styling (same reason as badge above).
    let details_text = Text::new("")
        .id(format!("{id}-details-text"))
        .bind(format!("/demo/exer-02/invariants/{bind_slug}/details"))
        .build();
    let (details, details_desc) = Container::new()
        .id(format!("{id}-details"))
        .class("text-xs font-mono text-muted-foreground")
        .children(vec![details_text])
        .build_tree();

    let (cell_root, cell_direct) = Container::new()
        .id(id)
        .class(INVARIANT_CELL_CLASS)
        .children(vec![icon_node, label_heading, badge, details])
        .build_tree();

    // Flatten descendants: cell's direct children + badge-text + details-text.
    let mut desc = Vec::with_capacity(cell_direct.len() + badge_desc.len() + details_desc.len());
    desc.extend(cell_direct);
    desc.extend(badge_desc);
    desc.extend(details_desc);
    (cell_root, desc)
}

// ---------- Card 4: Patch log ----------

#[cfg(feature = "gallery")]
fn build_patch_log_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Patch log")
        .id("exer-02-c4-h2")
        .level(2)
        .build();

    // Empty-state Text (UI-SPEC line 250 verbatim). Text has no .class() builder
    // (only a `text` prop); styling comes from the Log container's class if
    // additional visual treatment is needed.
    let empty_state = Text::new("No patches yet. Press \"Start patching\" to begin.")
        .id("exer-02-log-empty")
        .build();

    // Empty log container — backend SetChildren/SetNode populates rows per tick.
    // Wrap the empty-state text in a build_tree call so the container's
    // descendants are flattened into the overall tree.
    let (log_root, log_desc) = Container::new()
        .id("exer-02-log-container")
        .class(LOG_CLASS)
        .children(vec![empty_state])
        .build_tree();

    let (card_root, card_direct) = Container::new()
        .id("exer-02-c4-patch-log")
        .class(CARD_CLASS)
        .children(vec![heading, log_root])
        .build_tree();

    let mut desc = Vec::with_capacity(card_direct.len() + log_desc.len());
    desc.extend(card_direct);
    desc.extend(log_desc);
    (card_root, desc)
}

// ---------- Tests ----------

#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;

    #[test]
    fn root_id_is_exer_02_root() {
        let v = gallery_demo();
        assert_eq!(v[0].0, "exer-02-root");
    }

    #[test]
    fn outer_class_is_locked_string() {
        let v = gallery_demo();
        let val = serde_json::to_value(&v[0].1).expect("serialize");
        assert_eq!(val["props"]["class"], OUTER_CLASS);
    }

    #[test]
    fn focused_input_binds_exact_path() {
        let v = gallery_demo();
        let (_, comp) = v
            .iter()
            .find(|(id, _)| id == "exer-02-focused-input")
            .expect("focused input node present");
        let json = serde_json::to_value(comp).expect("serialize");
        assert_eq!(json["bind"], "/demo/exer-02/focused-value");
    }

    #[test]
    fn cadence_radiogroup_has_four_options() {
        let v = gallery_demo();
        let (_, comp) = v
            .iter()
            .find(|(id, _)| id == "exer-02-cadence")
            .expect("cadence radiogroup");
        let json = serde_json::to_value(comp).expect("serialize");
        let opts = json["props"]["options"]
            .as_array()
            .expect("options array");
        assert_eq!(opts.len(), 4);
        let values: Vec<&str> = opts
            .iter()
            .map(|o| o["value"].as_str().expect("str"))
            .collect();
        assert_eq!(values, ["250", "500", "1000", "2000"]);
    }

    #[test]
    fn invariant_dashboard_has_four_cells() {
        let v = gallery_demo();
        for id in [
            "exer-02-invariant-focus",
            "exer-02-invariant-cursor",
            "exer-02-invariant-typed",
            "exer-02-invariant-ime",
        ] {
            assert!(
                v.iter().any(|(nid, _)| nid == id),
                "missing invariant cell {id}"
            );
        }
    }

    #[test]
    fn three_cta_buttons_with_correct_actions() {
        let v = gallery_demo();
        for (btn_id, expected_action) in [
            ("exer-02-start", "gallery-demo/exer-02/start"),
            ("exer-02-pause", "gallery-demo/exer-02/pause"),
            ("exer-02-reset", "gallery-demo/exer-02/reset"),
        ] {
            let (_, comp) = v
                .iter()
                .find(|(id, _)| id == btn_id)
                .unwrap_or_else(|| panic!("button {btn_id} missing"));
            let json = serde_json::to_value(comp).expect("serialize");
            let action_name = json["action"]["name"].as_str().unwrap_or("");
            assert_eq!(
                action_name, expected_action,
                "button {btn_id} action.name mismatch"
            );
            let action_type = json["action"]["type"].as_str().unwrap_or("");
            assert_eq!(
                action_type, "click",
                "button {btn_id} action.type must be click"
            );
        }
    }

    #[test]
    fn patch_log_container_visible() {
        let v = gallery_demo();
        assert!(
            v.iter().any(|(id, _)| id == "exer-02-log-container"),
            "exer-02-log-container must be present in node tree"
        );
    }

    #[test]
    fn registered_demos_includes_exer_02() {
        let e = registered_demos()
            .find(|e| e.key == "exer-02")
            .expect("exer-02 registered");
        assert_eq!(e.display_name, "Exerciser: Rapid Patching");
    }

    // ---- Extra structural assertions not strictly required by the plan's
    //      Task 1 behaviour list, but useful invariants for Plan 19-03 Task 2
    //      handlers and Plan 19-05 UAT to rely on.

    #[test]
    fn four_invariant_badges_bind_under_invariants_namespace() {
        // The badge itself is a Badge-styled Container with a child Text that
        // carries the bind; assert on the child-Text binding (id
        // `{cell-id}-badge-text`).
        let v = gallery_demo();
        for slug in ["focus", "cursor", "typed", "ime"] {
            let id = format!("exer-02-invariant-{slug}-badge-text");
            let (_, comp) = v
                .iter()
                .find(|(nid, _)| nid == &id)
                .unwrap_or_else(|| panic!("badge-text node {id} missing"));
            let json = serde_json::to_value(comp).expect("serialize");
            assert_eq!(
                json["bind"],
                format!("/demo/exer-02/invariants/{slug}/state"),
                "badge-text {id} must bind under invariants namespace"
            );
        }
    }
}
