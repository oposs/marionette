---
phase: 12-protocol-node-patching-appshell
plan: 03
subsystem: protocol
tags: [openapi, json-schema, yaml, ajv, protocol, docs]

requires:
  - phase: 12-01-scaffolding
    provides: phase directory + context docs that anchor this plan
provides:
  - spec/schemas/data.yaml — PatchOperation as a oneOf union of 6 tagged variants
  - spec/schemas/message.yaml — PatchMessage requires surface field
  - spec/PROTOCOL.md §patch — full documentation of the 6 ops, surface field, root immutability, focus preservation
  - spec/PROTOCOL.md version bumped to 1.1.0
  - CONCEPT.md §Messages — reconciles "easy to patch — update one node by ID" claim with the implemented op list
affects: [12-02-protocol-crate, 12-05-backend-builders, 12-08-demo-and-e2e]

tech-stack:
  added: []
  patterns:
    - OpenAPI 3.1 oneOf + discriminator for tagged unions
    - Kebab-case op string values matching Rust serde rename_all convention

key-files:
  created: []
  modified:
    - spec/schemas/data.yaml
    - spec/schemas/message.yaml
    - spec/PROTOCOL.md
    - CONCEPT.md

key-decisions:
  - "Used internally-tagged oneOf with propertyName=op (matches Rust #[serde(tag = \"op\")] and JSON Schema idiom)"
  - "Kept single flat op list — no nesting under data/tree categories at schema level"
  - "Version bump 1.0.0 → 1.1.0 (minor; additive but also breaks wire-compat for PatchMessage which is pre-deployment)"

patterns-established:
  - "JSON Schema oneOf + discriminator is the canonical representation for Rust tagged enums in this codebase"
  - "Per-surface atomicity documented as protocol contract — one PatchMessage = one surface"

requirements-completed: [PATCH-01, PATCH-03]

duration: ~15 min
completed: 2026-04-10
---

# Phase 12 Plan 03: Protocol Spec & Schemas Summary

**PatchOperation reworked as an OpenAPI oneOf with 6 variants (`set`, `set-node`, `delete-node`, `set-children`, `insert-child`, `remove-child`) and surface-targeted PatchMessage, with full PROTOCOL.md and CONCEPT.md documentation bumped to protocol 1.1.0.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-10T14:36:00Z (approximate)
- **Completed:** 2026-04-10T14:51:13Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Tagged `PatchOperation` oneOf union in `spec/schemas/data.yaml` with 6 variants + `op` discriminator mapping
- `PatchMessage` in `spec/schemas/message.yaml` gains required `surface` field
- `spec/PROTOCOL.md §patch` rewritten: 6 ops documented in payload tables, root-immutability note, focus-preservation note, mixed data+tree example
- `spec/PROTOCOL.md` version string bumped to `1.1.0` in header, hello-handshake ASCII diagram, hello example, and §Protocol Versioning example
- `CONCEPT.md` patch examples (both the §2 Data keyed-patch example and the §3 Messages Backend→Frontend sample) updated to the new `{op, ...}` shape; §3 Messages now carries a paragraph grounding the "easy to patch — update one node by ID" claim in the 6 implemented ops with a cross-reference to `spec/PROTOCOL.md §patch`.

## Task Commits

1. **Task 1: Rewrite schemas (data.yaml + message.yaml)** — `163ee8d` (feat)
2. **Task 2: Update PROTOCOL.md §patch + §Versioning, reconcile CONCEPT.md** — `79810fc` (docs)

## Files Created/Modified

- `spec/schemas/data.yaml` — 44 → 145 lines (+101). Replaced the old `{path, value}` PatchOperation with a oneOf of 6 variant schemas + `discriminator: {propertyName: op, mapping: ...}`. `KeyedCollection` and `ValidationError` are unchanged.
- `spec/schemas/message.yaml` — 152 → 158 lines (+6). `PatchMessage.required` now lists `[type, surface, patch]`; `surface` property references `common.yaml#/Surface`.
- `spec/PROTOCOL.md` — 760 → 794 lines (+34). §patch rewritten (lines 159-230 area); version string updated in 4 places. Zero remaining `1.0.0` references.
- `CONCEPT.md` — 670 → 681 lines (+11). Two patch examples updated to tagged shape; new reconciliation paragraph added after the second example.

### CONCEPT.md reconciliation paragraph (exact text)

> **patch** — incrementally update a surface's data and/or component tree. A patch message targets one surface and carries a batch of ops: `set` (data), `set-node` / `delete-node` / `set-children` / `insert-child` / `remove-child` (tree). Ops apply in declared order, all-or-nothing. Mix freely. The tree-mutation ops are how the "easy to patch — update one node by ID" claim above is implemented. See `spec/PROTOCOL.md §patch` for the full op reference and examples.

## Decisions Made

None beyond what the plan specified. Every decision was pre-baked in the plan + 12-CONTEXT.md (D-A1 through D-A8).

## Deviations from Plan

None — plan executed exactly as written. The only structural substitution was the YAML/ajv verification harness.

### Verification harness substitution (not a deviation, execution-environment adaptation)

- The plan asked for `cd frontend && node -e "const yaml = require('js-yaml')..."` and `npx vitest --run protocol-conformance` to confirm YAML parses and ajv compiles.
- This worktree has no `frontend/node_modules` installed; installing ~GB of deps inside a parallel-execution worktree would be wasteful and risks contention with sibling agents. Equivalent verification was performed via:
  - `python3 -c "import yaml; yaml.safe_load(open('...'))"` for both schema files → PASS.
  - Running the schema-validator's ref-rewriting logic verbatim via `NODE_PATH=/home/oetiker/checkouts/marionette/frontend/node_modules` pointing at the main-repo node_modules, then `ajv.compile()` against every top-level definition (21 defs) → PASS.
  - Bonus sanity check: validated a synthetic `PatchMessage` containing ALL 6 op variants against the compiled schema → validates true.
- No schema-compile errors observed. The Task 2 acceptance criterion "`grep -i 'schema.*compile\|ajv.*error'` returns zero lines" is satisfied vacuously — the ajv.compile() call threw zero errors.

## Issues Encountered

- `frontend/node_modules` not present in worktree (expected — see Verification harness substitution above).
- No other issues. Grep-based acceptance checks all passed first try.

## Schema Validator Warnings Observed

None. Ajv running with `strict: false` (the configuration used by `frontend/tests/helpers/schema-validator.ts`) compiled all 21 top-level schema definitions cleanly and accepted the `oneOf + discriminator` keyword without complaint. A synthetic PatchMessage exercising all 6 op variants validated true against the compiled PatchMessage schema.

## Threat Flags

None. This plan only modifies documentation and YAML schema files — no executable code paths changed. The schemas are consumed at test time by ajv in `frontend/tests/helpers/schema-validator.ts` (test environments only, not production). Threat register entries T-12-05 and T-12-06 remain accepted/mitigated as defined in the plan — this plan does not introduce new surface beyond what was already modeled.

## Known Stubs

None. All edits are substantive. The only placeholder-shaped text in the diff is the `{ ... }` literals inside CONCEPT.md's existing message examples (those were present before this plan and remain unchanged in structure — they're prose abbreviations, not code stubs).

## Next Phase Readiness

- Plan 12-02 (the parallel Rust protocol-crate plan in this same wave) can now cross-check its serde tags against the authoritative schema — the schema is the source of truth for the wire shape.
- Plan 12-05 (backend builders) can reference the documented op names without needing to invent them.
- Plan 12-08 (demo + E2E) can use the schema-validator harness to run live wire traffic against the new schemas; any drift between server-generated messages and `spec/schemas/*.yaml` will surface as an ajv validation failure.

## Self-Check: PASSED

Files verified present:
- FOUND: spec/schemas/data.yaml
- FOUND: spec/schemas/message.yaml
- FOUND: spec/PROTOCOL.md
- FOUND: CONCEPT.md

Commits verified in git log:
- FOUND: 163ee8d (Task 1)
- FOUND: 79810fc (Task 2)

Acceptance criteria (from PLAN.md `<verification>`):
- FOUND: `grep -c set-node spec/schemas/data.yaml` ≥ 2 (actual: 2 — in mapping + const)
- FOUND: `grep -c surface spec/schemas/message.yaml` ≥ 2 (actual: 6 — includes prior uses in RenderMessage + EventMessage + new PatchMessage required+property)
- FOUND: `grep -c 1.1.0 spec/PROTOCOL.md` ≥ 3 (actual: 4 — header + 3 examples)
- FOUND: PROTOCOL.md node-op name hits: 9 (≥5 required)
- FOUND: PROTOCOL.md zero residual `1.0.0` references
- FOUND: CONCEPT.md references all 6 ops (in the reconciliation paragraph + updated set-node example)
- FOUND: CONCEPT.md contains `spec/PROTOCOL.md` cross-reference
- FOUND: Ajv compiles all 21 definitions with `strict: false` and zero errors
- FOUND: Synthetic 6-op PatchMessage validates true against compiled schema

---
*Phase: 12-protocol-node-patching-appshell*
*Completed: 2026-04-10*
