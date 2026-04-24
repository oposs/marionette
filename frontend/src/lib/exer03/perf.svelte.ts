/**
 * EXER-03 perf instrumentation (Phase 19 Plan 19-04).
 *
 * Captures 4 signals — TTFP / Scroll FPS / Memory growth / Patch latency
 * p95 — and reports them to the backend via `gallery-demo/exer-03/report-perf`
 * for advisory threshold evaluation (19-CONTEXT.md §D-3).
 *
 * Target consumer: the EXER-03 Pathological Scale gallery screen. This
 * module auto-arms when a perf-cell DOM node (`#exer-03-perf-ttfp`) appears;
 * that keeps the gallery build-out decoupled from the component registry
 * — we do not invent a new Svelte component just to host a lifecycle hook.
 *
 * Source: 19-RESEARCH.md §Pattern 4 (ready-to-copy outline adapted for
 * the module-scoped buffer + single Remeasure click wiring).
 *
 * Pitfall guards:
 *   #3 (TTFP observer registered late) — we use buffered
 *       `performance.getEntriesByType('paint')` instead of a live
 *       PerformanceObserver so late module registration still yields a
 *       valid reading.
 *   #4 (Chromium-only memory) — `performance.memory` is Chromium-only;
 *       `captureMemoryMb` returns `null` on browsers that lack it so the
 *       UI renders the "unavailable" copy instead of NaN.
 */

import { sendAction } from '$lib/transport/dispatcher';
import { installPatchProbe } from '$lib/init';

// -- Public types -----------------------------------------------------------

export interface PerfSnapshot {
	ttfp_ms: number | null;
	fps: number | null;
	memory_mb: number | null;
	latency_p95_ms: number | null;
}

// -- TTFP (buffered read; Pitfall 3) ----------------------------------------

/**
 * Return the `startTime` of the first-paint PerformanceEntry, or `null` if
 * the browser has not yet recorded one. The reading is buffered —
 * `getEntriesByType('paint')` returns all past entries, so late subscription
 * still yields the correct value.
 */
export function captureTTFP(): number | null {
	if (typeof performance === 'undefined' || typeof performance.getEntriesByType !== 'function') {
		return null;
	}
	const entries = performance.getEntriesByType('paint');
	const fp = entries.find((e) => e.name === 'first-paint');
	return fp ? fp.startTime : null;
}

// -- Scroll FPS (rAF delta loop over 5 s window) ----------------------------

const FPS_SAMPLE_WINDOW_MS = 5_000;

/**
 * Sample instantaneous FPS via `requestAnimationFrame` deltas for
 * `FPS_SAMPLE_WINDOW_MS` milliseconds, then invoke `onDone` with the median
 * FPS over the window (median, not mean, to shrug off scroll-jank spikes).
 *
 * Returns a `cancel` function so the caller can abort mid-window
 * (e.g. if the user navigates away).
 */
export function startFpsSampler(onDone: (fps: number) => void): () => void {
	const samples: number[] = [];
	let last = performance.now();
	let running = true;
	const startTime = last;

	const loop = (t: number) => {
		if (!running) return;
		const dt = t - last;
		if (dt > 0) samples.push(1000 / dt);
		last = t;
		if (t - startTime > FPS_SAMPLE_WINDOW_MS) {
			samples.sort((a, b) => a - b);
			const fps = samples[Math.floor(samples.length / 2)] ?? 0;
			onDone(fps);
			running = false;
			return;
		}
		requestAnimationFrame(loop);
	};
	requestAnimationFrame(loop);
	return () => {
		running = false;
	};
}

// -- Memory — Chromium-only guard (Pitfall 4) -------------------------------

/**
 * Read `performance.memory.usedJSHeapSize` and convert to MiB, or return
 * `null` on non-Chromium browsers where the extension is absent.
 *
 * Per UI-SPEC §Error messages (line 315), the backend/UI pair surfaces
 * a "Perf measurement API unavailable" copy when memory reads null — we
 * return null here and let the caller decide how to present that.
 */
export function captureMemoryMb(): number | null {
	if (typeof performance === 'undefined') return null;
	const p = performance as Performance & { memory?: { usedJSHeapSize: number } };
	if (!p.memory || typeof p.memory.usedJSHeapSize !== 'number') return null;
	return p.memory.usedJSHeapSize / (1024 * 1024);
}

// -- Patch-apply latency — via installPatchProbe() hook --------------------

const LATENCY_BUFFER_SIZE = 100;
const latencyBuffer: number[] = [];

/**
 * Record one patch-apply latency into the rolling buffer. Called from the
 * probe callback installed via `installPatchProbe()` — see `init.ts`
 * Phase 19 Plan 19-01 Task 1 for the hook's wire location.
 */
export function recordPatchLatency(ms: number): void {
	if (latencyBuffer.length >= LATENCY_BUFFER_SIZE) {
		latencyBuffer.shift();
	}
	latencyBuffer.push(ms);
}

/**
 * Compute the 95th percentile of the current latency buffer. Returns
 * `null` on an empty buffer so the caller can send `latency_p95_ms: null`
 * which the backend handler skips (no-op on that signal).
 */
export function getLatencyP95(): number | null {
	if (latencyBuffer.length === 0) return null;
	const sorted = [...latencyBuffer].sort((a, b) => a - b);
	const idx = Math.floor(sorted.length * 0.95);
	return sorted[Math.min(idx, sorted.length - 1)];
}

/**
 * Test-only hook: clear the rolling latency buffer.
 *
 * Exposed so vitest cases can assert getLatencyP95() behaviour without
 * cross-pollination from earlier tests. Safe in production — the cost is
 * one function-call indirection that the bundler can inline.
 */
export function __resetLatencyBufferForTests(): void {
	latencyBuffer.length = 0;
}

// -- Round-trip report ------------------------------------------------------

/**
 * Send the snapshot to `gallery-demo/exer-03/report-perf`. The backend
 * handler evaluates each signal against the advisory thresholds and emits
 * PatchMessage Set ops on `/demo/exer-03/perf/{slug}/{value,badge}`.
 */
export function reportPerf(snapshot: PerfSnapshot): void {
	sendAction('gallery-demo/exer-03/report-perf', snapshot as unknown as Record<string, unknown>);
}

// -- Auto-arm on EXER-03 screen mount ---------------------------------------
//
// The guarantee: when the EXER-03 perf readout cell mounts in the DOM,
// this module:
//   1. Installs the patch-latency probe (records into latencyBuffer).
//   2. Captures TTFP + memory(t0) ~100 ms after mount (enough for first-paint).
//   3. Listens for the first scroll → starts the FPS sampler.
//   4. Reports snapshot on mount, again at t+30 s (memory growth), and on
//      Remeasure click.
//
// We detect the mount via MutationObserver rather than a Svelte onMount
// so this module stays decoupled from the gallery's component registry.
// Import-time effect is gated on `typeof window !== 'undefined'` to keep
// SSR safe.

if (typeof window !== 'undefined') {
	let armed = false;
	let memoryT0: number | null = null;

	const arm = () => {
		if (armed) return;
		if (!document.getElementById('exer-03-perf-ttfp')) return;
		armed = true;

		// Install the latency probe before any patches flow in.
		installPatchProbe((dt) => recordPatchLatency(dt));

		// Capture initial values after a microtask so the first-paint entry
		// is already buffered.
		setTimeout(() => {
			memoryT0 = captureMemoryMb();
			reportPerf({
				ttfp_ms: captureTTFP(),
				fps: null,
				memory_mb: null, // growth delta is captured at t+30s, not t0
				latency_p95_ms: getLatencyP95(),
			});
		}, 100);

		// At t+30s, report memory GROWTH (delta from t0) + latency refresh.
		setTimeout(() => {
			const m = captureMemoryMb();
			const growth = m !== null && memoryT0 !== null ? m - memoryT0 : null;
			reportPerf({
				ttfp_ms: null,
				fps: null,
				memory_mb: growth,
				latency_p95_ms: getLatencyP95(),
			});
		}, 30_000);

		// First scroll → kick off the FPS sampler. Capture phase so the
		// DataTable's internal scroll container also trips the listener.
		const scrollHandler = () => {
			window.removeEventListener('scroll', scrollHandler, true);
			startFpsSampler((fps) => {
				reportPerf({ ttfp_ms: null, fps, memory_mb: null, latency_p95_ms: null });
			});
		};
		window.addEventListener('scroll', scrollHandler, true);
	};

	const obs = new MutationObserver(() => arm());
	obs.observe(document.body, { childList: true, subtree: true });
	if (document.readyState !== 'loading') arm();
	else document.addEventListener('DOMContentLoaded', arm);

	// Remeasure click → fresh capture. TTFP is whatever the browser
	// recorded on initial load (same value always); memory + latency are
	// live. FPS is intentionally omitted — a fresh scroll window is
	// required to retrigger the sampler.
	document.addEventListener(
		'click',
		(ev) => {
			const t = ev.target as HTMLElement | null;
			if (!t?.closest('#exer-03-remeasure')) return;
			const m = captureMemoryMb();
			const growth = m !== null && memoryT0 !== null ? m - memoryT0 : null;
			reportPerf({
				ttfp_ms: captureTTFP(),
				fps: null,
				memory_mb: growth,
				latency_p95_ms: getLatencyP95(),
			});
		},
		{ passive: true }
	);
}
