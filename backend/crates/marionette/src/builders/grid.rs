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
