# Phase 17: Gallery Crate Skeleton + Colocated Built-in Demos - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-22
**Phase:** 17-gallery-crate-skeleton-colocated-built-in-demos
**Areas discussed:** Demo content density, Sweep scope & coverage rules, Nav composition & landing page, Stateful fixtures (now or later)

---

## Gray Area Selection

**Question:** Which areas do you want to discuss for Phase 17 (Gallery Crate Skeleton + Colocated Built-in Demos)?

| Option | Description | Selected |
|--------|-------------|----------|
| Demo content density | How much does each `gallery_demo()` show? Minimal / rich / typical-usage. Ripples into Phase 18. | ✓ |
| Sweep scope & coverage rules | Which of the ~20 ComponentBuilder structs get a demo? All / visible-standalone / curated. | ✓ |
| Nav composition & landing page | Flat vs grouped; what the gallery shows before any click. | ✓ |
| Stateful fixtures: now or later | Real seeds in Phase 17 vs stubs now / Phase 18 handlers add fixtures. | ✓ |

**User's choice:** All four areas.

---

## Area 1: Demo content density

### Q1: What does each `gallery_demo()` emit?

| Option | Description | Selected |
|--------|-------------|----------|
| Canonical + composite (hybrid) | Leaves single canonical, composites substantive mini-compositions via nested calls. Phase 18 owns variant matrices. | ✓ |
| Comprehensive variant matrix | Every demo bakes in full variant × size × state. Phase 18 thin. Maintenance surface doubles. | |
| Typical-usage showroom | Opinionated representative instance per component. Editorializes the aesthetic. | |

**User's choice:** Canonical + composite (hybrid).

### Q2: Leaf demo richness within "canonical" bucket

| Option | Description | Selected |
|--------|-------------|----------|
| 2–3 representative instances in a Container | Default + disabled + destructive stacked. Meaningful when clicked directly; leaves variant-matrix to Phase 18. | ✓ |
| Single default instance | Strictly one Button, one TextInput. Austere; underwhelming as a gallery experience. | |
| All primary variants, no size/state | All 5 Button variants, default size. Pushes toward variant-matrix territory. | |

**User's choice:** 2–3 representative instances in a Container.

### Q3: Actions & bindings in pure-fn demos

| Option | Description | Selected |
|--------|-------------|----------|
| Decorative — no actions, no bindings | Inputs unbound, Buttons label-only. Strictest pure-fn; demos don't exercise the data store. | |
| Canonical action names with no-op handlers | Demos fire real actions in a `gallery-demo/*` namespace; gallery-demo registers no-ops. Feels alive. | ✓ |
| Real actions with state fixtures | Demo fns emit real actions; gallery-demo handlers ship fixtures. Overlaps heavily with Area 4. | |

**User's choice:** Canonical action names with no-op handlers.

### Q4: Composite composition — nested or inlined?

| Option | Description | Selected |
|--------|-------------|----------|
| Strictly nested via gallery_demo() calls | AppShell::gallery_demo calls SideNav::gallery_demo etc. Matches DEMO-02 verbatim. | |
| Inline-constructed slot content | AppShell builds slot content via direct builder calls. Decouples composites from leaves. | |
| Mixed — nest where ergonomic, inline where not | Per-composite planner call. | |

**User's choice:** "Other" (free-text) — "the appshell demo should be consciously designed regarding the content and nav bar sections, and not some automatic amalgamation … since the possibilities for content are too numerous."

**Interpretation recorded as D-A2:** AppShell's demo is hand-designed with deliberate sidebar/main content choices; other composites (Form, FieldSet, ConfirmDialog body) still follow DEMO-02 nested-call pattern where the leaf-demo shape fits.

---

## Area 2: Sweep scope & coverage rules

### Q1: Which builders must get a `#[gallery_demo]`?

| Option | Description | Selected |
|--------|-------------|----------|
| Every ComponentBuilder (including structural) | All ~20 structs get demos, even SurfaceMount / NavGroup / FieldSeparator. Maximum coverage; some contrived. | |
| Visible-standalone only, structural skipped with documented rationale | Skip SurfaceMount / NavItem / NavGroup / FieldSeparator; visible ones each get a demo; structural exercised transitively. | ✓ |
| Curated "meaningful demos" list | Planner picks per-component. Weakens auto-discovery promise; drift risk. | |

**User's choice:** Visible-standalone only, structural skipped with documented rationale.

### Q2: Which edge cases to skip? (multiSelect)

| Option | Description | Selected |
|--------|-------------|----------|
| SideNav | Outside an AppShell context looks wrong. Demoed transitively via AppShell::gallery_demo. | ✓ |
| Container | Empty renders nothing; "Container wrapping Text" is indistinguishable from the Text demo. | ✓ |
| Grid | Same rationale as Container. | |
| Keep all three — skip only the four structural ones | Maximum exhaustiveness. | |

**User's choice:** SideNav + Container (Grid KEEPS a demo).

**Final skip list:** SurfaceMount, NavItem, NavGroup, FieldSeparator (structural) + SideNav, Container (context-dependent) + TableColumn (not a component). Grid stays in scope.

### Q3: File placement for gallery_demo fns

| Option | Description | Selected |
|--------|-------------|----------|
| Same file, below the builder | standard.rs grows from ~700 → ~1100 lines. Same-screen colocated. | |
| Dedicated gallery_demos.rs | One file; demo drifts from builder code. | |
| Per-component file refactor | Break up standard.rs into button.rs, text_input.rs, etc. | ✓ |

**User's choice:** Per-component file refactor.

### Q4: Refactor edges

| Option | Description | Selected |
|--------|-------------|----------|
| Every ComponentBuilder struct — one file each | All ~24 files created; related props-structs colocated with owner; public API preserved via re-exports. | ✓ |
| Group related components into family files | form.rs, layout.rs, nav.rs, feedback.rs, actions.rs, data_table.rs. | |
| Only refactor components that ship a demo | Mixed state — half builders in files, half still in standard.rs. Inconsistent. | |

**User's choice:** Every ComponentBuilder struct — one file each.

---

## Area 3: Nav composition & landing page

### Q1: Flat vs grouped nav

| Option | Description | Selected |
|--------|-------------|----------|
| Flat alphabetical (by key) | registered_demos() ordering. Zero metadata extension. Grouping deferred if needed. | ✓ |
| Grouped via DemoEntry extension | Add `group: Option<&'static str>`. Expands Phase 16 shape. | |
| Grouped via hand-curated list in gallery-demo | Hard-code category assignments. Undermines auto-discovery promise. | |

**User's choice:** Flat alphabetical (by key).

### Q2: First-visit landing behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Curated Home page (Heading + Text + links) | Welcome + explanation + grid of demo tiles. Intentional first impression. | ✓ |
| Auto-redirect to first demo alphabetically | "button" is first. Abrupt; not representative. | |
| Empty main with stub message | "Select a demo from the sidebar". Unloved. | |

**User's choice:** Curated Home page.

### Q3: Nav click → render mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Single gallery-show action with key payload | Handler looks up registered_demos().find(), invokes render, emits Render message. Auto-extensible. | ✓ |
| Per-demo action names registered dynamically | Each demo gets its own action name at startup. Extra boilerplate. | |
| Client-side routing with initial-state hydration | URL hash determines demo; SPA reads hash. More moving parts. | |

**User's choice:** Single gallery-show action with key payload.

### Q4: No-op handler wiring strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Single gallery-demo/noop action that all demos point to | One handler emits a toast naming the source. | ✓ |
| Conventional per-action names enumerated at startup | More specific toast messages; small list to maintain. | |
| Extend ActionRouter with fallback-handler capability | Scope creep; silent-failure risk. | |

**User's choice:** Single gallery-demo/noop action that all demos point to.

**Softened by Area 4, Q2:** Modal and ConfirmDialog need purpose-built actions (modal-open/close, confirm-open/accept/reject). The `noop` action covers leaf-demo components; modals get their own registered actions within the `gallery-demo/*` namespace.

---

## Area 4: Stateful fixtures (now or later)

### Q1: Seed data scope for state-dependent demos

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal per-demo seeds (the "feels alive" bar) | 5–10 DataTable rows, 2–3 bound form fields, open Modal body, toast on nav. Phase 18 scales up. | ✓ |
| Zero fixtures — demos render structurally only | Empty DataTable headers; unbound inputs; Phase 18 handlers ship every fixture. | |
| Rich fixtures (CAT-ready data) | 500+ rows, full form state. Blurs Phase 17/18 boundary. | |

**User's choice:** Minimal per-demo seeds.

### Q2a: Bind-path declaration style (bundled question)

| Option | Description | Selected |
|--------|-------------|----------|
| Convention: `/demo/{key}/...` paths in demo fns | gallery-show handler seeds `/demo/{key}` scope per-demo match arm. | ✓ |
| No bindings in demo fns | Inputs accept typing locally but values aren't in the SDUI store. | |
| Ad-hoc bind paths per demo | Each fn picks whatever feels natural. Drift risk. | |

**User's choice:** Convention `/demo/{key}/...`.

### Q2b: Modal and ConfirmDialog open-state (bundled question)

| Option | Description | Selected |
|--------|-------------|----------|
| Statically rendered in open visual state | Modal returns with open=true; users SEE without needing to trigger. | |
| Trigger button + closed modal (real behavior) | Trigger fires action; opens via surface patch. Demonstrates real usage. | ✓ |
| Both — trigger button AND statically-open mini modal | Side-by-side idle + active. Resembles a CAT screen. | |

**User's choice:** Trigger button + closed modal (real behavior).

---

## Close-out

**Question:** We've captured density, sweep scope, nav composition, and stateful fixtures. Anything still unclear?

| Option | Description | Selected |
|--------|-------------|----------|
| I'm ready for context | Write CONTEXT.md and hand off to /gsd-plan-phase 17. | ✓ |
| Explore more gray areas | Surface additional decisions before planning. | |
| Revisit an earlier answer | Reconsider a locked decision. | |

**User's choice:** Ready for context.

---

## Claude's Discretion

Recorded in CONTEXT.md §Claude's Discretion:
- Gallery-demo port number (probable: 3002)
- `GALLERY-DEMOS.md` exact location
- `standard.rs` disposition (retire vs shim)
- Related props-struct placement (colocate vs shared `types.rs`)
- Home page tile rendering shape
- Whether Home tiles are registry-derived or hand-authored (or both)
- Noop-handler toast message wording
- `#[gallery_demo(name = "...")]` overrides for edge display names
- DataTable synthetic-data generator sharing with Phase 18/19
- Action-registration boilerplate style
- Modal/ConfirmDialog sub-surface targeting
- Deep-link URL hash bootstrap
- Whether `gallery-smoke` Cargo.toml needs adjustment

## Deferred Ideas

Recorded in CONTEXT.md §Deferred Ideas:
- DemoEntry grouping field (Phase 18/19 follow-up if nav becomes unwieldy)
- GALLERY-LINT (v1.3+)
- Deep-link URL hash handling (scope-flex on top of D-C3)
- Shared synthetic-data generator across Phases 17/18/19
- ActionRouter fallback/wildcard capability (rejected)
- Framework-level composite combinator machinery (rejected)
- Screenshot/documentation auto-generation (v1.3+)
- Theme editor (Phase 20, THEME-01)
- Third-party crate demo registration
- Standalone demos for skipped structural/context-dependent components
- Noop handler toast message richness
