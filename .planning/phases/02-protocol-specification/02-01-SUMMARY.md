---
phase: 02-protocol-specification
plan: 01
subsystem: api
tags: [openapi, json-schema, yaml, redocly, websocket, protocol]

# Dependency graph
requires:
  - phase: 01-project-infrastructure
    provides: Makefile with lint target, spec/ directory structure
provides:
  - OpenAPI 3.1 entry point (spec/openapi.yaml)
  - JSON Schema definitions for all 6 protocol message types
  - Component adjacency list schema with data binding
  - PatchOperation, KeyedCollection, ValidationError data schemas
  - Redocly lint tooling integrated into build system
affects: [02-protocol-specification, 03-frontend-library, 04-backend-toolkit]

# Tech tracking
tech-stack:
  added: ["@redocly/cli ^2.24.0", "@stoplight/spectral-cli ^6.15.0"]
  patterns: [tagged-union-discriminator, openapi-as-schema-registry, adjacency-list-component-model]

key-files:
  created:
    - spec/openapi.yaml
    - spec/schemas/common.yaml
    - spec/schemas/component.yaml
    - spec/schemas/data.yaml
    - spec/schemas/message.yaml
    - spec/package.json
    - spec/.redocly.yaml
    - spec/.gitignore
  modified:
    - Makefile

key-decisions:
  - "Redocly config requires explicit --config flag (not auto-discovered in spec/ subdirectory)"
  - "Disabled no-empty-servers, security-defined, operation-operationId rules for WebSocket-only spec"
  - "Added MIT SPDX identifier to license for strict info-license rule compliance"

patterns-established:
  - "Schema $ref convention: within-file #/TypeName, cross-file filename.yaml#/TypeName, from openapi.yaml schemas/filename.yaml#/TypeName"
  - "Tagged union: oneOf + discriminator on type field with const values per variant"
  - "WebSocket-only OpenAPI: use webhooks section for message schema documentation, empty paths"

requirements-completed: [PROT-01, PROT-02, PROT-03, PROT-04, PROT-05, PROT-06, PROT-07, PROT-08, PROT-09, PROT-10, PROT-11, PROT-12, PROT-14, DOC-01]

# Metrics
duration: 4min
completed: 2026-03-18
---

# Phase 2 Plan 1: Protocol Specification - Schemas Summary

**OpenAPI 3.1 spec with tagged union for 6 message types, component adjacency list schema, and JSON Pointer data binding using Redocly lint tooling**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-18T16:58:37Z
- **Completed:** 2026-03-18T17:02:52Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Complete OpenAPI 3.1 specification with all protocol message schemas validated by Redocly
- All 6 message types (hello, render, patch, action, event, error) defined as tagged union with discriminator
- Component adjacency list structure with data binding via JSON Pointer (RFC 6901)
- Spec tooling integrated into Makefile (lint-spec target + lint target extension)

## Task Commits

Each task was committed atomically:

1. **Task 1: Spec tooling and foundational schemas** - `fd147ee` (feat)
2. **Task 2: Message schemas and OpenAPI entry point** - `4de075a` (feat)

## Files Created/Modified
- `spec/openapi.yaml` - OpenAPI 3.1 entry point with webhooks section and all schema refs
- `spec/schemas/common.yaml` - Surface, JsonPointer, MessageId shared types
- `spec/schemas/component.yaml` - Component and ComponentAction schemas (adjacency list node)
- `spec/schemas/data.yaml` - PatchOperation, KeyedCollection, ValidationError schemas
- `spec/schemas/message.yaml` - ProtocolMessage tagged union with all 6 message types
- `spec/package.json` - Spec tooling dependencies (@redocly/cli, @stoplight/spectral-cli)
- `spec/.redocly.yaml` - Redocly lint configuration with WebSocket-appropriate rule overrides
- `spec/.gitignore` - Ignore node_modules and dist
- `Makefile` - Added lint-spec target and spec linting to lint target

## Decisions Made
- Redocly CLI does not auto-discover .redocly.yaml in subdirectories -- added explicit `--config .redocly.yaml` to npm scripts
- Disabled `no-empty-servers`, `security-defined`, and `operation-operationId` rules since this is a WebSocket-only protocol spec with no REST servers or security schemes
- Added SPDX `identifier: MIT` to license object for strict lint compliance

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Redocly config not auto-discovered in spec/ subdirectory**
- **Found during:** Task 2 (lint verification)
- **Issue:** Redocly CLI ignores .redocly.yaml when running from spec/ subdirectory, reports "No configurations provided"
- **Fix:** Added `--config .redocly.yaml` flag to npm lint and bundle scripts
- **Files modified:** spec/package.json
- **Verification:** `npm run lint` passes, `make lint-spec` passes
- **Committed in:** 4de075a (Task 2 commit)

**2. [Rule 3 - Blocking] Redocly recommended rules incompatible with WebSocket-only spec**
- **Found during:** Task 2 (lint verification)
- **Issue:** `no-empty-servers` and `security-defined` rules fail because WebSocket spec has no servers or security schemes
- **Fix:** Disabled inapplicable rules in .redocly.yaml with comments explaining why
- **Files modified:** spec/.redocly.yaml
- **Verification:** `npx @redocly/cli lint openapi.yaml --config .redocly.yaml` exits 0
- **Committed in:** 4de075a (Task 2 commit)

**3. [Rule 1 - Bug] Missing spec/.gitignore for node_modules**
- **Found during:** Task 1 (after npm install)
- **Issue:** spec/node_modules/ appeared as untracked in git
- **Fix:** Created spec/.gitignore with node_modules/ and dist/
- **Files modified:** spec/.gitignore (new)
- **Committed in:** fd147ee (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact on plan:** All auto-fixes necessary for lint tooling to work correctly. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All schema files ready for protocol manual (02-02-PLAN.md) to reference
- All schema files ready for example YAML files (02-03-PLAN.md) to validate against
- `make lint-spec` passes -- CI integration ready
- Bundling works (`redocly bundle` resolves all $ref paths)

---
*Phase: 02-protocol-specification*
*Completed: 2026-03-18*
