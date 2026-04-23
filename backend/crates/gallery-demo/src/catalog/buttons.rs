//! CAT-01 — Buttons & Actions catalog screen.
//!
//! Per CONTEXT.md §D-2-B, composes the full variant × size × state matrix
//! fresh from `marionette::builders::Button` — does NOT invoke the leaf
//! `button::gallery_demo()`. Outer layout + card + inner grid classes are
//! locked in UI-SPEC.md §Spacing Scale and §CAT-01.

#![allow(clippy::too_many_lines)]

use marionette::gallery::Node;

/// Locked CSS class strings (UI-SPEC §Spacing Scale).
const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";

const VARIANTS: [&str; 5] = ["default", "destructive", "outline", "ghost", "link"];

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-buttons", name = "Catalog: Buttons")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // RED stub — full implementation arrives in the next commit.
    Vec::new()
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
