//! Composite builder helpers (`form_shell`).
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use super::container::Container;

// -- Composite helpers --

/// Assemble the canonical form-screen envelope used by every Phase 15 CRM
/// edit form.
///
/// Composes `Container([heading, back_button, form_child])` as the outer
/// wrapper and merges `form_descendants` (`FieldSet`s, `FieldSeparator`s,
/// action row, etc.) into a flat `HashMap<String, Component>` suitable for
/// `RenderMessage.nodes`. Returns `(root_id, nodes_map)`.
///
/// # Arguments
///
/// * `root_id` — stable id for the outer `Container` (e.g. `"company-form-root"`).
/// * `heading` — `(id, Component)` tuple returned by `Heading::new(...).id(...).build()`.
/// * `back_button` — `(id, Component)` tuple returned by `Button::new("← Back").id(...).variant("outline")...build()`.
/// * `form_child` — `(id, Component)` tuple returned by `Form::new().id(...).children(...).build_tree().0` (the top-level `Form` adjacency node).
/// * `form_descendants` — flat vec of `(id, Component)` pairs collected from the `FieldSet`/`FieldSeparator`/`Container` children's `build_tree()` calls and `Form`'s own descendant vec.
///
/// # Returns
///
/// `(root_id, nodes_map)` where `nodes_map` contains the outer `Container`
/// plus the `heading`, `back_button`, `form_child`, and every form descendant
/// — ready to pass as `RenderMessage.nodes`.
///
/// # Example
///
/// ```ignore
/// let heading = Heading::new("Edit Company").id("company-form-heading").build();
/// let back_button = Button::new("← Back")
///     .id("company-form-back")
///     .variant("outline")
///     .action(ComponentAction::click("company_list"))
///     .build();
/// let (form_child, form_descendants) = Form::new()
///     .id("company-form")
///     .children(vec![details_set, separator, address_set, action_row])
///     .build_tree();
///
/// let (root, nodes) = form_shell(
///     "company-form-root",
///     heading,
///     back_button,
///     form_child,
///     form_descendants,
/// );
/// ```
#[must_use]
pub fn form_shell(
    root_id: impl Into<String>,
    heading: (String, marionette_protocol::Component),
    back_button: (String, marionette_protocol::Component),
    form_child: (String, marionette_protocol::Component),
    form_descendants: Vec<(String, marionette_protocol::Component)>,
) -> (
    String,
    std::collections::HashMap<String, marionette_protocol::Component>,
) {
    let root_id = root_id.into();

    // Outer Container wraps heading + back_button + form_child by id reference.
    // `Container::children(Vec<(id, Component)>)` collects tuples; the macro's
    // generated `build_with_children` then emits [container_tuple, ...children]
    // with the container's `.children` field populated with the child ids in
    // declaration order.
    let container_nodes = Container::new()
        .id(&root_id)
        .children(vec![
            heading.clone(),
            back_button.clone(),
            form_child.clone(),
        ])
        .build_with_children();

    let mut nodes: std::collections::HashMap<String, marionette_protocol::Component> =
        std::collections::HashMap::new();
    for (id, c) in container_nodes {
        nodes.insert(id, c);
    }
    // Re-insertion of the three tuples is idempotent — `build_with_children`
    // already includes them — but the explicit inserts guard against any
    // future change to the macro's emission order and keep the intent
    // readable.
    nodes.insert(heading.0, heading.1);
    nodes.insert(back_button.0, back_button.1);
    nodes.insert(form_child.0, form_child.1);
    for (id, c) in form_descendants {
        nodes.insert(id, c);
    }

    (root_id, nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::button::Button;
    use super::super::form::Form;
    use super::super::heading::Heading;

    #[test]
    fn form_shell_assembles_container_with_heading_back_form() {
        let heading = Heading::new("Edit Company")
            .id("company-form-heading")
            .build();
        let back_button = Button::new("← Back")
            .id("company-form-back")
            .variant("outline")
            .action(marionette_protocol::ComponentAction::click("company_list"))
            .build();
        // A minimal Form with no children — build_tree returns the root tuple
        // plus an empty descendants vec.
        let (form_child, form_descendants) = Form::new().id("company-form").build_tree();

        let (root_id, nodes) = form_shell(
            "company-form-root",
            heading,
            back_button,
            form_child,
            form_descendants,
        );

        assert_eq!(root_id, "company-form-root");

        // Nodes must include the root container, heading, back button, and form child.
        assert!(nodes.contains_key("company-form-root"));
        assert!(nodes.contains_key("company-form-heading"));
        assert!(nodes.contains_key("company-form-back"));
        assert!(nodes.contains_key("company-form"));

        let container = nodes.get("company-form-root").expect("root container");
        assert_eq!(container.r#type, "container");
        let children = container
            .children
            .as_ref()
            .expect("container has children");
        assert_eq!(
            children,
            &vec![
                "company-form-heading".to_string(),
                "company-form-back".to_string(),
                "company-form".to_string()
            ]
        );
    }
}
