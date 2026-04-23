# Phase 18: Catalog Screens - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-23
**Phase:** 18-catalog-screens
**Areas discussed:** Registration + nav placement, Catalog ↔ leaf demo relationship, CAT-02 Forms live validation patch design, CAT-03 DataTable synthetic data + shared generator

---

## Area 1 — Registration + nav placement

### Q1.1: How should catalog screens register with the nav?

| Option | Description | Selected |
|--------|-------------|----------|
| `#[gallery_demo]` in gallery-demo (Recommended) | Catalog fns in gallery-demo crate under `src/catalog/`; auto-discovery reuses existing nav-iteration + `gallery-show` handler. | ✓ |
| `#[gallery_demo]` in marionette crate | Catalog fns colocate with leaf `gallery_demo()` fns in marionette; bloats marionette with gallery-specific screens. | |
| Hand-wired nav + custom handlers in gallery-demo | 5 screens wired by hand; parallel registration path; every new screen touches main.rs. | |
| `#[gallery_demo]` + seed_for_key extension | Same as Option 1 but with explicit `seed_for_key` extension for bulk data. | |

**Notes:** User picked recommended. Rationale: consistent with Phase 17 auto-discovery pattern.

### Q1.2: Nav grouping at ~25 flat entries?

| Option | Description | Selected |
|--------|-------------|----------|
| Keep flat alphabetical (Recommended) | 25 entries reads fine; re-open grouping in Phase 19/20 if needed. | ✓ |
| Open deferred `group` field on DemoEntry | Touches marionette + marionette-macros; scope expansion mid-milestone. | |
| Hybrid: flat now, prefix-group via display-name | Zero framework change; display-name conventions cluster entries. | |

**Notes:** User picked recommended.

### Q1.3: Demo keys naming for catalog screens?

| Option | Description | Selected |
|--------|-------------|----------|
| `catalog-<family>` prefix (Recommended) | `catalog-buttons`, `catalog-forms`, `catalog-data-table`, `catalog-feedback`, `catalog-typography`. | ✓ |
| `<family>-catalog` suffix | `buttons-catalog`, etc.; singular/plural mismatch with leaves. | |
| No prefix — family-level names | `buttons-and-actions`, `all-forms`, etc.; loses semantic grouping. | |

**Notes:** User picked recommended. Alphabetical sort clusters the 5 entries contiguously.

### Q1.4: How should catalog screens route from nav clicks?

| Option | Description | Selected |
|--------|-------------|----------|
| Fully reuse `gallery-show` (Recommended) | Zero new framework machinery; consistent with Phase 17 D-C3. | ✓ |
| Reuse `gallery-show` + dedicated catalog actions for interactive flows | Mixed approach for complex catalog interactions. | |
| Parallel `catalog-show` action | Breaks Phase 17 "single action routes every nav click" decision. | |

**Notes:** User picked recommended.

**User's choice summary for Area 1:** All four recommended options — clean sweep.

---

## Area 2 — Catalog ↔ leaf demo relationship

### Q2.1: Relationship between leaf demos and catalog screens?

| Option | Description | Selected |
|--------|-------------|----------|
| Coexist unchanged (Recommended) | Leaves stay minimal per Phase 17; catalog adds 5 new nav entries. | ✓ |
| Catalog replaces leaves for the 5 families | Retires leaf demos for covered families; structural change. | |
| Upgrade cheap leaves + add catalog for big expansions | Blurs leaf/catalog line; per-family judgment call. | |

**Notes:** User picked recommended.

### Q2.2: How should catalog fns compose their content?

| Option | Description | Selected |
|--------|-------------|----------|
| Compose fresh via builder API (Recommended) | No leaf `gallery_demo()` calls; aligns with Phase 17 §domain. | ✓ |
| Include leaf `gallery_demo()` as catalog section 1 | Violates Phase 17 rule; leaf content shifts would propagate. | |
| Catalog imports shared constants from marionette | New exports; overkill abstraction. | |

**Notes:** User picked recommended.

### Q2.3: Does Phase 18 touch existing leaf `gallery_demo()` fns?

| Option | Description | Selected |
|--------|-------------|----------|
| No changes to leaves — purely additive (Recommended) | Smallest blast radius; Phase 17 stability preserved. | ✓ |
| Allow small leaf polish during catalog work | Bundles e.g. W-06 fix; scope creep risk. | |
| Dedicated Phase 18 Plan for leaf polish | Explicit bounded polish plan. | |

**Notes:** User picked recommended. W-06 ErrorDisplay `message` field fix is deferred.

### Q2.4: File layout inside gallery-demo?

| Option | Description | Selected |
|--------|-------------|----------|
| `gallery-demo/src/catalog/<family>.rs` (Recommended) | New sub-module; scales to Phase 19 `exerciser/`. | ✓ |
| Flat in `gallery-demo/src/` | Shallower tree; crate root gets crowded. | |
| One file per catalog screen, no sub-module | Root-level files; module naming awkwardness. | |

**Notes:** User picked recommended.

**User's choice summary for Area 2:** All four recommended options — clean sweep.

---

## Area 3 — CAT-02 Forms live validation patch design

### Q3.1: Which field(s) carry the live validation demo?

| Option | Description | Selected |
|--------|-------------|----------|
| Single email field (Recommended) | One TextInput with format rule; clear demo signal. | |
| Two fields — email + required name | Doubled complexity; still doesn't exercise other inputs. | |
| One per input type (comprehensive) | 6 live-validation stories; maximally exercises framework. | ✓ |

**Notes:** User picked most-comprehensive option over my recommended. Direction: maximally exercise the framework surface.

### Q3.2: When should validation fire?

| Option | Description | Selected |
|--------|-------------|----------|
| Submit button click (Recommended) | Classic SDUI; two cycles for full story. | |
| On-blur validation | Responsive; single-pass; needs blur-action wiring. | ✓ |
| On-change (live as-you-type) | Debouncing concerns; WS chatty. | |
| Explicit demo controls (Introduce / Fix buttons) | Unrealistic UX; state-machine explicit. | |

**Notes:** User picked on-blur. Open question flagged: current builders may not emit blur actions — researcher determines whether to add `.on_blur(ComponentAction)` to affected builders.

### Q3.3: Patch shape for error clear — which exercises Phase 12 node patching best? (REFRAMED after user feedback)

**Initial framing rejected** — user callout: "you keep mentioning the crm ... this is NOT about crm but about exercising and demoing marionette". Initial Q3.3 was anchored on CRM parity; reframed around protocol-surface exposure.

| Option | Description | Selected |
|--------|-------------|----------|
| Mix all three ops across the 6 inputs (Recommended) | Assigns `set-children`, `set-node`, `delete-node` across the 6 inputs; teaches full Phase 12 surface. | ✓ |
| `set-children` on the FieldGroup across all 6 | Uniform pattern; only exercises one op. | |
| `set-node` on a dedicated per-field error node (formsnap-inspired) | Exercises one op; aligns with per-field error anatomy. | |
| `delete-node` + inverse `set-node` to introduce | Two-op symmetry; id-stability concerns. | |

**Notes:** User picked recommended after reframing.

### Q3.4: Matrix layout — mobile-first? (REFRAMED after user feedback)

**Initial framing missed mobile** — user callout: "we must make sure this works on mobile too!". Reframed with mobile-first constraint.

| Option | Description | Selected |
|--------|-------------|----------|
| Per-input Cards with responsive inner grid (Recommended) | Outer stack of Cards (mobile-native); inner `grid-cols-1 sm:grid-cols-2 lg:grid-cols-5`. | ✓ |
| Accordion per input type (shadcn Accordion) | Dense; hides content behind clicks. | |
| Responsive outer grid — input rows × state columns | True matrix at desktop; mobile loses structure. | |
| Tabs per state (state selector at top) | Very dense; only one state visible at a time. | |

**Notes:** User picked recommended after reframing.

### Q3.5 (mid-flight addition): Formsnap adoption?

User question during discussion: "are we using formsnap for our forms? we should!"

Investigation: formsnap + sveltekit-superforms NOT installed. Current stack is raw shadcn-svelte `Field` anatomy imported directly.

**User's decision (verbatim):** "I don't want to corrupt our model in the sense that we do client side validation, what I like about this form component is that they also thought a lot about how to compose forms ... so maybe instead of using it directly is that we should take it as inspiration for improving our form component".

| Option | Description | Selected |
|--------|-------------|----------|
| Adopt formsnap as dependency | Client-side validation; conflicts with server-driven model. | |
| Defer as v1.3+ investigation | Separate spike; CAT-02 stays server-side. | |
| Small spike now before planning CAT-02 | Prototype; expands Phase 18 scope. | |
| Design reference only — no dependency | Study composition patterns; improve marionette Form/Field builders. | ✓ |

**Notes:** Server-driven validation model preserved. Researcher tasked with studying formsnap composition anatomy for inspiration; planner evaluates small Form-component polish pass as a Phase 18 pre-CAT-02 plan candidate.

**User's choice summary for Area 3:** Two recommended picks rejected initially (Q3.1 picked more-comprehensive; Q3.2 picked on-blur over submit). After reframing Q3.3 and Q3.4, both recommendations accepted. Plus mid-flight formsnap discussion landed on "design reference only".

---

## Area 4 — CAT-03 DataTable synthetic data + shared generator

### Q4.1: Data shape for CAT-03's 500+ synthetic rows?

| Option | Description | Selected |
|--------|-------------|----------|
| Generic synthetic records (Recommended) | `{id, name, email, status, score, joined_at}`; domain-neutral. | ✓ |
| Contact-style (reuse CRM field shape) | Couples to CRM schema; CRM concerns leak in. | |
| Themed characters (fantasy/fictional) | Cute but adds narrative distraction. | |

**Notes:** User picked recommended.

### Q4.2: Which ColumnKinds should the demo exercise?

| Option | Description | Selected |
|--------|-------------|----------|
| All available (Text, Number, Date, Badge, Actions) (Recommended) | Exhaust the ColumnKind enum; 7 columns. | ✓ |
| Text + Badge + Actions only | Minimal set; leaves ColumnKinds undocumented. | |
| All available + column visibility toggle demoed prominently | Same + one extra interaction. | |

**Notes:** User picked recommended.

### Q4.3: Where does the 500-row generator live?

| Option | Description | Selected |
|--------|-------------|----------|
| Shared `gallery-demo/src/fixtures.rs` — param-driven (Recommended) | Serves both CAT-03 (500) and Phase 19 EXER-03 (10k). | ✓ |
| Phase-18-local helper, Phase 19 re-extracts | Minimum Phase 18 scope; re-extraction risk. | |
| Unify across Phase 17 + 18 + 19 upfront | Maximum consistency; touches Phase 17 code. | |

**Notes:** User picked recommended. Phase 17's existing 5-row helper stays untouched (aligns with D-2-C "no leaf changes").

### Q4.4: Row seeding approach?

| Option | Description | Selected |
|--------|-------------|----------|
| Virtualized fetch-rows pagination (Recommended) | Exercises Phase 13 infinite scroll end-to-end. | ✓ |
| One-shot seed of all 500 rows | Defeats virtualization; breaks at 10k. | |
| Hybrid — seed first page, stub remaining client-side | Breaks protocol story. | |

**Notes:** User picked recommended.

**User's choice summary for Area 4:** All four recommended options — clean sweep.

---

## Claude's Discretion

These areas were explicitly deferred by the user to Claude's discretion during planning/execution:

- **CAT-01 Buttons & Actions screen layout** — standard approach: per-variant (or per-size) Cards with responsive inner grid, mirroring D-3-D's CAT-02 pattern.
- **CAT-04 Feedback screen composition** — standard approach: per-surface Cards with trigger buttons; compositional popup pattern per GALLERY-DEMOS.md §Popup composition; ConfirmDialog uses the structured contract from Phase 17 17-05.
- **CAT-05 Typography & tokens composition** — 3-section Card stack: text scale, lucide icon catalog (14 registered icons minimum), OKLCH swatches (27 tokens, grouped or flat). Dark-theme preview is stretch goal.
- **Whether the Form-component polish pass (D-3-E) lands in Phase 18 or a separate plan** — researcher/planner decides scope.
- **Whether blur-action wiring requires builder changes (D-3-B) or uses existing change-dispatch** — researcher determines.
- **Column visibility toggle prominence in CAT-03** — planner's call (Option A vs Option C of Q4.2).
- **Safelist extension for `grid-cols-5` Tailwind class** — researcher confirms whether it's currently safelisted or needs adding.
- **Deterministic RNG seed choice for the shared `synthetic_rows(n)` generator** — planner picks.

---

## Deferred Ideas

Ideas mentioned during discussion that were noted for future phases:

- **W-06 ErrorDisplay `message` field dead-state fix** — deferred to follow-up plan or Phase 19 polish pass.
- **Unifying Phase 17's `seed_table_rows()` helper with the new shared generator** — deferred; Phase 18 stays purely additive.
- **Formsnap as a dependency** — explicitly rejected (client-side validation conflicts with server-driven model); design reference only.
- **Leaf-demo bind-path drift fixes** — deferred; Phase 18 is additive only.
- **Grouping metadata on `DemoEntry`** — Phase 19/20 may re-open.
- **Full lucide library search / dynamic icon browsing** — CAT-05 ships registered set minimum; expansion is v1.3+.
- **Dark-theme preview pane for CAT-05** — Phase 20's Live Token Editor is the proper home; stretch goal for CAT-05.
- **Tabs / Tooltip / Popover primitives in marionette** — not in Phase 18 scope.
- **GALLERY-LINT CI enforcement** — v1.3+.

---

*Discussion log generated: 2026-04-23*
