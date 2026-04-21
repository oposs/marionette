---
phase: 12
plan: 05
type: execute
wave: 2
depends_on: [12-02]
files_modified:
  - backend/crates/marionette/src/builders/standard.rs
  - backend/crates/marionette/src/builders/app_shell.rs
  - backend/crates/marionette/src/builders/mod.rs
autonomous: true
requirements: [SHELL-04]
nyquist_compliant: true
tags: [backend, rust, builders, app-shell]
must_haves:
  truths:
    - "SurfaceMount is a standard derived builder with a single required `name: String` prop"
    - "AppShell is a hand-written builder exposing .sidebar(), .header(), .footer(), .main(), .popups(), .toasts(), .build_with_children()"
    - "AppShell.build() stores slot node IDs in props under sidebarNodeId, headerNodeId, footerNodeId, mainNodeId, popupsNodeId, toastsNodeId"
    - "AppShell.build_with_children() returns a flat Vec<(String, Component)> containing the shell + all slot roots + all descendants"
    - "All new builders have inline unit tests asserting props/wiring shapes"
  artifacts:
    - path: "backend/crates/marionette/src/builders/standard.rs"
      provides: "SurfaceMount derived builder"
      contains: "struct SurfaceMount"
    - path: "backend/crates/marionette/src/builders/app_shell.rs"
      provides: "hand-written AppShell builder"
      contains: "pub struct AppShellBuilder"
  key_links:
    - from: "app_shell.rs AppShellBuilder::build()"
      to: "Component props"
      via: "sidebarNodeId / headerNodeId / footerNodeId / mainNodeId / popupsNodeId / toastsNodeId props"
      pattern: "sidebarNodeId"
---

<objective>
Implement the backend toolkit half of Part B: a `SurfaceMount` derived builder (trivial — single `name` prop) and a hand-written `AppShell` builder with six slot methods and a flat-list `build_with_children` that feeds `RenderMessage.nodes`. Matches D-B2, D-B3, D-B4 verbatim.

Purpose: CRM handler migration (Plan 07) depends on this builder. By separating builder construction from handler migration, Plan 05 can run parallel with the frontend store work (Plan 04) — no file overlap.

Output: Two compiled builders with inline tests. `cargo test -p marionette` green. `cargo clippy -p marionette -- -D warnings` green.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md
@.planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md
@backend/crates/marionette/src/builders/standard.rs
@backend/crates/marionette/src/builders/app_shell.rs
@backend/crates/marionette/src/builders/mod.rs
@backend/crates/marionette/src/builders/node.rs
@backend/crates/marionette-macros/src/component_builder.rs
@backend/crates/marionette-protocol/src/component.rs

<interfaces>
Existing `#[derive(ComponentBuilder)]` shape (from standard.rs:97-99):

```rust
#[derive(ComponentBuilder)]
#[component(type = "side-nav")]
pub struct SideNav {}
```

The macro generates:
- `impl SideNav { pub fn new() -> SideNavBuilder }`
- `SideNavBuilder::build() -> (String, Component)`
- `SideNavBuilder::build_with_children() -> Vec<(String, Component)>`
- `SideNavBuilder::build_tree() -> ((String, Component), Vec<(String, Component)>)`
- `SideNavBuilder::id(impl Into<String>) -> Self`
- `SideNavBuilder::children(Vec<(String, Component)>) -> Self`
- Prop setters for each declared field

Required fields (non-Option) become mandatory arguments to `::new()`.

For `SurfaceMount`, the only prop is a required `name: String`, so `SurfaceMount::new(name)` matches the pattern of `TextInput::new(label)` / `Heading::new(text)`.

`Component` struct (from component.rs, verified earlier):

```rust
pub struct Component {
    pub r#type: String,
    pub props: Option<serde_json::Value>,
    pub children: Option<Vec<String>>,
    pub bind: Option<String>,
    pub action: Option<ComponentAction>,
    pub visible: Option<String>,
}
```

The scaffold `app_shell.rs` from Plan 01 has a placeholder `pub struct AppShell;`. This plan replaces it with the real builder.

`uuid::Uuid` crate is already in dependencies — used by the derive macro for id generation; confirm via `grep uuid backend/crates/marionette/Cargo.toml`.

Backend scaffold from Plan 01 already added `pub mod app_shell; pub use app_shell::*;` to `builders/mod.rs`, so no new module wiring is needed by this plan — just the impl inside `app_shell.rs`.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add SurfaceMount derived builder to standard.rs with inline test</name>
  <read_first>
    - backend/crates/marionette/src/builders/standard.rs (entire file, note the existing 18 builders and test patterns at the bottom)
    - backend/crates/marionette-macros/src/component_builder.rs (derive expansion)
    - .planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md D-B2
  </read_first>
  <behavior>
    - `SurfaceMount::new("content").build()` returns `(id, Component)` where `component.r#type == "surface-mount"` and `component.props["name"] == "content"`
    - `SurfaceMount::new("modal").id("shell-modal-mount").build()` returns an id of exactly `"shell-modal-mount"`
    - `SurfaceMount` has no children, no bind, no action by default
  </behavior>
  <action>
1. In `backend/crates/marionette/src/builders/standard.rs`, after the `NavGroup` struct definition (around line 114) and before `// -- Form components --`, add a new section:

```rust
// -- Sub-surface mounting --

/// Mount a named sub-surface at this position in the tree.
///
/// `SurfaceMount` is a leaf node that renders `<Surface name={props.name}/>`
/// on the frontend, recursively mounting another surface's tree. Used by
/// AppShell (and future tabs, split-panes, etc.) to compose content surfaces
/// into a shell surface.
#[derive(ComponentBuilder)]
#[component(type = "surface-mount")]
pub struct SurfaceMount {
    pub name: String,
}
```

2. In the same file's `#[cfg(test)] mod tests` block at the bottom (near line 183), add a test for `SurfaceMount`:

```rust
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
```

3. Find the `all_18_standard_types` test (around line 271 of standard.rs). Since we are adding `surface-mount`, extend it to `all_19_standard_types` (rename the function) and add `SurfaceMount::new("x").build().1.r#type` to the `types` Vec and `"surface-mount"` to the `expected` Vec at the matching index. Order does not matter as long as types and expected stay aligned.

4. Run `cd backend && cargo test -p marionette builders::standard::tests::surface_mount_builder builders::standard::tests::surface_mount_builder_custom_id builders::standard::tests::all_19_standard_types` — all three tests must pass.
  </action>
  <verify>
    <automated>cd backend &amp;&amp; cargo test -p marionette builders::standard::tests::surface_mount 2&gt;&amp;1 | tail -15</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'struct SurfaceMount' backend/crates/marionette/src/builders/standard.rs` succeeds
    - `grep -q 'component(type = "surface-mount")' backend/crates/marionette/src/builders/standard.rs` succeeds
    - `grep -q 'pub name: String' backend/crates/marionette/src/builders/standard.rs` (within the SurfaceMount struct — context-sensitive, acceptable to grep for the struct block line count)
    - `cd backend && cargo test -p marionette builders::standard::tests::surface_mount_builder` exits 0
    - `cd backend && cargo test -p marionette builders::standard::tests::surface_mount_builder_custom_id` exits 0
    - `cd backend && cargo test -p marionette builders::standard::tests::all_19_standard_types` exits 0 (the renamed test)
    - `cd backend && cargo clippy -p marionette -- -D warnings` exits 0
  </acceptance_criteria>
  <done>SurfaceMount derived builder added to standard.rs with three passing tests. Clippy pedantic is green.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Write hand-written AppShell builder in app_shell.rs with slot methods and build_with_children</name>
  <read_first>
    - backend/crates/marionette/src/builders/app_shell.rs (scaffold from Plan 01)
    - backend/crates/marionette/src/builders/standard.rs (idiom reference — Container, SideNav)
    - backend/crates/marionette-protocol/src/component.rs
    - backend/crates/marionette/Cargo.toml (confirm uuid is a dependency)
    - .planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md D-B3, D-B4
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Pattern 4 (hand-written builder reference implementation)
  </read_first>
  <behavior>
    - `AppShell::new().build()` returns `(id, Component)` with `component.r#type == "app-shell"`, `component.children == None` (no positional children — slot IDs live in props)
    - `AppShell::new().id("app-shell-root").build()` returns id `"app-shell-root"`
    - `.sidebar((id, component))` records the slot — `build()` writes `sidebarNodeId = id` into props
    - Analogous for `.header()`, `.footer()`, `.main()`, `.popups()`, `.toasts()` writing `headerNodeId`, `footerNodeId`, `mainNodeId`, `popupsNodeId`, `toastsNodeId`
    - `.with_descendants(Vec<(String, Component)>)` appends additional nodes (transitively harvested from sub-builders' `build_tree` outputs)
    - `build_with_children()` returns a flat Vec containing the shell component FIRST, then each provided slot root, then all descendants
    - Inline tests assert: (a) props shape, (b) flat list ordering + content
  </behavior>
  <action>
REPLACE the entire contents of `backend/crates/marionette/src/builders/app_shell.rs` with:

```rust
//! Hand-written AppShell builder.
//!
//! AppShell is a first-class SDUI component (per D-B3) whose "props" are
//! node-ID references into the top-level adjacency list of the shell surface.
//! Its slot methods accept pre-built `(id, Component)` tuples from other
//! builders and record BOTH the slot ID into props AND the slot sub-tree into
//! the children-collection for flattening by `build_with_children`.
//!
//! The derive macro in `marionette-macros` cannot express this shape (slot
//! methods have custom semantics, not plain prop setters), so AppShell is
//! the first hand-written builder in the toolkit. Its file is the template
//! for future structural components (tab-views, split-panes).

use marionette_protocol::Component;
use serde_json::{Map, Value};
use uuid::Uuid;

/// Entry point for building an `AppShell` component.
pub struct AppShell;

impl AppShell {
    /// Start a new AppShell builder. Chain `.sidebar()`, `.header()`, etc.
    /// then call `.build_with_children()` to get a flat `Vec<(id, Component)>`
    /// suitable for insertion into `RenderMessage.nodes`.
    #[must_use]
    pub fn new() -> AppShellBuilder {
        AppShellBuilder::default()
    }
}

/// Builder for an `AppShell` component. See `AppShell::new`.
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

    /// Build the AppShell node. Returns `(shell_id, component)` without
    /// surfacing slot child trees — call `build_with_children` for that.
    ///
    /// Slot IDs are written into `component.props` under the keys
    /// `sidebarNodeId`, `headerNodeId`, `footerNodeId`, `mainNodeId`,
    /// `popupsNodeId`, `toastsNodeId`.
    #[must_use]
    pub fn build(self) -> (String, Component) {
        let shell_id = self
            .id
            .clone()
            .unwrap_or_else(|| format!("app-shell-{}", Uuid::new_v4()));

        let mut props = Map::new();
        if let Some((ref id, _)) = self.sidebar_node {
            props.insert("sidebarNodeId".into(), Value::String(id.clone()));
        }
        if let Some((ref id, _)) = self.header_node {
            props.insert("headerNodeId".into(), Value::String(id.clone()));
        }
        if let Some((ref id, _)) = self.footer_node {
            props.insert("footerNodeId".into(), Value::String(id.clone()));
        }
        if let Some((ref id, _)) = self.main_node {
            props.insert("mainNodeId".into(), Value::String(id.clone()));
        }
        if let Some((ref id, _)) = self.popups_node {
            props.insert("popupsNodeId".into(), Value::String(id.clone()));
        }
        if let Some((ref id, _)) = self.toasts_node {
            props.insert("toastsNodeId".into(), Value::String(id.clone()));
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

    /// Build the AppShell and flatten all slot roots + their descendants into
    /// a single list suitable for insertion into `RenderMessage.nodes`.
    ///
    /// Order: `[shell, sidebar_root, header_root, footer_root, main_root,
    /// popups_root, toasts_root, ...descendants]`. Missing slots are skipped.
    #[must_use]
    pub fn build_with_children(self) -> Vec<(String, Component)> {
        // Capture slot root tuples before consuming self in build()
        let mut slot_roots: Vec<(String, Component)> = Vec::new();
        if let Some(s) = self.sidebar_node.clone() {
            slot_roots.push(s);
        }
        if let Some(s) = self.header_node.clone() {
            slot_roots.push(s);
        }
        if let Some(s) = self.footer_node.clone() {
            slot_roots.push(s);
        }
        if let Some(s) = self.main_node.clone() {
            slot_roots.push(s);
        }
        if let Some(s) = self.popups_node.clone() {
            slot_roots.push(s);
        }
        if let Some(s) = self.toasts_node.clone() {
            slot_roots.push(s);
        }
        let descendants = self.descendants.clone();

        let (shell_id, shell) = self.build();

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
```

Then run `cd backend && cargo test -p marionette builders::app_shell::tests` — all 5 tests must pass.

Run `cd backend && cargo clippy -p marionette -- -D warnings` — must be green. If `clippy::pedantic` complains about `#[must_use]` on `build()` (it's already there), `missing_const_for_fn`, or similar, address each warning rather than adding `#[allow]`.

Run `cd backend && cargo test --workspace` — the entire workspace must stay green.
  </action>
  <verify>
    <automated>cd backend &amp;&amp; cargo test -p marionette builders::app_shell::tests 2&gt;&amp;1 | tail -15</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'pub struct AppShellBuilder' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `grep -q 'pub fn sidebar' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `grep -q 'pub fn header' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `grep -q 'pub fn footer' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `grep -q 'pub fn main' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `grep -q 'pub fn popups' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `grep -q 'pub fn toasts' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `grep -q 'pub fn build_with_children' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `grep -q 'sidebarNodeId' backend/crates/marionette/src/builders/app_shell.rs` succeeds
    - `cd backend && cargo test -p marionette builders::app_shell::tests` exits 0 with 5 tests passing
    - `cd backend && cargo test --workspace` exits 0
    - `cd backend && cargo clippy -p marionette -- -D warnings` exits 0
  </acceptance_criteria>
  <done>AppShell hand-written builder is complete with 5 passing unit tests. Workspace tests and clippy pedantic are green.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries
Backend builder code only — no user-input handling, no network I/O, no auth decisions. Builders construct server-side SDUI trees from trusted handler code.

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-12-11 | Tampering | A handler could accidentally pass an invalid slot tuple (e.g., mismatched id) to `AppShellBuilder.sidebar()` | accept | Rust type system enforces `(String, Component)` shape; misusing slot methods causes visual errors but no security impact. |
| T-12-12 | Information Disclosure | AppShell props could accidentally expose internal node IDs the frontend shouldn't know | accept | Node IDs are already visible in `RenderMessage.nodes` keys — no new exposure surface. |
</threat_model>

<verification>
- `cd backend && cargo test -p marionette builders::app_shell::tests` exits 0 with 5 passing tests
- `cd backend && cargo test -p marionette builders::standard::tests::surface_mount_builder` exits 0
- `cd backend && cargo test --workspace` exits 0
- `cd backend && cargo clippy --workspace -- -D warnings` exits 0
- `grep -q 'surface-mount' backend/crates/marionette/src/builders/standard.rs` succeeds
- `grep -q 'app-shell' backend/crates/marionette/src/builders/app_shell.rs` succeeds
</verification>

<success_criteria>
- `SurfaceMount` is registered via `#[derive(ComponentBuilder)]` in `standard.rs` with `type = "surface-mount"` and required `name: String`
- `AppShell` is a hand-written builder in `app_shell.rs` with six slot methods (sidebar, header, footer, main, popups, toasts), `.with_descendants`, `.id`, `.build`, `.build_with_children`
- Slot IDs land in `Component.props` under `sidebarNodeId` / `headerNodeId` / `footerNodeId` / `mainNodeId` / `popupsNodeId` / `toastsNodeId`
- `.build_with_children()` returns a flat `Vec<(String, Component)>` with shell first, then slot roots, then descendants
- At least 7 inline unit tests pass (2 SurfaceMount + 5 AppShell)
- Workspace tests and clippy pedantic are green
</success_criteria>

<output>
After completion, create `.planning/phases/12-protocol-node-patching-appshell/12-05-SUMMARY.md` recording:
- Number of inline tests added (split between standard.rs and app_shell.rs)
- Any clippy warnings resolved (and how) during implementation
- Confirmation that `all_19_standard_types` test was renamed from `all_18_standard_types`
</output>
