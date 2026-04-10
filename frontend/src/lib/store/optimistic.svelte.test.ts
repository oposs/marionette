import { describe, it, expect, beforeEach } from 'vitest';
import { applyOptimistic, confirmOptimistic, rollbackOptimistic } from './optimistic.svelte.js';
import { getData, setData, resetStore } from './data.svelte.js';
import { resetDirty } from './dirty.svelte.js';

describe('optimistic updates', () => {
	beforeEach(() => {
		resetStore('main');
		resetDirty();
	});

	it('applyOptimistic snapshots and applies patch', () => {
		setData('main', '/user/name', 'Alice');
		applyOptimistic('req-1', 'main', [{ op: 'set', path: '/user/name', value: 'Optimistic' }]);
		expect(getData('main', '/user/name')).toBe('Optimistic');
	});

	it('confirmOptimistic removes snapshot (no-op on data)', () => {
		setData('main', '/user/name', 'Alice');
		applyOptimistic('req-1', 'main', [{ op: 'set', path: '/user/name', value: 'Optimistic' }]);
		confirmOptimistic('req-1');
		expect(getData('main', '/user/name')).toBe('Optimistic');
	});

	it('rollbackOptimistic restores original values', () => {
		setData('main', '/user/name', 'Alice');
		applyOptimistic('req-1', 'main', [{ op: 'set', path: '/user/name', value: 'Optimistic' }]);
		rollbackOptimistic('req-1');
		expect(getData('main', '/user/name')).toBe('Alice');
	});

	it('rollbackOptimistic for unknown ID is a no-op', () => {
		setData('main', '/user/name', 'Alice');
		rollbackOptimistic('nonexistent');
		expect(getData('main', '/user/name')).toBe('Alice');
	});
});
