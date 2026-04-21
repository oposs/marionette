---
phase: 12
plan: 03
type: execute
wave: 1
depends_on: [12-01]
files_modified:
  - spec/schemas/data.yaml
  - spec/schemas/message.yaml
  - spec/PROTOCOL.md
  - CONCEPT.md
autonomous: true
requirements: [PATCH-01, PATCH-03]
nyquist_compliant: true
tags: [protocol, spec, schemas, docs]
must_haves:
  truths:
    - "spec/schemas/data.yaml defines PatchOperation as a oneOf with 6 variants and `op` discriminator"
    - "spec/schemas/message.yaml defines PatchMessage with required `surface` field"
    - "spec/PROTOCOL.md §patch section documents the tagged-enum shape, surface field, and atomic ordering"
    - "spec/PROTOCOL.md §Protocol Versioning reflects version 1.1.0"
    - "CONCEPT.md's 'easy to patch — update one node by ID' claim is reconciled with the implemented protocol"
  artifacts:
    - path: "spec/schemas/data.yaml"
      provides: "tagged PatchOperation oneOf union"
      contains: "discriminator"
    - path: "spec/schemas/message.yaml"
      provides: "PatchMessage with required surface"
      contains: "surface"
    - path: "spec/PROTOCOL.md"
      provides: "node-patch semantics documented"
      contains: "set-node"
  key_links:
    - from: "spec/schemas/data.yaml PatchOperation oneOf"
      to: "variant schemas"
      via: "discriminator.propertyName = op"
      pattern: "propertyName:\\s*op"
---

<objective>
Update the OpenAPI schemas and protocol documentation so `PatchOperation` is a tagged `oneOf` with 6 variants, `PatchMessage` has a required `surface` field, `spec/PROTOCOL.md` documents the node-patch semantics, and `CONCEPT.md` reconciles its "easy to patch — update one node by ID" claim with the implemented protocol. Version string bumps to `1.1.0` throughout docs and schemas.

Purpose: This plan is the documentation/schema mirror of Plan 02. It runs parallel (Wave 1) because the two edit disjoint file sets — Rust sources vs. YAML/Markdown. The schema-validator test harness loads YAML at runtime, so wire conformance tests in Plan 08 depend on these schemas being accurate.

Output: Updated YAML schemas + Markdown docs that match the Rust protocol crate byte-for-byte on the wire format.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md
@.planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md
@spec/schemas/data.yaml
@spec/schemas/message.yaml
@spec/schemas/common.yaml
@spec/schemas/component.yaml
@spec/PROTOCOL.md
@CONCEPT.md
@frontend/tests/helpers/schema-validator.ts

<interfaces>
Current `spec/schemas/data.yaml` defines `PatchOperation` as a plain struct:

```yaml
PatchOperation:
  type: object
  required:
    - path
    - value
  properties:
    path:
      type: string
      format: json-pointer
      description: JSON Pointer to the data location to update
    value:
      description: New value to set at the path
  additionalProperties: false
```

Target: Rewrite as a tagged `oneOf` with 6 variant schemas, matching RESEARCH Pattern 3 verbatim.

Current `spec/schemas/message.yaml` `PatchMessage` has NO `surface` field and lists only `[type, patch]` as required. Target: add `surface` to required and to properties.

`CONCEPT.md` line 66 context (verified via bash `sed -n '55,75p'`):
- Line 66-67 says "**Why flat, not nested?**" followed by three bullets including "Easy to patch (update one node by ID)"

`spec/PROTOCOL.md §Messages > patch` (lines 159-192) currently documents ONLY the data-only patch (path/value shape). It must gain documentation for the 5 new node ops, the tagged-enum shape, and the `surface` field.

`spec/PROTOCOL.md §Protocol Versioning` (lines 719-736) and line 3 `**Version:** 1.0.0` + lines 39, 90, 725 with `"1.0.0"` strings: all must bump to `1.1.0`.

Ajv in `frontend/tests/helpers/schema-validator.ts` runs with `strict: false` and already supports OpenAPI 3.1 `oneOf + discriminator` keyword.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Rewrite spec/schemas/data.yaml PatchOperation as a tagged oneOf + update message.yaml PatchMessage</name>
  <read_first>
    - spec/schemas/data.yaml
    - spec/schemas/message.yaml
    - spec/schemas/common.yaml (Surface definition)
    - spec/schemas/component.yaml (Component reference target for set-node)
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Pattern 3 JSON Schema section
    - frontend/tests/helpers/schema-validator.ts (verify oneOf+discriminator keyword support)
  </read_first>
  <action>
1. REPLACE the current `PatchOperation` block in `spec/schemas/data.yaml` (the 10-line `type: object` struct at the top of the file) with the following tagged `oneOf` union. Do NOT remove `KeyedCollection` or `ValidationError` further down the file — they are unchanged:

```yaml
PatchOperation:
  description: >-
    A single patch operation. Data ops (`set`) update a JSON Pointer path in
    the surface's data store; node ops mutate the surface's component adjacency
    list. Operations in a `PatchMessage.patch` array are applied in declared
    order, all-or-nothing. Mix data and node ops freely in one batch.
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
      description: JSON Pointer to the data location to update
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
      description: Node ID to remove from the adjacency list
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
      description: Parent node ID whose children array is replaced
    children:
      type: array
      items:
        type: string
      description: Ordered list of child node IDs
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
      description: Parent node ID
    index:
      type: integer
      minimum: 0
      description: Zero-based insertion index
    childId:
      type: string
      description: Child node ID to insert
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
      description: Parent node ID
    childId:
      type: string
      description: Child node ID to remove
  additionalProperties: false
```

2. In `spec/schemas/message.yaml`, modify the `PatchMessage` block. Add `surface` to `required` and to `properties` between `id` and `patch`. New block:

```yaml
PatchMessage:
  type: object
  required:
    - type
    - surface
    - patch
  properties:
    type:
      type: string
      const: patch
    id:
      $ref: "common.yaml#/MessageId"
    surface:
      $ref: "common.yaml#/Surface"
      description: >-
        Target surface name. One message targets exactly one surface; ops
        apply in declared order, atomic per-surface.
    patch:
      type: array
      items:
        $ref: "data.yaml#/PatchOperation"
      description: Array of patch operations to apply in declared order
  additionalProperties: false
```

3. Validate the YAML parses: `cd frontend && node -e "const yaml = require('js-yaml'); const fs = require('fs'); yaml.load(fs.readFileSync('../spec/schemas/data.yaml', 'utf8')); yaml.load(fs.readFileSync('../spec/schemas/message.yaml', 'utf8')); console.log('OK')"`.

4. Run the schema-validator test harness via the existing protocol-conformance suite to confirm ajv accepts the new schemas (they will fail on actual Hello messages until Plan 02 ships and Plan 08 updates the E2E; for Wave 1 the acceptance is "schemas parse cleanly and ajv does not error at load time"). Run:

```bash
cd frontend && npx vitest --config vitest-browser.config.ts --run protocol-conformance 2>&1 | tail -20
```

Do NOT fix failing test cases here — that belongs to Plan 08. The acceptance is only that ajv loads the YAML without throwing schema-compile errors.

5. Do NOT touch `common.yaml` or `component.yaml` — their referenced types (`Surface`, `MessageId`, `Component`) are unchanged.
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; node -e "const yaml=require('js-yaml'); const fs=require('fs'); yaml.load(fs.readFileSync('../spec/schemas/data.yaml','utf8')); yaml.load(fs.readFileSync('../spec/schemas/message.yaml','utf8')); console.log('OK')" &amp;&amp; grep -q 'propertyName: op' ../spec/schemas/data.yaml &amp;&amp; grep -q 'set-node:' ../spec/schemas/data.yaml &amp;&amp; grep -A2 'PatchMessage:' ../spec/schemas/message.yaml | grep -q 'surface'</automated>
  </verify>
  <acceptance_criteria>
    - `js-yaml` parses `spec/schemas/data.yaml` without throwing
    - `js-yaml` parses `spec/schemas/message.yaml` without throwing
    - `grep -c 'PatchOperationSet\|PatchOperationSetNode\|PatchOperationDeleteNode\|PatchOperationSetChildren\|PatchOperationInsertChild\|PatchOperationRemoveChild' spec/schemas/data.yaml` returns at least 12 (each schema name appears in the oneOf list AND as a top-level definition)
    - `grep -q 'propertyName: op' spec/schemas/data.yaml` succeeds
    - `grep -q 'discriminator:' spec/schemas/data.yaml` succeeds
    - `grep -B1 -A10 'PatchMessage:' spec/schemas/message.yaml` shows `surface` in both `required` and `properties`
    - Loading the YAML via the existing schema-validator harness does not throw at ajv compile time (inspect output of the vitest run; accept failing test assertions, only gate on schema-compile errors)
  </acceptance_criteria>
  <done>data.yaml and message.yaml define the new PatchOperation oneOf union and PatchMessage.surface field. YAML parses. ajv compiles without errors.</done>
</task>

<task type="auto">
  <name>Task 2: Update spec/PROTOCOL.md §patch + §Protocol Versioning, and reconcile CONCEPT.md</name>
  <read_first>
    - spec/PROTOCOL.md (specifically lines 1-10 header, lines 159-192 §patch, lines 719-736 §Versioning)
    - CONCEPT.md (lines 55-75 around the "easy to patch" claim)
    - .planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md §domain (D-A1..D-A8 summaries)
  </read_first>
  <action>
1. Update `spec/PROTOCOL.md`:
   - Line 3: change `**Version:** 1.0.0` → `**Version:** 1.1.0`
   - Line 39: in the WebSocket handshake example, change `|  <--- hello { version: "1.0.0" }  |` → `|  <--- hello { version: "1.1.0" }  |`
   - Line 90: change `version: "1.0.0"` → `version: "1.1.0"` (inside the §hello example block)
   - Line 725: change `version: "1.0.0"` → `version: "1.1.0"` (inside the §Protocol Versioning example block)
   - Grep `spec/PROTOCOL.md` for any other `1.0.0` references and update them; leave any that refer to prior protocol versions in a changelog/history context, if present.

2. REWRITE the §patch subsection (lines ~159-192). New content:

```markdown
### patch

- **Direction:** Server to client
- **Purpose:** Incrementally update a surface's data and/or component tree without a full re-render
- **When to use:** When part of a surface changes — a data field, an added form row, a swapped-in sub-component — and you want to preserve focus, cursor position, and unrelated component state

A `patch` message targets exactly one surface (via the required `surface` field) and carries a batch of `PatchOperation` entries applied in declared order, all-or-nothing. Data ops and node-tree ops can be mixed freely in one batch.

**Fields:**

| Field     | Type              | Required | Description                                       |
|-----------|-------------------|----------|---------------------------------------------------|
| `type`    | string            | yes      | Always `"patch"`                                  |
| `id`      | string            | no       | Correlation ID                                    |
| `surface` | string            | yes      | Target surface name (e.g., `"main"`, `"content"`) |
| `patch`   | PatchOperation[]  | yes      | Operations applied in declared order              |

Each `PatchOperation` is a tagged object discriminated by `op`:

#### Data operations

| `op`  | Payload fields     | Effect                                               |
|-------|--------------------|------------------------------------------------------|
| `set` | `path`, `value`    | Set `value` at JSON Pointer `path` in surface data   |

#### Node tree operations

| `op`            | Payload fields                | Effect                                                                            |
|-----------------|-------------------------------|-----------------------------------------------------------------------------------|
| `set-node`      | `id`, `component`             | Replace (or create) the component at node `id` in the surface's adjacency list    |
| `delete-node`   | `id`                          | Remove the node with `id` from the adjacency list                                 |
| `set-children`  | `id`, `children: string[]`    | Replace `id`'s children array with the given ordered list of child IDs            |
| `insert-child`  | `parent`, `index`, `childId`  | Insert `childId` into `parent`'s children array at zero-based `index`             |
| `remove-child`  | `parent`, `childId`           | Remove `childId` from `parent`'s children array                                   |

**Root immutability:** A `root` pointer is immutable for the lifetime of a `Render`. Node patches can replace the component AT the root ID (via `set-node`) or mutate its children, but cannot re-point `root` to a different ID. Top-level transitions (login → shell, error → recovery) use a full `render` message instead.

**Unknown ops:** A `PatchMessage` containing an `op` not in this table is an error. Clients surface this as a stale-client prompt and should reload. Protocol version negotiation via `HelloMessage.version` is the mechanism for forward compatibility — see §Protocol Versioning and §Stale Client Handling.

**Focus preservation:** Frontends applying node patches must mutate node map entries in place rather than replacing parent tree objects wholesale. This guarantees that a focused input field retains its focus and cursor position across arbitrary patches to sibling nodes in the same surface.

**Example — data + tree ops mixed, swap a form field on country select:**

```yaml
type: patch
surface: content
patch:
  - op: set
    path: "/contact/country"
    value: "CH"
  - op: insert-child
    parent: contact-form
    index: 4
    childId: contact-canton
  - op: set-node
    id: contact-canton
    component:
      type: select
      bind: "/contact/canton"
      props:
        label: "Canton"
        options:
          - { value: "ZH", label: "Zürich" }
          - { value: "BE", label: "Bern" }
  - op: delete-node
    id: contact-us-state
```
```

3. Reconcile `CONCEPT.md`. The "Why flat, not nested?" section already contains the claim "Easy to patch (update one node by ID)". This claim is now accurate — the phase 12 protocol crate ships `set-node`. Do NOT edit the promise bullet (it is correct as-is). Instead, LOCATE §3 Messages (search for the `### 2. Data (Application State)` heading; the Messages subsection precedes it) and add a brief note about the enhanced patch semantics under the Messages heading:
   - Find the description of the three message types (render/patch/action or equivalent).
   - If the text describes `patch` as data-only, rewrite it to: "**patch** — incrementally update a surface's data and/or component tree. A patch message targets one surface and carries a batch of ops: `set` (data), `set-node` / `delete-node` / `set-children` / `insert-child` / `remove-child` (tree). Ops apply in declared order, all-or-nothing. Mix freely. See `spec/PROTOCOL.md §patch`."
   - If the text does not mention `patch` at all, or already describes the 6 ops correctly, add one sentence: "The tree-mutation ops (`set-node`, `delete-node`, `set-children`, `insert-child`, `remove-child`) are how the `patch` claim is implemented — see `spec/PROTOCOL.md §patch`."

4. After edits, grep `CONCEPT.md` for any `version 1.0` / `1.0.0` references that should bump. If none, no change.

5. Run `cd frontend && npx vitest --config vitest-browser.config.ts --run protocol-conformance 2>&1 | tail -20`. The test suite will still contain failing assertions (those are Plan 08's responsibility), but no new schema-compile errors should appear.
  </action>
  <verify>
    <automated>grep -q 'Version:\*\* 1.1.0' spec/PROTOCOL.md &amp;&amp; grep -q 'set-node' spec/PROTOCOL.md &amp;&amp; grep -q 'delete-node' spec/PROTOCOL.md &amp;&amp; grep -q 'set-children' spec/PROTOCOL.md &amp;&amp; grep -q 'insert-child' spec/PROTOCOL.md &amp;&amp; grep -q 'remove-child' spec/PROTOCOL.md &amp;&amp; ! grep -n 'version.*1\.0\.0' spec/PROTOCOL.md</automated>
  </verify>
  <acceptance_criteria>
    - `grep -n '1.0.0' spec/PROTOCOL.md` returns zero hits that represent the current protocol version (changelog references to historical 1.0.0 are OK if clearly marked; ideally no hits at all)
    - `grep -c '^\*\*Version:\*\* 1.1.0' spec/PROTOCOL.md` returns 1
    - `grep -c 'set-node\|delete-node\|set-children\|insert-child\|remove-child' spec/PROTOCOL.md` returns at least 5
    - `grep -q 'surface' spec/PROTOCOL.md` inside the §patch table (visible via `sed -n '159,250p' spec/PROTOCOL.md`)
    - The §patch section includes a worked YAML example that mixes at least one data op and at least 2 distinct node ops
    - `CONCEPT.md` references at least one of the 5 node-op names, OR its §Messages subsection explicitly points at `spec/PROTOCOL.md §patch` for the tree-mutation op list
    - `cd frontend && npx vitest --config vitest-browser.config.ts --run protocol-conformance 2>&1 | grep -i 'schema.*compile\|ajv.*error'` returns zero lines (test case failures are OK; schema load errors are not)
  </acceptance_criteria>
  <done>spec/PROTOCOL.md documents the 6 PatchOperation variants, the surface field, root immutability, and focus preservation. Version string bumped to 1.1.0 everywhere. CONCEPT.md's patch-by-node-ID claim is grounded in a reference to the new op list.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries
Documentation and schema updates only — no executable code paths changed by this plan. The YAML schemas ARE consumed at test time by ajv in `frontend/tests/helpers/schema-validator.ts`, but the validator runs in test environments only (not production).

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-12-05 | Information Disclosure | OpenAPI schema files exposed in public spec/ directory could reveal protocol shape | accept | The protocol is explicitly an open specification (CONCEPT.md publishes it). No secrets live in schemas. |
| T-12-06 | Tampering | Schema/code drift — if schemas and Rust types diverge, server-generated wire messages may fail client-side ajv validation | mitigate | Plan 08 extends `protocol-conformance.spec.ts` to run ajv against live wire traffic. Any drift becomes a test failure. |
</threat_model>

<verification>
- `js-yaml` parses both `spec/schemas/data.yaml` and `spec/schemas/message.yaml`
- `grep -c 'set-node' spec/schemas/data.yaml` ≥ 2 (in discriminator mapping AND variant schema)
- `grep -c 'surface' spec/schemas/message.yaml` ≥ 2 (in required AND properties)
- `grep -c '1.1.0' spec/PROTOCOL.md` ≥ 3 (header + example blocks)
- `grep -n 'PatchOperation\|set-node\|delete-node' spec/PROTOCOL.md` shows at least 5 lines
- `cd frontend && npx vitest --config vitest-browser.config.ts --run protocol-conformance 2>&1 | grep -i 'schema load\|ajv compile'` returns zero lines
</verification>

<success_criteria>
- `PatchOperation` in `data.yaml` is a tagged `oneOf` with 6 variants + `op` discriminator mapping
- `PatchMessage` in `message.yaml` has required `surface` field
- `spec/PROTOCOL.md §patch` documents all 6 ops with payload tables, a mixed example, and focus-preservation / root-immutability notes
- `spec/PROTOCOL.md` version string is `1.1.0` everywhere runtime-relevant
- `CONCEPT.md` "easy to patch by node ID" claim is grounded in a reference to the implemented op list
- ajv compiles the new schemas without errors (test-case failures tolerable — they are Plan 08's concern)
</success_criteria>

<output>
After completion, create `.planning/phases/12-protocol-node-patching-appshell/12-03-SUMMARY.md` recording:
- Lines changed in `spec/PROTOCOL.md` (before/after line counts)
- Confirmed YAML parse succeeds for both data.yaml and message.yaml
- Exact text of the CONCEPT.md reconciliation (the paragraph that changed)
- Any schema-validator warnings observed (non-blocking)
</output>
