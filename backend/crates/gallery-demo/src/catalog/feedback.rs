//! CAT-04 — Feedback catalog screen.
//!
//! Two Cards side-by-side: (Card 1) trigger surfaces for toast / modal /
//! confirm-dialog + (Card 2) placeholder states for empty / loading / error.
//! All three triggers reuse existing Phase 17 handlers (`gallery-demo/toast-fire`,
//! `gallery-demo/modal-open`, `gallery-demo/confirm-open`) — Plan 18-07 does
//! NOT introduce new handlers (D-2-C). The placeholder mini-Cards are purely
//! static visuals composed from Container + Heading + Text + Spinner + ErrorDisplay
//! builders. Per D-2-C, the W-06 ErrorDisplay `message`-field dead-state is
//! NOT fixed here (deferred to a later polish plan).
//!
//! Layout (UI-SPEC §CAT-04 lines 494-544):
//! ```text
//! Container[outer]
//!   Heading H1 "Feedback"
//!   Text    intro
//!   Container[card 1 — trigger surfaces]
//!     Heading H2 "Trigger surfaces"
//!     Container[trigger-grid: grid grid-cols-1 sm:grid-cols-3 gap-3]
//!       Button "Fire toast" + Button "Open modal" + Button "Open confirm dialog"
//!   Container[card 2 — placeholder states]
//!     Heading H2 "Placeholder states"
//!     Container[placeholder-grid: grid grid-cols-1 sm:grid-cols-3 gap-3]
//!       Container[empty: border-dashed]   H4 + Text
//!       Container[loading: border]        Spinner + Text
//!       ErrorDisplay (bound to /demo/catalog-feedback/errors)
//! ```

use marionette::builders::{Button, Container, ErrorDisplay, Heading, Spinner, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

// ---------- Locked CSS class strings (UI-SPEC §Spacing Scale / §CAT-04) ----------

const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
const INNER_GRID_CLASS: &str = "grid grid-cols-1 sm:grid-cols-3 gap-3";
const EMPTY_CLASS: &str = "rounded-md border-2 border-dashed p-8 flex flex-col items-center \
                           justify-center gap-2 text-center text-muted-foreground";
const LOADING_CLASS: &str =
    "rounded-md border p-8 flex flex-col items-center justify-center gap-3";

// ---------- Top-level demo fn ----------

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-feedback", name = "Catalog: Feedback")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // --- Title + intro (locked copy from UI-SPEC §Copywriting Contract) ---
    let title = Heading::new("Feedback")
        .id("catalog-feedback-title")
        .level(1)
        .build();
    let intro = Text::new(
        "Toast, modal, and confirm-dialog triggers side-by-side, plus the three \
         placeholder states (empty / loading / error) rendered statically.",
    )
    .id("catalog-feedback-intro")
    .build();

    // --- Build both Cards. Each helper returns (card_root_tuple, descendants). ---
    let (card1_root, card1_desc) = build_trigger_card();
    let (card2_root, card2_desc) = build_placeholder_card();

    // --- Outer root: title + intro + 2 card-root tuples. ---
    let (outer_root, outer_direct) = Container::new()
        .id("catalog-feedback-root")
        .class(OUTER_CLASS)
        .children(vec![title, intro, card1_root, card2_root])
        .build_tree();

    // --- Flatten: outer root + outer direct children + both cards' descendants. ---
    let mut result: Vec<Node> =
        Vec::with_capacity(1 + outer_direct.len() + card1_desc.len() + card2_desc.len());
    result.push(outer_root);
    result.extend(outer_direct);
    result.extend(card1_desc);
    result.extend(card2_desc);
    result
}

// ---------- Card 1: Trigger surfaces ----------

#[cfg(feature = "gallery")]
fn build_trigger_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Trigger surfaces")
        .id("catalog-feedback-card1-heading")
        .level(2)
        .build();

    // Three trigger Buttons — each fires an existing Phase 17 handler verbatim.
    let toast_btn = Button::new("Fire toast")
        .id("catalog-feedback-toast-trigger")
        .action(ComponentAction::click("gallery-demo/toast-fire"))
        .build();
    let modal_btn = Button::new("Open modal")
        .id("catalog-feedback-modal-trigger")
        .action(ComponentAction::click("gallery-demo/modal-open"))
        .build();
    let confirm_btn = Button::new("Open confirm dialog")
        .id("catalog-feedback-confirm-trigger")
        .action(ComponentAction::click("gallery-demo/confirm-open"))
        .build();

    let (grid_root, grid_desc) = Container::new()
        .id("catalog-feedback-trigger-grid")
        .class(INNER_GRID_CLASS)
        .children(vec![toast_btn, modal_btn, confirm_btn])
        .build_tree();

    let (card_root, card_direct) = Container::new()
        .id("catalog-feedback-card1")
        .class(CARD_CLASS)
        .children(vec![heading, grid_root])
        .build_tree();

    // Card direct children (heading + grid root tuple) + grid's descendants
    // (the 3 trigger buttons).
    let mut descendants: Vec<Node> = Vec::with_capacity(card_direct.len() + grid_desc.len());
    descendants.extend(card_direct);
    descendants.extend(grid_desc);
    (card_root, descendants)
}

// ---------- Card 2: Placeholder states ----------

#[cfg(feature = "gallery")]
fn build_placeholder_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Placeholder states")
        .id("catalog-feedback-card2-heading")
        .level(2)
        .build();

    // Empty placeholder (UI-SPEC §CAT-04 lines 537-538: locked copy).
    let empty_h = Heading::new("No data yet")
        .id("catalog-feedback-empty-h")
        .level(4)
        .build();
    let empty_body = Text::new(
        "Start by adding your first item — empty states should always tell users \
         what to do next.",
    )
    .id("catalog-feedback-empty-body")
    .build();
    let (empty_root, empty_desc) = Container::new()
        .id("catalog-feedback-empty")
        .class(EMPTY_CLASS)
        .children(vec![empty_h, empty_body])
        .build_tree();

    // Loading placeholder — Spinner md + Text "Loading…" (UI-SPEC §CAT-04 line 535).
    let spinner = Spinner::new()
        .size("md")
        .id("catalog-feedback-loading-spinner")
        .build();
    let loading_label = Text::new("Loading…")
        .id("catalog-feedback-loading-label")
        .build();
    let (loading_root, loading_desc) = Container::new()
        .id("catalog-feedback-loading")
        .class(LOADING_CLASS)
        .children(vec![spinner, loading_label])
        .build_tree();

    // Error placeholder — ErrorDisplay bound to the seed path. The positional
    // `message` arg is dead per Phase 17 W-06 (frontend reads errors from
    // `bind` only); we pass a short label for future-proofing if W-06 is
    // resolved as a bind-fallback.
    let error_display = ErrorDisplay::new("errors")
        .id("catalog-feedback-error")
        .bind("/demo/catalog-feedback/errors")
        .build();

    let (grid_root, grid_desc) = Container::new()
        .id("catalog-feedback-placeholder-grid")
        .class(INNER_GRID_CLASS)
        .children(vec![empty_root, loading_root, error_display])
        .build_tree();

    let (card_root, card_direct) = Container::new()
        .id("catalog-feedback-card2")
        .class(CARD_CLASS)
        .children(vec![heading, grid_root])
        .build_tree();

    // Flatten: card direct children + grid descendants (empty root + loading
    // root + error display tuple) + empty's own descendants (H4 + body) +
    // loading's own descendants (spinner + label).
    let mut descendants: Vec<Node> = Vec::with_capacity(
        card_direct.len() + grid_desc.len() + empty_desc.len() + loading_desc.len(),
    );
    descendants.extend(card_direct);
    descendants.extend(grid_desc);
    descendants.extend(empty_desc);
    descendants.extend(loading_desc);
    (card_root, descendants)
}

#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;

    fn find(v: &[Node], id: &str) -> Option<serde_json::Value> {
        v.iter()
            .find(|(i, _)| i == id)
            .map(|(_, c)| serde_json::to_value(c).expect("serialize"))
    }

    #[test]
    fn root_id() {
        let v = gallery_demo();
        assert_eq!(v[0].0, "catalog-feedback-root");
    }

    #[test]
    fn three_trigger_buttons_with_locked_actions() {
        // Component's `action` serializes at the top level (see
        // marionette_protocol::Component — `#[serde] pub action: Option<...>`
        // is a sibling of `type`, not nested under `props`). Plan 18-07's
        // test hint used `props.action`, which is wrong; fixed per Rule 1.
        let v = gallery_demo();
        let toast = find(&v, "catalog-feedback-toast-trigger").expect("toast trigger");
        assert_eq!(toast["type"], "button");
        assert_eq!(toast["action"]["name"], "gallery-demo/toast-fire");
        assert_eq!(toast["action"]["type"], "click");
        let modal = find(&v, "catalog-feedback-modal-trigger").expect("modal trigger");
        assert_eq!(modal["action"]["name"], "gallery-demo/modal-open");
        let confirm = find(&v, "catalog-feedback-confirm-trigger").expect("confirm trigger");
        assert_eq!(confirm["action"]["name"], "gallery-demo/confirm-open");
    }

    #[test]
    fn empty_placeholder_has_border_dashed_class() {
        let v = gallery_demo();
        let empty = find(&v, "catalog-feedback-empty").expect("empty placeholder");
        let class = empty["props"]["class"].as_str().unwrap_or("");
        assert!(
            class.contains("border-dashed"),
            "empty class missing border-dashed: {class}"
        );
        assert!(class.contains("text-muted-foreground"));
    }

    #[test]
    fn loading_placeholder_has_spinner() {
        let v = gallery_demo();
        let spinner = find(&v, "catalog-feedback-loading-spinner").expect("spinner");
        assert_eq!(spinner["type"], "spinner");
    }

    #[test]
    fn error_display_bound_to_seeded_path() {
        // `bind` is a top-level Component field, not nested under `props`.
        // Plan 18-07's hint used `props.bind`; fixed per Rule 1.
        let v = gallery_demo();
        let err = find(&v, "catalog-feedback-error").expect("error display");
        assert_eq!(err["type"], "error-display");
        assert_eq!(err["bind"], "/demo/catalog-feedback/errors");
    }

    #[test]
    fn registered_demos_includes_catalog_feedback() {
        let e = registered_demos()
            .find(|e| e.key == "catalog-feedback")
            .expect("registered");
        assert_eq!(e.display_name, "Catalog: Feedback");
    }
}
