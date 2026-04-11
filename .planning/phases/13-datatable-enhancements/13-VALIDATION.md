---
phase: 13
slug: datatable-enhancements
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-11
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Derived from `13-RESEARCH.md` §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework (frontend unit)** | vitest 4.1 (node env) |
| **Framework (frontend browser)** | vitest-browser-svelte 2.1 (real Chromium via `@vitest/browser-playwright`) |
| **Framework (frontend E2E)** | @playwright/test 1.58 |
| **Framework (backend)** | cargo test (standard) |
| **Config file — frontend unit** | `frontend/vite.config.ts` default |
| **Config file — frontend browser** | `frontend/vitest-browser.config.ts` |
| **Config file — frontend E2E (dev)** | `frontend/playwright.config.ts` (dev server on :5173) |
| **Config file — frontend E2E (backend)** | `frontend/playwright.e2e.config.ts` (backend on :3001) |
| **Quick run — frontend unit** | `cd frontend && npm test -- --run` |
| **Quick run — frontend browser (table)** | `cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/` |
| **Quick run — backend** | `cd backend && cargo test -p marionette` |
| **Full suite — frontend** | `cd frontend && npm test -- --run && npx vitest --config vitest-browser.config.ts --run && npx playwright test` |
| **Full suite — backend** | `cd backend && cargo test --workspace` |
| **Estimated runtime (full)** | ~3 min frontend (unit + browser + E2E) + ~1 min backend |

---

## Sampling Rate

- **After every task commit:** `cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/ && cd ../backend && cargo test -p marionette`
- **After every plan wave:** Full frontend + backend suites (`npm test -- --run && npx vitest --config vitest-browser.config.ts --run && npx playwright test && cd ../backend && cargo test --workspace`)
- **Before `/gsd-verify-work`:** Full suite green plus the two new E2E specs (`datatable-filter`, `datatable-infinite-scroll`)
- **Max feedback latency:** ~30 seconds (quick run), ~4 min (full suite)

---

## Per-Task Verification Map

> Task IDs are placeholders. The planner replaces `{plan}-{task}` with the real IDs once plans exist.

| # | Requirement | Plan | Wave | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---|-------------|------|------|-----------------|-----------|-------------------|-------------|--------|
| 1 | TABLE-01 | tbd | 2 | Filter bar renders text input and select dropdowns from `props.filters` | browser | `npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTable.browser-test.ts -t "renders filter bar"` | ⬜ W0 rewrite | ⬜ pending |
| 2 | TABLE-01 | tbd | 2 | Text filter debounced 300ms then dispatches `filter` action | browser + fake timers | `... -t "debounces filter"` | ⬜ W0 new | ⬜ pending |
| 3 | TABLE-01 | tbd | 2 | Enter in text filter flushes immediately | browser | `... -t "Enter flushes filter"` | ⬜ W0 new | ⬜ pending |
| 4 | TABLE-01 | tbd | 2 | Select filter fires immediately on change | browser | `... -t "Select filter no debounce"` | ⬜ W0 new | ⬜ pending |
| 5 | TABLE-01 | tbd | 2 | Empty/undefined filter values stripped from payload | browser | `... -t "filter payload omits empty"` | ⬜ W0 new | ⬜ pending |
| 6 | TABLE-01 | tbd | 1 | Backend `Filter::text / Filter::select / Filter::date_range` builders serialize correctly | Rust unit | `cd backend && cargo test -p marionette filter_builder` | ⬜ W0 new inline test in `standard.rs` | ⬜ pending |
| 7 | TABLE-01 | tbd | 3 | Filter input is validated via `Payload<FilterParams>` derive; malformed dates rejected with `ActionError::BadPayload` (V5 Input Validation) | Rust unit | `cargo test -p crm-demo filter_params_rejects_bad_date` | ⬜ W0 new | ⬜ pending |
| 8 | TABLE-01 | tbd | 3 | Live filter roundtrip against running backend | E2E playwright | `cd frontend && npx playwright test tests/e2e/datatable-filter.spec.ts` | ⬜ W0 new spec | ⬜ pending |
| 9 | TABLE-02 | tbd | 2 | IntersectionObserver sentinel fires `fetch-rows` when scrolled near tail | browser (real Chromium IO) | `... -t "sentinel triggers fetch-rows"` | ⬜ W0 new | ⬜ pending |
| 10 | TABLE-02 | tbd | 2 | Virtualizer renders only visible rows + overscan | browser | `... -t "virtualizer windows rows"` | ⬜ W0 new | ⬜ pending |
| 11 | TABLE-02 | tbd | 2 | Sort/filter reset resets scrollTop to 0 and re-arms sentinel | browser | `... -t "reset clears scroll"` | ⬜ W0 new | ⬜ pending |
| 12 | TABLE-02 | tbd | 2 | `total_rows` prop gates fetch when reached | browser | `... -t "stops fetching at total_rows"` | ⬜ W0 new | ⬜ pending |
| 13 | TABLE-02 | tbd | 2 | Fewer-than-limit response gates fetch | browser | `... -t "stops fetching on short chunk"` | ⬜ W0 new | ⬜ pending |
| 14 | TABLE-02 | tbd | 2 | Stale `fetch-rows` response discarded when action id doesn't match (D-H3 correlation) | browser | `... -t "drops stale fetch-rows patches"` | ⬜ W0 new | ⬜ pending |
| 15 | TABLE-02 | tbd | 3 | Server-side page limit hard-capped at 100 (DoS mitigation V5) | Rust unit | `cargo test -p crm-demo fetch_rows_caps_limit` | ⬜ W0 new | ⬜ pending |
| 16 | TABLE-02 | tbd | 3 | `fetch-rows` handler respects existing `AuthRequirement` matching the source list handler (V4 Access Control) | Rust integration | `cargo test -p crm-demo fetch_rows_requires_auth` | ⬜ W0 new | ⬜ pending |
| 17 | TABLE-02 | tbd | 3 | Live progressive scroll roundtrip against seeded dataset > `page_size` | E2E playwright | `cd frontend && npx playwright test tests/e2e/datatable-infinite-scroll.spec.ts` | ⬜ W0 new spec (depends on seed bump) | ⬜ pending |
| 18 | TABLE-03 | tbd | 2 | Columns DropdownMenu renders a CheckboxItem for each column where `getCanHide()` is true | browser | `... -t "columns dropdown lists hideable columns"` | ⬜ W0 new | ⬜ pending |
| 19 | TABLE-03 | tbd | 2 | Toggling a checkbox hides the column in the rendered table | browser | `... -t "toggle hides column"` | ⬜ W0 new | ⬜ pending |
| 20 | TABLE-03 | tbd | 2 | `hidden_default: true` columns start hidden | browser | `... -t "hidden_default starts hidden"` | ⬜ W0 new | ⬜ pending |
| 21 | TABLE-03 | tbd | 3 | Visibility state does NOT persist across reload | manual UAT | Chrome MCP walkthrough during `/gsd-verify-work` | ⬜ manual | ⬜ pending |
| 22 | Success-4 | tbd | 2 | Sorting resets scroll position and re-fetches from offset 0 | browser | `... -t "sort resets scroll"` | ⬜ W0 new | ⬜ pending |
| 23 | D-F1 | tbd | 2 | `kind: 'actions'` renders a DropdownMenu instead of `[object Object]` (resolves latent bug) | browser | `... -t "actions kind renders DropdownMenu"` | ⬜ W0 new | ⬜ pending |
| 24 | D-F1 | tbd | 2 | `kind: 'date'` uses `Intl.DateTimeFormat` | browser | `... -t "date kind formats"` | ⬜ W0 new | ⬜ pending |
| 25 | D-F1 | tbd | 2 | `kind: 'number'` right-aligns via tabular-nums | browser | `... -t "number kind right-aligns"` | ⬜ W0 new | ⬜ pending |
| 26 | D-F1 | tbd | 2 | `kind: 'badge'` renders shadcn `Badge` | browser | `... -t "badge kind renders Badge"` | ⬜ W0 new | ⬜ pending |
| 27 | D-F1 | tbd | 2 | `DataTableActions.svelte` escapes `item.label` via Svelte text interpolation (XSS mitigation) | browser | `... -t "DataTableActions escapes label"` | ⬜ W0 new | ⬜ pending |
| 28 | D-A2 | tbd | 1 | `TableScreen.svelte` file deleted | fs assertion | `test ! -e frontend/src/lib/components/screen/TableScreen.svelte` | ⬜ CI guard | ⬜ pending |
| 29 | TABLE-01/02/03 | tbd | 3 | Protocol conformance: filter + fetch-rows traffic validates against schemas | E2E playwright | `npx playwright test tests/e2e/protocol-conformance.spec.ts -t "filter action conforms"` | ⬜ W0 extend existing | ⬜ pending |
| 30 | CRM migration | tbd | 3 | Existing shell-nav E2E still passes after CRM handler migration | E2E playwright | `npx playwright test tests/e2e/shell-nav.spec.ts` | ✓ existing | ⬜ pending |
| 31 | Focus preservation | tbd | 2 | Filter input retains focus across server Render response (mirrors Phase 12's D-A6 proof) | browser | `... -t "filter focus preserved across reset"` | ⬜ W0 new | ⬜ pending |

**Counted:** 30 automated tests + 1 manual UAT (visibility non-persistence) = 31 verifications.

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Files to create/extend BEFORE any feature work. The planner MUST include these in the earliest wave so downstream plans can land green.

- [ ] **`frontend/src/lib/components/ui/data-table/`** — GENERATED by `npx shadcn-svelte@latest add data-table` in the `frontend/` working dir
- [ ] **`frontend/src/lib/components/ui/dropdown-menu/`** — GENERATED by `npx shadcn-svelte@latest add dropdown-menu` in the `frontend/` working dir
- [ ] **`frontend/package.json`** — add `@tanstack/table-core` and `@tanstack/svelte-virtual` runtime deps (`npm i @tanstack/table-core @tanstack/svelte-virtual`)
- [ ] **`frontend/src/lib/components/table/DataTable.browser-test.ts`** — REWRITE (existing file covers 4 basic assertions; Phase 13 adds ~20 new assertions against the recipe-shaped component)
- [ ] **`frontend/src/lib/components/table/DataTableActions.svelte`** — NEW component consumed by the `actions` cell kind (renders `DropdownMenu` items from row action arrays)
- [ ] **`frontend/src/lib/components/table/DataTableActions.browser-test.ts`** — NEW (clicking an item fires its `action.name` via `sendAction`; item label escaped via text interpolation)
- [ ] **`frontend/src/lib/actions/viewport.ts`** — NEW Svelte action (`onIntersect`) wrapping `IntersectionObserver` for the sentinel
- [ ] **`frontend/src/lib/transport/dispatcher.ts`** — EXTEND `sendAction` to RETURN the generated action id (currently returns `void`)
- [ ] **`frontend/src/lib/transport/dispatcher.test.ts`** — NEW or EXTEND (assert `sendAction` returns non-empty id string)
- [ ] **`frontend/tests/e2e/datatable-filter.spec.ts`** — NEW E2E (live filter roundtrip; `captureWebSocketFrames` + `__mrnSendAction` pattern from existing specs)
- [ ] **`frontend/tests/e2e/datatable-infinite-scroll.spec.ts`** — NEW E2E (depends on seed bump; verifies sentinel-driven fetch-rows against seeded > page_size dataset)
- [ ] **`backend/crates/crm-demo/src/seed.rs`** — BUMP contact seed count to `>2 × page_size` (per D-H4)
- [ ] **`backend/crates/marionette/src/builders/standard.rs`** — EXTEND with `Filter` + `ColumnKind` enums, fluent `.filter(...)` helper, inline tests (`filter_text_serializes`, `filter_select_serializes`, `filter_date_range_serializes`, `column_kind_serializes`, `data_table_fluent_filters_accumulate`)
- [ ] **`backend/crates/crm-demo/src/handlers/mod.rs` (or router registration)** — ADD new generic `fetch_rows` handler registration (per D-H1)
- [ ] **`backend/crates/crm-demo/src/handlers/` — per-screen `FilterParams` types** — NEW `#[derive(Deserialize)]` structs for each of `audit`, `contact`, `company`, `user` filter payloads

*Wave 0 must complete before Wave 1 (backend extensions) and Wave 2 (DataTable rewrite + CRM handler migration).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Column visibility state does NOT persist across page reload | TABLE-03 | Validates intentional NON-feature; automated tests prove the dropdown works, but "toggle, reload, observe reset" is a UX sanity check best done in Chrome MCP during `/gsd-verify-work` | 1. Open CRM contacts list in Chrome MCP. 2. Click Columns → uncheck "Company". 3. Verify the Company column disappears. 4. Reload the page. 5. Verify all columns are visible again (backend defaults). |

---

## Nyquist Sampling Concerns

- **Filter debounce timing** — browser tests MUST use `vi.useFakeTimers()` + `vi.advanceTimersByTime(300)` to avoid real-time waits. Pattern documented in `.planning/codebase/TESTING.md` ~line 170.
- **IntersectionObserver in browser tests** — real Chromium fires the observer naturally. Ensure the scroll container has meaningful `clientHeight` and rows have meaningful aggregate height (`virtualizer.getTotalSize()`). Use `element.scrollIntoView()` or programmatic scrollTop changes to trigger intersection.
- **Stale-response discard test** — fire two actions in rapid succession; mock the dispatcher to respond in reverse order; assert only the latest applies. Mock via a tiny fake dispatcher that delays dispatch per message.
- **Focus preservation test** — mirror the pattern in `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts:21-66` from Phase 12.

---

## Security Cross-Reference

Security threats that map to test rows in the Verification Map above (from 13-RESEARCH.md §Security Domain):

| Threat | Mitigation | Test Row(s) |
|--------|-----------|-------------|
| SQL injection via filter payload | SeaORM parameterized queries; reject unknown filter ids server-side | 6, 7 |
| XSS via `actions` column payload (`item.label` injection) | Svelte text interpolation, no `{@html}` | 23, 27 |
| DoS via giant page limits | Server-side page size hard cap (`limit.min(100)`) | 15 |
| Access control bypass via `fetch-rows` | `fetch-rows` handler enforces the same `AuthRequirement` as the source list handler | 16 |
| Malformed date payloads | `Payload<FilterParams>` derive rejects with `ActionError::BadPayload` | 7 |
| Client-side column hide used as privacy control (NOT A CONTROL) | Documentation note in `spec/PROTOCOL.md` — hidden columns are still transmitted; if sensitive, exclude from row payload at the server | (docs only) |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies assigned
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references above
- [ ] No watch-mode flags (all test commands use `--run`)
- [ ] Feedback latency < 30 s for quick runs, < 4 min for full suite
- [ ] `nyquist_compliant: true` set in frontmatter after planner populates task IDs

**Approval:** pending (awaits planner populating real task IDs in the Verification Map)
