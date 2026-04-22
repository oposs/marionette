//! `Form` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Form components --

#[derive(ComponentBuilder)]
#[component(type = "form")]
pub struct Form {
    #[builder(optional)]
    pub submit_label: Option<String>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01 composite) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "form")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    use marionette_protocol::ComponentAction;

    // D-A1 composite nesting: reuse text_input's + select's demo tree.
    let text_input_nodes = crate::builders::text_input::gallery_demo();
    let select_nodes = crate::builders::select::gallery_demo();

    let submit = crate::builders::button::Button::new("Submit")
        .id("demo-form-submit")
        .action(ComponentAction::submit("gallery-demo/noop"))
        .build();

    // Index 0 of each nested demo is the Container-root tuple that we feed
    // into Form.children; the remaining entries are the root's descendants.
    let (form_root, form_desc) = Form::new()
        .id("demo-form-root")
        .children(vec![
            text_input_nodes[0].clone(),
            select_nodes[0].clone(),
            submit,
        ])
        .build_tree();

    let mut all = vec![form_root];
    all.extend(text_input_nodes.into_iter().skip(1));
    all.extend(select_nodes.into_iter().skip(1));
    all.extend(form_desc);
    all
}
