use marionette::builders::standard::Text;
use marionette::gallery::Node;
use marionette_macros::gallery_demo;

#[gallery_demo]
pub fn has_args(_arg: u32) -> Node {
    Text::new("nope").build()
}

fn main() {}
