# Phase 12: Protocol Node Patching + AppShell - Research

**Researched:** 2026-04-10
**Domain:** OpenSDUI protocol extension + Svelte 5 fine-grained reactivity + shadcn-svelte Sidebar composable
**Confidence:** HIGH

## Summary

Phase 12 is primarily an *integration* research task — almost every technical decision is already locked in CONTEXT.md (D-A1..D-B15). The planner needs concrete values for **nine specific technical questions** to write actionable tasks. This document answers each, with verified citations from the repo, Svelte official docs, shadcn-svelte docs, and serde/OpenAPI references.

**Primary recommendation:** The focus-preservation requirement (D-A6) hinges on a single, small code-level change in `frontend/src/lib/store/surfaces.svelte.ts`: stop replacing `surfaceState[surface] = { root, nodes }` wholesale. Mutate `surfaceState[surface].nodes[id]` in place. Everything else (NodeRenderer keyed each blocks, per-node `$derived(nodes[nodeId])` reads) is already in place and will do the right thing once the store stops invalidating the whole proxy.

The AppShell half of the phase is straightforward once two traps are addressed: (1) the current `--sidebar-*` CSS tokens in `app.css` use the wrong names for the shadcn-svelte Sidebar component and will need renaming, and (2) every CRM handler currently renders to surface `"main"` and must change to `"content"` as part of the nav-layer migration.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Protocol — PatchOperation shape**
- **D-A1:** Tagged enum with `op` discriminator. `PatchOperation` becomes a Rust enum / JSON Schema `oneOf` tagged union serialized via `#[serde(tag = "op")]`. Single `patch` array holds both data ops and node-tree ops in declared order. All-or-nothing atomic per-surface. Breaking change from current `{path, value}` — pre-deployment, no backcompat.
- **D-A2:** Five node operations plus one data op: `set {path, value}`, `set-node {id, component}`, `delete-node {id}`, `set-children {id, children}`, `insert-child {parent, index, childId}`, `remove-child {parent, childId}`.

**Protocol — Surface Targeting**
- **D-A3:** Add `surface: Surface` required field to `PatchMessage`. One message targets exactly one surface. Also fixes existing latent bug in `frontend/src/lib/init.ts:47` that hardcodes every patch to `main`.

**Protocol — Root Immutability**
- **D-A4:** `root` is immutable per `Render`. No `set-root` op. Top-level transitions use full `Render`.

**Protocol — Version and Focus Preservation**
- **D-A5:** Protocol version bumps to `"1.1.0"` via `HelloMessage.version`.
- **D-A6:** **Focus preservation is mandatory and proven by test.** Surface store must mutate node entries in place; Svelte 5 fine-grained reactivity scopes re-renders. A browser test exercises "user typing in field A → backend sends a patch modifying field B → field A retains focus and cursor."
- **D-A7:** No backward compatibility. Clean bump to 1.1.0.

**Protocol — Orphan Handling**
- **D-A8:** Client-side walk-and-prune GC scoped to the target surface of each patch batch. O(N) in the target surface's size. Exists as a nicety for long-lived shell surfaces; sub-surface architecture handles the common case via `Render` replacement.

**AppShell — Architecture**
- **D-B1:** Sub-surface architecture via new `surface-mount` SDUI component type. `main` slot mounts `content` sub-surface; `popups` mounts `modal`; `toasts` mounts `toasts`. Screen nav = full `Render` into `content` → automatic cleanup.
- **D-B2:** `surface-mount` is a general common building block, not an AppShell special. Registered in `registry/defaults.ts`, backend builder via `#[derive(ComponentBuilder)]` in `standard.rs`. Props: `{ name: String }`. Svelte impl: `<Surface name={props.name}/>`.
- **D-B3:** AppShell has no special protocol powers. Normal SDUI component. Hand-written backend builder (slot methods have custom semantics). Slot children referenced by ID in props (`sidebarNodeId`, `headerNodeId`, `footerNodeId`, `mainNodeId`, `popupsNodeId`, `toastsNodeId`). Top-level adjacency list.

**AppShell — Backend Builder Shape**
- **D-B4:** Slot methods accept pre-built node references. `AppShell::new().sidebar(sidebar_id).header(header_id).footer(footer_id).main(content_mount_id).popups(modal_mount_id).toasts(toast_mount_id).build_with_children()`.

**AppShell — Slot Content**
- **D-B5:** Header: sidebar trigger (mobile), app title/branding, user menu.
- **D-B6:** Footer: version info ("Marionette v1.1 · Protocol 1.1.0"), connection status indicator (replaces retired `ConnectionBanner`), legal/copyright.
- **D-B7:** Sidebar: existing `SideNav` + `NavGroup` + `NavItem` from Phase 11 — no new nav components.
- **D-B8:** Sub-surface slots use `surface-mount` nodes: `name: "content"` (main), `name: "modal"` (popups), `name: "toasts"` (toasts).

**AppShell — Frontend Structural Changes**
- **D-B9:** `routes/+layout.svelte` collapses to `<Surface name="main"/>`. `ConnectionBanner` retired. `sidebar`/`modal`/`toast` top-level mounts removed.
- **D-B10:** `AppShell.svelte` uses shadcn Sidebar primitives. Install `sidebar`, `dialog` (already installed), shadcn Toast/Sonner.
- **D-B11:** Pre-auth/login flow is a full `Render` transition (root changes), respects D-A4.

**AppShell — CRM Integration (Phase 12 scope)**
- **D-B12:** Minimal + nav. Only (1) `routes/+layout.svelte` update, (2) nav/routing layer migration (initial shell render + `navigate` handler), (3) one end-to-end node-mutation demo. Per-screen CRUD handlers NOT touched in Phase 12 — they're Phase 15.

**Nav State & Events**
- **D-B13:** Nav active state via data binding on `/nav/active/*` paths. Existing Phase 11 pattern.
- **D-B14:** Modal close via existing `event { name: "close", surface: "modal" }` mechanism.
- **D-B15:** Toast lifecycle via `insert-child` / `delete-node` patches on `toasts` sub-surface.

### Claude's Discretion

Research below provides recommendations for each:
- Exact Rust type layout for tagged `PatchOperation` enum (serde attributes, struct vs. Box variants) → **see Finding 3**
- JSON Schema shape for `oneOf` tagged union in `data.yaml` → **see Finding 3**
- Svelte 5 reactivity wiring inside surface store → **see Finding 1**
- Walk-and-prune GC implementation detail → **see Finding 5**
- Exact AppShell Svelte slot composition, shadcn Sidebar sub-component mapping → **see Finding 2**
- `surface-mount` mounting semantics (recursion safety) → **see Finding 4**
- CRM contact form field-swap demo specifics → **see Finding 9**
- Shadcn toast primitive choice → Phase 11 D-04 already picked shadcn Toast (not Sonner); honor
- Ordering of plans within phase → recommend Part A end-to-end before Part B

### Deferred Ideas (OUT OF SCOPE)

**To future phases in v1.1:**
- Per-screen CRM handler cleanup — Phase 15
- DataTable node-patch features — Phase 13
- FormScreen node-patch features + orphan `FormScreen.svelte`/`TableScreen.svelte` rewrite — Phase 14
- Flowbite residue audit — Phase 15

**To v2:**
- `SHELL-05`: Persistent sidebar collapse state (cookie/localStorage)
- `SHELL-06`: Auto-generated breadcrumbs
- `SHELL-07`: Multiple sidebar variants (floating, inset)
- Stacked modals
- Cross-surface atomic transactions

**Noted but out of Phase 12:**
- Orphan cleanup for `FormScreen.svelte` / `TableScreen.svelte` — Phases 13/14
- Additional `surface-mount`-based components (tabs, split-pane) — future
- Walk-and-prune GC optimization (visited-set pre-allocation, microtask deferral) — planning detail
- Protocol version negotiation UX — already specified
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **PATCH-01** | `PatchMessage` carries data + node-tree ops in atomic batch. Frontend/backend Rust types + `spec/schemas/data.yaml` + `spec/schemas/message.yaml` + `spec/openapi.yaml` reflect new shape. | Finding 3 (serde enum), Finding 8 (schema update mechanics), Finding 1 (store dispatcher) |
| **PATCH-02** | Frontend surface store applies node patches reactively without remounting unrelated nodes; focused text input retains focus+cursor across sibling patches. Automated focus-preservation test. | Finding 1 (Svelte 5 reactivity), Finding 6 (browser test pattern) |
| **PATCH-03** | `spec/PROTOCOL.md` documents node-patch semantics; `CONCEPT.md` reconciled; `HelloMessage.version` = `"1.1.0"`. | Finding 8 (doc update sites) |
| **SHELL-01** | Collapsible sidebar on desktop + sheet overlay on mobile via shadcn Sidebar composable. | Finding 2 (Sidebar API) |
| **SHELL-02** | Header + footer areas for title/user menu and status/version info. | Finding 2 (slot mapping) |
| **SHELL-03** | CSS variable theming via `--sidebar-*` tokens. ⚠ **TRAP**: current tokens in `app.css` use wrong names — see Finding 2. | Finding 2 (token rename) |
| **SHELL-04** | AppShell is normal SDUI component: registered in `defaults.ts`, hand-written backend builder in `backend/crates/marionette/src/builders/`, slot children addressed by name in props referencing top-level adjacency-list nodes. | Finding 7 (builder shape) |
</phase_requirements>

## Project Constraints (from CLAUDE.md and codebase conventions)

- Tabs for indentation, single quotes for strings, 100-char print width (Prettier + prettier-plugin-svelte).
- Rust: `#![warn(clippy::pedantic)]`, `edition = "2024"`, rustfmt enforced.
- All SDUI components accept `props`, `bind?`, `action?`, `surface`, `children?` (`Snippet`).
- Component types are strings, registered in `frontend/src/lib/registry/defaults.ts`.
- Tests: Vitest unit, `vitest-browser-svelte` browser component tests, Playwright E2E.
- Inline `#[cfg(test)] mod tests` at bottom of Rust source files for protocol crate.
- Schemas live in `spec/schemas/*.yaml`, validated against live wire messages in `frontend/tests/e2e/protocol-conformance.spec.ts` via `frontend/tests/helpers/schema-validator.ts` (ajv + js-yaml at test time). **No codegen — Rust and TS types are hand-maintained mirrors of the YAML.**
- Commit hook: `commit_docs: true` in `.planning/config.json`; Nyquist validation enabled.

## Standard Stack

### Core (all already installed — no new deps for Part A)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `svelte` | ^5.53.0 | Svelte 5 runes + reactive proxies for fine-grained reactivity | Fine-grained reactivity is the primary mechanism for focus preservation [CITED: svelte.dev/docs/svelte/$state] |
| `bits-ui` | ^2.17.3 | Headless primitives underlying shadcn-svelte | Shadcn Sidebar is built on bits-ui [VERIFIED: package.json] |
| `@lucide/svelte` | ^1.8.0 | Icon source for connection/status indicators | Phase 11 registry already wires this [VERIFIED: registry/icons.ts] |
| `serde` | 1 | Rust protocol serialization via `#[serde(tag = "op")]` tagged enum | Existing pattern in `messages.rs` for `ProtocolMessage` [VERIFIED: messages.rs:13] |
| `serde_json` | 1 | JSON round-trip for `PatchOperation` tagged variants | Existing dependency [VERIFIED: Cargo.toml] |

### Supporting (Part B — new install)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `shadcn-svelte sidebar` | latest per registry | Responsive sidebar composable (desktop collapsible + mobile sheet) | Required for `AppShell.svelte` — install via `pnpm dlx shadcn-svelte@latest add sidebar` or the project's equivalent [CITED: shadcn-svelte.com/docs/components/sidebar] |

**Installation verification (from current repo):**
```bash
ls frontend/src/lib/components/ui/
# badge button card checkbox dialog input label select separator skeleton table
```
**Missing:** `sidebar` (required). `dialog` already installed (Phase 11). Toast primitive status: Phase 11 D-04 picked shadcn Toast but verify in `frontend/src/lib/components/ui/` during execution — it is NOT present in the current listing; the planner must install it OR confirm it's already in `feedback/` or elsewhere.

**Version verification command (planner must run before writing tasks):**
```bash
cd frontend && npx shadcn-svelte@latest add sidebar --dry-run 2>&1 | head
# or check currently-installed registry version:
cat frontend/components.json
```

### Alternatives Considered & Rejected (per CONTEXT)

| Instead of | Alternative | Why Rejected |
|------------|-------------|--------------|
| Tagged enum with `op` | Separate `PatchOperation.data[]` + `PatchOperation.tree[]` arrays | CONTEXT D-A1 explicitly picks tagged enum |
| Per-op `surface` field | Single message-level `surface` | CONTEXT D-A3 explicitly picks message-level |
| Globally-unique node IDs | Per-surface node IDs | Pushes invariant onto every generator; doesn't help data-side |
| Sonner toast lib | Shadcn Toast (Radix) | Phase 11 D-04 |
| TanStack Virtual for content | Custom virtual scroll | Already decided [CITED: STATE.md] |

## Architecture Patterns

### Recommended Project Structure
```
backend/crates/marionette-protocol/src/
├── data.rs           # PatchOperation (REWRITE as tagged enum) + ValidationError
├── messages.rs       # PatchMessage (ADD `surface` field) + others unchanged
└── lib.rs            # pub use — add new variant types if extracted

backend/crates/marionette/src/builders/
├── standard.rs       # ADD SurfaceMount struct with #[derive(ComponentBuilder)]
├── app_shell.rs      # NEW FILE — hand-written AppShell builder (slot semantics)
└── mod.rs            # Re-export app_shell

backend/crates/crm-demo/src/
├── main.rs           # REPLACE handle_navigate: build AppShell into `main`,
│                     #   delegate screen content to `content` sub-surface
└── handlers/         # Each handler that renders to "main" needs s/"main"/"content"/

frontend/src/lib/
├── store/
│   ├── surfaces.svelte.ts   # REWRITE: fine-grained mutation API
│   └── data.svelte.ts       # EXTEND applyPatch to dispatch on op variant
├── transport/messages.ts    # UPDATE PatchOperation (union type), PatchMessage (add surface)
├── init.ts                  # FIX line 47: applyPatch(msg.surface, ...)
├── registry/defaults.ts     # ADD 'app-shell' + 'surface-mount'
└── components/
    ├── core/
    │   ├── Surface.svelte         # Verify recursion-safe (it is)
    │   ├── NodeRenderer.svelte    # Verify nothing changes (it doesn't)
    │   └── SurfaceMount.svelte    # NEW FILE
    ├── shell/                     # NEW DIRECTORY
    │   ├── AppShell.svelte
    │   └── AppShell.browser-test.ts
    └── ConnectionBanner.svelte    # DELETE (move role to footer)

frontend/src/routes/
└── +layout.svelte               # COLLAPSE to <Surface name="main"/>

spec/schemas/
├── data.yaml        # PatchOperation → tagged oneOf with 6 variants
└── message.yaml     # PatchMessage: required `surface` field

spec/PROTOCOL.md                 # §Messages > patch: document node ops + version 1.1.0
CONCEPT.md                       # Reconcile line 66
```

### Pattern 1: Svelte 5 fine-grained reactive map mutation (the focus-preservation mechanism)

**What:** Mutate individual keys on a reactive object proxy; do not replace the proxy reference.

**When to use:** Surface store `setNode`, `deleteNode`, `setChildren`, `insertChild`, `removeChild`.

**Why it works:**

> "Proxies allow Svelte to run code when you read or write properties, including via methods like `array.push(...)`, triggering granular updates. ... State is proxified recursively until Svelte finds something other than an array or simple object ... modifying an individual todo's property will trigger updates to anything in your UI that depends on that specific property." [CITED: svelte.dev/docs/svelte/$state]

**Example (from the intended `surfaces.svelte.ts` rewrite):**
```typescript
// Source: Svelte 5 $state deep-reactivity pattern (svelte.dev/docs/svelte/$state)
interface SurfaceTree {
	root: string;
	nodes: Record<string, ComponentNode>;
}

// Single top-level proxy — created ONCE, never reassigned
const surfaceState: Record<string, SurfaceTree> = $state({});

export function setSurfaceTree(surface: string, root: string, nodes: Record<string, ComponentNode>) {
	// On full Render: it's fine to replace the tree because the whole subtree changes.
	// But for subsequent node patches on an existing surface, prefer fine-grained.
	surfaceState[surface] = { root, nodes };
}

export function setNode(surface: string, id: string, component: ComponentNode) {
	const tree = surfaceState[surface];
	if (!tree) return;
	// CRITICAL: mutate the inner map key in place. Do NOT do `tree.nodes = {...tree.nodes, [id]: component}`.
	// Svelte's proxy tracks per-key reads — only components that read `nodes[id]` will re-derive.
	tree.nodes[id] = component;
}

export function deleteNode(surface: string, id: string) {
	const tree = surfaceState[surface];
	if (!tree) return;
	delete tree.nodes[id];
}

export function setChildren(surface: string, id: string, children: string[]) {
	const tree = surfaceState[surface];
	const parent = tree?.nodes[id];
	if (!parent) return;
	// In-place mutation of the child array. Keyed {#each ... (childId)} block will reorder, not remount.
	parent.children = children;
}

export function insertChild(surface: string, parent: string, index: number, childId: string) {
	const tree = surfaceState[surface];
	const p = tree?.nodes[parent];
	if (!p) return;
	if (!p.children) p.children = [];
	p.children.splice(index, 0, childId);
}

export function removeChild(surface: string, parent: string, childId: string) {
	const tree = surfaceState[surface];
	const p = tree?.nodes[parent];
	if (!p?.children) return;
	const i = p.children.indexOf(childId);
	if (i >= 0) p.children.splice(i, 1);
}
```

**Why this preserves focus:**
1. `NodeRenderer.svelte` reads `let node = $derived(nodes[nodeId]);` — a *per-key* read on the proxy. [VERIFIED: NodeRenderer.svelte:15]
2. `NodeRenderer.svelte` uses `{#each node.children as childId (childId)}` — keyed each block. Existing child DOM nodes are reordered, not recreated. [VERIFIED: NodeRenderer.svelte:31; CITED: svelte.dev/docs/svelte/each]
3. `setNode('main', 'field-b', ...)` only invalidates the derived on the NodeRenderer whose `nodeId === 'field-b'`. Sibling NodeRenderers are untouched — their `$derived(nodes['field-a'])` never re-fires, so `<TextInput>` never remounts, so the DOM `<input>` keeps focus and cursor.

**The current bug:** `setSurfaceTree` assigns `surfaceState[surface] = { root, nodes }` — which replaces the entire tree object reference every time, invalidating ALL derived reads of `nodes[x]` and thus all component trees. Even a pure data patch followed by `setSurfaceTree` would blow focus. The dirty-path queue in `dirty.svelte.ts` masks this for data paths during active editing, but it cannot mask a node patch.

### Pattern 2: `surface-mount` recursion safety

**Analysis of current `Surface.svelte` + `NodeRenderer.svelte`** [VERIFIED: source read]:

- `Surface.svelte` reads `getSurfaceTree(name)` via `$derived` and renders `<NodeRenderer nodeId={tree.root} nodes={tree.nodes} surface={name} />`.
- `NodeRenderer.svelte` resolves a component from the registry and renders it, passing a `children` snippet that recurses into sub-`NodeRenderer`s for `node.children`.

**Mounting `<Surface name="content" />` from inside a `NodeRenderer` is safe** because:
1. A `surface-mount` node is a *leaf* from NodeRenderer's perspective — the component registered for `'surface-mount'` is a Svelte component that internally instantiates `<Surface name={props.name}/>`. NodeRenderer does not recurse into its children (surface-mount has none).
2. `Surface.svelte` reads `getSurfaceTree(name)` via `$derived` — a pure reactive read. No store subscription cycle exists because surfaces are independent key-namespaced maps.
3. Lifecycle: parent surface's NodeRenderer mounts `SurfaceMount` → `SurfaceMount` mounts `<Surface name="content"/>` → inner `Surface` waits for its own `surfaceState.content` to populate (shows `LoadingSkeleton` while undefined). When the backend sends `Render` to `content`, only the inner NodeRenderer tree mounts. No ordering ambiguity.
4. Infinite-loop guard: `surface-mount` props are `{ name: string }` — a constant per node. There's no reactive loop between the outer and inner surface unless a handler explicitly Renders surface X from a handler triggered by surface X. That's a handler-author mistake, not an architectural flaw.

**Recommended `SurfaceMount.svelte` (trivial):**
```svelte
<script lang="ts">
	import Surface from './Surface.svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	let {
		props = {},
		bind,
		action,
		surface,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();
	// `surface` prop is the PARENT surface (where this node is mounted);
	// `props.name` is the CHILD surface this mount exposes.
</script>

<Surface name={props.name as string} />
```

The `surface` prop is ignored here: the mount is a pure redirection. `bind`/`action` are accepted for SDUI interface uniformity but unused.

### Pattern 3: Tagged enum for `PatchOperation` (Rust + JSON Schema)

**Rust (serde internally-tagged):**

Existing codebase already uses the internally-tagged pattern for `ProtocolMessage` via `#[serde(tag = "type", rename_all = "lowercase")]` [VERIFIED: messages.rs:13]. `PatchOperation` should follow the same style for consistency.

```rust
// backend/crates/marionette-protocol/src/data.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::component::Component;

/// A single patch operation applied to a surface.
///
/// Operations are applied in declared order within a `PatchMessage`, all-or-nothing.
/// Mix data and node-tree ops freely in one batch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum PatchOperation {
    /// Data op — set a value at a JSON Pointer path in the surface's data store.
    Set {
        path: String,
        value: serde_json::Value,
    },
    /// Node op — replace (or create) the component at this node ID.
    SetNode {
        id: String,
        component: Component,
    },
    /// Node op — delete the node with this ID from the surface's adjacency list.
    DeleteNode { id: String },
    /// Node op — replace the children array of the given node.
    SetChildren {
        id: String,
        children: Vec<String>,
    },
    /// Node op — insert an existing child ID into a parent's children array at index.
    InsertChild {
        parent: String,
        index: usize,
        #[serde(rename = "childId")]
        child_id: String,
    },
    /// Node op — remove a child ID from a parent's children array.
    RemoveChild {
        parent: String,
        #[serde(rename = "childId")]
        child_id: String,
    },
}
```

**Why internally-tagged (`#[serde(tag = "op")]`):**
- Matches wire format `{"op": "set-node", "id": "...", "component": {...}}` — a single flat JSON object per op, as typical in JSON Patch / OpenAPI oneOf tagged unions.
- Matches existing codebase idiom (`ProtocolMessage` uses `#[serde(tag = "type")]`).
- Externally-tagged would wrap each op as `{"set-node": {...}}`, which adds a level of nesting and doesn't match idiomatic JSON Schema `oneOf` with `discriminator`.
- Adjacently-tagged would be `{"op": "set-node", "data": {...}}` — also extra nesting.

**Pitfall — variant size:** `SetNode` embeds a full `Component` which is larger than other variants. `cargo clippy::pedantic` may warn about `clippy::large_enum_variant`. Resolution options:
1. `#[allow(clippy::large_enum_variant)]` at the enum (pragmatic; minimal ceremony).
2. Box the `Component`: `SetNode { id: String, component: Box<Component> }` — trades a heap alloc for smaller stack footprint. [CITED: rust-lang.github.io/rust-clippy/master/#large_enum_variant]

**Recommendation:** Start with `#[allow]`; revisit if patch batches become hot path. The crate isn't perf-critical.

**Interaction with `Component`'s own `type` discriminator:** `Component` uses `pub r#type: String` as a plain field (not a tagged enum discriminator) [VERIFIED: component.rs:8]. No collision with `PatchOperation`'s `op` tag — they're at different nesting levels:
```json
{"op": "set-node", "id": "x", "component": {"type": "text-input", "props": {...}}}
```

**JSON Schema (`spec/schemas/data.yaml`):**

```yaml
PatchOperation:
  oneOf:
    - $ref: "#/PatchOperationSet"
    - $ref: "#/PatchOperationSetNode"
    - $ref: "#/PatchOperationDeleteNode"
    - $ref: "#/PatchOperationSetChildren"
    - $ref: "#/PatchOperationInsertChild"
    - $ref: "#/PatchOperationRemoveChild"
  discriminator:
    propertyName: op
    mapping:
      set: "#/PatchOperationSet"
      set-node: "#/PatchOperationSetNode"
      delete-node: "#/PatchOperationDeleteNode"
      set-children: "#/PatchOperationSetChildren"
      insert-child: "#/PatchOperationInsertChild"
      remove-child: "#/PatchOperationRemoveChild"

PatchOperationSet:
  type: object
  required: [op, path, value]
  properties:
    op:
      type: string
      const: set
    path:
      type: string
      format: json-pointer
    value:
      description: New value to set at the path
  additionalProperties: false

PatchOperationSetNode:
  type: object
  required: [op, id, component]
  properties:
    op:
      type: string
      const: set-node
    id:
      type: string
      description: Node ID within the target surface's adjacency list
    component:
      $ref: "component.yaml#/Component"
  additionalProperties: false

PatchOperationDeleteNode:
  type: object
  required: [op, id]
  properties:
    op:
      type: string
      const: delete-node
    id:
      type: string
  additionalProperties: false

PatchOperationSetChildren:
  type: object
  required: [op, id, children]
  properties:
    op:
      type: string
      const: set-children
    id:
      type: string
    children:
      type: array
      items:
        type: string
  additionalProperties: false

PatchOperationInsertChild:
  type: object
  required: [op, parent, index, childId]
  properties:
    op:
      type: string
      const: insert-child
    parent:
      type: string
    index:
      type: integer
      minimum: 0
    childId:
      type: string
  additionalProperties: false

PatchOperationRemoveChild:
  type: object
  required: [op, parent, childId]
  properties:
    op:
      type: string
      const: remove-child
    parent:
      type: string
    childId:
      type: string
  additionalProperties: false
```

Ajv (used by `frontend/tests/helpers/schema-validator.ts`) supports OpenAPI 3.1 `oneOf + discriminator` with `strict: false` — the existing schema-validator is already configured for this [VERIFIED: schema-validator.ts:85].

**`spec/schemas/message.yaml` update:**
```yaml
PatchMessage:
  type: object
  required:
    - type
    - surface   # <-- ADDED
    - patch
  properties:
    type:
      type: string
      const: patch
    id:
      $ref: "common.yaml#/MessageId"
    surface:    # <-- ADDED
      $ref: "common.yaml#/Surface"
    patch:
      type: array
      items:
        $ref: "data.yaml#/PatchOperation"
  additionalProperties: false
```

### Pattern 4: Hand-written backend builder for AppShell

The existing `#[derive(ComponentBuilder)]` macro assumes all fields are plain props that get serialized into the `props` JSON map [VERIFIED: component_builder.rs:119-142]. AppShell needs slot setters like `.sidebar(id)` that store a node ID in the component's `props` under a specific key (e.g., `sidebarNodeId`) but do NOT take an `Option<String>` required field in the usual sense — they also need to track the underlying child node trees.

**Existing precedent for hand-written builders:** None yet — all current builders in `standard.rs` use the derive macro. AppShell is the first hand-written builder, and its pattern will be a template for future high-level structural components (tab-views, split-panes, wizards).

**Recommended shape (new file `backend/crates/marionette/src/builders/app_shell.rs`):**

```rust
//! Hand-written AppShell builder.
//!
//! AppShell is a first-class SDUI component whose "props" are node-ID references
//! into the top-level adjacency list of the shell surface. Its slot methods accept
//! pre-built `(id, Component)` tuples from other builders and record both the slot
//! ID into props and the full sub-tree into the children-collection for flattening.

use marionette_protocol::Component;
use serde_json::{json, Map, Value};
use uuid::Uuid;

pub struct AppShell {}

impl AppShell {
    #[must_use]
    pub fn new() -> AppShellBuilder {
        AppShellBuilder::default()
    }
}

#[derive(Default)]
pub struct AppShellBuilder {
    sidebar_node: Option<(String, Component)>,
    header_node: Option<(String, Component)>,
    footer_node: Option<(String, Component)>,
    main_node: Option<(String, Component)>,
    popups_node: Option<(String, Component)>,
    toasts_node: Option<(String, Component)>,
    // Descendants of each slot child (flattened from build_with_children on the caller side).
    descendants: Vec<(String, Component)>,
    id: Option<String>,
}

impl AppShellBuilder {
    #[must_use]
    pub fn sidebar(mut self, slot: (String, Component)) -> Self {
        self.sidebar_node = Some(slot);
        self
    }
    #[must_use]
    pub fn header(mut self, slot: (String, Component)) -> Self { /* same shape */ self.header_node = Some(slot); self }
    #[must_use]
    pub fn footer(mut self, slot: (String, Component)) -> Self { self.footer_node = Some(slot); self }
    #[must_use]
    pub fn main(mut self, slot: (String, Component)) -> Self { self.main_node = Some(slot); self }
    #[must_use]
    pub fn popups(mut self, slot: (String, Component)) -> Self { self.popups_node = Some(slot); self }
    #[must_use]
    pub fn toasts(mut self, slot: (String, Component)) -> Self { self.toasts_node = Some(slot); self }
    /// Attach descendants harvested from sub-builder `.build_with_children()` calls.
    #[must_use]
    pub fn with_descendants(mut self, desc: Vec<(String, Component)>) -> Self {
        self.descendants.extend(desc);
        self
    }
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Build the AppShell node; returns (id, Component).
    /// Slot node IDs are written into props; slot child tuples are NOT returned here —
    /// call `build_with_children` for the flat-map form.
    #[must_use]
    pub fn build(self) -> (String, Component) {
        let shell_id = self.id.unwrap_or_else(|| format!("app-shell-{}", Uuid::new_v4()));

        // Collect slot IDs into props (using camelCase to match SDUI frontend convention)
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
            children: None,  // NO positional children — slots are in props
            bind: None,
            action: None,
            visible: None,
        };
        (shell_id, component)
    }

    /// Build the AppShell and flatten all slot nodes + their descendants into a single list
    /// suitable for insertion into `RenderMessage.nodes`.
    #[must_use]
    pub fn build_with_children(self) -> Vec<(String, Component)> {
        // Collect slot roots into a vec before consuming self in build()
        let mut out = Vec::new();
        let mut slots: Vec<(String, Component)> = Vec::new();
        if let Some(s) = &self.sidebar_node { slots.push(s.clone()); }
        if let Some(s) = &self.header_node { slots.push(s.clone()); }
        if let Some(s) = &self.footer_node { slots.push(s.clone()); }
        if let Some(s) = &self.main_node { slots.push(s.clone()); }
        if let Some(s) = &self.popups_node { slots.push(s.clone()); }
        if let Some(s) = &self.toasts_node { slots.push(s.clone()); }
        let desc = self.descendants.clone();

        let (shell_id, shell) = self.build();
        out.push((shell_id, shell));
        out.extend(slots);
        out.extend(desc);
        out
    }
}
```

**Usage site (in the migrated `handle_navigate` in `main.rs`):**
```rust
// Build sub-trees via normal builders
let side_nav_nodes = SideNav::new().id("shell-sidebar").children(nav_items).build_with_children();
let (sidebar_root, sidebar_desc) = split_head_tail(side_nav_nodes);

let (header_root, header_desc) = /* Container with title + user menu */;
let (footer_root, footer_desc) = /* Container with version + connection-status */;

let (content_mount, _) = SurfaceMount::new("content").id("shell-content-mount").build();
let (modal_mount, _) = SurfaceMount::new("modal").id("shell-modal-mount").build();
let (toast_mount, _) = SurfaceMount::new("toasts").id("shell-toasts-mount").build();

let shell_nodes = AppShell::new()
    .id("app-shell-root")
    .sidebar(sidebar_root)
    .header(header_root)
    .footer(footer_root)
    .main((content_mount.0.clone(), content_mount.1))
    .popups((modal_mount.0.clone(), modal_mount.1))
    .toasts((toast_mount.0.clone(), toast_mount.1))
    .with_descendants([sidebar_desc, header_desc, footer_desc].concat())
    .build_with_children();
```

> **Planner's call:** Whether to add a helper `split_head_tail` or to use `build_tree` (which returns `(root, descendants)` separately). The existing `build_tree()` method on the generated builders [VERIFIED: component_builder.rs:284] is designed exactly for this case — prefer `build_tree()` over manual splitting.

### Anti-Patterns to Avoid
- **Replacing `surfaceState[surface]` wholesale during incremental updates.** Kills fine-grained reactivity, blows focus.
- **Spreading into a new children array** (`parent.children = [...parent.children, x]`). The keyed each block will still work, but a new array reference may trigger re-evaluation of parent-level derived state. Use in-place `.splice` / `.push`.
- **Using `#[serde(tag = "op", content = "data")]` (adjacently-tagged).** Adds unnecessary `data:` nesting at the wire level; doesn't match idiomatic JSON Schema oneOf tagged unions.
- **Returning multiple Render messages from one handler to multiple surfaces for the shell case.** After Phase 12, the initial nav response should Render ONCE into `main` (the shell) and ONCE into `content` (the screen) — not into `sidebar`+`main` like today. The current main.rs `handle_navigate` pattern of rendering `side-nav` into a top-level `sidebar` surface goes away entirely.
- **Hand-rolling a graph walk for GC** when the patch batch already has node IDs. The walk-and-prune starts from `root`; it's a simple reachability BFS.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Mobile sidebar sheet overlay + desktop collapsible | Custom CSS media query + manual state | `shadcn-svelte sidebar` (`Sidebar.Provider` + `Sidebar.Root` + `Sidebar.Trigger`) | Responsive behavior, keyboard navigation, ARIA, and CSS variable theming already handled [CITED: shadcn-svelte.com/docs/components/sidebar] |
| Modal dialog overlay | Custom portal + focus trap | `shadcn-svelte dialog` (already installed) | Phase 11 already uses this for `ModalSurface.svelte` [VERIFIED: ModalSurface.svelte:5] |
| Toast region with queueing / auto-dismiss | Custom timers | Shadcn Toast (per Phase 11 D-04) — verify installed | Phase 11 already picked this; Phase 12 consumes it |
| Tagged-union serialization in Rust | Manual string-match / custom serde impl | `#[serde(tag = "op")]` on an enum | Matches existing `ProtocolMessage` pattern [VERIFIED: messages.rs:13] |
| JSON Schema oneOf validation | Custom validation code | `spec/openapi.yaml` + existing ajv validator | Infrastructure is already live [VERIFIED: schema-validator.ts] |
| Keyed list reordering without remounting DOM | Manual DOM node tracking | Svelte `{#each … as item (key)}` (already in NodeRenderer) | Svelte guarantees identity preservation [CITED: svelte.dev/docs/svelte/each] |
| Reactive map with per-key subscription | Custom pub/sub | Svelte 5 `$state({})` with in-place key mutation | Deep reactive proxies give this free [CITED: svelte.dev/docs/svelte/$state] |

**Key insight:** Every piece of hard technology Phase 12 needs (reactive proxies, keyed lists, tagged-enum serialization, schema validation, responsive sidebars, modals, toasts) exists and is already wired. The work is **plumbing and migration**, not invention.

## Runtime State Inventory

Phase 12 includes a *structural* migration (routes/+layout.svelte + handler surface renames + retirement of `ConnectionBanner`) but is NOT a rename phase in the usual sense. The table below answers the five categories explicitly so the planner doesn't miss anything.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| **Stored data** | None. The project uses SQLite via sea-orm for CRM entities, but the protocol changes don't touch any persisted content. No stored string referring to the old protocol shape or to surface names `sidebar`/`modal`/`toast` as database keys. [VERIFIED: no hits in `backend/crates/crm-demo/src/entities/` for `sidebar`/`modal`] | **None** — verified by grep |
| **Live service config** | None. No external services (n8n, Datadog, Tailscale, etc.) depend on the surface names or the `PatchMessage` shape. | **None** |
| **OS-registered state** | None. The project has no Windows Task Scheduler, systemd, launchd, or pm2 registrations referencing these names. `make dev` launches ephemeral processes. | **None** |
| **Secrets/env vars** | `LISTMONK_URL`, `LISTMONK_USER`, `LISTMONK_PASSWORD` exist [VERIFIED: crm-demo/src/main.rs:281-295] but are unrelated to this phase. | **None** |
| **Build artifacts** | `backend/target/` exists (Cargo build cache). A clean build after changing the `marionette-protocol` crate should be correct; `cargo test` invalidates caches automatically. No stale `.egg-info` / compiled binaries carry old shape. Frontend has no pre-compiled schema artifacts — schemas are loaded from YAML at test time. | **None** — standard `cargo build` / `npm install` re-runs cover it |

**Additional runtime concern — the wire-level breaking change:** Because `PatchMessage` shape changes, any **open WebSocket connection** at deploy time would fall into version-mismatch territory. Per CONTEXT D-A7 and `spec/PROTOCOL.md §Stale Client Handling`, `HelloMessage.version` bump to `1.1.0` plus page reload covers this — no additional work needed in this phase. Mentioned here for completeness.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust + Cargo | Backend protocol + builder changes | ✓ | 1.93+ per STACK.md | — |
| Node.js + npm | Frontend + schema-validator tests | ✓ | 25.4.0 | — |
| Playwright + Chromium | Browser component tests, E2E schema conformance tests | ✓ | ^1.58.2 | — |
| vitest-browser-svelte | Focus-preservation browser test | ✓ | ^2.1.0 | — |
| bits-ui | shadcn sidebar dependency | ✓ | ^2.17.3 | — |
| shadcn-svelte CLI | Install `sidebar` primitive | ✓ (remote via `pnpm dlx shadcn-svelte@latest`) | latest | Manually copy sidebar components from shadcn-svelte registry if CLI fails |
| ajv + js-yaml | Schema round-trip validation | ✓ | ^8.18.0 / ^4.1.1 | — |
| SQLite | CRM persistence (unrelated to phase but required for E2E) | ✓ | (via rusqlite/sea-orm) | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None required — `sidebar` is the only new component and has a manual copy-from-registry fallback.

## Common Pitfalls

### Pitfall 1: `--sidebar-*` CSS token name mismatch
**What goes wrong:** Current `app.css` defines `--sidebar-background` and `--sidebar-foreground`. The shadcn-svelte Sidebar component expects `--sidebar` (no suffix) as the background token, plus `--sidebar-foreground`, `--sidebar-primary`, `--sidebar-primary-foreground`, `--sidebar-accent`, `--sidebar-accent-foreground`, `--sidebar-border`, `--sidebar-ring`. [CITED: shadcn-svelte.com/docs/components/sidebar; VERIFIED: app.css:29-35]
**Why it happens:** Phase 10 defined the tokens based on an earlier mental model; the actual shadcn Sidebar registry uses a subtly different naming scheme.
**How to avoid:** When installing `sidebar` via `shadcn-svelte add sidebar`, the CLI will write its own default tokens to `app.css` if they're missing. Alternatively, the planner should include a task that:
1. Reads the shadcn-svelte sidebar block templates to confirm the canonical names (the install step prints them).
2. Renames `--sidebar-background` → `--sidebar` in both `:root` and `.dark` blocks of `app.css`.
3. Updates the `@theme inline` mapping: `--color-sidebar-background: var(--sidebar-background)` → `--color-sidebar: var(--sidebar)`.
4. Audits any Tailwind class usages of `bg-sidebar-background` (currently in `SideNav.svelte:20` and `Surface.svelte:16`) and replaces with `bg-sidebar`.
**Warning signs:** Sidebar renders with no background color in the AppShell browser test; `bg-sidebar-background` in grep still finds matches.

### Pitfall 2: Focus loss from `setSurfaceTree` replacing the whole tree on `Render`
**What goes wrong:** A full `Render` into surface X replaces `surfaceState[x] = {root, nodes}`. Every `$derived` on a descendant NodeRenderer re-evaluates. If the user was typing in a field on surface Y (a sibling surface), NOT surface X, no problem — surface Y's subtree is untouched. But if surface X receives a Render AND the user was typing in X, focus is lost (expected — Render is a full replacement).
**Why it happens:** `Render` is semantically "full replacement" so this is correct. The trap is that the initial implementation might accidentally use `Render` instead of `Patch` for small updates to X that should preserve focus.
**How to avoid:** The handler author chooses `Render` vs `Patch`. The store implementation must guarantee that *Patch* preserves focus on unrelated nodes even within the same surface. The focus-preservation test exercises a node-patch (not a Render) to prove this.
**Warning signs:** Focus-preservation test fails with "field-a lost focus after patch to field-b in same surface."

### Pitfall 3: `surface-mount` name collision between registry component type and SDUI component prop
**What goes wrong:** The component type string is `'surface-mount'` and its `props.name` refers to a sub-surface name like `'content'`. If a developer confuses the two — e.g. passes `name: "surface-mount"` — the inner `<Surface name="surface-mount"/>` renders a skeleton forever because no surface of that name is rendered to.
**Why it happens:** Two different namespaces (component type vs. surface name) with similar words.
**How to avoid:** Document clearly in the builder doc comment. The browser test for `SurfaceMount.svelte` should include an assertion that a legitimate name like `"content"` correctly mounts the child surface while a nonexistent name shows the loading skeleton.
**Warning signs:** AppShell browser test shows a blank skeleton where content should appear.

### Pitfall 4: Serde internally-tagged enums reject unknown `op` values by default
**What goes wrong:** `#[serde(tag = "op")]` with known variants rejects any wire message whose `op` doesn't match one of the enum variants. Good for strictness, bad for forward compatibility — if v1.2 adds a new op and a v1.1 client receives it, deserialization errors.
**Why it happens:** Strict tagged-enum semantics.
**How to avoid:** Per the pre-deployment no-backcompat posture, this is fine. The protocol version guards against shape mismatch via `HelloMessage.version`. Document in `spec/PROTOCOL.md §Stale Client Handling` that an unknown `op` is an error that the client should surface as a stale-client prompt. No code change.
**Warning signs:** N/A for this phase.

### Pitfall 5: `dirty.svelte.ts` only applies to data paths, not node ops
**What goes wrong:** The existing `isDirty` / `queuePatch` mechanism in `frontend/src/lib/store/dirty.svelte.ts` intercepts data patches to paths currently being edited. Node patches don't go through this queue — a `set-node` on the node whose TextInput is focused will still replace the underlying component definition, which may cascade to a DOM remount via `type` or `bind` changes.
**Why it happens:** The dirty mechanism was designed for data reactivity, not tree reactivity.
**How to avoid:** The focus-preservation test specifically patches a SIBLING node, not the focused node. That's the canonical demo and the design point: you can't preserve focus on a field whose component definition is being replaced. The test assertion is "patch to B preserves focus on A" — NOT "patch to A preserves focus on A." This should be clearly documented in the test and in `spec/PROTOCOL.md`. No code change, but a deliberate design clarification task.
**Warning signs:** Confused test writer adds an assertion that patching a focused field preserves its cursor.

### Pitfall 6: CRM handler surface migration misses handlers
**What goes wrong:** 8 handler files render to `"main"` today [VERIFIED: grep]. Missing one leaves a screen that Renders into the old `main` surface and bypasses the AppShell entirely — it renders in the parent surface instead of the `content` sub-surface.
**Why it happens:** Easy to miss one handler in a codebase-wide rename.
**How to avoid:** The rename is mechanical and small. A single grep + sed (surface: "main" → surface: "content") covers it, with the explicit exception of the shell-building code in `main.rs` `handle_navigate` which NOW renders `main` for the shell itself. The planner should include a verification step: `grep -rn 'surface:\s*"main"' backend/crates/crm-demo/src/` should return ONLY the shell-building site after migration.
**Warning signs:** Phase 12 E2E test sees a screen rendered outside the shell frame.

## Code Examples

### Example 1: applyPatch dispatcher (extended to handle node ops)

```typescript
// frontend/src/lib/store/data.svelte.ts — NEW applyPatch signature
// Source: codebase — extension of existing applyPatch

import type { PatchOperation } from '$lib/transport/messages.js';
import { resolvePointer, setAtPointer } from './pointer.js';
import { isDirty, queuePatch } from './dirty.svelte.js';
import {
	setNode,
	deleteNode,
	setChildren,
	insertChild,
	removeChild,
	gcOrphans,
} from './surfaces.svelte.js';

export function applyPatch(surface: string, operations: PatchOperation[]): void {
	for (const op of operations) {
		switch (op.op) {
			case 'set': {
				if (isDirty(op.path)) {
					queuePatch(op.path, op);
				} else {
					setAtPointer(getStore(surface).data, op.path, op.value);
				}
				break;
			}
			case 'set-node':
				setNode(surface, op.id, op.component);
				break;
			case 'delete-node':
				deleteNode(surface, op.id);
				break;
			case 'set-children':
				setChildren(surface, op.id, op.children);
				break;
			case 'insert-child':
				insertChild(surface, op.parent, op.index, op.childId);
				break;
			case 'remove-child':
				removeChild(surface, op.parent, op.childId);
				break;
		}
	}
	// Optional: run GC once per batch (D-A8). Debate: synchronous here vs. microtask.
	gcOrphans(surface);
}
```

### Example 2: Walk-and-prune GC (O(N) reachability BFS)

```typescript
// frontend/src/lib/store/surfaces.svelte.ts — gcOrphans
export function gcOrphans(surface: string): void {
	const tree = surfaceState[surface];
	if (!tree) return;

	// BFS reachability from root, using the children adjacency.
	const reachable = new Set<string>();
	const queue: string[] = [tree.root];
	while (queue.length > 0) {
		const id = queue.shift()!;
		if (reachable.has(id)) continue;
		reachable.add(id);
		const node = tree.nodes[id];
		if (node?.children) {
			for (const child of node.children) queue.push(child);
		}
	}

	// Delete unreachable nodes in place.
	for (const id of Object.keys(tree.nodes)) {
		if (!reachable.has(id)) {
			delete tree.nodes[id];
		}
	}
}
```

**BFS vs DFS:** No practical difference at these sizes. BFS is more iteration-friendly with a queue; DFS uses the call stack. Prefer BFS.

**Synchronous in applyPatch vs microtask:** Synchronous is simpler and the O(N) cost is negligible (a few hundred nodes at most per surface in realistic apps). Defer only if profiling shows it matters.

**Pre-allocate visited set:** Not worth it.

### Example 3: AppShell.svelte composition (shadcn Sidebar primitives)

```svelte
<!-- frontend/src/lib/components/shell/AppShell.svelte -->
<!-- Source: shadcn-svelte.com/docs/components/sidebar composition pattern -->
<script lang="ts">
	import type { ComponentAction } from '$lib/transport/messages';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';

	let {
		props = {},
		bind,
		action,
		surface,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();

	// Resolve slot IDs from props
	let sidebarId = $derived(props.sidebarNodeId as string | undefined);
	let headerId = $derived(props.headerNodeId as string | undefined);
	let footerId = $derived(props.footerNodeId as string | undefined);
	let mainId = $derived(props.mainNodeId as string | undefined);
	let popupsId = $derived(props.popupsNodeId as string | undefined);
	let toastsId = $derived(props.toastsNodeId as string | undefined);

	// Look up nodes from the current surface's tree (shell lives in its own surface, typically 'main')
	let tree = $derived(getSurfaceTree(surface));
	let nodes = $derived(tree?.nodes ?? {});
</script>

<Sidebar.Provider>
	<Sidebar.Root collapsible="offcanvas">
		<Sidebar.Content>
			{#if sidebarId && nodes[sidebarId]}
				<NodeRenderer nodeId={sidebarId} {nodes} {surface} />
			{/if}
		</Sidebar.Content>
	</Sidebar.Root>
	<Sidebar.Inset>
		<div class="flex min-h-screen flex-col">
			<header class="flex items-center gap-2 border-b px-4 py-2">
				<Sidebar.Trigger />
				{#if headerId && nodes[headerId]}
					<NodeRenderer nodeId={headerId} {nodes} {surface} />
				{/if}
			</header>
			<main class="flex-1 overflow-auto">
				{#if mainId && nodes[mainId]}
					<NodeRenderer nodeId={mainId} {nodes} {surface} />
				{/if}
			</main>
			<footer class="border-t px-4 py-2 text-xs text-muted-foreground">
				{#if footerId && nodes[footerId]}
					<NodeRenderer nodeId={footerId} {nodes} {surface} />
				{/if}
			</footer>
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>

<!-- Popups / toasts rendered as siblings at the top of the shell (above Sidebar.Provider
     is fine because they're fixed-position). Could equally sit inside Inset. -->
{#if popupsId && nodes[popupsId]}
	<NodeRenderer nodeId={popupsId} {nodes} {surface} />
{/if}
{#if toastsId && nodes[toastsId]}
	<NodeRenderer nodeId={toastsId} {nodes} {surface} />
{/if}
```

**Note on popups/toasts placement:** The shadcn Dialog and Toast primitives use portals (`bits-ui` builds on `@melt-ui` portal). They render into document.body regardless of where their component tree sits. Safe to mount the `surface-mount` for `modal`/`toasts` inside `Sidebar.Inset` or outside — same DOM destination.

### Example 4: Focus-preservation browser test (vitest-browser-svelte)

```typescript
// frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts
// Source: codebase TextInput.browser-test.ts pattern + NodeRenderer dispatcher
import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import { tick } from 'svelte';
import Surface from '$lib/components/core/Surface.svelte';
import {
	setSurfaceTree,
	setNode,
	clearSurfaceTree,
} from '$lib/store/surfaces.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import { resetDirty } from '$lib/store/dirty.svelte';
import { registerDefaults } from '$lib/registry/defaults';

beforeEach(() => {
	resetStore('fptest');
	clearSurfaceTree('fptest');
	resetDirty();
	registerDefaults();
});

test('patch to sibling node preserves focus and cursor on focused input', async () => {
	// Arrange: a container with two text inputs
	setFullState('fptest', { a: '', b: '' });
	setSurfaceTree('fptest', 'root', {
		root: { type: 'container', children: ['field-a', 'field-b'] },
		'field-a': { type: 'text-input', bind: '/a', props: { label: 'A' } },
		'field-b': { type: 'text-input', bind: '/b', props: { label: 'B' } },
	});

	const screen = await render(Surface, { props: { name: 'fptest' } });

	// Focus field A, type "hello", position cursor at 3
	const inputs = screen.baseElement.querySelectorAll('input');
	const inputA = inputs[0] as HTMLInputElement;
	inputA.focus();
	inputA.setSelectionRange(0, 0);
	inputA.value = 'hello';
	inputA.dispatchEvent(new Event('input', { bubbles: true }));
	inputA.setSelectionRange(3, 3);

	expect(document.activeElement).toBe(inputA);

	// Act: patch field-b to change its label
	setNode('fptest', 'field-b', {
		type: 'text-input',
		bind: '/b',
		props: { label: 'B (changed)' },
	});
	await tick();

	// Assert: input A still has focus and cursor at 3
	expect(document.activeElement).toBe(inputA);
	expect(inputA.selectionStart).toBe(3);
	expect(inputA.selectionEnd).toBe(3);
	expect(inputA.value).toBe('hello');

	// Also assert field B updated its label visibly
	await expect.element(screen.getByText('B (changed)')).toBeVisible();
});
```

**Why this test is sufficient proof:**
- It exercises the real `Surface` → `NodeRenderer` → `TextInput` path with real registered components.
- It bypasses the transport layer (`setNode` is called directly) — this is intentional; the transport dispatching is tested separately.
- It asserts both focus identity (`document.activeElement`) and cursor position (`selectionStart/End`) — the full focus-preservation contract.

## State of the Art

| Old Approach (pre-Phase 12) | Current Approach (post-Phase 12) | When Changed | Impact |
|------------------------------|----------------------------------|--------------|--------|
| `PatchMessage.patch: [{path, value}]` data-only | `PatchMessage.patch: [oneOf 6 variants]` mixed data + tree | Phase 12 | CONCEPT.md promise finally kept; CRM can swap fields dynamically |
| `PatchMessage` with no `surface` field (hardcoded client-side to `main`) | `PatchMessage.surface: Surface` required | Phase 12 | Enables patches to sidebar/modal/toast surfaces; fixes init.ts:47 bug |
| Top-level `routes/+layout.svelte` mounts 4 surfaces directly | Single `<Surface name="main"/>`; AppShell mounts sub-surfaces via `surface-mount` | Phase 12 | Shell becomes server-driven content, layout.svelte becomes trivial |
| `ConnectionBanner` as fixed top bar | Footer connection-status indicator inside AppShell | Phase 12 | Less obtrusive, always visible, no vertical space cost |
| `setSurfaceTree` replaces `{root, nodes}` wholesale | Fine-grained `setNode`/`deleteNode`/`setChildren`/`insertChild`/`removeChild` | Phase 12 | Focus preservation works; per-key Svelte 5 reactivity |
| `HelloMessage.version = "1.0.0"` | `"1.1.0"` | Phase 12 | Stale-client detection triggers on cached clients |

**Deprecated/outdated:**
- `PatchOperation { path: String, value: Value }` — replaced by tagged enum
- `ConnectionBanner.svelte` — retired (functionality in footer indicator)
- `frontend/src/lib/components/core/ConnectionBanner.browser-test.ts` — delete with the component

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | shadcn-svelte Sidebar installs via `shadcn-svelte@latest add sidebar` and creates `$lib/components/ui/sidebar/` with the standard sub-components listed. | Pattern: AppShell composition | [CITED docs] The exact component surface may differ slightly between shadcn-svelte releases. Planner should run the install command once at phase kickoff and adjust if the API diverges. |
| A2 | `ajv` with `strict: false` accepts OpenAPI 3.1 `oneOf + discriminator` without custom keyword registration. | Pattern: JSON Schema tagged union | [ASSUMED from schema-validator.ts being `strict: false`] If ajv rejects the discriminator keyword, the schema-validator helper needs `oneOf`-only (no discriminator) and relies on Ajv's `oneOf` validation alone — which still works but gives worse error messages. |
| A3 | Phase 11 D-04 "shadcn Toast (Radix)" is actually installed or can be installed via `shadcn-svelte add`. | Standard Stack | [ASSUMED — not visible in current `components/ui/` listing] Planner must verify at phase start: `ls frontend/src/lib/components/ui/ \| grep -i toast`. If missing, Phase 12 must include install. Could also mean Phase 11 landed Sonner instead — re-check Phase 11 context if in doubt. |
| A4 | Clippy will warn `clippy::large_enum_variant` on the `SetNode { component: Component }` variant under `pedantic` settings. | Pattern: tagged enum | [ASSUMED] If it doesn't, the `#[allow]` is unnecessary — harmless. |
| A5 | The existing `build_tree()` builder method works for the "build a sub-tree, pass root to AppShell, hand descendants separately" pattern in `main.rs`. | Pattern: Hand-written builder | [VERIFIED from component_builder.rs:284 — method exists and returns `(root, descendants)`] No risk. |
| A6 | `Dialog` and `Toast` primitives use DOM portals so their SDUI-mounted position in the shell tree doesn't affect rendering. | Pattern: AppShell composition | [ASSUMED — common pattern for bits-ui / Radix] If not, placing the popups `NodeRenderer` outside `Sidebar.Inset` is fine. |
| A7 | Per-screen CRM handlers that currently render to `surface: "main"` can be changed to `surface: "content"` without any other logic changes — the tree they build is identical. | Pitfall 6 | [VERIFIED from grep — 8 sites, all single-line changes] No risk. |
| A8 | `setSurfaceTree` is called only on full `Render` messages, not during incremental updates. After Phase 12, `setSurfaceTree` remains the mechanism for `Render` (where wholesale replacement is correct) and `setNode`/`deleteNode`/etc. handle patches. | Pattern 1 | [VERIFIED from init.ts:33 — `setSurfaceTree` is only called in the render handler] No risk. |

## Open Questions

1. **Toast primitive current status**
   - What we know: Phase 11 D-04 picked shadcn Toast; `components/ui/` currently has no `toast` directory
   - What's unclear: Is shadcn Toast installed under a different name, not yet installed, or did Phase 11 pivot silently to Sonner?
   - Recommendation: Phase 12 Wave 0 task should `ls frontend/src/lib/components/ui/` and install if missing. Don't block research on this.

2. **Location of AppShell hand-written builder: separate file vs. append to `standard.rs`?**
   - What we know: CONTEXT suggests new file `builders/app_shell.rs` is an option; no existing hand-written builders to mimic
   - What's unclear: Whether project style prefers one file per hand-written structural component or grouping in `standard.rs`
   - Recommendation: New file `backend/crates/marionette/src/builders/app_shell.rs`. Reasoning: (1) the hand-written shape is substantively different from the derive-macro shape in `standard.rs`; (2) establishes a clear pattern for future structural components. Update `builders/mod.rs` to re-export.

3. **Does the frontend TypeScript `PatchOperation` union need JSDoc on each variant?**
   - What we know: Current type is a plain interface with 2 fields
   - What's unclear: Project JSDoc discipline on discriminated unions
   - Recommendation: Brief JSDoc on each variant matching the spec/schemas/data.yaml descriptions. Low cost, high clarity.

4. **Protocol documentation order: tagged union example first or text-first?**
   - What we know: `spec/PROTOCOL.md §Messages > patch` currently has text-then-example
   - What's unclear: Whether node ops should get their own top-level subsection or inline within §patch
   - Recommendation: Inline within §patch. Add a new "Node tree operations" sub-heading below the existing "Data operations" description; add one worked example per node op variant.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework (backend) | `cargo test` — inline `#[cfg(test)] mod tests` |
| Framework (frontend unit) | Vitest 4.1 (node env) |
| Framework (frontend browser) | `vitest-browser-svelte` 2.1 + Playwright/Chromium |
| Framework (frontend E2E) | Playwright 1.58 against :5173 or :3001 |
| Config file (frontend unit) | `frontend/vite.config.ts` (default vitest wiring) |
| Config file (frontend browser) | `frontend/vitest-browser.config.ts` |
| Config file (frontend E2E) | `frontend/playwright.config.ts`, `frontend/playwright.e2e.config.ts` |
| Quick run (backend protocol only) | `cd backend && cargo test -p marionette-protocol` |
| Quick run (frontend unit) | `cd frontend && npm test -- --run surfaces` |
| Quick run (frontend browser) | `cd frontend && npx vitest --config vitest-browser.config.ts --run surfaces.focus-preservation` |
| Full suite (backend) | `cd backend && cargo test` |
| Full suite (frontend unit) | `cd frontend && npm test -- --run` |
| Full suite (frontend browser) | `cd frontend && npx vitest --config vitest-browser.config.ts --run` |
| Full suite (frontend E2E) | `cd frontend && npx playwright test --config playwright.e2e.config.ts` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| **PATCH-01** | `PatchOperation` tagged enum serializes/deserializes each of 6 variants losslessly | unit (Rust) | `cargo test -p marionette-protocol data::tests::patch_operation -- --exact` | ❌ Wave 0 — rewrite `data.rs` tests |
| **PATCH-01** | `PatchMessage` serializes with required `surface` field; wire example round-trips | unit (Rust) | `cargo test -p marionette-protocol messages::tests::patch_round_trip` | ❌ Update existing test — adds `surface` |
| **PATCH-01** | TS `PatchOperation` union type compiles and exhaustive-switches correctly | unit (TS) | `cd frontend && npm run check` | ✅ Exists via svelte-check |
| **PATCH-01** | Live wire patch messages validate against `spec/schemas/data.yaml`'s new `oneOf` | E2E (Playwright + ajv) | `cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/protocol-conformance.spec.ts` | ✅ Exists at `protocol-conformance.spec.ts` — needs new test cases for each op variant |
| **PATCH-02** | Frontend `setNode` mutates in place; `$derived(nodes[id])` on sibling NodeRenderer does NOT re-fire | unit (TS) | `cd frontend && npm test -- --run surfaces.svelte.test` | ❌ Wave 0 — new `frontend/src/lib/store/surfaces.svelte.test.ts` |
| **PATCH-02** | Focused TextInput retains focus + cursor position across patch to sibling node | browser (vitest-browser-svelte) | `cd frontend && npx vitest --config vitest-browser.config.ts --run surfaces.focus-preservation` | ❌ Wave 0 — new `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` |
| **PATCH-02** | Full tree Render still replaces correctly (no regression) | unit (TS) | `cd frontend && npm test -- --run surfaces.svelte.test` | ❌ Covered by unit test above |
| **PATCH-02** | Walk-and-prune GC removes orphan node IDs from `surfaceState[x].nodes` | unit (TS) | `cd frontend && npm test -- --run surfaces.svelte.test` | ❌ Covered by unit test above |
| **PATCH-03** | `HelloMessage.version` reports `"1.1.0"` on first connection | E2E | `cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/protocol-conformance.spec.ts -g "hello"` | ✅ Update assertion in existing test |
| **PATCH-03** | `spec/PROTOCOL.md` contains "node tree operations" section; `CONCEPT.md` line 66 promise reconciled | doc (grep) | `grep -q 'node tree operation' spec/PROTOCOL.md && grep -q 'set-node' spec/PROTOCOL.md` | ❌ Manual verification in planner-written verify step |
| **SHELL-01** | `<AppShell>` renders Sidebar.Provider + Sidebar.Root + Sidebar.Trigger from shadcn primitives | browser | `cd frontend && npx vitest --config vitest-browser.config.ts --run AppShell` | ❌ Wave 0 — new `frontend/src/lib/components/shell/AppShell.browser-test.ts` |
| **SHELL-01** | Mobile viewport (width < 768) collapses sidebar into sheet; trigger shows | browser | same file as above | ❌ Part of AppShell.browser-test.ts |
| **SHELL-02** | Header slot renders passed node; Footer slot renders passed node | browser | same | ❌ Part of AppShell.browser-test.ts |
| **SHELL-03** | Sidebar uses `--sidebar*` CSS tokens; `bg-sidebar` class applied | visual + browser | `cd frontend && npx playwright test tests/visual/components.spec.ts -g "sidebar"` | ✅ Exists, likely needs baseline refresh after token rename |
| **SHELL-04** | `registry/defaults.ts` has `'app-shell'` and `'surface-mount'` keys; components resolve via `getComponent` | unit (TS) | `cd frontend && npm test -- --run registry` | ✅ Registry test exists; add new-key assertions |
| **SHELL-04** | Backend `AppShell` builder creates a `Component { type: "app-shell", props: {sidebarNodeId, ...} }` | unit (Rust) | `cd backend && cargo test -p marionette app_shell_builder` | ❌ Wave 0 — new `backend/crates/marionette/src/builders/app_shell.rs` with inline tests |
| **SHELL-04** | Backend `SurfaceMount` builder creates a `Component { type: "surface-mount", props: {name: "content"} }` | unit (Rust) | `cargo test -p marionette surface_mount_builder` | ❌ Wave 0 — add to `standard.rs` tests |
| **End-to-end** | CRM nav renders inside AppShell; clicking nav item updates `content` sub-surface; no page reload; shell persists | E2E | `cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/` | ✅ E2E harness exists; extend `integration.spec.ts` with AppShell assertions |
| **End-to-end demo** | Country select change triggers patch that swaps sibling form fields; unrelated focused field retains focus | E2E | `cd frontend && npx playwright test --config playwright.e2e.config.ts -g "country field swap"` | ❌ Wave 0 — new E2E test; requires backend handler change in contact form |

### Sampling Rate
- **Per task commit:** `cargo check -p marionette-protocol` (for protocol crate changes) OR `cd frontend && npm run check && npm test -- --run <affected>` (frontend changes). Fast feedback — under 15s.
- **Per wave merge:** Full `cargo test` + full `npm test` + targeted browser test run for the wave's components. Under 2 min.
- **Phase gate:** `cargo test` + `npm test -- --run` + browser test suite + Playwright E2E against `:3001`. Before `/gsd-verify-work`. Under 5 min.

### Wave 0 Gaps (files that need to exist before executors run)

- [ ] `backend/crates/marionette-protocol/src/data.rs` — REWRITE (tagged enum) + rewrite inline `#[cfg(test)] mod tests` with round-trip tests for each of 6 variants
- [ ] `backend/crates/marionette-protocol/src/messages.rs` — update `PatchMessage` struct to add `surface: Surface`; update inline tests
- [ ] `backend/crates/marionette/src/builders/app_shell.rs` — NEW FILE; hand-written AppShell builder with inline tests
- [ ] `backend/crates/marionette/src/builders/standard.rs` — ADD `SurfaceMount` struct with `#[derive(ComponentBuilder)]` + inline test
- [ ] `backend/crates/marionette/src/builders/mod.rs` — add `pub mod app_shell; pub use app_shell::*;`
- [ ] `frontend/src/lib/store/surfaces.svelte.ts` — REWRITE with fine-grained API
- [ ] `frontend/src/lib/store/surfaces.svelte.test.ts` — NEW FILE; unit tests for `setNode`/`deleteNode`/`setChildren`/`insertChild`/`removeChild`/`gcOrphans`
- [ ] `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` — NEW FILE; the canonical D-A6 proof test
- [ ] `frontend/src/lib/components/shell/` — NEW DIRECTORY
- [ ] `frontend/src/lib/components/shell/AppShell.svelte` — NEW FILE
- [ ] `frontend/src/lib/components/shell/AppShell.browser-test.ts` — NEW FILE
- [ ] `frontend/src/lib/components/core/SurfaceMount.svelte` — NEW FILE
- [ ] `frontend/src/lib/components/core/SurfaceMount.browser-test.ts` — NEW FILE (verify it mounts Surface correctly)
- [ ] `frontend/tests/e2e/shell-nav.spec.ts` — NEW FILE (end-to-end shell rendering + nav)
- [ ] `frontend/tests/e2e/node-patch-focus.spec.ts` — NEW FILE (country-field-swap demo + focus assertion against real backend)
- [ ] Delete: `frontend/src/lib/components/core/ConnectionBanner.svelte` + `.browser-test.ts`

### Framework install commands (if needed)
- Shadcn sidebar: `cd frontend && npx shadcn-svelte@latest add sidebar`
- Shadcn toast (verify first): `cd frontend && ls src/lib/components/ui/ | grep -i toast` then install if missing

## Sources

### Primary (HIGH confidence)
- **Codebase direct reads** (VERIFIED):
  - `backend/crates/marionette-protocol/src/data.rs` — current `PatchOperation` shape
  - `backend/crates/marionette-protocol/src/messages.rs` — current `PatchMessage` + existing `ProtocolMessage` tagged-enum pattern
  - `backend/crates/marionette-protocol/src/component.rs` — `Component` struct
  - `backend/crates/marionette/src/builders/standard.rs` — existing derive-macro builders
  - `backend/crates/marionette-macros/src/component_builder.rs` — `#[derive(ComponentBuilder)]` expansion, including `build_tree()`
  - `backend/crates/crm-demo/src/main.rs` — `handle_navigate` (the site of the nav-layer migration)
  - `backend/crates/crm-demo/src/handlers/*.rs` — 8 handlers rendering to `"main"`
  - `frontend/src/lib/init.ts` — the `PatchMessage` bug at line 47
  - `frontend/src/lib/store/surfaces.svelte.ts` — current `setSurfaceTree` (the wholesale-replacement bug)
  - `frontend/src/lib/store/data.svelte.ts` — current `applyPatch`
  - `frontend/src/lib/store/dirty.svelte.ts` — existing dirty-path queue (interacts with focus)
  - `frontend/src/lib/components/core/Surface.svelte`, `NodeRenderer.svelte` — recursion path + keyed each block
  - `frontend/src/lib/components/form/TextInput.svelte`, `TextInput.browser-test.ts` — browser test pattern
  - `frontend/src/lib/components/popup/ModalSurface.svelte` — existing modal-surface hardcoding (informs the transition)
  - `frontend/src/lib/registry/defaults.ts`, `icons.ts` — component registration pattern
  - `frontend/src/app.css` — current (WRONG) `--sidebar-background` tokens
  - `frontend/tests/helpers/schema-validator.ts` — ajv + js-yaml schema loader (the mechanism for schema tests)
  - `frontend/tests/e2e/protocol-conformance.spec.ts` — wire schema conformance test harness
  - `spec/schemas/data.yaml`, `message.yaml`, `common.yaml`, `component.yaml` — current schemas
  - `spec/openapi.yaml` — entry point
  - `spec/PROTOCOL.md §Messages > patch` (lines 159-192), `§Protocol Versioning` (lines 719-736)
  - `.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md` — locked decisions
- **Svelte 5 official docs** (CITED):
  - https://svelte.dev/docs/svelte/$state — deep reactive proxies, per-key subscriptions
  - https://svelte.dev/docs/svelte/each — keyed each blocks preserve DOM identity on reorder
- **shadcn-svelte official docs** (CITED):
  - https://www.shadcn-svelte.com/docs/components/sidebar — Sidebar sub-components, CSS variables, responsive behavior, `useSidebar()` hook
- **Project documentation** (VERIFIED):
  - `.planning/codebase/TESTING.md` — vitest-browser-svelte + Playwright patterns
  - `.planning/codebase/CONVENTIONS.md` — Svelte 5 conventions, Rust clippy pedantic
  - `.planning/codebase/STACK.md` — dependency versions
  - `.planning/REQUIREMENTS.md` — PATCH-01..03, SHELL-01..04
  - `.planning/research/STACK.md` §AppShell — shadcn-svelte Sidebar as the correct primitive

### Secondary (MEDIUM confidence)
- clippy large_enum_variant lint behavior — CITED: rust-lang.github.io/rust-clippy/master/#large_enum_variant (general Rust knowledge)
- Ajv OpenAPI 3.1 discriminator support — ASSUMED from existing schema-validator being `strict: false` and the project already running these tests successfully against 1.0.0 messages

### Tertiary (LOW confidence)
- None for this research. Every substantive claim is either verified from the codebase or cited to an official documentation page.

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — all dependencies verified in `package.json`/`Cargo.toml`
- Architecture: **HIGH** — all new-code patterns verified against existing codebase idioms; Svelte 5 reactivity cited from official docs
- Serde enum layout: **HIGH** — matches existing `ProtocolMessage` pattern in the same crate
- shadcn Sidebar API: **HIGH** — fetched directly from current official docs
- CSS token mismatch: **HIGH** — verified `app.css` has `--sidebar-background`; docs confirm shadcn uses `--sidebar`
- CRM handler surface migration: **HIGH** — verified 8 sites via grep
- Focus preservation mechanism: **HIGH** — verified the bug location (`setSurfaceTree` wholesale replacement) and the fix pattern (in-place mutation on the reactive proxy, backed by the keyed each block already in NodeRenderer)
- Browser test pattern: **HIGH** — verified against existing `TextInput.browser-test.ts`
- Pitfalls: **HIGH** — each is backed by a verified codebase site or cited documentation

**Research date:** 2026-04-10
**Valid until:** 2026-05-10 (30 days — dependencies are stable v1.x, shadcn-svelte updates are backwards-compatible at the registry level)
