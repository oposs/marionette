// @vitest-environment jsdom
/**
 * EXER-02 invariants — unit tests for mountWatchers + tick loop (Plan 19-03).
 *
 * Scope:
 *  - mountWatchers installs the 4 DOM watchers (focusout / input /
 *    compositionstart / compositionupdate / compositionend) and a
 *    patch-probe observer.
 *  - The returned cleanup removes all listeners.
 *  - startTickLoop sends gallery-demo/exer-02/tick at cadence; stopTickLoop
 *    halts further ticks.
 *
 * Out of scope (verified at UAT time in Plan 19-05):
 *  - The autoArm() path, which wires MutationObserver + click handlers.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Hoisted mocks — vi.mock calls are lifted to the top of the file, so any
// variable referenced inside the factory must also be hoisted via
// vi.hoisted (otherwise we hit "Cannot access before initialization").
const { sentinel, probeInstall } = vi.hoisted(() => ({
	sentinel: vi.fn(),
	probeInstall: vi.fn()
}));

vi.mock('$lib/transport/dispatcher', () => ({ sendAction: sentinel }));
vi.mock('$lib/init', () => ({ installPatchProbe: probeInstall }));
vi.mock('$lib/store/data.svelte', () => ({ setData: vi.fn() }));

import {
	mountWatchers,
	startTickLoop,
	stopTickLoop,
	type ExpectedValueTracker
} from './invariants.svelte';

function makeExpected(initial = ''): ExpectedValueTracker {
	let v = initial;
	return {
		get: () => v,
		set: (x: string) => {
			v = x;
		}
	};
}

describe('EXER-02 invariants module — mountWatchers', () => {
	let input: HTMLInputElement;

	beforeEach(() => {
		sentinel.mockReset();
		probeInstall.mockReset();
		document.body.innerHTML = '';
		input = document.createElement('input');
		input.type = 'text';
		document.body.appendChild(input);
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('focus watcher reports FAIL on focusout', () => {
		const cb = vi.fn();
		const cleanup = mountWatchers(input, cb, makeExpected());

		input.dispatchEvent(new FocusEvent('focusout'));

		expect(cb).toHaveBeenCalledWith(
			expect.objectContaining({ name: 'focus', state: 'FAIL' })
		);
		cleanup();
	});

	it('cleanup fn removes focusout listener', () => {
		const cb = vi.fn();
		const cleanup = mountWatchers(input, cb, makeExpected());
		cleanup();

		input.dispatchEvent(new FocusEvent('focusout'));
		expect(cb).not.toHaveBeenCalled();
	});

	it('IME watcher reports PASS on compositionend after compositionstart', () => {
		const cb = vi.fn();
		const cleanup = mountWatchers(input, cb, makeExpected());

		input.dispatchEvent(new CompositionEvent('compositionstart'));
		input.dispatchEvent(new CompositionEvent('compositionend'));

		expect(cb).toHaveBeenCalledWith(
			expect.objectContaining({ name: 'ime', state: 'PASS' })
		);
		cleanup();
	});

	it('mountWatchers installs the patch probe on install and clears it on cleanup', () => {
		const cb = vi.fn();
		const cleanup = mountWatchers(input, cb, makeExpected());

		// Install should have been called once with a function.
		expect(probeInstall).toHaveBeenCalledTimes(1);
		expect(typeof probeInstall.mock.calls[0][0]).toBe('function');

		cleanup();
		// Cleanup must have called installPatchProbe(null).
		expect(probeInstall).toHaveBeenCalledTimes(2);
		expect(probeInstall.mock.calls[1][0]).toBeNull();
	});

	it('typed watcher syncs expected tracker on non-composing input events', () => {
		const cb = vi.fn();
		const expected = makeExpected();
		const cleanup = mountWatchers(input, cb, expected);

		input.value = 'hello';
		// Use a plain Event — InputEvent.isComposing defaults to false.
		input.dispatchEvent(new Event('input'));

		expect(expected.get()).toBe('hello');
		cleanup();
	});
});

describe('EXER-02 invariants module — tick loop', () => {
	beforeEach(() => {
		sentinel.mockReset();
	});

	afterEach(() => {
		stopTickLoop();
		vi.useRealTimers();
	});

	it('startTickLoop sends ticks at the configured cadence', () => {
		vi.useFakeTimers();
		startTickLoop(500);
		vi.advanceTimersByTime(1250); // ~2.5 intervals
		expect(sentinel).toHaveBeenCalledTimes(2);
		expect(sentinel).toHaveBeenCalledWith('gallery-demo/exer-02/tick');
		stopTickLoop();
	});

	it('stopTickLoop prevents further ticks', () => {
		vi.useFakeTimers();
		startTickLoop(500);
		vi.advanceTimersByTime(550);
		expect(sentinel).toHaveBeenCalledTimes(1);
		stopTickLoop();
		vi.advanceTimersByTime(2000);
		expect(sentinel).toHaveBeenCalledTimes(1); // no additional ticks
	});

	it('startTickLoop is idempotent — re-calling clears the previous interval', () => {
		vi.useFakeTimers();
		startTickLoop(500);
		startTickLoop(500); // restart immediately; previous interval must be cleared
		vi.advanceTimersByTime(550);
		expect(sentinel).toHaveBeenCalledTimes(1); // only the second interval ticks
		stopTickLoop();
	});

	it('startTickLoop clamps sub-floor cadence to [100, 60 000] ms', () => {
		vi.useFakeTimers();
		startTickLoop(10); // below floor
		vi.advanceTimersByTime(105); // just over clamped floor (100)
		expect(sentinel).toHaveBeenCalledTimes(1);
		stopTickLoop();
	});
});
