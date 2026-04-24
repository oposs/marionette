---
phase: 19-exerciser-screens
created: 2026-04-24
status: locked
upstream:
  - .planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-VERIFICATION.md
  - .planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-06-SUMMARY.md (G-02 nestability finding)
  - .planning/phases/18-catalog-screens/18-08-SUMMARY.md (Container icon primitive Phase 19 can reuse)
  - .planning/STATE.md (Blockers/Concerns: AppShell nestability handed to Phase 19)
---

# Phase 19 — Exerciser Screens — CONTEXT

## Phase scope (locked, from ROADMAP)

Three frontend robustness exercisers — Nested AppShell, Rapid Patching, Pathological Scale — surface capability edges that a clean business app never hits, and capture any gaps as deferred items.

**Requirements covered:** EXER-01, EXER-02, EXER-03.

## Locked decisions (this CONTEXT.md)

### D-1 — EXER-01 nestability investigation depth: Frame v1.3 fix proposal

EXER-01 ships the broken-nesting state as evidence (a 4-dimension observation matrix: provider context, mobile sheet, keyboard shortcuts, `--sidebar-*` token inheritance) AND drafts a concrete v1.3 framework extension proposal as a v1.3 seed.

**Why:** The ROADMAP wording "gaps deferred to v1.3 seeds rather than forcing fixes in v1.2" is the literal scope; attempting an actual v1.2 fix would exceed it. But Phase 17's STATE.md hand-off specifically says "Phase 19 EXER-01 owns the resolution — likely requires (a) scoped Sidebar.Provider context in shadcn-svelte, or (b) a scoped-surface-name framework extension". A drafted proposal turns Phase 17's hand-off into a runway for v1.3 instead of leaving the next milestone to start from scratch.

**What this means concretely:**
- Plan EXER-01 builds a real nested-AppShell screen (outer AppShell hosts an inner AppShell in its content slot — the visually-broken state Phase 17 documented).
- Plan EXER-01 instruments the 4 dimensions and writes findings to a structured matrix in the SUMMARY.md.
- Plan EXER-01 produces a separate v1.3 seed file under `.planning/seeds/v1.3-appshell-nestability.md` proposing the framework-extension shape (likely scoped-surface-name + scoped Sidebar.Provider context) with enough detail that it can become a v1.3 phase without re-research.

### D-2 — EXER-02 invariants exercised: all 4 (focus, cursor, typed input, IME)

The Rapid Patching screen exercises all of:
- **Focus retention** — input keeps focus through 60s of patches at default ~500ms cadence (PATCH-02 invariant; the locked goal).
- **Cursor position** — cursor stays at user's character position; doesn't jump to end or position 0 as patches fire.
- **Typed input integrity** — user types fast (or pastes); every keystroke survives, no characters lost. Tests input-event vs patch-tick race conditions.
- **IME composition** — composing CJK / Vietnamese / etc. via IME while patches fire; composition session not broken mid-character. High-value for international users.

**Why all 4:** The cost of adding the extra invariants is low (single screen with one focused input + a 60s timer) and the value is high (each one surfaces a different real-world failure mode the SDUI patch pipeline could have). User explicitly chose all 4.

### D-3 — EXER-03 perf treatment: soft thresholds (advisory) + 4 signals

The Pathological Scale screen captures perf as soft-threshold advisory measurements:

**Signals captured:**
- TTFP (time-to-first-paint) — navigation → first non-skeleton paint
- Scroll FPS — frames-per-second during sustained scroll on the 10k-row DataTable
- Memory snapshot — JS heap size (a) after page mount, (b) after 30s of sustained scroll
- Patch application latency — per-patch delivery time on a heavy page (10k rows + 80 fields mounted)

**Threshold style:**
- Set advisory targets per signal (suggested: scroll FPS ≥ 30, TTFP ≤ 3s, memory growth after 30s scroll ≤ +50MB, patch latency p95 ≤ 50ms — researcher to validate / refine these).
- If observed values miss the targets, flag in SUMMARY.md as a finding (subject to D-4 below).
- Phase verification does NOT fail on missed thresholds — the targets are advisory, not gating.

**Why advisory not gating:** Hardware-dependent perf gating risks false negatives on slower test machines; ROADMAP wording is "observed performance baselines are recorded" — literal interpretation is baseline-only, but soft thresholds add actionable signal without blocking.

### D-4 — Defer-vs-fix policy for findings: defer to v1.3, except trivial fixes

Default response when any of the three exercisers surfaces a real gap:
- **Defer to v1.3 seed** — write a `.planning/seeds/v1.3-{slug}.md` capturing the finding with reproduction steps and proposed scope.

**Exception — trivial fixes apply inline if ALL of:**
- < 30 minutes implementation
- No scope expansion (modifies only files the exerciser plan already touches)
- Doesn't introduce a new dependency or framework concept

**Why:** ROADMAP wording explicitly says "gaps deferred to v1.3 seeds rather than forcing fixes in v1.2" — the default IS defer. But forbidding all in-phase fixes makes the planner reject obvious one-line tweaks that have no downside; the trivial-fix carve-out preserves the principle while allowing pragmatism.

**Examples that qualify as trivial:**
- A 1-line `virtualizer.ts` config change (e.g., `overscan: 5` → `overscan: 10`)
- Adding a missing `aria-label` to an icon button surfaced by EXER-02's keyboard shortcut audit
- A typo in a console-warn message

**Examples that do NOT qualify (defer to v1.3):**
- Refactoring the surface-store update path
- Adding a new patch-batching mechanism
- Anything in the `marionette` framework crate that isn't a 1-line typo

## Decisions inherited from prior phases (NOT revisited)

- **Auto-discovery contract** — Each of the 3 exercisers ships as a `#[gallery_demo]` annotated `pub fn name() -> Node` (Phase 16/17 contract). Nav entry surfaces automatically.
- **Build_tree pattern** — Same flattening pattern catalog/buttons.rs, forms.rs, etc. use (Phase 18 reference).
- **Synthetic data** — EXER-03's 10k row DataTable can extend Plan 18-03's `synthetic_rows(n: usize)` generator (likely just bump the parameter from 500 → 10000). EXER-03's 80-field FormScreen needs a fresh generator.
- **Container icon primitive** — Plan 18-08 added a display-only `icon` field to Container; available for EXER-* screens that need lucide icons without a Button.
- **Modal sub-surface contract** — Phase 17 lessons (compositional popups, empty-Container = closed sentinel) apply if any exerciser triggers a modal.

## Wave / parallelization hint (advisory — final call is gsd-planner's)

The 3 exercisers touch disjoint files (each gets its own `backend/crates/gallery-demo/src/exerciser/{nested_appshell,rapid_patching,pathological_scale}.rs`). They share no state and have no execution dependency on each other. They can ship in a single parallel wave — though plan-checker may split them by complexity. All 3 should run after a (possibly trivial) Wave 1 if any framework polish is needed (TBD by the researcher).

## Deferred ideas / scope creep noted

- (none recorded yet — add here if user mentions ideas during this phase that aren't in EXER-01..03 scope)

## Unblocks

Phase 19 completion unblocks Phase 20 (Live Token Editor) and milestone v1.2 close-out (PROGRESS will be 5/5 phases for v1.2 milestone).
