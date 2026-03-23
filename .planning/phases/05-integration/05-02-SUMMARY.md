---
phase: 05-integration
plan: 02
subsystem: testing
tags: [playwright, e2e, websocket, ajv, schema-validation, protocol-conformance, spa-fallback]

# Dependency graph
requires:
  - phase: 05-integration
    plan: 01
    provides: Axum server with WS dispatch, static serving, SPA fallback, demo handlers
  - phase: 03-frontend-library
    provides: SvelteKit app with WebSocket transport and component rendering
  - phase: 02-protocol-specification
    provides: OpenAPI YAML schemas for protocol message validation
provides:
  - Playwright E2E integration tests validating full browser-to-backend round-trip
  - Protocol conformance tests validating messages against OpenAPI YAML schemas using AJV
  - WebSocket frame capture helper for Playwright
  - Schema validator utility loading OpenAPI YAML and validating protocol messages
  - Separate playwright.e2e.config.ts for integration tests against real backend
affects: [06-crm-features, e2e-testing, ci-pipeline]

# Tech tracking
tech-stack:
  added: [ajv, ajv-formats, js-yaml, @types/js-yaml]
  patterns: [WebSocket frame capture via Playwright page.on('websocket'), AJV schema validation with cross-file $ref rewriting, separate Playwright config for E2E vs component tests]

key-files:
  created:
    - frontend/tests/e2e/integration.spec.ts
    - frontend/tests/e2e/protocol-conformance.spec.ts
    - frontend/tests/helpers/ws-capture.ts
    - frontend/tests/helpers/schema-validator.ts
    - frontend/playwright.e2e.config.ts
  modified:
    - frontend/tests/e2e/smoke.spec.ts
    - frontend/package.json
    - frontend/src/lib/init.ts
    - backend/crates/crm-demo/src/main.rs

key-decisions:
  - "Defer router init until server hello received to avoid race condition with navigate action"
  - "Add bound text component for /message data path so patch has visible effect"
  - "Separate playwright.e2e.config.ts to avoid breaking Phase 3 visual/component tests"
  - "ESM-compatible schema validator using import.meta.url instead of __dirname"

patterns-established:
  - "WebSocket frame capture: register listener BEFORE page.goto to capture initial frames"
  - "Protocol conformance testing: AJV validator with cross-file YAML $ref rewriting"
  - "expect.poll() pattern for waiting on async WebSocket frames in Playwright"

requirements-completed: [INTEG-01, INTEG-02, INTEG-03]

# Metrics
duration: 10min
completed: 2026-03-23
---

# Phase 5 Plan 02: E2E Integration Tests Summary

**Playwright E2E tests validating WebSocket round-trip, component rendering, SPA fallback, and protocol conformance against OpenAPI schemas using AJV**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-23T06:42:27Z
- **Completed:** 2026-03-23T06:53:19Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- 10 E2E tests pass against the real backend: hello exchange, navigate/render round-trip, button click/patch round-trip, health endpoint, SPA fallback, and 4 protocol schema conformance tests
- WebSocket frame capture helper enables intercepting and asserting on protocol messages in browser context
- AJV-based schema validator loads all 4 OpenAPI YAML files, merges definitions, and validates protocol messages
- Fixed init.ts race condition where navigate action was dropped before WebSocket was open

## Task Commits

Each task was committed atomically:

1. **Task 1: Install test deps, create helpers, create separate E2E Playwright config** - `4dc55c0` (feat)
2. **Task 2: E2E integration and protocol conformance tests with SPA fallback** - `3114280` (test)

## Files Created/Modified
- `frontend/tests/e2e/integration.spec.ts` - 5 E2E tests: hello, navigate/render, click/patch, health, SPA fallback
- `frontend/tests/e2e/protocol-conformance.spec.ts` - 4 schema conformance tests: hello, render, action, patch
- `frontend/tests/helpers/ws-capture.ts` - WebSocket frame capture utility for Playwright
- `frontend/tests/helpers/schema-validator.ts` - AJV validator loading OpenAPI YAML schemas
- `frontend/playwright.e2e.config.ts` - Separate Playwright config for E2E against real backend at port 3001
- `frontend/tests/e2e/smoke.spec.ts` - Simplified for compatibility with both dev and E2E configs
- `frontend/package.json` - Added ajv, ajv-formats, js-yaml dev dependencies
- `frontend/src/lib/init.ts` - Deferred router init until server hello (race condition fix)
- `backend/crates/crm-demo/src/main.rs` - Added bound text component for /message data path

## Decisions Made
- Defer router initialization until server hello is received, ensuring the WebSocket is fully established before sending the initial navigate action. This fixes a race condition where the navigate was silently dropped.
- Added a text component bound to `/message` in the demo render so the patch update becomes visible in the browser.
- Created a separate `playwright.e2e.config.ts` rather than modifying the existing config, preserving Phase 3 visual/component tests.
- Used `import.meta.url` with `fileURLToPath` for ESM compatibility instead of `__dirname`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed init.ts race condition: navigate action dropped before WebSocket open**
- **Found during:** Task 2 (E2E tests revealed no navigate/render round-trip)
- **Issue:** `initRouter(sendAction)` was called synchronously after `connect()`, but the WebSocket `onopen` hadn't fired yet. The `send()` function silently dropped messages when socket wasn't open, so the navigate action was lost.
- **Fix:** Registered a `hello` handler that initializes the router only after the server hello is received, ensuring the WebSocket is fully open.
- **Files modified:** frontend/src/lib/init.ts
- **Verification:** All E2E tests pass; navigate action now appears in captured frames
- **Committed in:** 3114280 (Task 2 commit)

**2. [Rule 1 - Bug] Added bound text component for /message data path in crm-demo**
- **Found during:** Task 2 (button click test)
- **Issue:** The demo_click handler returned a patch setting `/message` to "Button was clicked!", but no component was bound to `/message`, so the text was never rendered in the browser.
- **Fix:** Added a `Text` component with `.bind("/message")` to the demo navigate render, making the patch update visible.
- **Files modified:** backend/crates/crm-demo/src/main.rs
- **Verification:** "Button was clicked!" text appears after clicking the button
- **Committed in:** 3114280 (Task 2 commit)

**3. [Rule 3 - Blocking] Fixed ESM __dirname error in schema-validator.ts**
- **Found during:** Task 2 (first test run)
- **Issue:** `__dirname` is not defined in ES modules. The project uses `"type": "module"` in package.json.
- **Fix:** Used `import.meta.url` with `fileURLToPath` and `path.dirname` to derive `__dirname`.
- **Files modified:** frontend/tests/helpers/schema-validator.ts
- **Verification:** Schema validator loads YAML files successfully
- **Committed in:** 3114280 (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 blocking)
**Impact on plan:** All fixes necessary for correctness. The init.ts race condition was a pre-existing bug exposed by E2E testing. The bound component and ESM fix were required for the tests to work correctly.

## Issues Encountered
- Initial Playwright webServer timeout (120s) during first compilation. Resolved by pre-building the backend before running tests. In CI, the webServer timeout should be sufficient since cargo build artifacts are cached.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full E2E test suite validates the complete protocol round-trip in the browser
- 10 passing tests cover: hello, navigate/render, click/patch, health, SPA fallback, and schema conformance
- Phase 3 visual/component tests remain unaffected (separate Playwright config)
- Ready for Phase 6+ to add real CRM business logic with confidence in integration testing

---
*Phase: 05-integration*
*Completed: 2026-03-23*
