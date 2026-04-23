---
phase: 18-catalog-screens
plan: 05
subsystem: gallery-demo
tags: [gallery, catalog, cat-02, forms, validation, node-patch, phase-12]
requirements: [CAT-02]

requires:
  - 18-02  # Wave-1 blur dispatch on SelectInput / Checkbox / Switch / RadioGroup
  - 18-04  # catalog/ module skeleton + CAT-01 buttons sibling

provides:
  - "catalog-forms gallery demo (Catalog: Forms nav entry, renders from registered_demos)"
  - "6 blur-validate handlers — one per input type — each emitting the UI-SPEC-locked Phase 12 op mix"
  - "seed_for_key(\"catalog-forms\") arm with 36 value paths + 6 pre-seeded error messages"

affects:
  - backend/crates/gallery-demo

tech-stack:
  added: []
  patterns:
    - "Pre-mounted empty-Container error slots at stable ids — SetNode / SetChildren / DeleteNode target existing nodes (RESEARCH.md §Pitfall 4)"
    - "Hand-constructed ComponentAction { type: \"blur\" } — no .blur() builder helper yet (RESEARCH.md §Q1)"
    - "Per-Card composite builder returning (card_root_node, descendants) — shared assemble_card helper keeps the 6 Cards structurally identical"
    - "/_errors/demo/catalog-forms/<input>-value as sibling data path for server-authored error strings — value store stays under /demo/catalog-forms/<input>-value"

key-files:
  created:
    - backend/crates/gallery-demo/src/catalog/forms.rs
    - backend/crates/gallery-demo/src/handlers/catalog_forms.rs
  modified:
    - backend/crates/gallery-demo/src/catalog/mod.rs
    - backend/crates/gallery-demo/src/handlers/mod.rs
    - backend/crates/gallery-demo/src/handlers/show.rs

decisions:
  - "Error-slot Container is pre-mounted as direct child of each Card at first render; SetChildren / DeleteNode targets existing nodes only (never leaks orphan-node / ghost-parent bugs)"
  - "Select + Textarea invalid-blur handlers SetNode the slot with an ErrorDisplay whose OWN id (`catalog-forms-<input>-error`) differs from the slot id — so the valid-blur DeleteNode can target the NODE, not the slot"
  - "Checkbox + Switch use pure set-node (slot → ErrorDisplay on invalid, slot → empty Container on valid) to exercise the simplest node-patch op in the demo"
  - "ComponentAction::blur() constructor NOT added in this plan — hand-construct inline to match Plan 18-02's wiring; a dedicated helper is Phase 18 Plan 18-08 polish scope (optional)"

metrics:
  duration_min: 14
  tasks_completed: 3
  files_created: 2
  files_modified: 3
  tests_added: 21
  completed: 2026-04-23
---

# Phase 18 Plan 05: CAT-02 Forms Catalog Screen + Live-Validate Handlers Summary

CAT-02 Forms catalog screen ships the only catalog surface with backend-driven interaction: six Cards × (5-field state matrix + live-validate field) demonstrate the full Phase 12 node-tree op mix (`set-node` / `set-children` / `delete-node`) rotated across every shipped form input type.

## What Shipped

**Nav entry** `Catalog: Forms` (key `catalog-forms`) renders an outer column with H1 + intro Text + 6 per-input Cards:

| Card | State-matrix demo fields (5) | Interactive field | Error slot id |
|------|------------------------------|-------------------|---------------|
| TextInput | Normal, Disabled, With error, Focused (click me), With description | Email (type then tab out) | `catalog-forms-text-error-slot` |
| Select | Normal, Disabled, With error, Open (click me), With description | Country (required — pick one then tab out) | `catalog-forms-select-error-slot` |
| Checkbox | Normal, Checked, Disabled, With error, With description | I agree to the terms | `catalog-forms-checkbox-error-slot` |
| Switch | Off, On, Disabled, With error, With description | Enable notifications | `catalog-forms-switch-error-slot` |
| Radio Group | Normal, Selected, Disabled, With error, With description | Plan (pick one) | `catalog-forms-radio-error-slot` |
| Textarea | Normal, Disabled, With error, Focused, With description | Bio (min. 20 characters) | `catalog-forms-textarea-error-slot` |

**Locked responsive grid class** `grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3` on each Card's state-matrix Container (D-3-D).

## Per-Card Id Inventory (for Plan 18-08 UAT)

Each Card carries 6 stable ids following the `catalog-forms-<input>-<role>` convention:

| Role | Pattern | Purpose |
|------|---------|---------|
| card | `catalog-forms-<input>-card` | Container — target of SetChildren ops on invalid/valid blur |
| heading | `catalog-forms-<input>-heading` | H2 — first child of card |
| state-grid | `catalog-forms-<input>-state-grid` | Container holding the 5 state-demo fields |
| sep | `catalog-forms-<input>-sep` | FieldSeparator |
| interactive | `catalog-forms-<input>-interactive` | The live-validate field — fires blur action |
| error-slot | `catalog-forms-<input>-error-slot` | Pre-mounted empty Container — SetNode replaces component at this id |

For Select / Textarea, a seventh id appears only after the first invalid blur: the ErrorDisplay node's own id (`catalog-forms-select-error`, `catalog-forms-textarea-error`) — this is the DeleteNode target on subsequent valid blur.

Interactive fields' additional per-demo stable ids for state-grid children: `catalog-forms-<input>-{normal,disabled,with-error,focused|checked|selected|on,desc}` (the 3rd-to-last token varies by input — see `catalog/forms.rs` per-card `state_fields` vecs).

## Op-Mix Matrix — Handler → Patch Ops

Each of the 6 handlers emits the UI-SPEC §CAT-02 locked op mix. All six also write an error string to `/_errors/demo/catalog-forms/<input>-value` (empty on valid).

| Handler | Invalid-blur ops | Valid-blur ops |
|---------|------------------|-----------------|
| `validate_text_input` | SetNode(text-error-slot → ErrorDisplay) + SetChildren(text-card, +slot) + Set(error) | SetChildren(text-card, -slot) + Set("") |
| `validate_select` | SetNode(select-error-slot → ErrorDisplay id=select-error) + SetChildren(select-card, +slot) + Set(error) | DeleteNode(select-error) + Set("") |
| `validate_checkbox` | SetNode(checkbox-error-slot → ErrorDisplay) + Set(error) | SetNode(checkbox-error-slot → empty Container) + Set("") |
| `validate_switch` | SetNode(switch-error-slot → ErrorDisplay) + Set(error) | SetNode(switch-error-slot → empty Container) + Set("") |
| `validate_radio` | SetNode(radio-error-slot → ErrorDisplay) + SetChildren(radio-card, +slot) + Set(error) | SetChildren(radio-card, -slot) + Set("") |
| `validate_textarea` | SetNode(textarea-error-slot → ErrorDisplay id=textarea-error) + SetChildren(textarea-card, +slot) + Set(error) | DeleteNode(textarea-error) + Set("") |

Phase 12 op coverage across the 6 handlers:
- **SetNode** — 8 distinct occurrences across invalid paths + 2 in valid paths (checkbox/switch "revert to empty Container")
- **SetChildren** — 4 invalid + 2 valid (text + radio — both add-and-remove)
- **DeleteNode** — 2 valid (select + textarea)

## Validation Rules & Error Strings (UI-SPEC §Copywriting — server-authored only)

| Input | Rule | Error message |
|-------|------|---------------|
| text-input | value contains `@` AND `.` | `Enter a valid email address.` |
| select | value is non-empty | `Please make a selection.` |
| checkbox | value == true | `You must agree to continue.` |
| switch | value == true | `Notifications must be enabled.` |
| radio | value is non-empty | `Please pick one option.` |
| textarea | value.chars().count() >= 20 | `Bio must be at least 20 characters.` |

## Seed Table

36 value paths under `/demo/catalog-forms/` seed the state-matrix + interactive fields on first visit; 6 pre-seeded error strings under `/_errors/demo/catalog-forms/` make the "With error" state-demo cells render red on initial load. Locked verbatim to UI-SPEC §CAT-02 lines 384-429.

Hard contract enforced by `catalog_forms_seed_covers_every_bind_path_in_the_demo` test (Phase 17 G-05 lesson): every `.bind(...)` path used in `catalog/forms.rs` has a matching key in the seed arm, checked at build time.

## Test Count

21 new tests across the plan:

| File | Count | What it covers |
|------|-------|----------------|
| `handlers/catalog_forms.rs` | 13 | 12 invalid+valid paths across 6 handlers + 1 `/_errors/` prefix invariant across all handlers |
| `catalog/forms.rs` | 6 | root id, 6 card ids, 6 error-slot Containers, 6 interactive fields w/ blur actions, registered_demos entry, ≥30 bind paths under namespace |
| `handlers/show.rs` | 2 | bind-alignment with seed + pre-seeded error strings for "With error" cells |

Gallery-demo lib suite: 20 → 41 tests. All green. `cargo build --workspace --all-features` green. `cargo clippy -p gallery-demo --all-targets -- -D warnings` clean.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | `daa07db` | feat(18-05): add 6 CAT-02 blur-validate handlers with Phase 12 op mix |
| 2 | `d70c101` | feat(18-05): add CAT-02 catalog/forms.rs with 6 per-input Cards |
| 3 | `1f48ab7` | feat(18-05): seed catalog-forms state — 36 value paths + 6 pre-errors |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking issue] Field `name` on DemoEntry is actually `display_name`**
- **Found during:** Task 2 (writing catalog/forms.rs unit tests)
- **Issue:** Plan template at line 813 used `found.unwrap().name` on a `DemoEntry`. The struct field is `display_name` (see `backend/crates/marionette/src/gallery.rs:36`), so the template would not compile. Plan 18-04's `buttons.rs` test at line 230 already uses the correct name — plan 18-05 was stale.
- **Fix:** Wrote test against `entry.display_name`, matching `buttons.rs` and the actual struct.
- **Files modified:** `backend/crates/gallery-demo/src/catalog/forms.rs`
- **Commit:** `d70c101`

**2. [Rule 3 — Blocking issue] ComponentAction schema fields differ from plan stub**
- **Found during:** Task 2
- **Issue:** Plan stub at line 612 builds `ComponentAction { payload: None, optimistic: None, extra: Default::default() }`. The real struct (`backend/crates/marionette-protocol/src/component.rs:32-51`) has no `payload` or `optimistic` fields — only `r#type`, `name`, `target`, `id_path` (renamed to `idPath`), and a flattened `extra` map.
- **Fix:** Constructed the ComponentAction with the actual fields (`id_path: None`, `extra: serde_json::Map::new()`).
- **Files modified:** `backend/crates/gallery-demo/src/catalog/forms.rs`
- **Commit:** `d70c101`

**3. [Rule 3 — Blocking issue] `payload` accessor on ComponentAction in test assertions**
- **Found during:** Task 2 (self-review)
- **Issue:** Plan template test `v["props"]["action"]["type"]` assumes action sits inside props. Serialization of `Component` actually puts `bind` and `action` as TOP-LEVEL fields, not inside `props` (see `backend/crates/marionette-protocol/src/component.rs` Component struct — has sibling fields for `props`, `bind`, `action`).
- **Fix:** Tests read `v["action"]["type"]` and `v["bind"]` directly.
- **Files modified:** `backend/crates/gallery-demo/src/catalog/forms.rs`, `backend/crates/gallery-demo/src/handlers/show.rs`
- **Commits:** `d70c101`, `1f48ab7`

**4. [Rule 3 — Blocking issue] `build_with_children` does not flatten grandchildren**
- **Found during:** Task 2 (reading `component_builder.rs` macro output)
- **Issue:** Plan template at lines 638-660 called `.build_with_children()` on the outer Container expecting a flat node list, then skipped index 0 via `.into_iter().skip(1)` and extended with grid descendants separately. But `build_with_children()` only clones direct children (`self.__children`) — it does not recurse. Worse, the plan's code passes 8 `root_children` that include grandchildren-bearing tuples from each card; those grandchildren would be lost.
- **Fix:** Use `build_tree()` (returns `(root, descendants)` cleanly) on the outer Container AND on each Card's state-grid Container, then manually concat all descendant vecs. Matches the pattern in `catalog/buttons.rs` Plan 18-04.
- **Files modified:** `backend/crates/gallery-demo/src/catalog/forms.rs`
- **Commit:** `d70c101`

**5. [Rule 1 — Bug] Clippy lints (`if_not_else`, `similar_names`, `unnecessary_wraps`)**
- **Found during:** Task 1 verification (`cargo clippy -p gallery-demo --all-targets -- -D warnings`)
- **Issue:** Plan template used the `if !valid { invalid-branch } else { valid-branch }` pattern and a `patch_response` helper that returns `ActionResult`. Clippy rejected both under the crate's `-D warnings` quality bar. Also test variable `path` shadowed `patch` too closely.
- **Fix:** Flipped branches to happy-path-first; renamed test locals; added `#[allow(clippy::unnecessary_wraps)]` with a comment explaining the handler-contract parity reason.
- **Files modified:** `backend/crates/gallery-demo/src/handlers/catalog_forms.rs`
- **Commit:** `daa07db` (pre-commit clippy fix)

None of these deviations changed observable behaviour — they are purely plan-template-vs-reality corrections.

### Out-of-Scope Observations (Not Fixed)

**ErrorDisplay frontend reads an ARRAY at the bind path, plan specifies a STRING.**
- `frontend/src/lib/components/feedback/ErrorDisplay.svelte:26-41` iterates `errors as ErrorEntry[]` where `ErrorEntry = { path?: string, message: string }`. The plan's handlers write a plain string at `/_errors/demo/catalog-forms/<input>-value`. With a string value, the frontend's `{#if errors.length > 0}` guard will evaluate against `String.length` (not Array.length), which happens to render non-empty strings — but the `{#each errors as error}` loop will iterate over characters, rendering a per-character box.
- **Why not fixed here:** The plan's `<interfaces>` block explicitly locks string values at that path and the plan's acceptance tests assert strings. This is a shape mismatch between the UI-SPEC (which says "reads message from data store") and the actual frontend component. Resolving it requires either:
  - (a) handlers emit `[{message: "..."}]` array instead of string, OR
  - (b) frontend ErrorDisplay handles string-as-message fallback, OR
  - (c) a small Polish-scope Svelte edit to `ErrorDisplay.svelte` to treat a string value as `[{message: value}]`.
- **Flagged for:** Plan 18-08 Chrome MCP UAT — if the error chrome renders as per-character boxes on blur, Plan 18-08 will apply the smallest-surface fix (option c). Tracked in threat-model T-18-05-06 (aria-live attribute) as the related polish pass.

## TDD Gate Compliance

Per plan frontmatter `tdd_mode: opportunistic` (not `tdd`), separate RED/GREEN commits are not required at the plan level. Each task was committed as `feat(...)` with tests and implementation together — the tests act as spec-lock per the behavioural contract in the plan's `<behavior>` block and all 21 tests validated the first implementation pass (no RED phase was observed because the implementation was derived directly from the UI-SPEC's locked op-assignment table).

## Self-Check: PASSED

Created files exist:
- `FOUND: backend/crates/gallery-demo/src/catalog/forms.rs`
- `FOUND: backend/crates/gallery-demo/src/handlers/catalog_forms.rs`

Modified files exist:
- `FOUND: backend/crates/gallery-demo/src/catalog/mod.rs`
- `FOUND: backend/crates/gallery-demo/src/handlers/mod.rs`
- `FOUND: backend/crates/gallery-demo/src/handlers/show.rs`

Commits exist in git log:
- `FOUND: daa07db` — Task 1
- `FOUND: d70c101` — Task 2
- `FOUND: 1f48ab7` — Task 3

All plan acceptance-criteria greps confirmed: 6 validate fns, 6 action regs, 6 error-slot id refs, 36 seed paths, 6 pre-seeded errors. `cargo test -p gallery-demo` green (41 lib + 1 nav_auto_discovery + 1 smoke_boot). `cargo clippy -p gallery-demo --all-targets -- -D warnings` clean. `cargo build --workspace --all-features` green.
