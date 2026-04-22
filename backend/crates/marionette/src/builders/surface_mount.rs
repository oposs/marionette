//! `SurfaceMount` component builder.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

// -- Sub-surface mounting --

/// Mount a named sub-surface at this position in the tree.
///
/// `SurfaceMount` is a leaf node that renders `<Surface name={props.name}/>`
/// on the frontend, recursively mounting another surface's tree. Used by
/// `AppShell` (and future tabs, split-panes, etc.) to compose content surfaces
/// into a shell surface.
#[derive(ComponentBuilder)]
#[component(type = "surface-mount")]
pub struct SurfaceMount {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_mount_builder() {
        let (id, component) = SurfaceMount::new("content").build();
        assert!(!id.is_empty());
        assert!(id.starts_with("surface-mount-"));
        assert_eq!(component.r#type, "surface-mount");
        let props = component.props.as_ref().unwrap();
        assert_eq!(props["name"], "content");
        assert!(component.children.is_none());
        assert!(component.bind.is_none());
        assert!(component.action.is_none());
    }

    #[test]
    fn surface_mount_builder_custom_id() {
        let (id, _) = SurfaceMount::new("modal").id("shell-modal-mount").build();
        assert_eq!(id, "shell-modal-mount");
    }
}
