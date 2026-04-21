use marionette::builders::standard::Text;
use marionette::gallery::Node;
use marionette_macros::gallery_demo;

#[gallery_demo]
fn private_demo() -> Node {
    Text::new("nope").build()
}

fn main() {}
