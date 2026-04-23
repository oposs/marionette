//! CAT-05 — Typography & Tokens catalog screen.
//!
//! Three Cards stacked: Type scale, Lucide icon catalog (14 icons locked),
//! OKLCH semantic tokens (27 swatches: 26 colour tokens + 1 radius).
//! Pure static — no bind, no action, no interactive state.
//!
//! Icon-cell design LOCKED by UI-SPEC §Resolutions line 844 to "plain
//! Container with icon + label" — no Button, no action. Requires the
//! Container builder's `.icon()` setter from Plan 18-08 Task 0.
//!
//! Layout (UI-SPEC §CAT-05):
//! ```text
//! Container[outer — flex flex-col gap-6 p-6]
//!   Heading H1 "Typography & Tokens"
//!   Text    intro
//!   Container[card1 — type scale]
//!     Heading H2 "Type scale"
//!     Heading H1..H6 (one of each)
//!     Text    body
//!     Container[caption-wrapper — text-xs text-muted-foreground]
//!       Text    caption
//!   Container[card2 — icon catalog]
//!     Heading H2 "Lucide icon catalog"
//!     Container[icon-grid — grid grid-cols-4 sm:grid-cols-6 lg:grid-cols-8 gap-3]
//!       Container[icon-cell-<name> — icon="<name>"] × 14
//!         Container[labelwrap — text-xs text-muted-foreground]
//!           Text label (kebab name)
//!   Container[card3 — swatches]
//!     Heading H2 "OKLCH semantic tokens"
//!     Container[swatch-grid — grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-6 gap-3]
//!       Container[swatch-cell-<token>]                        × 27
//!         Container[swatch-box-<token> — w-full h-16 bg-<t>]
//!         Container[labelwrap — text-xs font-mono]
//!           Text --<token>
//! ```

#![allow(clippy::too_many_lines)]

use marionette::builders::{Container, Heading, Text};
use marionette::gallery::Node;

// ---------- Locked CSS class strings (UI-SPEC §Spacing Scale / §CAT-05) ----------

const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
const ICON_GRID_CLASS: &str = "grid grid-cols-4 sm:grid-cols-6 lg:grid-cols-8 gap-3";
const SWATCH_GRID_CLASS: &str = "grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-6 gap-3";
const ICON_CELL_CLASS: &str = "flex flex-col items-center gap-1 p-2 rounded border";
const SWATCH_CELL_CLASS: &str = "flex flex-col gap-2";
const LABEL_CLASS_ICON: &str = "text-xs text-muted-foreground";
const LABEL_CLASS_TOKEN: &str = "text-xs font-mono text-muted-foreground";
const CAPTION_CLASS: &str = "text-xs text-muted-foreground";
const RADIUS_BOX_CLASS: &str = "w-16 h-16 rounded border bg-muted";

/// Icons from UI-SPEC §CAT-05 line 594 — the 14 names registered in
/// `frontend/src/lib/registry/icons.ts` (kebab-case, in declared order).
const ICONS: [&str; 14] = [
    "plus",
    "chevron-up",
    "chevron-down",
    "alert-circle",
    "x",
    "menu",
    "arrow-left",
    "search",
    "filter",
    "pencil",
    "trash",
    "check",
    "loader",
    "circle-help",
];

/// OKLCH tokens from UI-SPEC §CAT-05 lines 601-630 — 26 colour tokens in
/// `:root` declaration order (see `frontend/src/app.css`).
const COLOUR_TOKENS: [&str; 26] = [
    "background",
    "foreground",
    "card",
    "card-foreground",
    "popover",
    "popover-foreground",
    "primary",
    "primary-foreground",
    "secondary",
    "secondary-foreground",
    "muted",
    "muted-foreground",
    "accent",
    "accent-foreground",
    "destructive",
    "border",
    "input",
    "ring",
    "sidebar",
    "sidebar-foreground",
    "sidebar-primary",
    "sidebar-primary-foreground",
    "sidebar-accent",
    "sidebar-accent-foreground",
    "sidebar-border",
    "sidebar-ring",
];

// ---------- Top-level demo fn ----------

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-typography", name = "Catalog: Typography")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // Title + intro (locked copy from UI-SPEC §Copywriting Contract).
    let title = Heading::new("Typography & Tokens")
        .id("catalog-typography-title")
        .level(1)
        .build();
    let intro = Text::new(
        "Type scale, lucide-svelte icon catalog, and OKLCH semantic colour tokens \
         used by every screen in the gallery.",
    )
    .id("catalog-typography-intro")
    .build();

    let (card1_root, card1_desc) = build_type_scale_card();
    let (card2_root, card2_desc) = build_icon_catalog_card();
    let (card3_root, card3_desc) = build_swatch_card();

    let (outer_root, outer_direct) = Container::new()
        .id("catalog-typography-root")
        .class(OUTER_CLASS)
        .children(vec![title, intro, card1_root, card2_root, card3_root])
        .build_tree();

    let mut result: Vec<Node> = Vec::with_capacity(
        1 + outer_direct.len() + card1_desc.len() + card2_desc.len() + card3_desc.len(),
    );
    result.push(outer_root);
    result.extend(outer_direct);
    result.extend(card1_desc);
    result.extend(card2_desc);
    result.extend(card3_desc);
    result
}

// ---------- Card 1: Type scale ----------

#[cfg(feature = "gallery")]
fn build_type_scale_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Type scale")
        .id("catalog-typo-card1-heading")
        .level(2)
        .build();

    let h1 = Heading::new("Heading 1 — sample")
        .id("catalog-typo-h1")
        .level(1)
        .build();
    let h2 = Heading::new("Heading 2 — sample")
        .id("catalog-typo-h2")
        .level(2)
        .build();
    let h3 = Heading::new("Heading 3 — sample")
        .id("catalog-typo-h3")
        .level(3)
        .build();
    let h4 = Heading::new("Heading 4 — sample")
        .id("catalog-typo-h4")
        .level(4)
        .build();
    let h5 = Heading::new("Heading 5 — sample")
        .id("catalog-typo-h5")
        .level(5)
        .build();
    let h6 = Heading::new("Heading 6 — sample")
        .id("catalog-typo-h6")
        .level(6)
        .build();

    let body = Text::new("Body text. The quick brown fox jumps over the lazy dog.")
        .id("catalog-typo-body")
        .build();

    // Caption: Text has no .class() prop (PATTERNS.md line 360-364) — wrap
    // in a small Container that carries the `text-xs text-muted-foreground`
    // class so the caption renders at the right scale.
    let caption_inner = Text::new("Caption / label — small descriptive text")
        .id("catalog-typo-caption-inner")
        .build();
    let (caption_root, caption_desc) = Container::new()
        .id("catalog-typo-caption-wrapper")
        .class(CAPTION_CLASS)
        .children(vec![caption_inner])
        .build_tree();

    let (card_root, card_direct) = Container::new()
        .id("catalog-typo-card1")
        .class(CARD_CLASS)
        .children(vec![heading, h1, h2, h3, h4, h5, h6, body, caption_root])
        .build_tree();

    let mut descendants: Vec<Node> = Vec::with_capacity(card_direct.len() + caption_desc.len());
    descendants.extend(card_direct);
    descendants.extend(caption_desc);
    (card_root, descendants)
}

// ---------- Card 2: Lucide icon catalog (14 icons, display-only) ----------

#[cfg(feature = "gallery")]
fn build_icon_catalog_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Lucide icon catalog")
        .id("catalog-typo-card2-heading")
        .level(2)
        .build();

    // Build 14 icon cells. Each cell is a plain Container carrying the
    // `.icon(name)` prop (new in Plan 18-08 Task 0) + a label-wrapper child
    // holding the kebab name. NO Button — UI-SPEC §Resolutions line 844
    // locks the icon catalog to display-only.
    let mut cell_roots: Vec<Node> = Vec::with_capacity(ICONS.len());
    let mut cell_descendants: Vec<Node> = Vec::new();

    for name in ICONS {
        // Text label is wrapped in a small Container so it picks up the
        // `text-xs text-muted-foreground` class (Text has no .class()).
        let label_inner = Text::new(name)
            .id(format!("catalog-typo-icon-label-{name}"))
            .build();
        let (labelwrap_root, labelwrap_desc) = Container::new()
            .id(format!("catalog-typo-icon-labelwrap-{name}"))
            .class(LABEL_CLASS_ICON)
            .children(vec![label_inner])
            .build_tree();

        let (cell_root, cell_direct) = Container::new()
            .id(format!("catalog-typo-icon-cell-{name}"))
            .class(ICON_CELL_CLASS)
            .icon(name)
            .children(vec![labelwrap_root])
            .build_tree();

        cell_roots.push(cell_root);
        cell_descendants.extend(cell_direct);
        cell_descendants.extend(labelwrap_desc);
    }

    let (grid_root, grid_desc) = Container::new()
        .id("catalog-typo-icon-grid")
        .class(ICON_GRID_CLASS)
        .children(cell_roots)
        .build_tree();

    let (card_root, card_direct) = Container::new()
        .id("catalog-typo-card2")
        .class(CARD_CLASS)
        .children(vec![heading, grid_root])
        .build_tree();

    let mut descendants: Vec<Node> = Vec::with_capacity(
        card_direct.len() + grid_desc.len() + cell_descendants.len(),
    );
    descendants.extend(card_direct);
    descendants.extend(grid_desc);
    descendants.extend(cell_descendants);
    (card_root, descendants)
}

// ---------- Card 3: OKLCH semantic tokens (26 colours + 1 radius) ----------

#[cfg(feature = "gallery")]
fn build_swatch_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("OKLCH semantic tokens")
        .id("catalog-typo-card3-heading")
        .level(2)
        .build();

    let mut cell_roots: Vec<Node> = Vec::with_capacity(COLOUR_TOKENS.len() + 1);
    let mut cell_descendants: Vec<Node> = Vec::new();

    for token in COLOUR_TOKENS {
        // Colour box — a standalone Container whose only job is to paint
        // the `bg-<token>` swatch area. No children.
        let colour_box = Container::new()
            .id(format!("catalog-typo-swatch-box-{token}"))
            .class(format!("w-full h-16 rounded-md border bg-{token}"))
            .build();

        // Token label (e.g. "--primary") wrapped in the mono-font small class.
        let label_inner = Text::new(format!("--{token}"))
            .id(format!("catalog-typo-swatch-label-{token}"))
            .build();
        let (labelwrap_root, labelwrap_desc) = Container::new()
            .id(format!("catalog-typo-swatch-labelwrap-{token}"))
            .class(LABEL_CLASS_TOKEN)
            .children(vec![label_inner])
            .build_tree();

        let (cell_root, cell_direct) = Container::new()
            .id(format!("catalog-typo-swatch-cell-{token}"))
            .class(SWATCH_CELL_CLASS)
            .children(vec![colour_box, labelwrap_root])
            .build_tree();

        cell_roots.push(cell_root);
        cell_descendants.extend(cell_direct);
        cell_descendants.extend(labelwrap_desc);
    }

    // Radius demo cell (UI-SPEC §CAT-05 line 631) — shows the --radius token
    // as a rendered 64×64 box so the value is visible, not just named.
    let radius_box = Container::new()
        .id("catalog-typo-swatch-box-radius")
        .class(RADIUS_BOX_CLASS)
        .build();
    let radius_label_inner = Text::new("--radius")
        .id("catalog-typo-swatch-label-radius")
        .build();
    let (radius_labelwrap_root, radius_labelwrap_desc) = Container::new()
        .id("catalog-typo-swatch-labelwrap-radius")
        .class(LABEL_CLASS_TOKEN)
        .children(vec![radius_label_inner])
        .build_tree();
    let (radius_cell_root, radius_cell_direct) = Container::new()
        .id("catalog-typo-swatch-cell-radius")
        .class(SWATCH_CELL_CLASS)
        .children(vec![radius_box, radius_labelwrap_root])
        .build_tree();
    cell_roots.push(radius_cell_root);
    cell_descendants.extend(radius_cell_direct);
    cell_descendants.extend(radius_labelwrap_desc);

    let (grid_root, grid_desc) = Container::new()
        .id("catalog-typo-swatch-grid")
        .class(SWATCH_GRID_CLASS)
        .children(cell_roots)
        .build_tree();

    let (card_root, card_direct) = Container::new()
        .id("catalog-typo-card3")
        .class(CARD_CLASS)
        .children(vec![heading, grid_root])
        .build_tree();

    let mut descendants: Vec<Node> = Vec::with_capacity(
        card_direct.len() + grid_desc.len() + cell_descendants.len(),
    );
    descendants.extend(card_direct);
    descendants.extend(grid_desc);
    descendants.extend(cell_descendants);
    (card_root, descendants)
}

// ---------- Tests ----------

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
        assert_eq!(v[0].0, "catalog-typography-root");
    }

    #[test]
    fn six_heading_levels_present() {
        let v = gallery_demo();
        for level in 1..=6 {
            let id = format!("catalog-typo-h{level}");
            let node = find(&v, &id)
                .unwrap_or_else(|| panic!("missing heading {id}"));
            assert_eq!(node["type"], "heading");
            assert_eq!(node["props"]["level"], level);
        }
    }

    #[test]
    fn fourteen_icon_cells_with_locked_names() {
        // LOCKED by UI-SPEC §Resolutions line 844: each icon cell is a
        // plain Container with `icon` + label — NOT a Button-without-action.
        let v = gallery_demo();
        for name in ICONS {
            let id = format!("catalog-typo-icon-cell-{name}");
            let node = find(&v, &id)
                .unwrap_or_else(|| panic!("missing icon cell {id}"));
            assert_eq!(node["type"], "container", "cell {id} must be container, not button");
            assert_eq!(node["props"]["icon"], name, "cell {id} must carry icon={name}");
            let class = node["props"]["class"].as_str().unwrap_or("");
            assert!(class.contains("flex"), "cell {id} class missing flex: {class}");
            // Regression guard: no action on the cell (display-only per
            // UI-SPEC line 844). `action` is a top-level Component field
            // (see marionette_protocol::Component), so check with .get().
            assert!(
                node.get("action").is_none() || node["action"].is_null(),
                "cell {id} must have no action — UI-SPEC §Resolutions line 844 \
                 locks display-only"
            );
        }
    }

    #[test]
    fn no_buttons_in_icon_catalog_subtree() {
        // Hard regression guard against the superseded PATTERNS.md §CAT-05
        // Button-based composition. UI-SPEC §Resolutions line 844 locks
        // the icon catalog to display-only Containers.
        let v = gallery_demo();
        for (id, c) in &v {
            if id.starts_with("catalog-typo-icon-") {
                let s = serde_json::to_value(c).expect("serialize");
                assert_ne!(
                    s["type"], "button",
                    "node {id} is a button — violates UI-SPEC §Resolutions line 844 \
                     (CAT-05 icon cells must be plain Containers, not Buttons)"
                );
            }
        }
    }

    #[test]
    fn twenty_seven_swatch_cells() {
        let v = gallery_demo();
        let count = v
            .iter()
            .filter(|(id, _)| id.starts_with("catalog-typo-swatch-cell-"))
            .count();
        assert_eq!(
            count, 27,
            "expected 26 colour tokens + 1 radius = 27 swatch cells"
        );
    }

    #[test]
    fn colour_swatch_boxes_use_bg_token_class() {
        let v = gallery_demo();
        for token in COLOUR_TOKENS {
            let id = format!("catalog-typo-swatch-box-{token}");
            let node = find(&v, &id)
                .unwrap_or_else(|| panic!("missing swatch box {id}"));
            let class = node["props"]["class"].as_str().unwrap_or("");
            let expected = format!("bg-{token}");
            assert!(
                class.contains(&expected),
                "swatch {token} class missing {expected}: {class}"
            );
        }
    }

    #[test]
    fn radius_swatch_cell_present() {
        let v = gallery_demo();
        assert!(
            v.iter().any(|(i, _)| i == "catalog-typo-swatch-cell-radius"),
            "missing radius cell"
        );
        let box_node = find(&v, "catalog-typo-swatch-box-radius").expect("radius box");
        let class = box_node["props"]["class"].as_str().unwrap_or("");
        assert!(
            class.contains("rounded") && class.contains("bg-muted"),
            "radius demo box class missing rounded/bg-muted: {class}"
        );
    }

    #[test]
    fn registered_demos_includes_catalog_typography() {
        let e = registered_demos()
            .find(|e| e.key == "catalog-typography")
            .expect("registered");
        assert_eq!(e.display_name, "Catalog: Typography");
    }
}
