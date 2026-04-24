//! EXER-03 Pathological Scale — body filled in by Plan 19-04.
//!
//! Plan 19-01 ships only the `#[gallery_demo]` stub so the module compiles
//! and the linkme registry carries `exer-03` from the start.

use marionette::builders::{Container, Heading, Text};
use marionette::gallery::Node;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "exer-03", name = "Exerciser: Pathological Scale")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    let title = Heading::new("Pathological Scale (stub)")
        .id("exer-03-title")
        .level(1)
        .build();
    let intro = Text::new("Plan 19-01 stub — real content ships in Plan 19-04.")
        .id("exer-03-intro")
        .build();
    let (root, descendants) = Container::new()
        .id("exer-03-root")
        .class("flex flex-col gap-6 p-6")
        .children(vec![title, intro])
        .build_tree();
    let mut out = Vec::with_capacity(1 + descendants.len());
    out.push(root);
    out.extend(descendants);
    out
}
