---
phase: 14
slug: formscreen-enhancements
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-17
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: `14-RESEARCH.md` §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework (frontend unit)** | `vitest@4.1` |
| **Framework (frontend browser)** | `vitest-browser-svelte@2.1` (Chromium) |
| **Framework (E2E)** | `@playwright/test@1.58` |
| **Framework (backend)** | `cargo test` |
| **Config files** | `frontend/vite.config.ts` (unit), `frontend/vitest-browser.config.ts` (browser), `frontend/playwright.config.ts` + `frontend/playwright.e2e.config.ts` (E2E), `backend/Cargo.toml` (Rust) |
| **Quick run command (frontend unit)** | `cd frontend && npm test` |
| **Quick run command (frontend browser)** | `cd frontend && npx vitest --config vitest-browser.config.ts --run` |
| **Quick run command (backend)** | `cd backend && cargo test -p marionette` |
| **Typecheck command (frontend)** | `cd frontend && npm run check` |
| **Full suite command** | `cd frontend && npm test && npx vitest --config vitest-browser.config.ts --run && npx playwright test && cd ../backend && cargo test` |
| **Estimated runtime** | ~90s (quick) / ~6 min (full suite incl. E2E + visual) |

---

## Sampling Rate

- **After every task commit:** Run quick unit + browser test for the touched component (`npx vitest --config vitest-browser.config.ts <specific-file> --run` or `cargo test -p marionette <specific-test>`).
- **After every plan wave:** Run full frontend browser suite + `cargo test -p marionette -p marionette-protocol` + `npm run check`.
- **Before `/gsd-verify-work`:** Full suite (unit + browser + E2E + visual + backend + `npm run check`) must be green, plus Chrome-MCP UAT of contact-edit form.
- **Max feedback latency:** ≤90 seconds per task commit.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 0 | FORM-01, FORM-02 | — | N/A | install | `cd frontend && npx shadcn-svelte@latest add field textarea radio-group switch --yes` | ❌ W0 creates | ⬜ pending |
| 14-01-02 | 01 | 0 | FORM-01 | — | N/A | browser | `cd frontend && npx vitest --config vitest-browser.config.ts src/lib/components/form/Textarea.browser-test.ts --run` | ❌ W0 | ⬜ pending |
| 14-01-03 | 01 | 0 | FORM-01 | — | N/A | browser | `cd frontend && npx vitest --config vitest-browser.config.ts src/lib/components/form/RadioGroup.browser-test.ts --run` | ❌ W0 | ⬜ pending |
| 14-01-04 | 01 | 0 | FORM-01 | — | N/A | browser | `cd frontend && npx vitest --config vitest-browser.config.ts src/lib/components/form/Switch.browser-test.ts --run` | ❌ W0 | ⬜ pending |
| 14-01-05 | 01 | 0 | FORM-02 | — | N/A | browser | `cd frontend && npx vitest --config vitest-browser.config.ts src/lib/components/form/FieldSet.browser-test.ts --run` | ❌ W0 | ⬜ pending |
| 14-02-01 | 02 | 1 | FORM-01 | — | `data-invalid`/`aria-invalid` on error | browser | `cd frontend && npx vitest --config vitest-browser.config.ts src/lib/components/form/TextInput.browser-test.ts --run` | ✅ (extend) | ⬜ pending |
| 14-02-02 | 02 | 1 | FORM-01 | — | Label `for`/`id` match → click focuses input | browser | Same TextInput harness (new assertion) | ✅ | ⬜ pending |
| 14-02-03 | 02 | 1 | D-E1 | — | `input_type="password"` → `<input type="password">` | browser | Same TextInput harness (Phase-13 test must stay green) | ✅ | ⬜ pending |
| 14-02-04 | 02 | 1 | FORM-01 | — | `<Field.Description>` renders | browser | Same TextInput harness (new case) | ✅ | ⬜ pending |
| 14-02-05 | 02 | 1 | Backend | — | `description` + `full_width` serialize | unit | `cd backend && cargo test -p marionette standard::text_input` | ✅ (extend) | ⬜ pending |
| 14-03-01 | 03 | 1 | FORM-01 | — | SelectInput Field.Field wrap | browser | `cd frontend && npx vitest --config vitest-browser.config.ts src/lib/components/form/SelectInput.browser-test.ts --run` | ✅ (extend) | ⬜ pending |
| 14-03-02 | 03 | 1 | Backend | — | `description` + `full_width` on SelectInput builder | unit | `cargo test -p marionette standard::select` | ✅ | ⬜ pending |
| 14-04-01 | 04 | 1 | FORM-01 | — | Checkbox Field.Field wrap | browser | `cd frontend && npx vitest --config vitest-browser.config.ts src/lib/components/form/Checkbox.browser-test.ts --run` | ✅ (extend) | ⬜ pending |
| 14-04-02 | 04 | 1 | Backend | — | `description` + `full_width` on Checkbox builder | unit | `cargo test -p marionette standard::checkbox` | ✅ | ⬜ pending |
| 14-05-01 | 05 | 2 | D-E3 | — | Textarea renders placeholder, rows, description, error | browser | `cd frontend && npx vitest --config vitest-browser.config.ts src/lib/components/form/Textarea.browser-test.ts --run` | ❌ W0 | ⬜ pending |
| 14-05-02 | 05 | 2 | Backend | — | Textarea builder serialization | unit | `cargo test -p marionette standard::textarea` | ❌ new | ⬜ pending |
| 14-06-01 | 06 | 2 | D-E4 | — | RadioGroup renders options, selection, error | browser | Existing RadioGroup browser-test | ❌ W0 | ⬜ pending |
| 14-06-02 | 06 | 2 | D-E4 | — | Switch toggle + Field.Field wrap | browser | Existing Switch browser-test | ❌ W0 | ⬜ pending |
| 14-06-03 | 06 | 2 | Backend | — | RadioGroup/Switch builder serialization | unit | `cargo test -p marionette standard::radio_group standard::switch_` | ❌ new | ⬜ pending |
| 14-07-01 | 07 | 3 | FORM-02 | — | `FieldSet` renders `Field.Set + Field.Legend + Field.Group` | browser | `FieldSet.browser-test.ts` | ❌ W0 | ⬜ pending |
| 14-07-02 | 07 | 3 | FORM-02 | — | Default grid: 1-col mobile / 2-col desktop | browser+visual | `FieldSet.browser-test.ts` + `tests/visual/form.spec.ts` | ❌ W0 + ✅ | ⬜ pending |
| 14-07-03 | 07 | 3 | FORM-02 | — | `FieldSet.cols=N` forces fixed N columns | browser | `FieldSet.browser-test.ts` | ❌ W0 | ⬜ pending |
| 14-07-04 | 07 | 3 | FORM-02 | — | Per-field `full_width` spans all columns | browser | `FieldSet.browser-test.ts` | ❌ W0 | ⬜ pending |
| 14-07-05 | 07 | 3 | FORM-02 | — | Sibling FieldSets separated by `Field.Separator` | browser+visual | `FieldSet.browser-test.ts` + visual | ❌ W0 | ⬜ pending |
| 14-07-06 | 07 | 3 | Backend | — | FieldSet builder serialization | unit | `cargo test -p marionette standard::field_set` | ❌ new | ⬜ pending |
| 14-08-01 | 08 | 4 | D-A1 | — | `FormScreen.svelte` no longer exists | smoke | `grep -r "FormScreen" frontend/src backend/crates` → zero matches | n/a (grep) | ⬜ pending |
| 14-08-02 | 08 | 4 | D-E2 | — | TextInput blur during parent patch does not throw | browser+E2E | New browser test + `tests/e2e/contact-edit.spec.ts` | ❌ W0 + ✅ | ⬜ pending |
| 14-08-03 | 08 | 4 | FORM-01,02 | — | contact-edit form renders every new primitive | E2E | `npx playwright test --config playwright.e2e.config.ts tests/e2e/contact-edit.spec.ts` | ✅ (extend) | ⬜ pending |
| 14-08-04 | 08 | 4 | FORM-02 | — | Visual snapshot matches baseline | visual | `npx playwright test tests/visual/form.spec.ts` | ✅ (rebaseline) | ⬜ pending |
| 14-08-05 | 08 | 4 | FORM-01,02 | — | Chrome-MCP UAT all primitives + responsive | manual-automated | Chrome-MCP script (see §Manual-Only) | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Task IDs above are indicative scaffolding — planner may split/merge. Every task in PLAN.md must map to a row here.*

---

## Wave 0 Requirements

Wave 0 installs framework primitives and scaffolds missing browser-test files so downstream waves can rely on green tests:

- [ ] `npx shadcn-svelte@latest add field textarea radio-group switch --yes` — installs 4 primitives under `frontend/src/lib/components/ui/`.
- [ ] `frontend/src/lib/components/form/Textarea.browser-test.ts` — stubs for D-E3 assertions.
- [ ] `frontend/src/lib/components/form/RadioGroup.browser-test.ts` — stubs for D-E4 (radio).
- [ ] `frontend/src/lib/components/form/Switch.browser-test.ts` — stubs for D-E4 (switch).
- [ ] `frontend/src/lib/components/form/FieldSet.browser-test.ts` — stubs for FORM-02 (legend, group, default grid, `cols`, separator handling, `full_width`).
- [ ] `frontend/src/lib/components/core/NodeRenderer.browser-test.ts` — stub for D-E2 blur-during-patch regression.
- [ ] Extend `frontend/src/lib/components/form/{TextInput,SelectInput,Checkbox}.browser-test.ts` — new assertions for `<Field.Field>` markup, `data-invalid`, `aria-invalid`, `<Field.Description>`, label `for`/`id` focus.
- [ ] Extend `frontend/tests/e2e/contact-edit.spec.ts` + `frontend/tests/visual/form.spec.ts` — cover responsive grid and new primitives.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Responsive grid transition at 768px breakpoint on real viewport | FORM-02 | Chromium DevTools emulation doesn't exercise the full container query + scrollbar reflow the same way a real resize does | Chrome-MCP: open contact-edit, `resize_window` to 375px, capture GIF, resize to 1024px, capture GIF, confirm fields stack vs grid |
| Label-click focus works across all primitives | FORM-01 | A11y verification — must see focus ring + observe cursor entry | Chrome-MCP: open contact-edit, click label for name, assert input focused (via JS `document.activeElement`) |
| Error-state visual feedback (destructive color on `aria-invalid`) | FORM-01 | Visual regression snapshots approximate this but color-contrast verification needs a human eye | Chrome-MCP: submit empty form, screenshot, confirm red ring + error text on at least 2 fields |
| Textarea typing preserves focus under live patch | D-E2 | Rapid-blur race is timing-sensitive; Playwright E2E covers baseline, Chrome-MCP covers real-user typing cadence | Chrome-MCP: focus textarea, type 20 chars with 100ms inter-keystroke, read console for zero `TypeError` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags (`--run` / `--run` everywhere, never `vitest` bare)
- [ ] Feedback latency ≤ 90s quick / ≤ 6 min full
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
