---
phase: 13
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - frontend/package.json
  - frontend/package-lock.json
  - frontend/src/lib/components/ui/data-table/
  - frontend/src/lib/components/ui/dropdown-menu/
  - frontend/src/lib/actions/viewport.ts
  - frontend/src/lib/transport/dispatcher.ts
  - frontend/src/lib/transport/dispatcher.test.ts
  - frontend/src/lib/components/table/SvelteVirtualSmoke.svelte
  - frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts
  - frontend/src/lib/utils/virtualizer.svelte.ts
  - backend/crates/crm-demo/src/seed.rs
autonomous: true
requirements: [TABLE-01, TABLE-02]
must_haves:
  truths:
    - "`@tanstack/table-core` and `@tanstack/svelte-virtual` are installed runtime deps"
    - "`frontend/src/lib/components/ui/data-table/` exists with recipe helper files (createSvelteTable, FlexRender, renderSnippet, renderComponent)"
    - "`frontend/src/lib/components/ui/dropdown-menu/` exists with shadcn DropdownMenu primitive files"
    - "Svelte-virtual store-based adapter either works on Svelte 5 OR the virtual-core-direct fallback `virtualizer.svelte.ts` is in place and the smoke test passes"
    - "`sendAction(...)` in `frontend/src/lib/transport/dispatcher.ts` returns the generated `id: string`"
    - "`onIntersect` Svelte action in `frontend/src/lib/actions/viewport.ts` wraps IntersectionObserver and fires a callback on intersection"
    - "`backend/crates/crm-demo/src/seed.rs` seeds > 2 × 50 = 100+ contacts when the contact table is empty"
  artifacts:
    - path: "frontend/src/lib/components/ui/data-table/index.js"
      provides: "createSvelteTable, FlexRender, renderSnippet, renderComponent re-exports"
    - path: "frontend/src/lib/components/ui/dropdown-menu/index.ts"
      provides: "DropdownMenu namespace re-export from bits-ui-backed primitives"
    - path: "frontend/src/lib/actions/viewport.ts"
      provides: "onIntersect Svelte action"
      exports: ["onIntersect"]
    - path: "frontend/src/lib/transport/dispatcher.ts"
      provides: "sendAction returns string id"
    - path: "frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts"
      provides: "Svelte 5 + @tanstack/svelte-virtual compatibility proof"
    - path: "backend/crates/crm-demo/src/seed.rs"
      provides: "100+ contact rows for infinite-scroll E2E"
  key_links:
    - from: "future DataTable.svelte (Plan 05)"
      to: "createSvelteTable, FlexRender from $lib/components/ui/data-table"
      via: "import"
      pattern: "from '\\$lib/components/ui/data-table"
    - from: "future DataTable.svelte (Plan 05) sentinel"
      to: "onIntersect action"
      via: "use: directive"
      pattern: "use:onIntersect"
    - from: "future DataTable.svelte (Plan 05)"
      to: "returned sendAction id"
      via: "const id = sendAction('fetch-rows', ...)"
      pattern: "sendAction\\(.*\\)"
---

<objective>
Land ALL Wave 0 infrastructure that downstream Phase 13 plans depend on. After this plan the repo has:

1. `@tanstack/table-core` and `@tanstack/svelte-virtual` installed + the shadcn-svelte `data-table` and `dropdown-menu` helper packages added via CLI
2. A proven story for Svelte 5 + `@tanstack/svelte-virtual` interop (either the store-based adapter works or the virtual-core-direct fallback is in place, decided empirically via a smoke test)
3. `sendAction` extended to RETURN the generated action id (required by D-H3 stale-response discard)
4. A reusable `onIntersect` Svelte action wrapping `IntersectionObserver` (required by D-D1 sentinel)
5. Seed data bumped to produce >100 contacts (required by D-H4b infinite-scroll E2E)

Purpose: Every Wave 2+ plan imports from the paths this plan creates. Without this plan, downstream plans cannot compile, test, or run.

Output: New runtime deps, new frontend artifacts under `$lib/components/ui/data-table`, `$lib/components/ui/dropdown-menu`, `$lib/actions/viewport.ts`, a smoke-test component + test, extended `dispatcher.ts` + test, optional `virtualizer.svelte.ts` fallback, and a seed.rs bump.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/13-datatable-enhancements/13-CONTEXT.md
@.planning/phases/13-datatable-enhancements/13-RESEARCH.md
@.planning/phases/13-datatable-enhancements/13-VALIDATION.md
@.planning/codebase/CONVENTIONS.md
@.planning/codebase/TESTING.md
@frontend/src/lib/transport/dispatcher.ts
@frontend/src/lib/components/table/DataTable.svelte
@frontend/package.json
@backend/crates/crm-demo/src/seed.rs

<interfaces>
<!-- Executor needs these BEFORE touching any file. Extracted from current code. -->

Current `sendAction` signature (frontend/src/lib/transport/dispatcher.ts:38-67):
```typescript
export function sendAction(
  name: string,
  payload?: Record<string, unknown>,
  source?: string,
  optimistic?: { patch: PatchOperation[] }
): void {
  const id = crypto.randomUUID();
  // ... builds msg, calls send(msg), returns void
}
```

Required new signature:
```typescript
export function sendAction(
  name: string,
  payload?: Record<string, unknown>,
  source?: string,
  optimistic?: { patch: PatchOperation[] }
): string {
  // ... same body, RETURN id
}
```

Current seed_contacts (backend/crates/crm-demo/src/seed.rs:89-128): seeds exactly 3 contacts (Alice, Bob, Carol) with a hand-written tuple list. Needs bump to >100 via a deterministic loop.

Existing shadcn-svelte primitive example (already present):
- `frontend/src/lib/components/ui/table/` exists, exports `Table.Root`, `Table.Header`, `Table.Body`, `Table.Row`, `Table.Head`, `Table.Cell`
- `frontend/src/lib/components/ui/input/`, `select/`, `button/`, `badge/` are present

Existing browser-test pattern:
- Framework: `vitest-browser-svelte` 2.1
- Runner: `npx vitest --config vitest-browser.config.ts --run <path>`
- Example: `frontend/src/lib/components/table/DataTable.browser-test.ts` (existing — will be rewritten in Plan 05)
</interfaces>

<research_references>
- 13-RESEARCH.md §Standard Stack — pinned versions `@tanstack/table-core@^8.21.3`, `@tanstack/svelte-virtual@^3.13.23`
- 13-RESEARCH.md §Architecture Patterns §Pattern 1 — `createSvelteTable` from `$lib/components/ui/data-table/index.js`
- 13-RESEARCH.md §Virtualization — the Svelte 5 store-vs-direct tradeoff and GitHub issue #866
- 13-VALIDATION.md §Wave 0 Requirements — checklist this plan MUST satisfy
</research_references>
</context>

<mcp_tool_usage>
Use the `svelte` MCP server (`mcp__svelte__*`) when writing `SvelteVirtualSmoke.svelte`, `onIntersect` action, and `virtualizer.svelte.ts` fallback. Query for idiomatic Svelte 5 patterns (runes, Svelte actions, `.svelte.ts` modules). After writing each Svelte file, re-invoke the svelte MCP to validate.
</mcp_tool_usage>

<tasks>

<task type="auto">
  <name>Task 1: Install TanStack deps + run shadcn-svelte CLI for data-table and dropdown-menu</name>
  <files>
    frontend/package.json,
    frontend/package-lock.json,
    frontend/src/lib/components/ui/data-table/,
    frontend/src/lib/components/ui/dropdown-menu/,
    frontend/components.json
  </files>
  <read_first>
    - frontend/package.json (verify current deps, target install section)
    - frontend/components.json (shadcn-svelte CLI config — confirms component paths)
    - .planning/phases/13-datatable-enhancements/13-RESEARCH.md §Standard Stack (for exact version pins)
    - frontend/src/lib/components/ui/table/ (reference for what a shadcn ui/ subfolder looks like after install)
  </read_first>
  <action>
    Working directory: `frontend/`. Run these commands in order:

    ```bash
    cd frontend
    npm i @tanstack/table-core@^8.21.3 @tanstack/svelte-virtual@^3.13.23
    npx shadcn-svelte@latest add data-table
    npx shadcn-svelte@latest add dropdown-menu
    ```

    Notes:
    - `@tanstack/svelte-virtual` will transitively pull `@tanstack/virtual-core@^3.13.23`. Do NOT add `virtual-core` as a direct dep.
    - If the shadcn-svelte CLI prompts for overwrite on `table` or other already-installed primitives, answer "no" (the `data-table` command only needs to write the helper package files under `src/lib/components/ui/data-table/`).
    - The `dropdown-menu` command writes a fresh subfolder `src/lib/components/ui/dropdown-menu/` (does NOT exist pre-phase).
    - After the install, verify `frontend/src/lib/components/ui/data-table/index.js` (or `.ts`) exists and exports `createSvelteTable`, `FlexRender`, `renderSnippet`, `renderComponent`. If the shadcn-svelte CLI version writes these with a different filename or export shape, adapt but preserve the four named exports — downstream Plan 05 imports all four by name from `$lib/components/ui/data-table/index.js`.

    Commit the resulting `frontend/package.json`, `frontend/package-lock.json`, and every file under the new `ui/data-table/` and `ui/dropdown-menu/` directories.
  </action>
  <verify>
    <automated>cd frontend && node -e "const pkg=require('./package.json'); if(!pkg.dependencies['@tanstack/table-core']||!pkg.dependencies['@tanstack/svelte-virtual']) process.exit(1)" && test -d src/lib/components/ui/data-table && test -d src/lib/components/ui/dropdown-menu && grep -rq "createSvelteTable" src/lib/components/ui/data-table && grep -rq "FlexRender" src/lib/components/ui/data-table</automated>
  </verify>
  <acceptance_criteria>
    - `frontend/package.json` `dependencies` contains `@tanstack/table-core` at version starting with `8.` and `@tanstack/svelte-virtual` at version starting with `3.`
    - `frontend/src/lib/components/ui/data-table/` directory exists and contains at least one file whose contents reference `createSvelteTable` AND `FlexRender`
    - `frontend/src/lib/components/ui/dropdown-menu/` directory exists with at least `index.ts` (or `.js`) that re-exports a `Root` / `Trigger` / `Content` / `CheckboxItem` namespace (typical shadcn-svelte shape)
    - `frontend/package-lock.json` has been updated (git shows modified)
    - `cd frontend && npx tsc --noEmit` exits 0 (the new files type-check)
  </acceptance_criteria>
  <done>Deps installed, shadcn helper files written, TypeScript compiles.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Svelte 5 + @tanstack/svelte-virtual smoke test (decide store-vs-direct path)</name>
  <files>
    frontend/src/lib/components/table/SvelteVirtualSmoke.svelte,
    frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts,
    frontend/src/lib/utils/virtualizer.svelte.ts
  </files>
  <read_first>
    - .planning/phases/13-datatable-enhancements/13-RESEARCH.md §Virtualization §Svelte 5 interop (the store-vs-direct tradeoff and GitHub issue TanStack/virtual#866)
    - frontend/src/lib/components/table/DataTable.browser-test.ts (existing — pattern for render() + expect.element())
    - .planning/codebase/TESTING.md §Browser component test structure (vitest-browser-svelte patterns)
    - node_modules/@tanstack/svelte-virtual/dist/*.js (read the installed adapter source to confirm whether it exports a store-based `createVirtualizer` or a rune-based one)
  </read_first>
  <behavior>
    - Mount `SvelteVirtualSmoke` with `count=100` and `estimateSize=40`
    - Assert `virtualItems` array is non-empty after mount
    - Assert the total scroll height === `100 * 40 = 4000px` (or close — TanStack may add padding)
    - Assert at least one virtual item has `index < 100` and has a stable `key`
    - Assert that resizing the scroll container re-computes `virtualItems` (scrollTop=2000 shows rows 45-55 or similar mid-range)
  </behavior>
  <action>
    **Step A — Try the store-based adapter first (preferred, simpler).**

    Create `frontend/src/lib/components/table/SvelteVirtualSmoke.svelte`:

    ```svelte
    <script lang="ts">
      import { createVirtualizer } from '@tanstack/svelte-virtual';

      let scrollRef: HTMLDivElement | undefined = $state();
      const count = 100;

      // Store-based adapter. Svelte 5 subscribes via $store auto-subscription.
      const virtualizer = createVirtualizer<HTMLDivElement, HTMLDivElement>({
        count,
        getScrollElement: () => scrollRef ?? null,
        estimateSize: () => 40,
        overscan: 5,
      });
    </script>

    <div bind:this={scrollRef} style="height: 300px; overflow: auto;" data-testid="scroll">
      <div style="height: {$virtualizer.getTotalSize()}px; position: relative;" data-testid="inner">
        {#each $virtualizer.getVirtualItems() as item (item.key)}
          <div
            data-testid="row-{item.index}"
            style="position: absolute; top: 0; left: 0; width: 100%; height: {item.size}px; transform: translateY({item.start}px);"
          >
            Row {item.index}
          </div>
        {/each}
      </div>
    </div>
    ```

    Create `frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts`:

    ```typescript
    import { render } from 'vitest-browser-svelte';
    import { expect, test } from 'vitest';
    import SvelteVirtualSmoke from './SvelteVirtualSmoke.svelte';

    test('store-based svelte-virtual renders virtualItems on mount', async () => {
      const screen = await render(SvelteVirtualSmoke);
      // First few rows must be in the DOM
      await expect.element(screen.getByTestId('row-0')).toBeVisible();
      // Far rows must NOT be in the DOM (virtualization proves itself)
      expect(await screen.getByTestId('row-99').query()).toBeNull();
      // Inner container reports total size
      const inner = await screen.getByTestId('inner').element();
      const h = Number((inner as HTMLElement).style.height.replace('px', ''));
      expect(h).toBeGreaterThanOrEqual(4000);
    });
    ```

    Run the test:

    ```bash
    cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/SvelteVirtualSmoke.browser-test.ts
    ```

    **Step B — IF the test FAILS** (symptoms: empty table, `virtualItems` is empty, TypeError on `$virtualizer`, or scroll is jumpy), fall back to the virtual-core-direct wrapper:

    Create `frontend/src/lib/utils/virtualizer.svelte.ts`:

    ```typescript
    import { Virtualizer, type VirtualizerOptions, type VirtualItem } from '@tanstack/virtual-core';

    /**
     * Svelte 5 rune-based wrapper around @tanstack/virtual-core's Virtualizer.
     * Used as a fallback when the @tanstack/svelte-virtual store-based adapter
     * has Svelte 5 interop bugs (see TanStack/virtual#866).
     */
    export function createRuneVirtualizer<TScroll extends Element, TItem extends Element>(
      options: () => Omit<VirtualizerOptions<TScroll, TItem>, 'observeElementRect' | 'observeElementOffset' | 'scrollToFn'>
    ) {
      let totalSize = $state(0);
      let virtualItems = $state<VirtualItem[]>([]);

      const baseOptions: VirtualizerOptions<TScroll, TItem> = {
        ...(options() as VirtualizerOptions<TScroll, TItem>),
        observeElementRect: (instance, cb) => {
          const el = instance.scrollElement;
          if (!el) return () => {};
          const ro = new ResizeObserver(() => cb(el.getBoundingClientRect()));
          ro.observe(el);
          cb(el.getBoundingClientRect());
          return () => ro.disconnect();
        },
        observeElementOffset: (instance, cb) => {
          const el = instance.scrollElement;
          if (!el) return () => {};
          const onScroll = () => cb(el.scrollTop, true);
          el.addEventListener('scroll', onScroll, { passive: true });
          cb(el.scrollTop, false);
          return () => el.removeEventListener('scroll', onScroll);
        },
        scrollToFn: (offset, _opts, instance) => {
          instance.scrollElement?.scrollTo({ top: offset, behavior: 'auto' });
        },
      };

      const instance = new Virtualizer(baseOptions);
      instance.setOptions({
        ...baseOptions,
        onChange: (v) => {
          totalSize = v.getTotalSize();
          virtualItems = v.getVirtualItems();
        },
      });
      instance._didMount();
      instance._willUpdate();

      return {
        get totalSize() { return totalSize; },
        get virtualItems() { return virtualItems; },
        instance,
      };
    }
    ```

    Then rewrite `SvelteVirtualSmoke.svelte` to use the rune wrapper and re-run the test. Iterate until the test above passes.

    **Step C — Record the decision** in a comment at the top of `SvelteVirtualSmoke.svelte`:

    ```svelte
    <!--
      svelte-virtual Svelte 5 compatibility decision (Phase 13 Wave 0 smoke test):
      - Path chosen: [STORE-BASED | VIRTUAL-CORE-DIRECT]
      - Reason: [one sentence]
      - Downstream DataTable.svelte (Plan 05) MUST use the same path.
    -->
    ```

    Do NOT delete `SvelteVirtualSmoke.svelte` — keep it as a living regression check. Plan 05 reads this decision comment to know which path to use.
  </action>
  <verify>
    <automated>cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/SvelteVirtualSmoke.browser-test.ts</automated>
  </verify>
  <acceptance_criteria>
    - `frontend/src/lib/components/table/SvelteVirtualSmoke.svelte` exists with a `<!-- svelte-virtual Svelte 5 compatibility decision ... Path chosen: ... -->` comment block at the top
    - `frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts` exists and passes
    - Test output shows `row-0` visible and `row-99` not in DOM (virtualization proven)
    - IF the fallback path was taken, `frontend/src/lib/utils/virtualizer.svelte.ts` exists with an exported `createRuneVirtualizer` function
    - IF the store path worked, `virtualizer.svelte.ts` does NOT exist (do not create it prematurely)
    - `grep -l "Path chosen" frontend/src/lib/components/table/SvelteVirtualSmoke.svelte` finds the decision comment
  </acceptance_criteria>
  <done>Smoke test green; downstream Plan 05 knows which virtualizer integration path to use.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Extend sendAction + add onIntersect action + bump seed.rs contacts</name>
  <files>
    frontend/src/lib/transport/dispatcher.ts,
    frontend/src/lib/transport/dispatcher.test.ts,
    frontend/src/lib/actions/viewport.ts,
    backend/crates/crm-demo/src/seed.rs
  </files>
  <read_first>
    - frontend/src/lib/transport/dispatcher.ts (full file — signature change)
    - frontend/src/lib/init.ts (if exists — confirms no caller expects `sendAction` to return void in a type-incompatible way)
    - frontend/src/lib/transport/websocket.svelte.ts (exports `send`)
    - backend/crates/crm-demo/src/seed.rs §seed_contacts (lines 89-128 — the tuple list to replace)
    - .planning/codebase/TESTING.md §Unit test structure (for `dispatcher.test.ts` shape)
    - .planning/phases/13-datatable-enhancements/13-CONTEXT.md §D-H3 (confirms sendAction must return the id; establishes this is the only correlation mechanism needed)
  </read_first>
  <action>
    **Part A — `sendAction` returns the generated id.**

    Edit `frontend/src/lib/transport/dispatcher.ts`. Change the signature from `): void {` to `): string {` and add `return id;` as the last line of the function. The rest of the function body stays unchanged. Update the JSDoc to say "Returns the generated correlation ID (UUID) so callers can track which response corresponds to which request."

    Exact final function:
    ```typescript
    /**
     * Send an action message to the backend.
     * Generates a correlation ID via crypto.randomUUID().
     * If optimistic patch is provided, applies it immediately before sending.
     * @returns The generated correlation ID (UUID) so callers can track which
     *          response corresponds to which request (used by DataTable for
     *          stale fetch-rows discard, per Phase 13 D-H3).
     */
    export function sendAction(
      name: string,
      payload?: Record<string, unknown>,
      source?: string,
      optimistic?: { patch: PatchOperation[] }
    ): string {
      const id = crypto.randomUUID();

      const msg: ActionMessage = {
        type: 'action',
        id,
        name
      };

      if (payload !== undefined) {
        msg.payload = payload;
      }

      if (source !== undefined) {
        msg.source = source;
      }

      if (optimistic) {
        msg.optimistic = optimistic;
        applyOptimistic(id, '', optimistic.patch);
      }

      send(msg);
      return id;
    }
    ```

    Grep for existing callers to confirm NONE of them break on the new return type (the change from `void` to `string` is backward-compatible at call sites because `string` is discardable):

    ```bash
    grep -rn "sendAction(" frontend/src --include="*.ts" --include="*.svelte"
    ```

    Expected: callers like `sendAction('sort', ...)` continue to work because they ignore the return value. If any caller uses `const x: void = sendAction(...)`, fix it.

    Create `frontend/src/lib/transport/dispatcher.test.ts` (new file):

    ```typescript
    import { describe, it, expect, vi, beforeEach } from 'vitest';

    vi.mock('./websocket.svelte', () => ({
      send: vi.fn(),
    }));
    vi.mock('$lib/store/optimistic.svelte', () => ({
      applyOptimistic: vi.fn(),
    }));

    import { sendAction } from './dispatcher';

    beforeEach(() => {
      vi.clearAllMocks();
      // Stub crypto.randomUUID for determinism
      vi.stubGlobal('crypto', { randomUUID: vi.fn(() => 'test-uuid-1234') });
    });

    describe('sendAction return value', () => {
      it('returns the generated action id', () => {
        const id = sendAction('filter', { search: 'alice' });
        expect(id).toBe('test-uuid-1234');
      });

      it('returns a string even without payload', () => {
        const id = sendAction('noop');
        expect(typeof id).toBe('string');
        expect(id.length).toBeGreaterThan(0);
      });

      it('generates a fresh id per call', () => {
        let counter = 0;
        vi.stubGlobal('crypto', { randomUUID: vi.fn(() => `id-${counter++}`) });
        expect(sendAction('a')).toBe('id-0');
        expect(sendAction('b')).toBe('id-1');
      });
    });
    ```

    **Part B — `onIntersect` Svelte action.**

    Create `frontend/src/lib/actions/viewport.ts`:

    ```typescript
    /**
     * Svelte action that fires a callback when the element enters the viewport
     * of its scroll container (or the root viewport if no container is given).
     *
     * Used by DataTable.svelte (Phase 13 Plan 05) as the infinite-scroll sentinel.
     *
     * Usage:
     * ```svelte
     * <div use:onIntersect={{ onEnter: () => sendAction('fetch-rows', ...), rootMargin: '100px' }}></div>
     * ```
     *
     * The observer disconnects and re-creates when options change. The callback
     * is fired on the leading edge of an intersection (when `isIntersecting` flips
     * from false to true), NOT on every intersectionRatio change.
     */
    export interface OnIntersectOptions {
      onEnter: () => void;
      root?: Element | null;
      rootMargin?: string;
      threshold?: number | number[];
      enabled?: boolean;
    }

    export function onIntersect(node: Element, options: OnIntersectOptions) {
      let observer: IntersectionObserver | undefined;
      let wasIntersecting = false;

      function start(opts: OnIntersectOptions) {
        stop();
        if (opts.enabled === false) return;
        observer = new IntersectionObserver(
          (entries) => {
            for (const entry of entries) {
              if (entry.isIntersecting && !wasIntersecting) {
                wasIntersecting = true;
                opts.onEnter();
              } else if (!entry.isIntersecting) {
                wasIntersecting = false;
              }
            }
          },
          {
            root: opts.root ?? null,
            rootMargin: opts.rootMargin ?? '0px',
            threshold: opts.threshold ?? 0,
          }
        );
        observer.observe(node);
      }

      function stop() {
        observer?.disconnect();
        observer = undefined;
        wasIntersecting = false;
      }

      start(options);

      return {
        update(newOptions: OnIntersectOptions) {
          start(newOptions);
        },
        destroy() {
          stop();
        },
      };
    }
    ```

    **Part C — Bump seed.rs contact count.**

    Edit `backend/crates/crm-demo/src/seed.rs` function `seed_contacts` (current lines 89-128). Replace the 3-contact tuple list with a deterministic loop that seeds 120 contacts. Keep the first three named contacts (Alice, Bob, Carol) for existing tests that assert their presence. Append 117 generated contacts after them.

    Replace the body of `seed_contacts` after `let globex = ... ;` with:

    ```rust
        #[allow(clippy::type_complexity)]
        let named_contacts: Vec<(&str, &str, Option<&str>, Option<&str>, Option<i32>)> = vec![
            ("Alice Johnson", "alice@acme.example.com", Some("+1-555-0101"), Some("CEO"), acme.as_ref().map(|c| c.company_id)),
            ("Bob Smith", "bob@globex.example.com", Some("+1-555-0102"), Some("CTO"), globex.as_ref().map(|c| c.company_id)),
            ("Carol Williams", "carol@example.com", None, Some("Freelancer"), None),
        ];

        for (name, email, phone, title, company_id) in named_contacts {
            let model = contact::ActiveModel {
                contact_id: NotSet,
                contact_name: Set(name.into()),
                contact_email: Set(email.into()),
                contact_phone: Set(phone.map(String::from)),
                contact_title: Set(title.map(String::from)),
                contact_company: Set(company_id),
                contact_created_at: NotSet,
                contact_updated_at: NotSet,
            };
            model.insert(db).await?;
        }

        // Bulk-seed additional contacts so Phase 13's infinite-scroll E2E has
        // > 2 × page_size (50) rows. Deterministic naming for test assertions.
        let titles = ["Engineer", "Manager", "Analyst", "Designer", "Director"];
        let company_ids: Vec<Option<i32>> = vec![
            acme.as_ref().map(|c| c.company_id),
            globex.as_ref().map(|c| c.company_id),
            None,
        ];
        for i in 0..117 {
            let name = format!("Seed Contact {i:03}");
            let email = format!("seed{i:03}@example.com");
            let title = titles[i % titles.len()];
            let company_id = company_ids[i % company_ids.len()];
            let model = contact::ActiveModel {
                contact_id: NotSet,
                contact_name: Set(name),
                contact_email: Set(email),
                contact_phone: Some(format!("+1-555-{:04}", 1000 + i)).map(Set).unwrap_or(NotSet),
                contact_title: Set(Some(title.into())),
                contact_company: Set(company_id),
                contact_created_at: NotSet,
                contact_updated_at: NotSet,
            };
            model.insert(db).await?;
        }

        tracing::info!("Seeded 120 demo contacts (3 named + 117 generated)");
        Ok(())
    }
    ```

    IMPORTANT: adapt the `phone` field handling to whatever the SeaORM `ActiveValue::Set` / `NotSet` idiom is for `Option<String>` in this crate — look at existing usage in the file. The block above may need a small syntactic tweak. The goal is: 120 contacts inserted, total count > 100.

    Also update any existing Rust test that asserts `contacts.len() == 3` — grep for it:

    ```bash
    grep -rn "Seeded 3 demo contacts\|contacts.len.*== *3" backend/ 2>/dev/null
    ```

    Update the log string assertion to `"Seeded 120 demo contacts"` if such a test exists, and bump any length assertions to `120`.
  </action>
  <verify>
    <automated>cd frontend && npx vitest --run src/lib/transport/dispatcher.test.ts && cd ../backend && cargo test -p crm-demo seed_contacts 2>/dev/null; cd .. ; cd backend && cargo build -p crm-demo</automated>
  </verify>
  <acceptance_criteria>
    - `grep -n "): string {" frontend/src/lib/transport/dispatcher.ts` matches the `sendAction` signature line
    - `grep -c "return id;" frontend/src/lib/transport/dispatcher.ts` is 1
    - `frontend/src/lib/transport/dispatcher.test.ts` exists and `cd frontend && npx vitest --run src/lib/transport/dispatcher.test.ts` passes with 3 passing tests
    - `frontend/src/lib/actions/viewport.ts` exists and exports `onIntersect` function (`grep -c "export function onIntersect" frontend/src/lib/actions/viewport.ts` == 1)
    - `cd frontend && npx tsc --noEmit` passes
    - `cd backend && cargo build -p crm-demo` succeeds
    - `grep -c "Seeded 120 demo contacts" backend/crates/crm-demo/src/seed.rs` == 1
    - `grep -c "for i in 0..117" backend/crates/crm-demo/src/seed.rs` == 1
    - No callers of `sendAction` are broken by the return-type change (check: `cd frontend && npx tsc --noEmit` exits 0)
  </acceptance_criteria>
  <done>Dispatcher returns id, onIntersect action exists, seed produces 120 contacts, all existing tests still green.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| CLI → repo filesystem | shadcn-svelte CLI writes files under `frontend/src/lib/components/ui/`. We only commit files the CLI writes; we do not execute arbitrary post-install hooks. |
| npm registry → repo | Two new transitive dep trees (`@tanstack/table-core`, `@tanstack/svelte-virtual` + `@tanstack/virtual-core`). Both are TanStack-maintained; low supply-chain risk relative to unrelated untrusted packages. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-13-01-01 | Tampering | `@tanstack/svelte-virtual` supply chain | accept | TanStack is a widely-used, reputable maintainer; version pinned in `package.json`; `package-lock.json` locks transitive deps. No action beyond pinning. |
| T-13-01-02 | I (Information disclosure) | `onIntersect` action fires callback → may leak scroll position via action dispatch | accept | Scroll position is not sensitive; the action only fires pagination requests, not PII. |
| T-13-01-03 | D (DoS) | `IntersectionObserver` fires rapidly on flaky scroll | mitigate | `wasIntersecting` latch in `onIntersect` collapses edge changes so the callback fires only on the leading edge of entry. Downstream DataTable (Plan 05) adds its own `lastFetchRowsActionId` guard. |
| T-13-01-04 | Tampering | `sendAction` return-value change breaks caller assumptions | mitigate | Task 3 greps all callers and runs `tsc --noEmit` to catch any callers that treat the return as `void`. Return-type widens from `void` to `string` — ignorable at call sites. |

No HIGH severity threats. Plan proceeds.
</threat_model>

<verification>
Run after all three tasks complete:

```bash
# Frontend type-check + unit + browser test suite
cd frontend
npx tsc --noEmit
npm test -- --run
npx vitest --config vitest-browser.config.ts --run src/lib/components/table/SvelteVirtualSmoke.browser-test.ts
npx vitest --run src/lib/transport/dispatcher.test.ts

# Backend builds and existing seed logic still compiles
cd ../backend
cargo build -p crm-demo
cargo test -p crm-demo seed 2>/dev/null || true  # tolerate no matching tests
```

All four frontend commands MUST exit 0. `cargo build` MUST exit 0.
</verification>

<success_criteria>
- `@tanstack/table-core` and `@tanstack/svelte-virtual` present in `frontend/package.json`
- `frontend/src/lib/components/ui/data-table/` and `ui/dropdown-menu/` directories exist with shadcn-svelte CLI output
- `SvelteVirtualSmoke.browser-test.ts` passes — Svelte 5 interop path is DECIDED and DOCUMENTED in the component's top-of-file comment
- `sendAction` returns `string`; `dispatcher.test.ts` proves it
- `onIntersect` Svelte action exists at `frontend/src/lib/actions/viewport.ts`
- `backend/crates/crm-demo/src/seed.rs` seeds 120 contacts; backend builds
- Every existing test still passes (no regressions)
</success_criteria>

<output>
After completion, create `.planning/phases/13-datatable-enhancements/13-01-scaffolding-SUMMARY.md` recording:
- Exact versions installed (`npm ls @tanstack/table-core @tanstack/svelte-virtual`)
- The svelte-virtual Svelte 5 path decision (STORE-BASED or VIRTUAL-CORE-DIRECT) and the reason
- Any deviations from the task actions (e.g., if shadcn-svelte CLI wrote `.ts` instead of `.js`)
- Any caller-site fixups needed because of the `sendAction` return-type change
</output>
