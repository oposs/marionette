//! CAT-01 — Buttons & Actions catalog screen.
//!
//! Per CONTEXT.md §D-2-B, composes the full variant × size × state matrix
//! fresh from `marionette::builders::Button` — does NOT invoke the leaf
//! `button::gallery_demo()`. Outer layout + card + inner grid classes are
//! locked in UI-SPEC.md §Spacing Scale and §CAT-01.
//!
//! Total cell count: 5 variants × 3 sizes × 4 states = **60 Buttons**.
//! Layout: 5 per-variant Cards, each with an H3 legend + a 4-column
//! responsive grid of 12 cells.

#![allow(clippy::too_many_lines)]

use marionette::builders::{Button, Container, Heading, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

/// Locked CSS class strings (UI-SPEC §Spacing Scale).
const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
const INNER_GRID_CLASS: &str = "grid grid-cols-1 sm:grid-cols-4 lg:grid-cols-4 gap-3";

const VARIANTS: [&str; 5] = ["default", "destructive", "outline", "ghost", "link"];
const SIZES: [&str; 3] = ["sm", "default", "lg"];

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-buttons", name = "Catalog: Buttons")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // --- title + intro (locked copy from UI-SPEC §Copywriting Contract) ---
    let title = Heading::new("Buttons & Actions")
        .id("catalog-buttons-title")
        .level(1)
        .build();
    let intro = Text::new(
        "Every Button variant × size × state visible at once. \
         Mobile: stacks vertically. Desktop: 4-column grid inside each variant Card.",
    )
    .id("catalog-buttons-intro")
    .build();

    // --- Build 5 per-variant Cards. Each call returns:
    //   (card_root_tuple, card_descendants)
    // where card_descendants already includes legend + grid_root + 12 cells.
    let mut card_roots: Vec<Node> = Vec::with_capacity(VARIANTS.len());
    let mut all_descendants: Vec<Node> = Vec::with_capacity(VARIANTS.len() * 14);
    for variant in VARIANTS {
        let (card_root, card_desc) = build_variant_card(variant);
        card_roots.push(card_root);
        all_descendants.extend(card_desc);
    }

    // --- outer root Container: title + intro + 5 card-root tuples ---
    let mut root_children: Vec<Node> = Vec::with_capacity(2 + card_roots.len());
    root_children.push(title);
    root_children.push(intro);
    root_children.extend(card_roots);

    let outer = Container::new()
        .id("catalog-buttons-root")
        .class(OUTER_CLASS)
        .children(root_children)
        .build_with_children();

    // `build_with_children` returns [root_tuple, ...direct_child_tuples].
    // Descendants below the direct-child layer (legend/grid/cells) were
    // collected in `all_descendants` above. Splice them in so the final
    // Vec<Node> is flat and every node appears exactly once.
    let mut result: Vec<Node> = Vec::with_capacity(outer.len() + all_descendants.len());
    result.extend(outer);
    result.extend(all_descendants);
    result
}

/// Build one per-variant Card: H3 legend + responsive inner grid of 12 cells.
///
/// Returns `(card_root_tuple, descendants)` where `descendants` is a flat
/// `Vec<Node>` containing: `[legend, grid_root, cell_1, cell_2, …, cell_12]`.
#[cfg(feature = "gallery")]
fn build_variant_card(variant: &str) -> (Node, Vec<Node>) {
    // --- H3 legend ---
    let legend = Heading::new(format!("variant = {variant}"))
        .id(format!("catalog-buttons-{variant}-legend"))
        .level(3)
        .build();

    // --- 12 cells = 3 sizes × 4 states (normal, disabled, loading, icon-only) ---
    let mut cells: Vec<Node> = Vec::with_capacity(12);
    for size in SIZES {
        // normal
        cells.push(
            Button::new(format!("{variant}/{size}"))
                .id(format!("cb-{variant}-{size}-normal"))
                .variant(variant)
                .size(size)
                .action(ComponentAction::click("gallery-demo/noop"))
                .build(),
        );
        // disabled (no action — click suppressed by the disabled attribute)
        cells.push(
            Button::new(format!("{variant}/{size}"))
                .id(format!("cb-{variant}-{size}-disabled"))
                .variant(variant)
                .size(size)
                .disabled(true)
                .build(),
        );
        // loading (Loader2 spinner rendered by Button.svelte per Plan 18-01)
        cells.push(
            Button::new(format!("{variant}/{size}"))
                .id(format!("cb-{variant}-{size}-loading"))
                .variant(variant)
                .size(size)
                .loading(true)
                .build(),
        );
        // icon-only (no label; `plus` icon from registry; aria-label required)
        cells.push(
            Button::new("")
                .id(format!("cb-{variant}-{size}-icon"))
                .variant(variant)
                .size(size)
                .icon("plus")
                .aria_label(format!("{variant} {size} icon button"))
                .action(ComponentAction::click("gallery-demo/noop"))
                .build(),
        );
    }

    // --- Inner responsive grid (Container with the locked grid class) ---
    // `build_tree` returns (root, descendants) so we can keep the 12 cells
    // as siblings in our descendants list without re-flattening.
    let (grid_root, grid_desc) = Container::new()
        .id(format!("catalog-buttons-{variant}-grid"))
        .class(INNER_GRID_CLASS)
        .children(cells)
        .build_tree();

    // --- Card (Container with the locked Card class) ---
    // The Card's direct children are the legend + the inner-grid ROOT tuple.
    let (card_root, card_direct) = Container::new()
        .id(format!("catalog-buttons-card-{variant}"))
        .class(CARD_CLASS)
        .children(vec![legend, grid_root])
        .build_tree();

    // Flatten: card's direct children (legend + grid_root) + grid's own
    // descendants (the 12 cells). `build_tree` already separated root from
    // descendants, so we concatenate the leftover tuples here.
    let mut descendants: Vec<Node> = Vec::with_capacity(card_direct.len() + grid_desc.len());
    descendants.extend(card_direct);
    descendants.extend(grid_desc);
    (card_root, descendants)
}

#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;

    #[test]
    fn root_id_is_catalog_buttons_root() {
        let v = gallery_demo();
        assert_eq!(v[0].0, "catalog-buttons-root", "first entry is the root");
    }

    #[test]
    fn outer_class_is_locked_string() {
        let v = gallery_demo();
        let root_comp = &v[0].1;
        let val = serde_json::to_value(root_comp).expect("serialize");
        assert_eq!(
            val["props"]["class"], OUTER_CLASS,
            "outer class must match locked spec"
        );
    }

    #[test]
    fn exactly_five_card_roots_with_locked_class() {
        let v = gallery_demo();
        let mut card_count = 0;
        for (id, comp) in &v {
            if id.starts_with("catalog-buttons-card-") {
                let val = serde_json::to_value(comp).expect("serialize");
                assert_eq!(val["props"]["class"], CARD_CLASS);
                card_count += 1;
            }
        }
        assert_eq!(card_count, VARIANTS.len(), "expected 5 variant cards");
    }

    #[test]
    fn exactly_sixty_button_instances() {
        let v = gallery_demo();
        let count = v
            .iter()
            .filter(|(_, c)| {
                let s = serde_json::to_value(c).expect("serialize");
                s["type"] == "button"
            })
            .count();
        assert_eq!(count, 60, "5 variants × 3 sizes × 4 states = 60");
    }

    #[test]
    fn every_button_has_expected_variant_and_size_props() {
        let v = gallery_demo();
        let allowed_variants = ["default", "destructive", "outline", "ghost", "link"];
        let allowed_sizes = ["sm", "default", "lg"];
        for (id, c) in &v {
            let s = serde_json::to_value(c).expect("serialize");
            if s["type"] == "button" {
                let variant = s["props"]["variant"].as_str().unwrap_or("");
                let size = s["props"]["size"].as_str().unwrap_or("");
                assert!(
                    allowed_variants.contains(&variant),
                    "button {id} variant={variant}"
                );
                assert!(allowed_sizes.contains(&size), "button {id} size={size}");
            }
        }
    }

    #[test]
    fn registered_demos_includes_catalog_buttons() {
        let found = registered_demos().find(|e| e.key == "catalog-buttons");
        assert!(found.is_some(), "catalog-buttons must register via linkme");
        let entry = found.unwrap();
        assert_eq!(entry.display_name, "Catalog: Buttons");
        let rendered = (entry.render)();
        assert_eq!(rendered[0].0, "catalog-buttons-root");
    }
}
