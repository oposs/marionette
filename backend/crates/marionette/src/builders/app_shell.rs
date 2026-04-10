//! Hand-written `AppShell` builder.
//!
//! `AppShell` is a first-class SDUI component (per D-B3) whose "props" are
//! node-ID references into the top-level adjacency list of the shell surface.
//! Its slot methods accept pre-built `(id, Component)` tuples from other
//! builders and record BOTH the slot ID into props AND the slot sub-tree into
//! a descendant collection for flattening by `build_with_children`.
//!
//! The derive macro in `marionette-macros` cannot express this shape (slot
//! methods have custom semantics, not plain prop setters), so `AppShell` is
//! the first hand-written builder in the toolkit. Its file is the template
//! for future structural components (tab-views, split-panes).

use marionette_protocol::Component;
use serde_json::{Map, Value};
use uuid::Uuid;

/// Entry point for building an `AppShell` component.
pub struct AppShell;

impl AppShell {
    /// Start a new `AppShell` builder. Chain `.sidebar()`, `.header()`, etc.
    /// then call `.build_with_children()` to get a flat `Vec<(id, Component)>`
    /// suitable for insertion into `RenderMessage.nodes`.
    ///
    /// This mirrors the `Type::new() -> TypeBuilder` convention emitted by the
    /// `#[derive(ComponentBuilder)]` macro for every other component, which is
    /// why we deliberately return the builder (not `Self`) here and silence
    /// the `clippy::new_ret_no_self` lint for this specific constructor.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new() -> AppShellBuilder {
        AppShellBuilder::default()
    }
}

/// Builder for an `AppShell` component. See [`AppShell::new`].
#[derive(Default)]
pub struct AppShellBuilder {
    sidebar_node: Option<(String, Component)>,
    header_node: Option<(String, Component)>,
    footer_node: Option<(String, Component)>,
    main_node: Option<(String, Component)>,
    popups_node: Option<(String, Component)>,
    toasts_node: Option<(String, Component)>,
    /// Descendants harvested from sub-builders' `.build_tree()` calls.
    descendants: Vec<(String, Component)>,
    id: Option<String>,
}

impl AppShellBuilder {
    /// Attach a sidebar slot child (usually a `SideNav` root).
    #[must_use]
    pub fn sidebar(mut self, slot: (String, Component)) -> Self {
        self.sidebar_node = Some(slot);
        self
    }

    /// Attach a header slot child (usually a `Container` with title + user menu).
    #[must_use]
    pub fn header(mut self, slot: (String, Component)) -> Self {
        self.header_node = Some(slot);
        self
    }

    /// Attach a footer slot child (usually a `Container` with version + status).
    #[must_use]
    pub fn footer(mut self, slot: (String, Component)) -> Self {
        self.footer_node = Some(slot);
        self
    }

    /// Attach the main content slot (usually a `SurfaceMount` for the `content` sub-surface).
    #[must_use]
    pub fn main(mut self, slot: (String, Component)) -> Self {
        self.main_node = Some(slot);
        self
    }

    /// Attach the popups slot (usually a `SurfaceMount` for the `modal` sub-surface).
    #[must_use]
    pub fn popups(mut self, slot: (String, Component)) -> Self {
        self.popups_node = Some(slot);
        self
    }

    /// Attach the toasts slot (usually a `SurfaceMount` for the `toasts` sub-surface).
    #[must_use]
    pub fn toasts(mut self, slot: (String, Component)) -> Self {
        self.toasts_node = Some(slot);
        self
    }

    /// Append descendants harvested from sub-builders' `build_tree()` calls.
    /// Order is preserved in the final flat list.
    #[must_use]
    pub fn with_descendants(mut self, desc: Vec<(String, Component)>) -> Self {
        self.descendants.extend(desc);
        self
    }

    /// Override the generated UUID with a stable id (e.g., `"app-shell-root"`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Build the `AppShell` node. Returns `(shell_id, component)` without
    /// surfacing slot child trees — call [`Self::build_with_children`] for that.
    ///
    /// Slot IDs are written into `component.props` under the keys
    /// `sidebarNodeId`, `headerNodeId`, `footerNodeId`, `mainNodeId`,
    /// `popupsNodeId`, `toastsNodeId`.
    #[must_use]
    pub fn build(self) -> (String, Component) {
        let shell_id = self
            .id
            .unwrap_or_else(|| format!("app-shell-{}", Uuid::new_v4()));

        let mut props = Map::new();
        if let Some((id, _)) = self.sidebar_node {
            props.insert("sidebarNodeId".into(), Value::String(id));
        }
        if let Some((id, _)) = self.header_node {
            props.insert("headerNodeId".into(), Value::String(id));
        }
        if let Some((id, _)) = self.footer_node {
            props.insert("footerNodeId".into(), Value::String(id));
        }
        if let Some((id, _)) = self.main_node {
            props.insert("mainNodeId".into(), Value::String(id));
        }
        if let Some((id, _)) = self.popups_node {
            props.insert("popupsNodeId".into(), Value::String(id));
        }
        if let Some((id, _)) = self.toasts_node {
            props.insert("toastsNodeId".into(), Value::String(id));
        }

        let component = Component {
            r#type: "app-shell".into(),
            props: Some(Value::Object(props)),
            children: None, // no positional children — slots are in props
            bind: None,
            action: None,
            visible: None,
        };
        (shell_id, component)
    }

    /// Build the `AppShell` and flatten all slot roots + their descendants into
    /// a single list suitable for insertion into `RenderMessage.nodes`.
    ///
    /// Order: `[shell, sidebar_root, header_root, footer_root, main_root,
    /// popups_root, toasts_root, ...descendants]`. Missing slots are skipped.
    #[must_use]
    pub fn build_with_children(mut self) -> Vec<(String, Component)> {
        // Take each slot root out of self in canonical order. Missing slots
        // are skipped. We consume the slots here so build() below will not
        // see them, which is fine because we re-populate the *NodeId props
        // directly from the drained tuples.
        let mut slot_roots: Vec<(String, Component)> = Vec::new();
        let mut props = Map::new();

        if let Some(slot) = self.sidebar_node.take() {
            props.insert("sidebarNodeId".into(), Value::String(slot.0.clone()));
            slot_roots.push(slot);
        }
        if let Some(slot) = self.header_node.take() {
            props.insert("headerNodeId".into(), Value::String(slot.0.clone()));
            slot_roots.push(slot);
        }
        if let Some(slot) = self.footer_node.take() {
            props.insert("footerNodeId".into(), Value::String(slot.0.clone()));
            slot_roots.push(slot);
        }
        if let Some(slot) = self.main_node.take() {
            props.insert("mainNodeId".into(), Value::String(slot.0.clone()));
            slot_roots.push(slot);
        }
        if let Some(slot) = self.popups_node.take() {
            props.insert("popupsNodeId".into(), Value::String(slot.0.clone()));
            slot_roots.push(slot);
        }
        if let Some(slot) = self.toasts_node.take() {
            props.insert("toastsNodeId".into(), Value::String(slot.0.clone()));
            slot_roots.push(slot);
        }

        let descendants = std::mem::take(&mut self.descendants);

        let shell_id = self
            .id
            .unwrap_or_else(|| format!("app-shell-{}", Uuid::new_v4()));

        let shell = Component {
            r#type: "app-shell".into(),
            props: Some(Value::Object(props)),
            children: None,
            bind: None,
            action: None,
            visible: None,
        };

        let mut out = Vec::with_capacity(1 + slot_roots.len() + descendants.len());
        out.push((shell_id, shell));
        out.extend(slot_roots);
        out.extend(descendants);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builders::standard::{Container, Heading, SideNav, SurfaceMount};

    fn simple_container(id: &str) -> (String, Component) {
        Container::new().id(id).build()
    }

    #[test]
    fn app_shell_build_with_all_slots_populates_props() {
        let sidebar = simple_container("shell-sidebar");
        let header = simple_container("shell-header");
        let footer = simple_container("shell-footer");
        let content_mount = SurfaceMount::new("content").id("shell-content-mount").build();
        let modal_mount = SurfaceMount::new("modal").id("shell-modal-mount").build();
        let toast_mount = SurfaceMount::new("toasts").id("shell-toasts-mount").build();

        let (id, component) = AppShell::new()
            .id("app-shell-root")
            .sidebar(sidebar)
            .header(header)
            .footer(footer)
            .main(content_mount)
            .popups(modal_mount)
            .toasts(toast_mount)
            .build();

        assert_eq!(id, "app-shell-root");
        assert_eq!(component.r#type, "app-shell");
        assert!(component.children.is_none());
        let props = component.props.unwrap();
        assert_eq!(props["sidebarNodeId"], "shell-sidebar");
        assert_eq!(props["headerNodeId"], "shell-header");
        assert_eq!(props["footerNodeId"], "shell-footer");
        assert_eq!(props["mainNodeId"], "shell-content-mount");
        assert_eq!(props["popupsNodeId"], "shell-modal-mount");
        assert_eq!(props["toastsNodeId"], "shell-toasts-mount");
    }

    #[test]
    fn app_shell_build_without_slots_yields_empty_props() {
        let (id, component) = AppShell::new().id("x").build();
        assert_eq!(id, "x");
        let props = component.props.unwrap();
        assert!(props.as_object().unwrap().is_empty());
    }

    #[test]
    fn app_shell_build_with_children_flattens_all_nodes() {
        let sidebar = simple_container("side-1");
        let header = simple_container("head-1");
        let content_mount = SurfaceMount::new("content").id("content-mount-1").build();

        // Simulate a descendant harvested from a sub-builder's build_tree
        let descendants = vec![(
            "descendant-1".to_string(),
            Heading::new("Nested").id("descendant-1").build().1,
        )];

        let flat = AppShell::new()
            .id("shell-1")
            .sidebar(sidebar)
            .header(header)
            .main(content_mount)
            .with_descendants(descendants)
            .build_with_children();

        // Expected order: shell, sidebar, header, main, descendant
        assert_eq!(flat.len(), 5);
        assert_eq!(flat[0].0, "shell-1");
        assert_eq!(flat[0].1.r#type, "app-shell");
        // Shell props should reference the three populated slots by id.
        let shell_props = flat[0].1.props.as_ref().unwrap();
        assert_eq!(shell_props["sidebarNodeId"], "side-1");
        assert_eq!(shell_props["headerNodeId"], "head-1");
        assert_eq!(shell_props["mainNodeId"], "content-mount-1");
        assert!(shell_props.get("footerNodeId").is_none());
        assert!(shell_props.get("popupsNodeId").is_none());
        assert!(shell_props.get("toastsNodeId").is_none());
        assert_eq!(flat[1].0, "side-1");
        assert_eq!(flat[2].0, "head-1");
        assert_eq!(flat[3].0, "content-mount-1");
        assert_eq!(flat[3].1.r#type, "surface-mount");
        assert_eq!(flat[4].0, "descendant-1");
    }

    #[test]
    fn app_shell_generates_uuid_id_when_not_set() {
        let (id, _) = AppShell::new().build();
        assert!(id.starts_with("app-shell-"));
        assert!(id.len() > "app-shell-".len());
    }

    #[test]
    fn app_shell_with_sidenav_build_tree_pattern() {
        // Canonical usage pattern: use SideNav::build_tree to get (root, descendants)
        // and pass them separately to AppShell.
        let nav_item_heading = Heading::new("Home").id("nav-home").build();
        let (sidebar_root, sidebar_desc) = SideNav::new()
            .id("shell-side-nav")
            .children(vec![nav_item_heading])
            .build_tree();

        let flat = AppShell::new()
            .id("shell-2")
            .sidebar(sidebar_root)
            .with_descendants(sidebar_desc)
            .build_with_children();

        // Expected: shell, sidebar_root, descendant (heading)
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].0, "shell-2");
        assert_eq!(flat[1].0, "shell-side-nav");
        assert_eq!(flat[2].0, "nav-home");
    }
}
