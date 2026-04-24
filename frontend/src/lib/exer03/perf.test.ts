/**
 * EXER-03 perf module unit tests (Phase 19 Plan 19-04 Task 3).
 *
 * The perf module is side-effecting at import time (installs a
 * MutationObserver + click listener). Tests run in `environment: 'node'`
 * per vite.config.ts — `typeof window === 'undefined'` in node, so the
 * side-effect block is bypassed and we can exercise the pure helpers in
 * isolation. The `performance` shim is Node's own (`perf_hooks.performance`
 * is polyfilled into `globalThis.performance` by vitest/node>=16).
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const sentinel = vi.fn();
vi.mock('$lib/transport/dispatcher', () => ({ sendAction: sentinel }));
vi.mock('$lib/init', () => ({ installPatchProbe: vi.fn() }));

describe('EXER-03 perf module', () => {
	beforeEach(() => {
		sentinel.mockReset();
		// Reset the `performance` shim to a minimal one per test.
		(globalThis as unknown as { performance: Partial<Performance> }).performance = {
			getEntriesByType: vi.fn(() => []),
			now: () => Date.now(),
		} as unknown as Performance;
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('captureTTFP returns null when no paint entries are buffered', async () => {
		const { captureTTFP } = await import('./perf.svelte');
		expect(captureTTFP()).toBeNull();
	});

	it('captureTTFP returns startTime for the first-paint entry', async () => {
		const perf = globalThis.performance as unknown as {
			getEntriesByType: ReturnType<typeof vi.fn>;
		};
		perf.getEntriesByType.mockReturnValue([
			{ name: 'first-paint', startTime: 1234.5 },
			{ name: 'first-contentful-paint', startTime: 1300 },
		]);
		const { captureTTFP } = await import('./perf.svelte');
		expect(captureTTFP()).toBe(1234.5);
	});

	it('captureMemoryMb returns null when performance.memory is absent', async () => {
		const { captureMemoryMb } = await import('./perf.svelte');
		expect(captureMemoryMb()).toBeNull();
	});

	it('captureMemoryMb converts usedJSHeapSize bytes to MiB', async () => {
		(globalThis as unknown as { performance: { memory: { usedJSHeapSize: number } } }).performance =
			{
				...(globalThis.performance as unknown as object),
				memory: { usedJSHeapSize: 10 * 1024 * 1024 },
			} as never;
		const { captureMemoryMb } = await import('./perf.svelte');
		expect(captureMemoryMb()).toBeCloseTo(10, 1);
	});

	it('getLatencyP95 returns null on an empty buffer', async () => {
		const { getLatencyP95, __resetLatencyBufferForTests } = await import('./perf.svelte');
		__resetLatencyBufferForTests();
		expect(getLatencyP95()).toBeNull();
	});

	it('getLatencyP95 computes the 95th percentile over 100 samples', async () => {
		const { recordPatchLatency, getLatencyP95, __resetLatencyBufferForTests } = await import(
			'./perf.svelte'
		);
		__resetLatencyBufferForTests();
		for (let i = 1; i <= 100; i++) recordPatchLatency(i);
		const p95 = getLatencyP95();
		expect(p95).not.toBeNull();
		// Implementations differ on the off-by-one; accept [95, 96].
		expect(p95!).toBeGreaterThanOrEqual(95);
		expect(p95!).toBeLessThanOrEqual(96);
	});

	it('recordPatchLatency caps the rolling buffer at 100 entries', async () => {
		const { recordPatchLatency, getLatencyP95, __resetLatencyBufferForTests } = await import(
			'./perf.svelte'
		);
		__resetLatencyBufferForTests();
		// 150 entries should shift the oldest 50 out, leaving the latest 100 (51..150).
		for (let i = 1; i <= 150; i++) recordPatchLatency(i);
		const p95 = getLatencyP95();
		expect(p95).not.toBeNull();
		// p95 of 51..150 should be >= 145 (latest 100 samples in sorted order).
		expect(p95!).toBeGreaterThanOrEqual(145);
	});

	it('reportPerf calls sendAction with the exact snapshot payload', async () => {
		const { reportPerf } = await import('./perf.svelte');
		const snap = { ttfp_ms: 1500, fps: 45, memory_mb: 20, latency_p95_ms: 10 };
		reportPerf(snap);
		expect(sentinel).toHaveBeenCalledTimes(1);
		expect(sentinel).toHaveBeenCalledWith('gallery-demo/exer-03/report-perf', snap);
	});

	it('reportPerf handles snapshots with null signals', async () => {
		const { reportPerf } = await import('./perf.svelte');
		const snap = { ttfp_ms: null, fps: 45, memory_mb: null, latency_p95_ms: null };
		reportPerf(snap);
		expect(sentinel).toHaveBeenCalledTimes(1);
		expect(sentinel).toHaveBeenCalledWith('gallery-demo/exer-03/report-perf', snap);
	});

	it('startFpsSampler resolves with a positive fps after the sample window elapses', async () => {
		// Shim requestAnimationFrame to fire synchronously with a fixed
		// frame delta until we pass the 5 s window boundary.
		let t = 0;
		const originalRaf = globalThis.requestAnimationFrame;
		(globalThis as unknown as { requestAnimationFrame: (cb: (t: number) => void) => number }).requestAnimationFrame =
			(cb: (t: number) => void) => {
				t += 16.67;
				if (t > 5200) {
					// Let the last frame still run to trigger onDone.
					setTimeout(() => cb(t), 0);
					return 0;
				}
				cb(t);
				return 1;
			};
		// Override performance.now so samples converge to ~60 FPS.
		(globalThis.performance as unknown as { now: () => number }).now = () => t;

		const { startFpsSampler } = await import('./perf.svelte');
		const fps = await new Promise<number>((resolve) => startFpsSampler(resolve));
		expect(fps).toBeGreaterThan(0);
		// 16.67 ms delta → ~60 FPS; allow slack for any boundary-condition skew.
		expect(fps).toBeGreaterThanOrEqual(50);
		expect(fps).toBeLessThanOrEqual(70);

		(globalThis as unknown as { requestAnimationFrame: typeof originalRaf }).requestAnimationFrame =
			originalRaf;
	});
});
