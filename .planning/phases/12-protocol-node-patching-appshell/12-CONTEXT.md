# Phase 12: Protocol Node Patching + AppShell - Context

**Gathered:** 2026-04-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 12 delivers two coupled things, in order:

**Part A — Protocol: Node Patching (prerequisite).** Extend the OpenSDUI protocol so `PatchMessage` can mutate the component tree incrementally (`set-node`, `delete-node`, `set-children`, `insert-child`, `remove-child`) alongside data operations, in one atomic per-surface batch. This closes the contradiction between `CONCEPT.md` line 66 ("Easy to patch — update one node by ID") and the implemented data-only `PatchMessage`. Also adds a `surface` field to `PatchMessage`, which fixes a pre-existing latent bug in `frontend/src/lib/init.ts:47` where every patch is hardcoded to `main`. Protocol version bumps to `1.1.0` via `HelloMessage`. Frontend surface store and dispatcher are extended to apply node patches reactively while preserving focus on unrelated fields.

**Part B — AppShell (built on Part A).** Implement the AppShell as a normal first-class SDUI component — no special protocol powers. AppShell uses the shadcn Sidebar composable for visual primitives. It establishes the application frame with named slots for `sidebar`, `header`, `footer`, and three dynamic sub-surface mounts (`content`, `popups`, `toasts`) via a new general `surface-mount` SDUI component type. The CRM demo's `routes/+layout.svelte` collapses to `<Surface name="main"/>` and mounts AppShell as the root. Phase 12's CRM scope is "Minimal + nav" — mount AppShell, migrate the nav/routing layer, and demonstrate at least one end-to-end flow that exercises node-level tree mutation (e.g., a select change swapping a field in place). Per-screen CRUD cleanup is deferred to Phase 15.

**What this phase is NOT:**
- Not a CRM rewrite — Phase 15 owns per-screen cleanup and the Flowbite residue audit
- Not a rewrite of the orphan `FormScreen`/`TableScreen` Svelte files in `frontend/src/lib/components/screen/` — Phases 13/14 own those
- Not persistent sidebar state, breadcrumbs, multiple sidebar variants — those are v2 (`SHELL-05..07`)
- Not a stacked-modal feature — one modal at a time in the `modal` sub-surface for v1.1

</domain>

<decisions>
## Implementation Decisions

### Protocol: PatchOperation Shape

- **D-A1:** **Tagged enum with `op` discriminator.** `PatchOperation` becomes a Rust enum (and JSON Schema `oneOf` tagged union) serialized via `#[serde(tag = "op")]`. A single `patch` array on `PatchMessage` holds both data operations and node-tree operations in declared order. Frontend applies them sequentially within the message — all or nothing, no reordering. This is a breaking change to the existing `{path, value}` shape, and that is fine: pre-deployment, no backward compatibility required.
- **D-A2:** **Five node operations.** `set-node {id, component}`, `delete-node {id}`, `set-children {id, children}`, `insert-child {parent, index, childId}`, `remove-child {parent, childId}`. Plus the existing data `set {path, value}` op, now explicitly discriminated. The child-sugar ops (`insert-child`, `remove-child`) make common cases like "append a nav item" one-op messages instead of requiring the server to recompute a full children array.

### Protocol: Surface Targeting

- **D-A3:** **Add `surface: Surface` field to `PatchMessage`.** Required on every patch. Mirrors `RenderMessage.surface` exactly. One message targets exactly one surface; ops apply in declared order, atomic per-surface. This is not only necessary for node patches (since node IDs are per-surface, not global) — it also fixes a pre-existing latent bug in `frontend/src/lib/init.ts:44-47` which currently hardcodes every incoming `PatchMessage` to the `main` surface regardless of intent, making it impossible to patch data into `sidebar` / `modal` / `toast` surfaces. Per-op surface fields and globally-unique node IDs were rejected: the former is over-designed for the stated use cases; the latter pushes an invariant onto every ID generator and doesn't solve the data-side ambiguity.

### Protocol: Root Immutability

- **D-A4:** **`root` is immutable per `Render`.** Once a `RenderMessage` sets a surface's `root` pointer, it stays constant until the next `RenderMessage`. Node patches can replace the component **at** the root ID (via `set-node` on the root ID) or mutate its children (via `set-children` / `insert-child` / `remove-child`), but cannot re-point `root` to a different ID. No `set-root` op. Top-level transitions (login → shell, error → recovery) use a full `Render` because they change the whole tree anyway.

### Protocol: Version and Focus Preservation

- **D-A5:** **Protocol version bumps to `1.1.0`.** Additive change — new ops, new `surface` field on `PatchMessage`. `HelloMessage.version` field is already wired for this per `spec/PROTOCOL.md §Protocol Versioning`.
- **D-A6:** **Focus preservation is mandatory and proven by test.** A text-input with focus and a cursor position must retain both across arbitrary node patches targeting sibling nodes in the same surface. The surface store implementation must mutate individual node entries in place rather than replacing the parent tree object wholesale, so that Svelte 5's fine-grained reactivity scopes re-renders to only the changed nodes. A browser test exercises "user typing in field A → backend sends a patch modifying field B → field A retains focus and cursor" explicitly.
- **D-A7:** **No backward compatibility.** The protocol crate, schemas, and docs update to v1.1.0 cleanly. The existing `PatchOperation { path, value }` shape is broken; all call sites migrate. This reflects the pre-deployment posture: Marionette has no deployed base outside this repo, so root-cause fixes are preferred over compat shims.

### Protocol: Orphan Handling

- **D-A8:** **Client-side walk-and-prune garbage collection, scoped to the target surface of each patch batch.** After `PatchMessage` ops are applied to surface X, the frontend walks the tree from `surfaceState[X].root` following `children` references, marking reachable node IDs. Any node in `surfaceState[X].nodes` not in the reachable set is deleted. GC cost is O(N) in the target surface's size, not in total protocol state. Walk-and-prune is optional correctness-wise because the sub-surface architecture (D-B1) means dynamic content lives in sub-surfaces that get full `Render` replacement on navigation (automatic cleanup); GC exists as a nicety for long-lived surfaces that only receive node patches.

### AppShell: Architecture

- **D-B1:** **Sub-surface architecture via a new `surface-mount` SDUI component type.** AppShell is the root of a `main` surface. Its `main` / `popups` / `toasts` slots contain `surface-mount` nodes that mount named sub-surfaces (`content`, `modal`, `toasts`). Screen navigation = full `Render` into the `content` sub-surface → automatic orphan cleanup via `Render` semantics, zero walk-and-prune overhead for the common case. Modals and toasts live in their own sub-surfaces (`modal`, `toasts`). Sidebar / header / footer children live at the top level of the shell (`main`) surface's adjacency list as in-tree slot children, updated via node patches when their structure changes (which is rare).
- **D-B2:** **`surface-mount` is a general common building block, not an AppShell special.** It is registered in `frontend/src/lib/registry/defaults.ts` like any other component. Its backend builder lives in `backend/crates/marionette/src/builders/standard.rs` with `#[derive(ComponentBuilder)]`. Its props are minimal: `{ name: String }` — the sub-surface name to mount at its position. Its Svelte implementation simply renders `<Surface name={props.name}/>` and delegates to the existing `Surface.svelte` machinery. Future components (tabbed views, split panes, any shell-like composition) can reuse `surface-mount` without touching AppShell.
- **D-B3:** **AppShell has no special protocol powers.** It is a normal SDUI component: registered in `defaults.ts`, built by a hand-written backend builder (not derived, because slot methods have custom semantics), with slot children addressed by name in props (`props.sidebarNodeId`, `props.headerNodeId`, `props.footerNodeId`, `props.mainNodeId`, `props.popupsNodeId`, `props.toastsNodeId`). All slot children live at the top level of the shell surface's adjacency list — AppShell props only reference them by ID, they are not nested inside `props.nodes` (the orphan `FormScreen`/`TableScreen` pattern is explicitly not adopted).

### AppShell: Backend Builder Shape

- **D-B4:** **Slot methods accept pre-built node references.** Canonical shape:
  ```rust
  let (sidebar_id, sidebar_children) = SideNav::new()
      .nav_item("Companies", "/companies")
      .nav_item("Contacts", "/contacts")
      .build_with_children();
  let (header_id, header_children) = /* Container with title + user menu */;
  let (footer_id, footer_children) = /* Container with version + status */;
  let (content_mount_id, content_mount) = SurfaceMount::new("content").build();
  let (modal_mount_id, modal_mount) = SurfaceMount::new("modal").build();
  let (toast_mount_id, toast_mount) = SurfaceMount::new("toasts").build();

  let (shell_id, shell_children) = AppShell::new()
      .sidebar(sidebar_id)
      .header(header_id)
      .footer(footer_id)
      .main(content_mount_id)
      .popups(modal_mount_id)
      .toasts(toast_mount_id)
      .build_with_children();
  ```
  The AppShell builder returns the shell node + all accumulated children (transitively) for insertion into the `RenderMessage.nodes` map. Slot IDs are stored in AppShell's props. No positional `children` array by convention — every slot is explicit and named.

### AppShell: Slot Content

- **D-B5:** **Header slot content.** Three elements in the first implementation:
  1. **Sidebar trigger** — mobile hamburger button, visible only on narrow viewports. Required by the shadcn Sidebar composable for mobile sheet behavior.
  2. **App title / branding** — plain text (or small logo later). Data-bound or static per-deployment — planner's call.
  3. **User menu** — shows logged-in user name (from CRM auth data path like `/auth/currentUser`) with a dropdown containing profile / logout actions.

- **D-B6:** **Footer slot content.**
  1. **Version info** — protocol version + app version, e.g., `"Marionette v1.1 · Protocol 1.1.0"`. Sourced from a data path the backend sets once at startup.
  2. **Connection status indicator** — WebSocket connection state (connected / reconnecting / offline). The existing `ConnectionBanner` component at the top of `routes/+layout.svelte` is retired; its role moves here. Less obtrusive than a top banner, always visible, doesn't eat vertical space.
  3. **Legal / copyright text** — small `© 2026 …` or equivalent.

- **D-B7:** **Sidebar slot composition reuses existing Phase 11 components.** The sidebar slot's child is a `SideNav` containing `NavGroup` and `NavItem` children — exactly the hierarchy Phase 11 shipped. No new nav-specific component types. The existing `NavItem.svelte` already uses shadcn Button with `bg-sidebar-accent` classes, so it slots in visually.

- **D-B8:** **Sub-surface slots use `surface-mount` nodes.** The `main` slot's child is a `surface-mount` node with `name: "content"`. The `popups` slot's child is a `surface-mount` node with `name: "modal"`. The `toasts` slot's child is a `surface-mount` node with `name: "toasts"`.

### AppShell: Frontend Structural Changes

- **D-B9:** **`routes/+layout.svelte` collapses drastically.** Current:
  ```svelte
  <ConnectionBanner />
  <div class="flex h-screen">
      <Surface name="sidebar" />
      <Surface name="main" />
  </div>
  <Surface name="modal" />
  <Surface name="toast" />
  ```
  New:
  ```svelte
  <Surface name="main" />
  ```
  The `sidebar`, `modal`, and `toast` top-level surface mounts go away. Those surfaces are now mounted by AppShell via `surface-mount` nodes. `ConnectionBanner` is retired; its role is subsumed by the footer's connection status indicator.

- **D-B10:** **`AppShell.svelte` implementation uses shadcn Sidebar primitives.** Install `npx shadcn-svelte add sidebar` (plus `dialog` and `sonner` or the chosen shadcn toast primitive if not already installed). The frontend `AppShell.svelte` component wires its slots to the shadcn composable:
  - `Sidebar.Provider` wraps everything (required by shadcn for state)
  - `Sidebar.Root` + `Sidebar.Content` receives the sidebar slot child
  - A flex column holds the header slot (top), the main slot (middle, flex-1), and the footer slot (bottom)
  - The popups slot wraps in a shadcn `Dialog` overlay
  - The toasts slot wraps in a shadcn `Toaster` region
  - Header slot includes the shadcn `Sidebar.Trigger` as its mobile hamburger

- **D-B11:** **Pre-auth / login flow.** On initial connection, the backend inspects the session/auth state and Renders the `main` surface with either:
  - An unauthenticated login screen (not AppShell) — user logs in, backend sends a full `Render` of `main` replacing it with AppShell + a Render into `content` for the initial landing screen.
  - An authenticated AppShell directly, plus a content-surface Render.
  This is a full `Render` transition because the root component changes. Root immutability (D-A4) is respected — the transition goes through `Render`, not a patch.

### AppShell: CRM Integration (Phase 12 scope)

- **D-B12:** **Minimal + nav.** Phase 12 does three things in CRM:
  1. Updates `routes/+layout.svelte` to the single-`<Surface name="main"/>` form.
  2. Migrates the CRM nav/routing layer — the backend handler(s) responsible for the initial shell render and for responding to `navigate` actions. These are updated to construct AppShell (via `AppShell::new()` builder) in the `main` surface and Render screen content into the `content` sub-surface.
  3. Delivers the end-to-end node-mutation demo flow: a CRM form where changing a select value triggers a node patch that swaps a related field in place (without clobbering focus on sibling fields). Concrete candidate: a Contact form where selecting "Country" reveals country-specific extra fields via `set-children` + `set-node` patches.
  Individual per-screen CRUD handlers (audit, notes, listmonk, etc.) are **not** touched in Phase 12. They will continue rendering their trees into whichever surface gets hooked up after the nav-layer migration — the planner decides whether they render into `content` unchanged, or need a minor surface-name change. Full per-screen cleanup is Phase 15's scope.

### Nav State & Events

- **D-B13:** **Nav active state via data binding on `/nav/active/*` paths.** Exactly as demonstrated in `CONCEPT.md` line 443's worked example. `NavItem` binds to a boolean data path; a `navigate` action updates the path via a data patch (`op: "set"`). No tree mutation for active-state toggling.
- **D-B14:** **Modal close reuses existing `event { name: "close", surface: "modal" }` mechanism.** The protocol already supports this per `spec/PROTOCOL.md §event`. The frontend listens for close events targeting the `modal` sub-surface and clears its tree (sets root to empty or similar). No new message type.
- **D-B15:** **Toast lifecycle via `insert-child` / `delete-node` patches on the `toasts` sub-surface.** Backend pushes a new toast by inserting a child into the toast region's root; frontend times out and sends a dismissal action, or the backend explicitly deletes.

### Claude's Discretion

The following are not gray-area decisions — the planner has latitude to pick the cleanest implementation:

- **Exact Rust type layout for the tagged `PatchOperation` enum** — e.g., whether internal-tagged, externally-tagged, or adjacently-tagged via serde attributes; whether each variant is a struct variant or a separate struct behind `Box` to keep enum size reasonable.
- **JSON Schema shape for the `oneOf` tagged union** in `spec/schemas/data.yaml` — multiple valid formulations exist; pick the cleanest for OpenAPI 3.1 draft 2020-12.
- **Svelte 5 reactivity wiring inside the surface store** — how exactly to mutate `$state` maps such that a `set-node` on node X re-renders only the component bound to X and not its siblings. The constraint is the focus-preservation test; the implementation strategy is the planner's call.
- **Walk-and-prune GC implementation detail** — BFS vs. DFS, whether it runs synchronously in `applyPatch` or deferred via microtask, whether a visited-set is pre-allocated. Any correct implementation that meets the O(N)-per-patch bound is acceptable.
- **Exact AppShell Svelte slot composition** — which shadcn Sidebar sub-components wrap which slots, what CSS classes, how mobile hamburger visibility is driven. The shadcn docs are the authoritative reference.
- **`surface-mount` mounting semantics** — whether `Surface.svelte` handles being nested inside a `NodeRenderer`-mounted `surface-mount` node via an `onMount`/`onDestroy` registration, or via a derived reactive lookup. Planner verifies recursion works cleanly.
- **CRM contact form "swap a field on country select" demo** — exact field choice, what fields get swapped, how the backend handler computes the patch. The constraint is that the demo proves node-level mutation end-to-end with focus preservation; the concrete scenario is flexible.
- **Shadcn toast primitive choice** — `sonner` vs. `shadcn toast` — Phase 11 already picked shadcn Toast (per `11-CONTEXT.md` D-04). Honor that choice and install accordingly if not already installed.
- **Ordering of plans within the phase** — whether to land Part A (protocol) end-to-end before starting Part B (AppShell), or to interleave. Either works; the planner sequences based on dependency tightness.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents (researcher, planner, executors) MUST read these before planning or implementing.**

### Protocol Specification (authoritative — will be modified)

- `CONCEPT.md` — vision document; line 66 currently promises "easy to patch — update one node by ID" which this phase reconciles with reality. Update Section 3 (Messages) to include node-patch semantics.
- `spec/PROTOCOL.md` §Messages > patch (lines ~159-192) — authoritative description of `PatchMessage` semantics; must be extended with the tagged-enum shape, node operations, and `surface` field.
- `spec/PROTOCOL.md` §Surfaces (lines ~617-717) — describes per-surface independent trees; defines the invariant that PatchMessage.surface aligns with.
- `spec/PROTOCOL.md` §Protocol Versioning (lines ~719-736) — wire the version bump to 1.1.0.
- `spec/openapi.yaml` — OpenAPI 3.1 entry point; references schema files below.
- `spec/schemas/data.yaml` — defines `PatchOperation`; must be rewritten as a tagged `oneOf` with 6 variants (1 data op + 5 node ops).
- `spec/schemas/message.yaml` — defines `PatchMessage`; add required `surface` field.
- `spec/schemas/component.yaml` — `Component` schema (referenced by `set-node` op); no changes expected but needs inspection.
- `spec/schemas/common.yaml` — `Surface`, `MessageId` primitives.

### Backend Protocol Crate (will be modified)

- `backend/crates/marionette-protocol/src/data.rs` — `PatchOperation` struct; becomes a tagged enum.
- `backend/crates/marionette-protocol/src/messages.rs` — `PatchMessage`; add `surface` field.
- `backend/crates/marionette-protocol/src/common.rs` — `Surface` type alias.
- `backend/crates/marionette-protocol/src/component.rs` — `Component` struct.
- `backend/crates/marionette-protocol/src/lib.rs` — crate re-exports.

### Backend Toolkit Crate (will be modified)

- `backend/crates/marionette/src/builders/standard.rs` — existing 18 component builders; `AppShell` and `SurfaceMount` builders will be added here (SurfaceMount via `#[derive(ComponentBuilder)]`, AppShell hand-written because of slot semantics).
- `backend/crates/marionette/src/builders/mod.rs` — re-exports; update.
- `backend/crates/marionette/src/builders/node.rs` — `Node` type alias and `node_id` helper.
- `backend/crates/marionette-macros/src/component_builder.rs` — derive macro; consult if AppShell can use it or needs hand-writing.

### Backend CRM Handlers (partially modified — nav layer only)

- `backend/crates/crm-demo/src/handlers/mod.rs` — handler registry.
- `backend/crates/crm-demo/src/handlers/auth.rs` — initial `main` surface Render (login or post-auth shell); this is where AppShell gets constructed after login.
- `backend/crates/crm-demo/src/main.rs` — startup; may need to change initial handler wiring.
- Other CRM handlers (`company.rs`, `contact.rs`, `interaction.rs`, etc.) — scope-check only; Phase 12 does NOT rewrite these. The "nav layer" migration may require them to render into `content` instead of `main`, which is likely a one-line change per handler.

### Frontend Transport & Stores (will be modified)

- `frontend/src/lib/transport/messages.ts` — TypeScript types for protocol messages; update for new `PatchOperation` variants and `PatchMessage.surface` field.
- `frontend/src/lib/transport/dispatcher.ts` — message dispatch plumbing.
- `frontend/src/lib/init.ts` lines 44-52 — current patch handler hardcodes `applyPatch('main', msg.patch)`; must be updated to route to `msg.surface` and dispatch both data ops and node ops.
- `frontend/src/lib/store/data.svelte.ts` — current `applyPatch(surface, operations)` only handles `{path, value}`; extend to dispatch on the tagged union and route node ops to the surface store.
- `frontend/src/lib/store/surfaces.svelte.ts` — current `setSurfaceTree(surface, root, nodes)` replaces the tree object wholesale; must be extended with fine-grained mutation APIs (`setNode`, `deleteNode`, `setChildren`, `insertChild`, `removeChild`, `gcOrphans`) that mutate entries in place to preserve Svelte 5 reactivity scoping.
- `frontend/src/lib/store/dirty.svelte.ts` — current dirty-field tracking; ensure it still works under the new patch shape.

### Frontend Components (will be modified + added)

- `frontend/src/lib/components/core/Surface.svelte` — surface mount point; verify it handles being nested inside a `NodeRenderer`-mounted `surface-mount` node.
- `frontend/src/lib/components/core/NodeRenderer.svelte` — node walker; must dispatch to `surface-mount` nodes cleanly.
- `frontend/src/lib/components/core/FallbackComponent.svelte` — fallback for unknown component types.
- `frontend/src/lib/components/core/ErrorBoundary.svelte` — surface-level error boundary.
- `frontend/src/lib/components/nav/SideNav.svelte` — existing; no changes expected, will be used as sidebar slot child.
- `frontend/src/lib/components/nav/NavGroup.svelte` — existing; no changes expected.
- `frontend/src/lib/components/nav/NavItem.svelte` — existing; no changes expected.
- `frontend/src/lib/components/ui/` — shadcn primitives; `sidebar`, `dialog`, and toast primitives need `npx shadcn-svelte add` if not yet installed.
- `frontend/src/lib/components/ConnectionBanner.svelte` — currently at the top of `routes/+layout.svelte`; **retired** in this phase, replaced by the footer connection-status indicator.
- `frontend/src/lib/components/screen/FormScreen.svelte` — orphan file; **out of scope**, Phases 13/14 own rewriting.
- `frontend/src/lib/components/screen/TableScreen.svelte` — orphan file; **out of scope**.
- **NEW**: `frontend/src/lib/components/shell/AppShell.svelte` — new file, wires shadcn Sidebar composable.
- **NEW**: `frontend/src/lib/components/core/SurfaceMount.svelte` — new file, renders `<Surface name={props.name}/>` at its tree position.

### Frontend Registry & Routes (will be modified)

- `frontend/src/lib/registry/defaults.ts` — register new `app-shell` and `surface-mount` component types.
- `frontend/src/lib/index.ts` — barrel file re-exports.
- `frontend/src/routes/+layout.svelte` — collapses to a single `<Surface name="main"/>` mount (D-B9).
- `frontend/src/app.css` — already has `--sidebar-*` tokens from Phase 10; verify shadcn Sidebar's expected token names match exactly.

### Requirements & Roadmap

- `.planning/REQUIREMENTS.md` §Protocol Extension — `PATCH-01`, `PATCH-02`, `PATCH-03` (new, mapped to Phase 12).
- `.planning/REQUIREMENTS.md` §AppShell — `SHELL-01`, `SHELL-02`, `SHELL-03`, `SHELL-04` (updated wording for "normal first-class SDUI component").
- `.planning/ROADMAP.md` Phase 12 section (lines 74-88) — updated goal and 8 success criteria covering both parts.

### Prior Phase Context

- `.planning/phases/10-foundation/10-CONTEXT.md` — Default shadcn base style, 0.25rem radius, Zinc theme, CSS variables mode. All carry forward.
- `.planning/phases/11-leaf-component-migration/11-CONTEXT.md` — "Compose from shadcn parts" philosophy (D-03), shadcn Toast primitive chosen over Sonner (D-04), dynamic lucide icon registry (D-05). All carry forward.

### Codebase Maps

- `.planning/codebase/CONVENTIONS.md` — Svelte 5 component patterns, SDUI interface contract, tabs/single-quotes/100-char.
- `.planning/codebase/STACK.md` — current tech stack.
- `.planning/codebase/STRUCTURE.md` — repo layout.
- `.planning/codebase/TESTING.md` — Vitest unit, Playwright component, Playwright E2E conventions; focus-preservation test will live under the component-test harness.
- `.planning/codebase/ARCHITECTURE.md` — high-level architecture.

### Research

- `.planning/research/STACK.md` §AppShell → shadcn-svelte Sidebar (lines 65-72) — confirms shadcn Sidebar composable (Provider / Root / Header / Content / Footer / Trigger) is the right visual primitive. Built-in responsive behavior, dedicated `--sidebar-*` tokens.
- `.planning/research/PITFALLS.md` — Flowbite-to-shadcn migration pitfalls (may still apply in edge cases).
- `.planning/research/ARCHITECTURE.md` — architecture research.
- `.planning/research/FEATURES.md` — feature research.
- `.planning/research/SUMMARY.md` — research synthesis.

### External References

- shadcn-svelte Sidebar docs — canonical source for `Sidebar.Provider` / `Sidebar.Root` / `Sidebar.Trigger` / responsive behavior / `--sidebar-*` tokens.
- shadcn-svelte Dialog docs — for the modal overlay primitive.
- shadcn-svelte Toast (Radix) docs — for the toast primitive (Phase 11 D-04 picked Radix Toast, not Sonner).
- RFC 6901 JSON Pointer — for data-op paths.
- OpenAPI 3.1 / JSON Schema draft 2020-12 — authoritative spec for the schema updates.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (no changes expected)

- **`frontend/src/lib/components/nav/SideNav.svelte`** — minimal wrapper (`<nav>` with tailwind classes, renders children). Will be the direct child of AppShell's sidebar slot.
- **`frontend/src/lib/components/nav/NavGroup.svelte`** — group container for nav items.
- **`frontend/src/lib/components/nav/NavItem.svelte`** — uses shadcn Button with `bg-sidebar-accent` active classes, binds to data path for active state, dispatches navigate actions. Already compatible with the new shell architecture.
- **`frontend/src/app.css`** — `--sidebar-*` CSS variable tokens already defined for both light and dark modes (lines 29-35 and 58-65). `@theme inline` mapping at line 87+. Phase 12 does NOT rewrite app.css.
- **`backend/crates/marionette-macros`** — `#[derive(ComponentBuilder)]` works for plain components with a flat prop list. `SurfaceMount` can use it. AppShell probably cannot — slot methods have custom semantics that need hand-writing.
- **`frontend/src/lib/store/data.svelte.ts`** — already per-surface (`const surfaces: Record<string, { data }> = $state({})`). `applyPatch(surface, operations)` API exists and needs extending for node ops.
- **`frontend/src/lib/store/surfaces.svelte.ts`** — already per-surface tree storage (`surfaceState: Record<string, SurfaceTree>`). `setSurfaceTree` replaces the tree object wholesale (wrong for node patches). Needs fine-grained mutation API.
- **`frontend/src/lib/store/dirty.svelte.ts`** — dirty-field tracking for focus preservation already exists; ensure it continues to work with node patches (which may patch form fields).

### Established Patterns

- **All SDUI components accept `surface`, `props`, `bind?`, `action?`, `children?`** — AppShell and SurfaceMount will both honor this contract. SurfaceMount's `children?` will be unused; AppShell's `children?` is unused too (slot children are referenced via props by ID).
- **Builders return `(id, Component)` tuples via `.build()`**; containers use `.build_with_children()` returning `Vec<(id, Component)>` for insertion into the nodes map. AppShell follows the latter pattern.
- **Component types are strings, registered in `defaults.ts`** via `registerAll({...})`. AppShell registers as `'app-shell'`; SurfaceMount registers as `'surface-mount'`.
- **Svelte 5 runes throughout** — `$state`, `$derived`, `$props()`. The surface store must use `$state` in a way that mutates individual map entries rather than replacing whole objects (for focus preservation).
- **`frontend/src/lib/components/core/Surface.svelte`** reads `getSurfaceTree(name)` via `$derived` and renders `<NodeRenderer nodeId={tree.root} nodes={tree.nodes} surface={name}/>`. Nested Surface mounting must not cause double-mount loops.
- **Tabs for indentation, single quotes, 100-char print width** — project conventions.

### Current Patch Handler (the bug fix)

`frontend/src/lib/init.ts:44-52`:
```ts
registerHandler('patch', (raw: unknown) => {
    const msg = raw as PatchMessage;
    // Apply patch to main surface (protocol lacks surface field on patch messages)
    applyPatch('main', msg.patch);
    if (msg.id) confirmOptimistic(msg.id);
});
```

The comment on line 46 is the admission of the gap we're closing. After D-A3, this becomes:
```ts
registerHandler('patch', (raw: unknown) => {
    const msg = raw as PatchMessage;
    applyPatch(msg.surface, msg.patch);  // routes by target surface
    if (msg.id) confirmOptimistic(msg.id);
});
```

And `applyPatch` internally dispatches on `op` to route data vs. node operations.

### Current Top-Level Layout (the structural simplification)

`frontend/src/routes/+layout.svelte`:
```svelte
<script>
    import '../app.css';
    import { ConnectionBanner, Surface } from '$lib';
    let { children } = $props();
</script>

<ConnectionBanner />
<div class="flex h-screen">
    <Surface name="sidebar" />
    <Surface name="main" />
</div>
<Surface name="modal" />
<Surface name="toast" />
{@render children()}
```

Becomes:
```svelte
<script>
    import '../app.css';
    import { Surface } from '$lib';
    let { children } = $props();
</script>

<Surface name="main" />
{@render children()}
```

`ConnectionBanner` is no longer imported (retired). Sub-surfaces (`content`, `modal`, `toasts`) are mounted via `surface-mount` nodes from inside AppShell, not at the top level.

### Integration Points

- `frontend/src/lib/index.ts` — barrel file; add AppShell and SurfaceMount exports.
- `frontend/src/lib/registry/defaults.ts` — add `'app-shell': AppShell` and `'surface-mount': SurfaceMount` to `registerAll`.
- `backend/crates/marionette/src/builders/mod.rs` — ensure AppShell and SurfaceMount are re-exported.
- `backend/crates/crm-demo/src/handlers/auth.rs` or similar — where the initial shell Render lives post-auth.

### Components To Be Added

- `frontend/src/lib/components/shell/AppShell.svelte` (new directory `shell/`)
- `frontend/src/lib/components/shell/AppShell.browser-test.ts`
- `frontend/src/lib/components/core/SurfaceMount.svelte`
- `frontend/src/lib/components/core/SurfaceMount.browser-test.ts`
- `backend/crates/marionette/src/builders/app_shell.rs` — or inline into `standard.rs` / a new `shell.rs` file; planner decides
- Browser tests for focus preservation under node patches
- Protocol crate tests for new `PatchOperation` variants and `PatchMessage.surface` field

### Components To Be Removed / Retired

- `frontend/src/lib/components/ConnectionBanner.svelte` — retired (functionality moves to footer connection-status indicator)
- Any usages of `ConnectionBanner` in `routes/+layout.svelte` — removed
- The `sidebar`, `modal`, `toast` top-level Surface mounts in `routes/+layout.svelte` — removed

### Shadcn Primitives To Install

Verified `frontend/src/lib/components/ui/` currently has: badge, button, card, checkbox, dialog, input, label, select, separator, skeleton, table. Still needed for Phase 12:

- `sidebar` — **required**, not yet installed
- `dialog` — **already installed** (from Phase 11)
- shadcn Toast (Radix) — verify status; install if missing

Install with `npx shadcn-svelte add sidebar` (and others as needed) early in the phase.

</code_context>

<specifics>
## Specific Ideas

### User's AppShell Vision (verbatim from discussion)

> "My idea of the appshell is/was that it would be the starting point of an application, setting down its primary design and structure. Navbar, header, footer, popups, toasts … places for all these elements to appear … the content is updated via patching … and because of automatic garbage collection orphan nodes get killed automatically."

This is the anchor: **AppShell is the application frame**. Every primary UI region (sidebar/navbar, header, footer, main content, popups, toasts) has a place in AppShell. Nothing primary lives above it at the `routes/+layout.svelte` level.

### User's Protocol Gap Discovery (verbatim)

> "Well maybe we have found a shortcoming in CONCEPT.md and spec/PROTOCOL.md. I would like the server to be able to add a new form (for example) to the appshell on the fly, or add new fields to a form or switch them out as a selectbox is switched in the form."

The three use cases that drove node patching into Phase 12:
1. Add a new form to the AppShell on the fly
2. Add new fields to an existing form
3. Swap fields in a form driven by a select (without clobbering focus on sibling fields)

Use case 3 is the strongest — it can't be cleanly worked around with full re-renders because it would lose focus/cursor state on sibling fields being edited. **This is why focus preservation (D-A6) is non-negotiable and must be proven by test.**

### User's Pre-Deployment Posture (verbatim)

> "We are still in the full dev phase despite having completed the first cycle … no deployed base, no backward compat necessary. Lets redo the planning for 12 and include the new core feature and then build the appshell on top."

Fix the protocol cleanly. No migration shims. Bump version. Delete what's in the way.

### Concrete End-to-End Demo Scenario

Success criterion 8 requires "at least one interactive flow demonstrates node-level mutation end-to-end." Candidate: **Contact form with country-dependent fields**.

- User opens a Contact create/edit form via the CRM navigation.
- Form includes `Country` select field. No extra fields visible initially.
- User types their name in the `Name` field (cursor at position 4).
- User changes the country select to "Switzerland".
- Backend responds with a `PatchMessage` containing:
  - `insert-child` on the form's children array to add `ch-canton` and `ch-postal-code` field nodes
  - `set-node` entries defining those new field components
  - Possibly `delete-node` on previously-visible country-specific fields if the country changed from another value
- Frontend applies the patch; the two new fields appear below the country select.
- **Critical assertion**: the `Name` field retains focus and cursor position at character 4. The user continues typing without interruption.

The focus-preservation browser test encodes exactly this scenario (or a minimal version of it).

### Orphan GC Example

After many rounds of node patches on a long-lived shell surface, some nodes may become unreachable. GC walks from root:

```
Before GC:  nodes = {shell-1, header-1, sidebar-1, footer-1, content-mount-1,
                     modal-mount-1, toast-mount-1, orphan-x, orphan-y}
Walk from root=shell-1:
  shell-1 → [header-1, sidebar-1, footer-1, content-mount-1, modal-mount-1, toast-mount-1]
  header-1 → [title-1, user-menu-1]
  ... (none reference orphan-x or orphan-y)
Reachable = {shell-1, header-1, sidebar-1, footer-1, content-mount-1, modal-mount-1,
             toast-mount-1, title-1, user-menu-1, ...}
After GC:  nodes = reachable set; orphan-x and orphan-y deleted.
```

GC is scoped to the target surface of the just-applied patch batch (D-A8).

</specifics>

<deferred>
## Deferred Ideas

### To Future Phases in v1.1

- **Per-screen CRM handler cleanup** — each CRUD handler's internal tree construction. Phase 15 (CRM Migration & Validation) owns this.
- **DataTable enhancements** that rely on node patching (filter bar, infinite scroll triggering node patches for additional rows) — Phase 13 scope.
- **FormScreen enhancements** that rely on node patching (grouped card sections, field-swap interactions) — Phase 14 scope. This phase also owns rewriting or deleting the orphan `FormScreen.svelte` / `TableScreen.svelte` files.
- **Flowbite residue audit** — Phase 15 scope.

### To v2

- **SHELL-05**: Persistent sidebar collapse state across sessions (cookie or localStorage).
- **SHELL-06**: Auto-generated breadcrumbs from navigation structure.
- **SHELL-07**: Multiple sidebar variants (floating, inset).
- **Stacked modals** — multiple modals open at once. v1.1 has one modal at a time in the `modal` sub-surface.
- **Cross-surface atomic transactions** — a wire message that atomically updates multiple surfaces. No current use case; node patches are per-surface.

### Noted but Out of Phase 12 Scope

- **Orphan file cleanup** for `frontend/src/lib/components/screen/FormScreen.svelte` and `TableScreen.svelte` — they use a nested `props.nodes` pattern explicitly NOT adopted by AppShell's builder. Phases 13/14 rewrite them.
- **Additional SDUI component types** inspired by the `surface-mount` pattern (e.g., tabbed-view, split-pane) — future phases can add them cheaply now that `surface-mount` exists as a primitive.
- **Walk-and-prune GC optimization** (visited-set pre-allocation, microtask deferral, incremental marking) — planning-time detail, not a phase-level decision.
- **Protocol version negotiation / upgrade UX** — `HelloMessage.version` mismatch handling is already specified in `PROTOCOL.md §Stale Client Handling`; no new work required.

</deferred>

---

*Phase: 12-protocol-node-patching-appshell*
*Context gathered: 2026-04-10*
