---
phase: 13
plan: 07
type: execute
wave: 4
depends_on: [13-06]
files_modified:
  - frontend/src/lib/components/form/TextInput.svelte
  - frontend/src/lib/components/form/TextInput.browser-test.ts
  - frontend/tests/e2e/datatable-filter.spec.ts
  - frontend/tests/e2e/datatable-infinite-scroll.spec.ts
  - frontend/tests/e2e/protocol-conformance.spec.ts
autonomous: false
requirements: [TABLE-01, TABLE-02, TABLE-03]
must_haves:
  truths:
    - "`TextInput.svelte` reads `props.input_type` (with fallback to `props.type` for backward compat) so backend-serialized `input_type: 'password'` renders `<input type=\"password\">` instead of `<input type=\"text\">` (D-H4a)"
    - "`datatable-filter.spec.ts` drives the contact list screen, types a search string, waits 350ms, and asserts the WebSocket frames contain a `filter` action with the typed value"
    - "`datatable-infinite-scroll.spec.ts` loads the contact list (seeded > 100 rows from Plan 01), scrolls to the tail, and asserts the WebSocket frames contain a `fetch-rows` action with `source: 'contact_list'` and a non-zero offset, followed by a `patch` with multiple `set` ops"
    - "`protocol-conformance.spec.ts` is extended to validate that `filter` and `fetch-rows` WebSocket traffic conforms to the schemas in `spec/schemas/` (message.yaml + data.yaml)"
    - "A manual UAT checkpoint verifies column visibility does NOT persist across reload"
  artifacts:
    - path: "frontend/src/lib/components/form/TextInput.svelte"
      provides: "Fixed password-field rendering"
    - path: "frontend/tests/e2e/datatable-filter.spec.ts"
      provides: "Live filter roundtrip E2E"
    - path: "frontend/tests/e2e/datatable-infinite-scroll.spec.ts"
      provides: "Sentinel-driven fetch-rows E2E"
  key_links:
    - from: "Test specs"
      to: "Backend fetch_rows handler + frontend DataTable sentinel"
      via: "Real WebSocket frames captured via existing `captureWebSocketFrames` helper"
      pattern: "captureWebSocketFrames"
---

<objective>
Close the three remaining Phase 13 items that don't naturally fit into the earlier plans:

1. **TextInput `input_type` bug fix (D-H4a).** The frontend reads `props.type` but the backend serializes `props.input_type`; password fields have been rendering as text inputs silently. Two-line fix in one Svelte component + a browser test.
2. **Two new E2E specs** that drive the real CRM through the new DataTable: `datatable-filter.spec.ts` (live filter roundtrip) and `datatable-infinite-scroll.spec.ts` (sentinel-driven fetch-rows against the seeded > page_size dataset from Plan 01).
3. **Extend `protocol-conformance.spec.ts`** to validate the two new WebSocket traffic patterns (`filter` action + `fetch-rows` action → patch response) against the schemas under `spec/schemas/`.
4. **Human-verify checkpoint** for column visibility non-persistence (the one manual UAT item from 13-VALIDATION.md row 21 — Chrome MCP walkthrough during `/gsd-verify-work`).

Purpose: Without this plan, the new DataTable is not proven to work end-to-end against a running backend, and the Phase 12 TextInput password bug would carry into Phase 14 (FormScreen enhancements). This plan closes both.

Output: One Svelte fix, one browser test, two new E2E specs, one extended protocol-conformance spec, and one human-verify checkpoint task.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/13-datatable-enhancements/13-CONTEXT.md
@.planning/phases/13-datatable-enhancements/13-RESEARCH.md
@.planning/phases/13-datatable-enhancements/13-VALIDATION.md
@.planning/codebase/TESTING.md
@frontend/src/lib/components/form/TextInput.svelte
@frontend/tests/e2e/

<interfaces>
<!-- Executor MUST read these before starting. -->

Current `TextInput.svelte` (line 59):
```svelte
<Input
  type={(props.type as string) ?? 'text'}
  ...
/>
```

Backend `TextInput` builder (from Plan 02 context, standard.rs:23-35):
```rust
#[derive(ComponentBuilder)]
#[component(type = "text-input")]
pub struct TextInput {
    pub label: String,
    #[builder(optional)]
    pub placeholder: Option<String>,
    #[builder(optional)]
    pub required: Option<bool>,
    #[builder(optional)]
    pub input_type: Option<String>,  // ← serialized as "input_type"
    #[builder(optional)]
    pub disabled: Option<bool>,
}
```

The mismatch: Svelte reads `props.type` but backend writes `props.input_type`. Fix: read `input_type` first, fall back to `type` for legacy callers. The snake_case vs camelCase convention here is snake_case (confirmed by Plan 02's discussion of `total_rows` staying snake_case).

Existing playwright E2E patterns:
- `frontend/tests/e2e/protocol-conformance.spec.ts` (already exists from Phase 12-08) — uses `captureWebSocketFrames(page)` helper
- `frontend/tests/e2e/shell-nav.spec.ts` — pattern for driving the CRM through a navigation flow
- `frontend/tests/helpers/ws-capture.ts` — `captureWebSocketFrames` returns a list of parsed frames
- Login helper: unknown — search for `login` in existing specs; adapt the existing pattern

Playwright configs:
- `frontend/playwright.config.ts` — dev server on :5173
- `frontend/playwright.e2e.config.ts` — backend on :3001 (builds frontend + starts backend)

**For these E2E tests to work, the backend must be running with the seeded data from Plan 01** (120 contacts). The `playwright.e2e.config.ts` already handles this — it spins up a real `crm-demo` binary.

Schema files for protocol-conformance:
- `spec/schemas/message.yaml` — ActionMessage, PatchMessage, RenderMessage envelope
- `spec/schemas/component.yaml` — Component shape
- `spec/schemas/data.yaml` — PatchOperation variants
</interfaces>

<research_references>
- 13-CONTEXT.md §Post-research refinements §D-H4a — TextInput fix rationale
- 13-VALIDATION.md §Per-Task Verification Map rows 8, 17, 21, 29, 30 — this plan satisfies these
- 13-VALIDATION.md §Manual-Only Verifications — the column visibility UAT walkthrough
- 13-RESEARCH.md §Project Constraints — Phase 12 verification flagged the TextInput bug as "Deferred to Phase 13"
</research_references>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Fix TextInput.svelte input_type bug + browser test</name>
  <files>
    frontend/src/lib/components/form/TextInput.svelte,
    frontend/src/lib/components/form/TextInput.browser-test.ts
  </files>
  <read_first>
    - frontend/src/lib/components/form/TextInput.svelte (74 lines — specifically line 59 `type={...}`)
    - frontend/src/lib/components/form/ (for adjacent component test patterns — e.g., `Button.browser-test.ts` if present)
    - .planning/codebase/TESTING.md §Browser component test structure
    - backend/crates/marionette/src/builders/standard.rs §`TextInput` struct (confirms the serialized field name is `input_type`)
  </read_first>
  <behavior>
    - `render(TextInput, { props: { label: 'Password', input_type: 'password' }, surface: 'test' })` produces an `<input type="password">` element
    - `render(TextInput, { props: { label: 'Email', input_type: 'email' }, surface: 'test' })` produces an `<input type="email">` element
    - `render(TextInput, { props: { label: 'Name' }, surface: 'test' })` (no `input_type`) produces an `<input type="text">` element (default)
    - Backward compat: `render(TextInput, { props: { label: 'Legacy', type: 'password' }, surface: 'test' })` also produces `<input type="password">` (fallback)
    - If BOTH `input_type` and `type` are set, `input_type` wins (it's the authoritative one)
  </behavior>
  <action>
    **Step 1 — Fix the Svelte file.** Edit `frontend/src/lib/components/form/TextInput.svelte`. Line 59 currently reads:

    ```svelte
    type={(props.type as string) ?? 'text'}
    ```

    Replace with:

    ```svelte
    type={(props.input_type as string) ?? (props.type as string) ?? 'text'}
    ```

    Precedence: `input_type` (backend-authoritative, Phase 13 fix) → `type` (legacy frontend-authored trees, backward compat) → `'text'` (default).

    No other changes to the file.

    **Step 2 — Create `frontend/src/lib/components/form/TextInput.browser-test.ts`** (if it doesn't already exist — check first via `ls`).

    If the file EXISTS, ADD the new tests below to the existing file's test block. If it does NOT exist, create:

    ```typescript
    import { render } from 'vitest-browser-svelte';
    import { expect, test, vi, beforeEach } from 'vitest';
    import TextInput from './TextInput.svelte';
    import { resetStore } from '$lib/store/data.svelte';

    vi.mock('$lib/transport/dispatcher', () => ({ sendAction: vi.fn() }));

    beforeEach(() => {
      resetStore('test');
      vi.clearAllMocks();
    });

    test('defaults to type="text" when no input_type set', async () => {
      const screen = await render(TextInput, {
        props: { props: { label: 'Name' }, surface: 'test' },
      });
      const input = await screen.container.querySelector('input');
      expect(input?.getAttribute('type')).toBe('text');
    });

    test('reads props.input_type (backend-authoritative) — password field', async () => {
      const screen = await render(TextInput, {
        props: { props: { label: 'Password', input_type: 'password' }, surface: 'test' },
      });
      const input = await screen.container.querySelector('input');
      expect(input?.getAttribute('type')).toBe('password');
    });

    test('reads props.input_type for email', async () => {
      const screen = await render(TextInput, {
        props: { props: { label: 'Email', input_type: 'email' }, surface: 'test' },
      });
      const input = await screen.container.querySelector('input');
      expect(input?.getAttribute('type')).toBe('email');
    });

    test('backward compat: reads props.type if input_type absent', async () => {
      const screen = await render(TextInput, {
        props: { props: { label: 'Legacy', type: 'password' }, surface: 'test' },
      });
      const input = await screen.container.querySelector('input');
      expect(input?.getAttribute('type')).toBe('password');
    });

    test('input_type takes precedence over type when both set', async () => {
      const screen = await render(TextInput, {
        props: { props: { label: 'Both', input_type: 'password', type: 'text' }, surface: 'test' },
      });
      const input = await screen.container.querySelector('input');
      expect(input?.getAttribute('type')).toBe('password');
    });
    ```

    **Step 3 — Run the test.**

    ```bash
    cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/form/TextInput.browser-test.ts
    ```

    All 5 tests MUST pass.
  </action>
  <verify>
    <automated>cd frontend && grep -c "props.input_type" src/lib/components/form/TextInput.svelte && npx vitest --config vitest-browser.config.ts --run src/lib/components/form/TextInput.browser-test.ts</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "props.input_type" frontend/src/lib/components/form/TextInput.svelte` == 1
    - `grep -c "props.type" frontend/src/lib/components/form/TextInput.svelte` >= 1 (fallback still present)
    - `frontend/src/lib/components/form/TextInput.browser-test.ts` exists
    - All 5 new tests pass
    - `cd frontend && npx tsc --noEmit` exits 0
  </acceptance_criteria>
  <done>TextInput password-rendering bug is fixed with backward compat; browser test covers all 5 cases.</done>
</task>

<task type="auto">
  <name>Task 2: Write datatable-filter.spec.ts + datatable-infinite-scroll.spec.ts E2E specs</name>
  <files>
    frontend/tests/e2e/datatable-filter.spec.ts,
    frontend/tests/e2e/datatable-infinite-scroll.spec.ts
  </files>
  <read_first>
    - frontend/tests/e2e/shell-nav.spec.ts (navigation + login pattern — reuse)
    - frontend/tests/e2e/protocol-conformance.spec.ts (WebSocket frame capture pattern — reuse)
    - frontend/tests/helpers/ws-capture.ts (the `captureWebSocketFrames` helper signature)
    - frontend/playwright.e2e.config.ts (confirms which config spawns the real backend with seeded data)
    - backend/crates/crm-demo/src/seed.rs (Plan 01 bumped this to 120 contacts)
    - .planning/phases/13-datatable-enhancements/13-06-crm-list-handler-migration-SUMMARY.md (for the exact filter-id → column mapping on contact_list)
    - .planning/phases/13-datatable-enhancements/13-VALIDATION.md rows 8, 17 (these are the spec targets)
  </read_first>
  <action>
    **Spec 1 — `frontend/tests/e2e/datatable-filter.spec.ts`.**

    ```typescript
    import { test, expect } from '@playwright/test';
    import { captureWebSocketFrames } from '../helpers/ws-capture';

    // Drive the CRM contact list, type into the search filter, wait past
    // the 300ms debounce, and assert that a `filter` action fired with the
    // typed payload. Proves TABLE-01 end-to-end against a real backend.

    test.describe('DataTable filter roundtrip (TABLE-01)', () => {
      test('typing in text filter dispatches debounced filter action', async ({ page }) => {
        const frames = await captureWebSocketFrames(page);

        // Navigate + login — reuse existing pattern. If helper exists, use it;
        // otherwise inline the login action.
        await page.goto('/');
        // Login as admin (admin@localhost / admin — seeded by crm-demo)
        await page.fill('input[type="email"]', 'admin@localhost');
        await page.fill('input[type="password"]', 'admin');
        await page.click('button:has-text("Log in"), button:has-text("Sign in"), button[type="submit"]');
        // Wait for post-login navigation
        await page.waitForURL(/\/(home|contacts|dashboard)/, { timeout: 10000 });

        // Navigate to contacts
        await page.click('a:has-text("Contacts"), [data-testid="nav-contacts"], nav >> text=Contacts');
        await expect(page.getByText(/Seed Contact 0|Alice Johnson/)).toBeVisible({ timeout: 10000 });

        // Find the search filter — the DataTable renders it with aria-label="Search"
        // or placeholder "Filter contacts..."
        const searchInput = page.getByPlaceholder('Filter contacts...').or(page.getByLabel('Search'));
        await searchInput.fill('Alice');

        // Wait past the debounce
        await page.waitForTimeout(400);

        // Assert that a filter action was dispatched
        await expect.poll(
          () => frames.filter((f) => f.direction === 'sent' && String(f.raw).includes('"name":"filter"')).length,
          { timeout: 5000 }
        ).toBeGreaterThan(0);

        const filterFrames = frames.filter(
          (f) => f.direction === 'sent' && String(f.raw).includes('"name":"filter"')
        );
        const lastFilter = filterFrames[filterFrames.length - 1];
        const parsed = typeof lastFilter.raw === 'string' ? JSON.parse(lastFilter.raw) : lastFilter.raw;
        expect(parsed.name).toBe('filter');
        expect(parsed.payload.search).toBe('Alice');
      });

      test('Enter in text filter flushes immediately without debounce wait', async ({ page }) => {
        const frames = await captureWebSocketFrames(page);
        // Same login + navigate setup as above
        // ...
        const searchInput = page.getByPlaceholder('Filter contacts...').or(page.getByLabel('Search'));
        await searchInput.fill('Bob');
        await searchInput.press('Enter');

        // Assert immediately (no 300ms wait)
        await expect.poll(
          () => frames.filter((f) => f.direction === 'sent' && String(f.raw).includes('"search":"Bob"')).length,
          { timeout: 500 }
        ).toBeGreaterThan(0);
      });
    });
    ```

    **Spec 2 — `frontend/tests/e2e/datatable-infinite-scroll.spec.ts`.**

    ```typescript
    import { test, expect } from '@playwright/test';
    import { captureWebSocketFrames } from '../helpers/ws-capture';

    // Load the contact list (seeded with 120 contacts by Plan 01's seed.rs bump),
    // scroll the DataTable to its tail, and assert that a `fetch-rows` action
    // was dispatched with source="contact_list" and a non-zero offset, followed
    // by a `patch` message with set ops appending rows. Proves TABLE-02 end-to-end.

    test.describe('DataTable infinite scroll (TABLE-02)', () => {
      test('scrolling to tail triggers fetch-rows with non-zero offset', async ({ page }) => {
        const frames = await captureWebSocketFrames(page);

        // Login + navigate to contacts (reuse the login pattern)
        await page.goto('/');
        await page.fill('input[type="email"]', 'admin@localhost');
        await page.fill('input[type="password"]', 'admin');
        await page.click('button[type="submit"]');
        await page.waitForURL(/contacts|home/, { timeout: 10000 });
        await page.click('a:has-text("Contacts"), nav >> text=Contacts');

        // Wait for initial render (first page_size rows)
        await expect(page.getByText(/Seed Contact 0|Alice Johnson/)).toBeVisible({ timeout: 10000 });

        // Get the scroll container — DataTable.svelte sets data-testid="datatable-scroll"
        const scroller = page.locator('[data-testid="datatable-scroll"]');
        await expect(scroller).toBeVisible();

        // Scroll to the bottom (a big scrollTop value forces sentinel intersection)
        await scroller.evaluate((el) => {
          (el as HTMLElement).scrollTop = 10000;
        });

        // Wait for fetch-rows dispatch
        await expect.poll(
          () => frames.filter((f) => f.direction === 'sent' && String(f.raw).includes('"name":"fetch-rows"')).length,
          { timeout: 5000 }
        ).toBeGreaterThan(0);

        const fetchFrames = frames.filter(
          (f) => f.direction === 'sent' && String(f.raw).includes('"name":"fetch-rows"')
        );
        const firstFetch = fetchFrames[0];
        const parsed = typeof firstFetch.raw === 'string' ? JSON.parse(firstFetch.raw) : firstFetch.raw;
        expect(parsed.name).toBe('fetch-rows');
        expect(parsed.payload.source).toBe('contact_list');
        expect(parsed.payload.offset).toBeGreaterThan(0);
        expect(parsed.payload.limit).toBeLessThanOrEqual(100);

        // Assert a corresponding patch message arrived with set ops
        await expect.poll(
          () => frames.filter(
            (f) => f.direction === 'received'
              && String(f.raw).includes('"type":"patch"')
              && String(f.raw).includes('"/contacts/')
          ).length,
          { timeout: 5000 }
        ).toBeGreaterThan(0);
      });

      test('fetch-rows action id is echoed into the response patch id', async ({ page }) => {
        const frames = await captureWebSocketFrames(page);
        // Same setup as above
        // ...
        // After scroll triggers fetch-rows, find the sent action id and the
        // received patch id. They must match (D-H3 correlation).
        const fetchFrame = frames.find((f) => f.direction === 'sent' && String(f.raw).includes('"name":"fetch-rows"'));
        expect(fetchFrame).toBeDefined();
        const sent = JSON.parse(String(fetchFrame!.raw));
        const patchFrame = frames.find(
          (f) => f.direction === 'received' && String(f.raw).includes(`"id":"${sent.id}"`)
        );
        expect(patchFrame).toBeDefined();
      });
    });
    ```

    **If the login flow helper already exists** in `frontend/tests/helpers/` (e.g., `login.ts` or similar), use it instead of inlining. Grep:
    ```bash
    grep -rn "helpers/login\|loginAsAdmin\|login.*admin" frontend/tests/ 2>/dev/null
    ```

    **Run the specs** using the backend config:
    ```bash
    cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/datatable-filter.spec.ts tests/e2e/datatable-infinite-scroll.spec.ts
    ```

    If the dev-config `playwright.config.ts` runs against :5173 without a real backend, those specs cannot work there — only the e2e config with backend works. Document this in the spec comment header.

    Iterate until both specs pass.

    **Fallback for the infinite-scroll spec:** If the DataTable's `data-testid="datatable-scroll"` isn't being set by Plan 05 (check grep — it should be there per the plan template), fall back to finding the scroll container via `locator('div[style*="overflow"]').first()` or a similar structural selector.
  </action>
  <verify>
    <automated>cd frontend && test -e tests/e2e/datatable-filter.spec.ts && test -e tests/e2e/datatable-infinite-scroll.spec.ts && npx playwright test --config playwright.e2e.config.ts tests/e2e/datatable-filter.spec.ts tests/e2e/datatable-infinite-scroll.spec.ts 2>&1 | tail -40</automated>
  </verify>
  <acceptance_criteria>
    - `frontend/tests/e2e/datatable-filter.spec.ts` exists with at least 2 `test(...)` calls
    - `frontend/tests/e2e/datatable-infinite-scroll.spec.ts` exists with at least 2 `test(...)` calls
    - Both specs use `captureWebSocketFrames` from the existing helper
    - Both specs assert against the `filter` or `fetch-rows` action name in the sent-frame list
    - The infinite-scroll spec asserts `payload.source === 'contact_list'` AND `payload.offset > 0`
    - Both specs pass when run against `playwright.e2e.config.ts` (real backend)
    - No regressions in existing E2E specs: `npx playwright test --config playwright.e2e.config.ts` all pass
  </acceptance_criteria>
  <done>Two new E2E specs green; infinite-scroll and filter paths proven end-to-end against the real backend with seeded data.</done>
</task>

<task type="auto">
  <name>Task 3: Extend protocol-conformance.spec.ts to validate filter + fetch-rows traffic shape</name>
  <files>frontend/tests/e2e/protocol-conformance.spec.ts</files>
  <read_first>
    - frontend/tests/e2e/protocol-conformance.spec.ts (entire file — added in Phase 12-08; understand the schema-validation helper pattern it uses)
    - spec/schemas/message.yaml (the ActionMessage and PatchMessage envelopes)
    - spec/schemas/data.yaml (the PatchOperation variants)
    - frontend/tests/e2e/datatable-filter.spec.ts (Task 2 — for the capture pattern)
  </read_first>
  <action>
    Open `frontend/tests/e2e/protocol-conformance.spec.ts`. It already validates existing CRM traffic against the YAML schemas. Add two new `test(...)` calls:

    ```typescript
    test('filter action payload conforms to ActionMessage schema', async ({ page }) => {
      // 1. Capture frames as in existing tests
      const frames = await captureWebSocketFrames(page);

      // 2. Drive a filter interaction: login, navigate to contacts, type in search
      // (reuse the same login flow the existing tests use in this file)
      // ... login + navigate ...
      const searchInput = page.getByPlaceholder('Filter contacts...').or(page.getByLabel('Search'));
      await searchInput.fill('Acme');
      await page.waitForTimeout(400);

      // 3. Find the filter action frame and validate against ActionMessage schema
      const filterFrame = frames.find(
        (f) => f.direction === 'sent' && String(f.raw).includes('"name":"filter"')
      );
      expect(filterFrame).toBeDefined();
      const parsed = JSON.parse(String(filterFrame!.raw));

      // Use the existing schema-validate helper (same one the other tests use).
      // If it's ajv-based with pre-compiled validators, target "ActionMessage".
      const errors = validateAgainstSchema('ActionMessage', parsed);
      expect(errors).toEqual([]);
    });

    test('fetch-rows response patch conforms to PatchMessage schema', async ({ page }) => {
      const frames = await captureWebSocketFrames(page);
      // Login + navigate, then trigger infinite scroll
      // ... login ...
      const scroller = page.locator('[data-testid="datatable-scroll"]');
      await scroller.evaluate((el) => { (el as HTMLElement).scrollTop = 10000; });
      await page.waitForTimeout(1000);

      const patchFrame = frames.find(
        (f) => f.direction === 'received'
          && String(f.raw).includes('"type":"patch"')
          && String(f.raw).includes('"/contacts/')
      );
      expect(patchFrame).toBeDefined();
      const parsed = JSON.parse(String(patchFrame!.raw));

      const errors = validateAgainstSchema('PatchMessage', parsed);
      expect(errors).toEqual([]);
    });
    ```

    If the existing file uses a different helper name (not `validateAgainstSchema`), use whatever is already there. If the existing tests import a shared `schemaValidator` module from `frontend/tests/helpers/`, reuse it. The goal: two new tests that run through the SAME schema-validation machinery the existing tests use.

    Run:
    ```bash
    cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/protocol-conformance.spec.ts
    ```

    All tests (existing + new) must pass.
  </action>
  <verify>
    <automated>cd frontend && grep -c "filter action payload conforms\|fetch-rows response patch conforms" tests/e2e/protocol-conformance.spec.ts && npx playwright test --config playwright.e2e.config.ts tests/e2e/protocol-conformance.spec.ts 2>&1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "filter action payload conforms" frontend/tests/e2e/protocol-conformance.spec.ts` == 1
    - `grep -c "fetch-rows response patch conforms" frontend/tests/e2e/protocol-conformance.spec.ts` == 1
    - All tests in `protocol-conformance.spec.ts` pass (existing + 2 new)
    - The new tests use the SAME schema-validation helper as the existing tests (no ad-hoc ajv instance)
  </acceptance_criteria>
  <done>Protocol-conformance spec validates the two new Phase 13 traffic patterns; full E2E suite green.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 4: Manual UAT — column visibility does NOT persist across reload</name>
  <files>manual-uat-only (no files modified)</files>
  <action>Execute the Chrome MCP walkthrough described in `<how-to-verify>` below. This is a human-driven verification — the assistant should NOT automate past the checkpoint. Pause and wait for the user to confirm all 16 steps passed.</action>
  <verify>
    <automated>echo "manual UAT — no automated verify"</automated>
  </verify>
  <done>User typed "approved" after completing the 16-step Chrome MCP walkthrough with all steps passing; OR user described deviations and the assistant addressed them.</done>
  <what-built>
    Phase 13 delivered:
    - A new DataTable with filter bar, column visibility dropdown, virtualized rows, and sentinel-driven infinite scroll
    - CRM contact/company/user/audit lists rewritten to use the new DataTable shape
    - Per-row actions render as a DropdownMenu instead of `[object Object]`
    - TextInput password fields now render correctly

    Automated tests cover everything EXCEPT the intentional non-feature: column visibility should NOT persist across page reload (per D-E1). This is a UX invariant that's cheap to verify manually with Chrome MCP and harder to automate reliably.
  </what-built>
  <how-to-verify>
    Using Chrome MCP (claude-in-chrome tools) or a normal browser, walk through these steps against a running `make dev` stack:

    1. Open http://localhost:5173 (or whatever the dev URL is)
    2. Log in as `admin@localhost` / `admin`
    3. Navigate to the Contacts list
    4. Click the "Columns" dropdown in the DataTable's top region
    5. Uncheck the "Company" column (or any visible column)
    6. Verify the column disappears from the table immediately
    7. Verify at least one Seed Contact row (e.g., "Seed Contact 000") is visible in the table (confirms Plan 01's seed bump took effect)
    8. Scroll the table to the bottom slowly — verify rows continue loading past the initial page (confirms infinite scroll works in the real app, not just tests)
    9. Type "Alice" into the Search filter — verify the table filters live (no Apply button) after ~300ms
    10. Press Enter in the Search filter — verify nothing visibly hiccups (Enter just flushes, which was already in flight)
    11. Clear the search filter, scroll back to top — verify the initial rows render again
    12. Hover or click a row's actions column (three-dots) — verify a DropdownMenu appears with "Edit" and "Delete" items (NOT `[object Object]`)
    13. Navigate to Audit (admin-only) — verify infinite scroll also works there
    14. Reload the page (Cmd-R / Ctrl-R)
    15. Navigate back to Contacts
    16. Click "Columns" again — **verify all columns are checked** (the previous "Company" unchecking did NOT persist)

    Expected outcome: every step works as described. Step 16 is the key non-feature validation: visibility MUST reset to backend defaults on reload.
  </how-to-verify>
  <resume-signal>Type "approved" if all 16 steps pass. Describe any deviations otherwise (screenshot + step number).</resume-signal>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| E2E test → real backend | Tests drive a real crm-demo binary with seeded data. No trust issues — test environment only. |
| TextInput password-type fix | Existing server → client attribute passthrough. The fix doesn't introduce new trust concerns. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-13-07-01 | I (Information disclosure) | TextInput password field rendered as text | mitigate | **THIS IS THE EXISTING BUG being fixed.** Pre-fix, password fields render as `<input type="text">`, so keystrokes are visible on screen and may end up in screen recordings / browser autofill for the wrong field type. The fix makes the attribute match the backend intent. Test proves `type="password"` is emitted. |
| T-13-07-02 | I (Information disclosure via E2E test creds) | `datatable-filter.spec.ts` logs in as `admin@localhost / admin` — default dev credentials visible in the test file | accept | Dev-only credentials. Matches existing shell-nav.spec.ts pattern. No production secrets. |
| T-13-07-03 | Tampering | Protocol-conformance spec could false-positive if the schema has a permissive `additionalProperties: true` | mitigate | The existing spec framework already handles this in Phase 12-08. The new tests reuse the same machinery, so they inherit its guarantees. |

No HIGH severity threats. The TextInput fix directly reduces an existing information-disclosure risk.
</threat_model>

<verification>
```bash
# Unit + browser: TextInput fix
cd frontend
npx tsc --noEmit
npx vitest --config vitest-browser.config.ts --run src/lib/components/form/TextInput.browser-test.ts

# E2E: new specs (requires real backend via playwright.e2e.config.ts)
npx playwright test --config playwright.e2e.config.ts tests/e2e/datatable-filter.spec.ts tests/e2e/datatable-infinite-scroll.spec.ts tests/e2e/protocol-conformance.spec.ts

# Manual UAT: Task 4 checkpoint (human-verify)
```

All automated checks MUST exit 0 before the manual UAT gate is reached.
</verification>

<success_criteria>
- TextInput correctly renders password fields (D-H4a fixed)
- `datatable-filter.spec.ts` passes — live filter roundtrip proven end-to-end
- `datatable-infinite-scroll.spec.ts` passes — sentinel-driven fetch-rows proven end-to-end against 120-contact seed data
- `protocol-conformance.spec.ts` extended with 2 new tests validating filter + fetch-rows traffic against schemas; full file passes
- Human-verify checkpoint confirms column visibility non-persistence UX (Task 4)
- No regressions in any other E2E spec or browser test
</success_criteria>

<output>
After completion, create `.planning/phases/13-datatable-enhancements/13-07-e2e-and-textinput-fix-SUMMARY.md` recording:
- TextInput fix diff (one-line change)
- Any deviations in the E2E specs from the stub code (e.g., if login flow had to be adapted, if data-testid selectors had to be changed, if the seeded contact count required a different scroll depth)
- Final test run output: counts of passed/failed/skipped tests across all three playwright config runs
- The human-verify walkthrough result (16 steps + any issues noted by the user)
- A note for Phase 14 (FormScreen) picking up from here: TextInput now reads `props.input_type` — Phase 14 builders should prefer that field on new form fields
</output>
