//! `Grid` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "grid")]
pub struct Grid {
    #[builder(optional)]
    pub cols: Option<u8>,
    #[builder(optional)]
    pub gap: Option<String>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "grid")]
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    // 2x3 Grid of Heading placeholders showing the grid layout shape.
    // Single-char names mirror spreadsheet-cell style (A/B/C/D/E/F) for
    // readability; clippy's many_single_char_names lint is silenced
    // locally because the names intentionally reflect the rendered labels.
    let a = crate::builders::heading::Heading::new("A")
        .id("demo-grid-a")
        .build();
    let b = crate::builders::heading::Heading::new("B")
        .id("demo-grid-b")
        .build();
    let c = crate::builders::heading::Heading::new("C")
        .id("demo-grid-c")
        .build();
    let d = crate::builders::heading::Heading::new("D")
        .id("demo-grid-d")
        .build();
    let e = crate::builders::heading::Heading::new("E")
        .id("demo-grid-e")
        .build();
    let f = crate::builders::heading::Heading::new("F")
        .id("demo-grid-f")
        .build();

    Grid::new()
        .id("demo-grid-root")
        .cols(3)
        .gap("1rem")
        .children(vec![a, b, c, d, e, f])
        .build_with_children()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_with_optional_fields() {
        let (_, component) = Grid::new().cols(3).gap("1rem").build();
        let props = component.props.unwrap();
        assert_eq!(props["cols"], 3);
        assert_eq!(props["gap"], "1rem");
    }
}
