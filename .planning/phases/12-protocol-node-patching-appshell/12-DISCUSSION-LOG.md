# Phase 12: Protocol Node Patching + AppShell - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `12-CONTEXT.md` — this log preserves the alternatives considered
> and the reasoning that led to each selection.

**Date:** 2026-04-10
**Phase:** 12-protocol-node-patching-appshell
**Areas discussed:** Scope rescope, PatchOperation shape, Operation set, Surface targeting, Root mutability, Surface architecture, Header content, Footer content, CRM integration scope, Sub-surface mechanism

---

## Phase Rescope (pre-existing condition)

The phase originally covered only AppShell (`SHELL-01..04`). During discussion, the user's question "doesn't the update call let us switch out aspects of the appshell component?" triggered a protocol audit. The result:

- `CONCEPT.md` line 66 claims "Easy to patch — update one node by ID" as a benefit of the adjacency list pattern
- The actual protocol (`PatchMessage` in `spec/PROTOCOL.md` and `marionette-protocol::PatchMessage`) only supports data patches `{path, value}`
- The only way to mutate the component tree today is a full `RenderMessage` which replaces the entire surface

User's response: "We are still in the full dev phase despite having completed the first cycle … no deployed base, no backward compat necessary. Lets redo the planning for 12 and include the new core feature and then build the appshell on top."

**Outcome:** Phase 12 rescoped to include a protocol extension (node patching) as a prerequisite for AppShell. Roadmap and requirements updated (commit `8cfea91`) before CONTEXT was written. New requirements `PATCH-01..03` added.

---

## Area A — Protocol: Node Patching

### A1 — PatchOperation shape

| Option | Description | Selected |
|--------|-------------|----------|
| Tagged enum, one array | `PatchOperation` becomes a tagged enum (`op` discriminator). Single `patch` array holds data + node ops in declared order. Breaking change to current `{path, value}` shape. | ✓ |
| Parallel arrays on PatchMessage | Keep current `PatchOperation` for data. Add separate `node_patch` array to `PatchMessage`. Preserves data-only consumers but requires spec on array-ordering semantics. | |
| New NodePatchMessage type | Separate message type. Two messages = no atomicity across them by default. | |

**User's choice:** Tagged enum, one array.
**Rationale:** Cleanest single pipe; atomicity implicit; no cross-array ordering spec needed; breaking change is free in pre-deployment posture.

### A2 — Operation set

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal (3 ops) | `set-node`, `delete-node`, `set-children`. Server computes new children list for any structural change. | |
| Minimal + child sugar (5 ops) | Above plus `insert-child`, `remove-child`. Makes "append a nav entry" a one-op message. | ✓ |

**User's choice:** Minimal + child sugar (5 ops).
**Rationale:** DX for the common "append a nav item" / "remove a toast" cases; server doesn't have to recompute full children arrays for incremental changes.

### A3 — Surface targeting on PatchMessage

Additional context surfaced during discussion: `frontend/src/lib/init.ts:44-47` currently contains the comment `// Apply patch to main surface (protocol lacks surface field on patch messages)` and hardcodes `applyPatch('main', msg.patch)`. This is a pre-existing latent bug — the data store is already per-surface on the frontend, but the dispatcher can only route to `main` because the protocol doesn't carry the target.

| Option | Description | Selected |
|--------|-------------|----------|
| Add `surface` to PatchMessage | One message = one surface. Mirrors `RenderMessage`. Fixes init.ts:47 hardcode-to-main bug as a side effect. | ✓ |
| Per-op surface field | Each `PatchOperation` variant carries its own surface. Cross-surface atomicity in one message. Repetitive wire format; no current use case. | |
| Globally unique node IDs | Require every node ID globally unique across surfaces. Invariant pushed onto every ID generator; silent clobber risk; doesn't solve the data-side ambiguity. | |

**User's choice:** Add `surface` to PatchMessage.
**Rationale:** Smallest change, fixes a known bug, symmetric with `RenderMessage`. Neither alternative buys anything for the stated use cases (all three in-surface). User initially wanted to clarify before deciding; chose after seeing the init.ts:47 evidence.

### A4 — Root pointer mutability

Walkthrough of all scenarios showed every real mutation of the root node is covered by `set-node` / `set-children` / `insert-child` / `remove-child` targeting the existing root ID. The only thing a `set-root` op would enable is "root has a different ID now" — no observable benefit, adds edge cases (new root ID not yet in nodes map).

| Option | Description | Selected |
|--------|-------------|----------|
| Root is immutable per Render | `root` set by `RenderMessage`, frozen until next `RenderMessage`. Node patches mutate root's component in place via `set-node`. Top-level transitions (login → shell) use full `Render`. | ✓ |
| Add `set-root` op | Sixth op. Re-points root. No walkthrough scenario benefits. | |

**User's choice:** Root is immutable per Render.
**Rationale:** Simplest invariant; no YAGNI; none of the stated use cases need set-root. User asked for more detail before deciding; chose after the scenario walkthrough.

---

## Area B — AppShell

### B1 — Surface architecture (first round)

**First round options presented:**

| Option | Description | Selected |
|--------|-------------|----------|
| SDUI component (single tree) | AppShell as a real registered SDUI component. One AppShell node in a surface owns its slots. | (initially) |
| Multi-surface frame | AppShell as structural frame in `routes/+layout.svelte` with named slot mount points. | |
| Hybrid | AppShell as component with sub-surfaces inside. | |

**User's feedback mid-question:** "Doesn't the update call let us switch out aspects of the appshell component, even though the appshell is a first class component? I would like for appshell not to be something special but a common building block and we could have other top-level builds too."

**Outcome of round 1:** User's feedback reshaped the entire question — AppShell should be a normal common building block, not special. This also surfaced the protocol gap (see Phase Rescope above). Round 1 options were withdrawn; round 2 reframed after rescope.

### B1 — Surface architecture (second round, post-rescope)

| Option | Description | Selected |
|--------|-------------|----------|
| Split — shell + content via surface-mount | AppShell lives in `main` surface. Main slot is a `surface-mount {name: "content"}` component. CRM handlers render into `content`. Screen nav = full Render of content surface → automatic orphan cleanup. New `surface-mount` primitive is reusable. | |
| Single — everything in one main surface | AppShell is root of `main`. Header, sidebar, footer, all screen content are nodes in one tree. Screen nav = node patches. Walk-and-prune GC for orphans. One mental model. | ✓ (initially) |
| Split with rename (shell + content) | Same mechanics as split, different naming. | |

**User's choice (round 2):** Single — everything in one main surface.
**User's notes:** "My idea of the appshell is/was that it would be the starting point of an application, setting down its primary design and structure. Navbar, header, footer, popups, toasts … places for all these elements to appear … the content is updated via patching … and because of automatic garbage collection orphan nodes get killed automatically."
**Rationale (round 2):** User's vision is AppShell as the application frame with all primary regions as slots. Content updated via patches. GC handles orphans automatically.

### B1 — Surface architecture (third round, after follow-up question)

Mid-discussion the user asked: **"Could the shell declare new surfaces inside itself?"**

This re-opened B1 with a clearer articulation of the split option. Extensive analysis presented: sub-surfaces would give automatic cleanup via Render semantics (no walk-and-prune needed for the common case), zero coupling between CRM handlers and shell structure, and `surface-mount` becomes a reusable primitive for other shell-like patterns. Only real downside is no cross-surface atomic transactions (which no current handler needs).

| Option | Description | Selected |
|--------|-------------|----------|
| Flip to sub-surfaces via surface-mount | Add new SDUI component type `surface-mount`. AppShell's main/popups/toasts slots contain surface-mount nodes mounting `content`, `modal`, `toasts` sub-surfaces. Screen navigation = full Render of content surface. AppShell has zero special powers. | ✓ |
| Stay with single-surface + walk-and-prune GC | Keep the earlier round-2 decision. | |
| Hybrid — content sub-surface only | Only the main content is a sub-surface. Modal and toast stay in-tree. | |

**User's choice (final):** Flip to sub-surfaces via surface-mount.
**Rationale:** The walk-and-prune GC is a correctness liability; sub-surfaces get automatic cleanup for free. CRM handler migration becomes a one-line surface-name change. `surface-mount` is a real reusable primitive aligned with the "common building block" philosophy.

### B1 GC scope refinement

After B1 flipped, user added: **"Since the patch always targets a single surface the orphan cleanup can be limited to the surface targeted by the patch."**

Captured as D-A8: walk-and-prune GC is scoped to the target surface of each patch batch, O(N) in surface size, not in total protocol state. This aligns with A3 (per-message surface targeting) naturally. GC becomes cheap-and-rare rather than cheap-and-frequent, because sub-surface Renders handle bulk cleanup.

### B2 — Backend builder shape

Locked in without a discussion turn — obvious given the "common building block" philosophy and the existing `(id, Component)` tuple pattern used by `Container`, `SideNav`, etc. AppShell slot methods accept child node IDs from other builders' `.build()` outputs; the AppShell builder's `build_with_children()` returns the shell node plus all accumulated slot children for insertion into the `RenderMessage.nodes` map.

### B3a — Header slot content

| Option | Description | Selected |
|--------|-------------|----------|
| Sidebar trigger (mobile hamburger) | Required by shadcn Sidebar composable for mobile sheet. Hard to skip — mobile users can't open nav without it. | ✓ |
| App title / branding | Static or data-bound app name. | ✓ |
| User menu (name + logout) | Logged-in user with dropdown (profile/logout). Needed for CRM auth flow. | ✓ |
| Connection status indicator | Small badge showing WebSocket state. Moved to footer instead. | |

**User's choice:** Sidebar trigger + App title/branding + User menu.
**Rationale:** Connection status goes in the footer instead — less prominent when everything is fine.

### B3b — Footer slot content

| Option | Description | Selected |
|--------|-------------|----------|
| Version info | Protocol + app version, static data path. | ✓ |
| Connection status (alternative to header) | WebSocket state indicator. | ✓ |
| Keep footer minimal / empty by default | Slot present, empty unless filled. | |
| Legal / copyright text | "© 2026 …" small text. | ✓ |

**User's choice:** Version info + Connection status + Legal/copyright text.
**Rationale:** Footer is the right location for always-visible meta information; `ConnectionBanner` component at the top of `routes/+layout.svelte` is retired, its role moving here.

### B4 — Sidebar composition

Locked in without a discussion turn — reuse existing `SideNav` / `NavGroup` / `NavItem` (Phase 11 output) as children of AppShell's sidebar slot. Consistent with "common building block" philosophy; no new nav-specific component types needed.

### B5 — CRM integration scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal — one screen proves it works | Mount AppShell, update one CRM handler for the demo flow, include end-to-end node-mutation scenario. Phase 15 finishes CRM migration. | |
| Thorough — all CRM handlers in Phase 12 | Every handler updated to the new pattern. Phase 12 grows significantly. | |
| Minimal + nav | Phase 12 migrates the nav/routing layer (sidebar contents, top-level navigation) so every screen reaches the right handler. Individual CRUD screens keep their current tree construction; Phase 15 does per-screen cleanup. | ✓ |

**User's choice:** Minimal + nav.
**Rationale:** Keeps Phase 12 scope bounded while ensuring the shell architecture is actually exercised end-to-end. Phase 15 has a clearly defined scope for the rest.

---

## Items Locked In Without Explicit Discussion

The following were either obvious from prior-phase context, mandated by the rescope, or raised as "flagged but not a gray area" during the discussion:

- **Protocol version bumps to `1.1.0`** (mandated by rescope; `HelloMessage.version` already wired)
- **Focus preservation is mandatory and proven by automated test** (non-negotiable consequence of adopting node patching for the select-swaps-field use case)
- **No backward compatibility** (pre-deployment posture, saved as durable memory)
- **Carrying forward from Phase 10**: Zinc/Neutral theme, 0.25rem radius, CSS variables mode, `--sidebar-*` tokens already defined in `app.css`
- **Carrying forward from Phase 11**: "Compose from shadcn parts" philosophy, shadcn Toast primitive (not Sonner) per Phase 11 D-04, dynamic lucide icon registry
- **shadcn Sidebar composable** (Provider/Root/Header/Content/Footer/Trigger) is the visual primitive — confirmed by `.planning/research/STACK.md` §AppShell
- **Modal close via existing `event { name: "close", surface: "modal" }`** — already in protocol, no new message type
- **Toast lifecycle via `insert-child` / `delete-node` patches on the `toasts` sub-surface**
- **Nav active state via `bind` on `/nav/active/*` data paths** — exactly as CONCEPT.md Example Flow demonstrates
- **`routes/+layout.svelte` collapses to a single `<Surface name="main"/>`** — all other top-level surface mounts retire

---

## Claude's Discretion (Planner Has Latitude)

- Exact Rust layout for the tagged `PatchOperation` enum (serde tagging style, whether variants are boxed)
- Exact JSON Schema `oneOf` shape for the tagged union in `spec/schemas/data.yaml`
- Svelte 5 reactivity wiring inside the surface store (how exactly to mutate `$state` maps for fine-grained re-renders)
- Walk-and-prune GC implementation (BFS vs. DFS, sync vs. microtask, visited-set allocation)
- Exact AppShell Svelte slot composition (which shadcn Sidebar sub-components wrap which slots)
- `surface-mount` mounting semantics (onMount/onDestroy registration vs. derived reactive lookup)
- CRM contact form "swap a field on country select" demo — exact field choice and handler logic
- Plan ordering within the phase (protocol end-to-end first vs. interleaved with AppShell)

---

## Deferred Ideas (Noted for Later)

- Per-screen CRM handler cleanup — Phase 15
- DataTable enhancements that leverage node patching (filter bar, infinite scroll) — Phase 13
- FormScreen enhancements that leverage node patching — Phase 14 (also owns rewriting the orphan `FormScreen.svelte` / `TableScreen.svelte` files)
- Flowbite residue audit — Phase 15
- `SHELL-05` persistent sidebar collapse — v2
- `SHELL-06` auto-generated breadcrumbs — v2
- `SHELL-07` multiple sidebar variants (floating, inset) — v2
- Stacked modals (multiple modals open at once) — future
- Cross-surface atomic transactions — no current use case
- Walk-and-prune GC optimization heuristics — planning-time detail
- Additional SDUI component types inspired by `surface-mount` pattern (tabbed-view, split-pane, etc.) — future phases can add them cheaply
