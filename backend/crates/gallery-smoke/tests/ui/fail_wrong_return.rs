use marionette_macros::gallery_demo;

#[gallery_demo]
pub fn wrong_return() -> Vec<u32> {
    vec![]
}

fn main() {}
