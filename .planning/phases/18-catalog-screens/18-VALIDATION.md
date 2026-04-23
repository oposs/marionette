---
phase: 18
slug: catalog-screens
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-23
---

# Phase 18 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Nyquist-compliant sampling: every task either has an `<automated>` verify command or declares a Wave 0 dependency that will install one. No 3 consecutive tasks without automated verification.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework (Rust)** | `cargo test` — workspace-wide unit + integration tests |
| **Framework (Rust lints)** | `cargo clippy --workspace --all-targets -- -D warnings` |
| **Framework (Svelte unit/browser)** | `vitest` — already configured (`frontend/vitest.config.ts`) |
| **Framework (E2E)** | `playwright test` — already configured (`frontend/playwright.config.ts`) |
| **Framework (UAT)** | Chrome MCP navigation (per `feedback_use_chrome_for_uat.md`) |
| **Config file** | `backend/Cargo.toml`, `frontend/package.json`, `frontend/playwright.config.ts` |
| **Quick run command** | `cargo build --workspace --all-features && cd frontend && pnpm check` |
| **Full suite command** | `make ci` (if target exists) OR: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cd frontend && pnpm test && pnpm build` |
| **Estimated runtime** | ~60–180 seconds (depends on caching) |

---

## Sampling Rate

- **After every task commit:** Run `cargo build --workspace --all-features` + any task-specific `<automated>` verify command.
- **After every plan wave:** Run `cargo test --workspace && cd frontend && pnpm check`.
- **Before `/gsd-verify-work`:** Full suite must be green + Chrome MCP UAT walk of every new catalog screen at desktop + mobile viewport widths.
- **Max feedback latency:** ~120 seconds for the quick command; full suite <5 min.

---

## Per-Task Verification Map

> Populated by planner during Plan creation. Each plan's tasks must have either an `<automated>` verify command or a Wave 0 dependency. Sampling continuity = no 3 consecutive tasks without automated verify.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| {populated by planner} | | | | | | | | | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Expected coverage per CONTEXT.md + RESEARCH.md wave structure:**
- Plan 18-01 (Wave 0 — framework polish): Rust unit tests for Button variant/size/state/loading/icon prop wiring, Rust unit tests for new blur-action builder methods, Svelte browser tests for SelectInput/Checkbox/Switch/RadioGroup blur-action emission, Tailwind safelist extension verified by grep.
- Plan 18-02 (Wave 1 — fixtures.rs): Rust unit tests for `synthetic_rows(n)` — deterministic output, correct row shape, offset/limit paging. Rust unit test for `fetch-rows` handler's new `catalog-synthetic-rows` source arm.
- Plan 18-03..18-07 (Wave 2 — catalog screens 01–05): Each catalog fn has a Rust unit test asserting `gallery_demo()` returns non-empty `Vec<Node>` with the expected root key. Each has a Chrome MCP UAT walk at desktop + mobile viewport (automated via Playwright optionally, manual otherwise).
- Plan 18-XX (GALLERY-DEMOS.md coverage matrix update): grep-verifiable acceptance criterion.

---

## Wave 0 Requirements

Per research §Wave Structure, Plan 18-01 lands framework polish required before any catalog screen can be built:

- [ ] `backend/crates/marionette/src/builders/button.rs` — wire `variant`, `size`, `loading`, `icon` props end-to-end to `frontend/src/lib/components/form/Button.svelte`. Add Rust unit tests covering all 4 new prop paths.
- [ ] `backend/crates/marionette/src/builders/{select.rs, checkbox.rs, switch.rs, radio_group.rs}` — add `.on_blur(ComponentAction)` builder method paralleling TextInput's (RESEARCH §Gap 2).
- [ ] `frontend/src/lib/components/form/{SelectInput.svelte, Checkbox.svelte, Switch.svelte, RadioGroup.svelte}` — dispatch blur action matching TextInput's existing pattern.
- [ ] `frontend/src/app.css:7` — extend `@source inline(...)` safelist with `sm:grid-cols-2 lg:grid-cols-5 lg:grid-cols-6 lg:grid-cols-8`. Grep-verifiable.
- [ ] Rust + Svelte unit/browser tests for all above. Phase 18 catalog plans depend on these.

*If skipped: CAT-01 cannot satisfy SC #1 (missing loading + icon-only states); CAT-02 cannot satisfy SC #2 for 4 of 6 inputs (missing blur); CAT-02/05 layout breaks on tablet/desktop (missing safelist classes).*

---

## Manual-Only Verifications

Catalog screens are UI-heavy; every catalog screen gets a Chrome MCP UAT walk per the `feedback_use_chrome_for_uat.md` global-memory rule.

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CAT-01 full 60-combo matrix visible on one page | CAT-01 SC #1 | Visual correctness (all 60 Button permutations render without layout breakage) is only assessable by eye; test infrastructure cannot judge "visible on one page" without a snapshot framework. | Chrome MCP: visit `/#catalog-buttons`, resize to 375px (phone) + 768px (tablet) + 1280px (desktop), screenshot each, verify all variant × size × state cells are reachable by scroll. |
| CAT-02 live validation clears error via node patch | CAT-02 SC #2 | Round-trip behavior (blur → server validate → patch → error clear) requires real WS + backend — Chrome MCP exercises the full stack. | Chrome MCP: visit `/#catalog-forms`, for each of 6 input types: type invalid value, tab out, verify ErrorDisplay appears; correct value, tab out, verify ErrorDisplay clears; observe network tab for the Phase 12 node-tree patch op. |
| CAT-03 virtualization engages at 500+ rows | CAT-03 SC #3 | Virtualization engagement is observable only by DOM inspection (item count < total row count) and scroll-triggered fetch-rows dispatch. | Chrome MCP: visit `/#catalog-data-table`, scroll to bottom, verify more rows load via WS fetch-rows; open DevTools Elements panel, confirm rendered row count << 500. |
| CAT-04 each feedback surface triggerable individually | CAT-04 SC #4 | UX flow (click trigger → overlay opens → correct content renders → close/accept fires expected action) is a manual walk. | Chrome MCP: click each trigger (Toast, Confirm-Open, Modal-Open, Placeholder), verify overlay/state shown, close via each dismiss path. |
| CAT-05 all semantic tokens rendered as swatches | CAT-05 SC #5 | Visual accuracy of OKLCH swatches (color matches `app.css`) requires eyeballing. | Chrome MCP: visit `/#catalog-typography`, verify 27+ swatches labeled with token names and matching visual colors; toggle dark mode (if implemented) and verify swatches update. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies (populated by planner)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (planner enforces)
- [ ] Wave 0 covers all framework gaps (Button props + 4 blur-wiring + safelist) — this validation strategy assumes Plan 18-01 satisfies it
- [ ] No watch-mode flags in sampling commands
- [ ] Feedback latency < 180s for full suite (measured)
- [ ] `nyquist_compliant: true` set in frontmatter once planner + checker pass

**Approval:** pending
