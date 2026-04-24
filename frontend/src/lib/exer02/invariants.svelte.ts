/**
 * EXER-02 invariant watchers + client-initiated tick loop (Plan 19-03).
 *
 * Provides four DOM-event-driven watchers (focus / cursor / typed / ime)
 * plus a setInterval-driven tick dispatcher that fires
 * `gallery-demo/exer-02/tick` at the configured cadence.
 *
 * A1 resolution: the backend has no broadcast channel (19-CONTEXT.md §D-4
 * locks framework-crate edits out of scope). The FRONTEND drives the cadence
 * via setInterval; each tick calls sendAction('gallery-demo/exer-02/tick').
 * The backend responds with a real PatchMessage via the normal ActionResult
 * path (handle_exer02_tick rotates 3 op kinds per tick — see
 * backend/crates/gallery-demo/src/handlers/exer02.rs).
 *
 * Each watcher reports PASS/FAIL/PENDING locally (no server round-trip)
 * by writing directly into the data store at
 * `/demo/exer-02/invariants/{slug}/state` and `/demo/exer-02/invariants/
 * {slug}/details`. No new action route is registered for update-invariant:
 * Plan 19-01 locked handlers/mod.rs as immutable for Wave 2, and adding
 * local-only updates keeps this plan's files disjoint from Plans 19-02/04.
 *
 * Cursor + IME invariants coordinate with the installable patch probe from
 * `frontend/src/lib/init.ts` (Plan 19-01 Task 1) — the probe fires AFTER
 * each applyPatch, giving the watchers a tick-after moment to sample
 * cursor position and IME composition state.
 *
 * Pitfall 5 / Pitfall 6 (19-RESEARCH.md): the cursor + IME watchers rely on
 * the patch probe to detect post-patch cursor drift; the IME watcher uses
 * compositionstart/compositionupdate/compositionend to bracket composition
 * sessions; InputEvent.isComposing is inspected on each `input` event.
 */

import { sendAction } from '$lib/transport/dispatcher';
import { installPatchProbe } from '$lib/init';
import { setData } from '$lib/store/data.svelte';

// -----------------------------------------------------------------------------
// Public types
// -----------------------------------------------------------------------------

export type InvariantName = 'focus' | 'cursor' | 'typed' | 'ime';
export type InvariantState = 'PASS' | 'FAIL' | 'PENDING';

export interface InvariantUpdate {
	name: InvariantName;
	state: InvariantState;
	details: string;
	timestamp: number;
}

export type UpdateCallback = (u: InvariantUpdate) => void;

export interface ExpectedValueTracker {
	get: () => string;
	set: (v: string) => void;
}

// The surface EXER-02 renders into. Locked to "content" so this module's
// setData calls target the same store that the backend's PatchMessages
// reach. Keep in sync with handle_exer02_start/tick/reset surface value.
const SURFACE = 'content';

// Cadence clamp mirrors the backend's T-19-01 mitigation (defense in depth).
const CADENCE_MIN_MS = 100;
const CADENCE_MAX_MS = 60_000;
const DEFAULT_CADENCE_MS = 500;

// -----------------------------------------------------------------------------
// mountWatchers — four DOM watchers + patch-probe coordination
// -----------------------------------------------------------------------------

/**
 * Mount the four invariant watchers on `input`. Returns a cleanup fn.
 *
 * The onUpdate callback is invoked every time a watcher observes a state
 * transition (PASS / FAIL). Callers typically pass a closure that writes
 * the update into the local data store so the invariant dashboard cells
 * re-render.
 */
export function mountWatchers(
	input: HTMLInputElement,
	onUpdate: UpdateCallback,
	expectedValue: ExpectedValueTracker
): () => void {
	const cleanups: Array<() => void> = [];

	// --- Invariant 1: Focus retention ---
	// Any focusout on the tracked input during a live patching session is a
	// FAIL. If the user intentionally tabs away, the next click on Start
	// resets the invariant via the backend /reset flow.
	const focusOutHandler = () => {
		onUpdate({
			name: 'focus',
			state: 'FAIL',
			details: `Focus lost at ${new Date().toISOString()}`,
			timestamp: performance.now()
		});
	};
	input.addEventListener('focusout', focusOutHandler);
	cleanups.push(() => input.removeEventListener('focusout', focusOutHandler));

	// --- Invariant 3: Typed input integrity ---
	// On each non-composing `input` event, sync the expected tracker with the
	// observed value (user just typed). Drift detection happens later in the
	// patch-probe block — after each patch, expected must still equal the
	// input's live value.
	const typedHandler = (e: Event) => {
		const ev = e as InputEvent;
		if (ev.isComposing) return; // IME handled by Invariant 4
		expectedValue.set(input.value);
	};
	input.addEventListener('input', typedHandler);
	cleanups.push(() => input.removeEventListener('input', typedHandler));

	// --- Invariant 4: IME composition ---
	// We don't have a reliable way to detect mid-composition breakage without
	// platform-specific signals; the observable fact we track is "a
	// composition session that started and also ended cleanly". On each
	// compositionend we report PASS. On a patch-probe observation where
	// composing is still true but the input's value changed in a way that
	// suggests early commit, we could report FAIL — but in v1.2 scope, we
	// report FAIL only if focus is lost mid-composition (handled by
	// Invariant 1) or the input is blurred during composition.
	let composing = false;
	const compStart = () => {
		composing = true;
	};
	const compUpdate = () => {
		// `compositionupdate` is the intra-composition event. No state
		// transition needed here; listen to satisfy the test and to exercise
		// the full composition event lifecycle.
		composing = true;
	};
	const compEnd = () => {
		composing = false;
		onUpdate({
			name: 'ime',
			state: 'PASS',
			details: 'IME composition completed without interruption',
			timestamp: performance.now()
		});
	};
	input.addEventListener('compositionstart', compStart);
	input.addEventListener('compositionupdate', compUpdate);
	input.addEventListener('compositionend', compEnd);
	cleanups.push(() => input.removeEventListener('compositionstart', compStart));
	cleanups.push(() => input.removeEventListener('compositionupdate', compUpdate));
	cleanups.push(() => input.removeEventListener('compositionend', compEnd));

	// --- Invariants 2 + 3: Patch-probe coordination (Pitfall 5) ---
	// installPatchProbe fires AFTER each applyPatch. We sample cursor
	// position + check for typed-value drift.
	let lastSampledSelection = input.selectionStart ?? 0;
	const probe = (latencyMs: number) => {
		// Cursor invariant: the negative signal is "cursor jumped to column 0
		// from a non-zero position while focus is still on the tracked input".
		// More rigorous signals require correlation with recent keydown / input
		// events which is out of scope for v1.2 (documented in SUMMARY).
		const currentSel = input.selectionStart ?? 0;
		if (
			currentSel === 0 &&
			lastSampledSelection > 0 &&
			document.activeElement === input
		) {
			onUpdate({
				name: 'cursor',
				state: 'FAIL',
				details: `Cursor jumped from col ${lastSampledSelection} to 0 after patch (Δ ${latencyMs.toFixed(1)}ms)`,
				timestamp: performance.now()
			});
		}
		lastSampledSelection = currentSel;

		// Typed invariant: after patch, expected should still equal input.value
		// — sibling patches must not touch the focused input's value.
		if (document.activeElement === input && expectedValue.get() !== input.value) {
			onUpdate({
				name: 'typed',
				state: 'FAIL',
				details: `Expected "${expectedValue.get()}" got "${input.value}" after patch`,
				timestamp: performance.now()
			});
		}

		// IME invariant: if patch fired while composing and composition was
		// observably interrupted (the input lost its composition session
		// without a compositionend — detectable only by blur), this is
		// already caught by Invariant 1 (focusout). Log for future diagnostic
		// use without triggering a FAIL here.
		if (composing && document.activeElement !== input) {
			onUpdate({
				name: 'ime',
				state: 'FAIL',
				details: `IME composition interrupted by patch (Δ ${latencyMs.toFixed(1)}ms)`,
				timestamp: performance.now()
			});
		}
	};
	installPatchProbe(probe);
	cleanups.push(() => installPatchProbe(null));

	return () => {
		for (const fn of cleanups) fn();
	};
}

// -----------------------------------------------------------------------------
// Client-initiated tick loop (A1 resolution)
// -----------------------------------------------------------------------------

let tickHandle: ReturnType<typeof setInterval> | null = null;

/**
 * Start the client-initiated tick loop. Fires
 * `sendAction('gallery-demo/exer-02/tick')` every cadenceMs ms until
 * stopTickLoop() is called. Calling start while running is idempotent
 * (the previous interval is cleared first).
 *
 * cadenceMs is clamped to [100, 60 000] ms (T-19-01 defense-in-depth
 * mirror of the backend's handle_exer02_start clamp).
 */
export function startTickLoop(cadenceMs: number): void {
	stopTickLoop(); // idempotent
	const safe = Math.max(
		CADENCE_MIN_MS,
		Math.min(CADENCE_MAX_MS, Math.floor(cadenceMs))
	);
	tickHandle = setInterval(() => {
		sendAction('gallery-demo/exer-02/tick');
	}, safe);
}

/** Stop the tick loop. Idempotent — safe to call when not running. */
export function stopTickLoop(): void {
	if (tickHandle !== null) {
		clearInterval(tickHandle);
		tickHandle = null;
	}
}

// -----------------------------------------------------------------------------
// Auto-arm helper (opt-in, NOT import side effect)
// -----------------------------------------------------------------------------
//
// Per plan's execution note: make auto-arm opt-in via explicit function
// rather than import side-effect to avoid vitest cross-test pollution and
// make testing the module's individual exports clean. The EXER-02 screen's
// root component (or the initMarionette host) calls autoArm() to wire
// the DOM observers for this exerciser.
//
// autoArm() returns a cleanup function that can be called to tear down all
// listeners + the MutationObserver + any running tick loop. Idempotent on
// re-arm: calling autoArm() twice without tearing down first is a no-op.

let armed = false;

/**
 * Auto-wire watchers + CTA click handlers once the EXER-02 screen is
 * visible in the DOM. Returns a cleanup function.
 *
 * Behaviour:
 *  - MutationObserver watches document.body; when an input with id
 *    `exer-02-focused-input` (or a wrapping node containing one) appears,
 *    mountWatchers() is called on it.
 *  - Each invariant update writes to the local data store at
 *    /demo/exer-02/invariants/{name}/{state,details}.
 *  - Clicks on #exer-02-start / #exer-02-pause / #exer-02-reset
 *    start / stop the client tick loop. Cadence is read from the
 *    currently-selected radio in #exer-02-cadence (falls back to 500).
 */
export function autoArm(): () => void {
	if (typeof window === 'undefined') {
		return () => {
			/* no-op on non-browser environments */
		};
	}
	if (armed) {
		return () => {
			/* already armed — caller kept a prior cleanup */
		};
	}
	armed = true;

	let mountedCleanup: (() => void) | null = null;
	const expected: ExpectedValueTracker = (() => {
		let v = '';
		return {
			get: () => v,
			set: (x: string) => {
				v = x;
			}
		};
	})();

	const writeInvariant = (u: InvariantUpdate) => {
		setData(SURFACE, `/demo/exer-02/invariants/${u.name}/state`, u.state);
		setData(SURFACE, `/demo/exer-02/invariants/${u.name}/details`, u.details);
	};

	const tryMount = () => {
		if (mountedCleanup) return;
		const el = document.getElementById('exer-02-focused-input');
		const inputEl =
			el instanceof HTMLInputElement ? el : (el?.querySelector('input') ?? null);
		if (!inputEl) return;
		mountedCleanup = mountWatchers(inputEl, writeInvariant, expected);
	};

	const clickHandler = (ev: Event) => {
		const target = ev.target as HTMLElement | null;
		if (!target) return;
		const startBtn = target.closest('#exer-02-start');
		const pauseBtn = target.closest('#exer-02-pause');
		const resetBtn = target.closest('#exer-02-reset');
		if (startBtn) {
			const radio = document.querySelector<HTMLInputElement>(
				'#exer-02-cadence input[type=radio]:checked'
			);
			const cadence = Number(radio?.value ?? DEFAULT_CADENCE_MS);
			startTickLoop(Number.isFinite(cadence) ? cadence : DEFAULT_CADENCE_MS);
		}
		if (pauseBtn || resetBtn) {
			stopTickLoop();
		}
	};
	document.addEventListener('click', clickHandler, { passive: true });

	const obs = new MutationObserver(() => tryMount());
	obs.observe(document.body, { childList: true, subtree: true });

	if (document.readyState !== 'loading') {
		tryMount();
	} else {
		document.addEventListener('DOMContentLoaded', tryMount, { once: true });
	}

	return () => {
		obs.disconnect();
		document.removeEventListener('click', clickHandler);
		stopTickLoop();
		mountedCleanup?.();
		mountedCleanup = null;
		armed = false;
	};
}
