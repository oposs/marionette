//! EXER-01 Nested AppShell — body filled in by Plan 19-02.
//!
//! Plan 19-01 ships only the `#[gallery_demo]` stub so the module compiles
//! and the linkme registry carries `exer-01` from the start.

use marionette::builders::{Container, Heading, Text};
use marionette::gallery::Node;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "exer-01", name = "Exerciser: Nested AppShell")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    let title = Heading::new("Nested AppShell (stub)")
        .id("exer-01-title")
        .level(1)
        .build();
    let intro = Text::new("Plan 19-01 stub — real content ships in Plan 19-02.")
        .id("exer-01-intro")
        .build();
    let (root, descendants) = Container::new()
        .id("exer-01-root")
        .class("flex flex-col gap-6 p-6")
        .children(vec![title, intro])
        .build_tree();
    let mut out = Vec::with_capacity(1 + descendants.len());
    out.push(root);
    out.extend(descendants);
    out
}
