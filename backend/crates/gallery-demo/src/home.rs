//! Curated Home page — welcome Heading + explanatory Text + Grid of tiles
//! derived from `registered_demos()`.
//!
//! Per CONTEXT.md §D-C2, the Home page is the gallery's intentional first
//! impression: a hand-authored entry point that also auto-adapts to new
//! demo registrations (tile list is registry-derived).

use std::collections::HashMap;

use marionette::builders::{Button, Container, Grid, Heading, Text};
use marionette::gallery::registered_demos;
use marionette_protocol::{Component, ComponentAction};

/// Build the Home page: welcome Heading + intro Text + Grid of tiles.
///
/// Returns `(root_id, nodes_map, data)` — the shape expected by the
/// `content` sub-surface Render emitted by `handle_navigate`.
///
/// Tiles are derived from `registered_demos()` so new `#[gallery_demo]`
/// additions appear automatically on the next rebuild (CONTEXT.md §D-C2).
#[must_use]
pub fn build_home_page() -> (String, HashMap<String, Component>, serde_json::Value) {
    let welcome = Heading::new("Marionette Gallery")
        .id("home-welcome")
        .level(1)
        .build();
    let intro = Text::new(
        "Visual-iteration harness and SDUI-frontend exerciser. \
         Pick a component from the sidebar, or click a tile below.",
    )
    .id("home-intro")
    .build();

    // Tile per registered demo (CONTEXT.md §D-C2: registry-derived).
    let tiles: Vec<(String, Component)> = registered_demos()
        .map(|entry| {
            let mut action = ComponentAction::click("gallery-show");
            action.extra.insert(
                "payload".into(),
                serde_json::json!({ "key": entry.key }),
            );
            Button::new(entry.display_name)
                .id(format!("home-tile-{}", entry.key))
                .variant("outline")
                .action(action)
                .build()
        })
        .collect();

    let (grid_root, grid_desc) = Grid::new()
        .id("home-grid")
        .cols(3)
        .gap("1rem")
        .children(tiles)
        .build_tree();

    let root_id = "home-root".to_string();
    let container_nodes = Container::new()
        .id(&root_id)
        .children(vec![welcome, intro, grid_root])
        .build_with_children();

    let mut nodes: HashMap<String, Component> = HashMap::new();
    for (id, c) in container_nodes {
        nodes.insert(id, c);
    }
    for (id, c) in grid_desc {
        nodes.insert(id, c);
    }

    (root_id, nodes, serde_json::json!({}))
}
