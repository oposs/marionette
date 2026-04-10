import { describe, it, expect, beforeEach, vi } from 'vitest';
import { markDirty, clearDirty, isDirty, queuePatch, resetDirty } from './dirty.svelte.js';

describe('dirty tracking', () => {
	beforeEach(() => {
		resetDirty();
	});

	it('markDirty then isDirty returns true', () => {
		markDirty('/user/name');
		expect(isDirty('/user/name')).toBe(true);
	});

	it('isDirty returns false for unmarked paths', () => {
		expect(isDirty('/user/name')).toBe(false);
	});

	it('isDirty returns true for child path when parent is dirty', () => {
		markDirty('/user/name');
		expect(isDirty('/user/name/first')).toBe(true);
	});

	it('clearDirty then isDirty returns false', () => {
		markDirty('/user/name');
		clearDirty('/user/name', () => {});
		expect(isDirty('/user/name')).toBe(false);
	});

	it('queuePatch queues ops; clearDirty applies them via callback', () => {
		markDirty('/user/name');
		queuePatch('/user/name', { op: 'set', path: '/user/name', value: 'ServerUpdate' });

		const applied: unknown[] = [];
		clearDirty('/user/name', (op) => applied.push(op));

		expect(applied).toEqual([{ op: 'set', path: '/user/name', value: 'ServerUpdate' }]);
	});

	it('multiple queued patches applied in order on clearDirty', () => {
		markDirty('/user/name');
		queuePatch('/user/name', { op: 'set', path: '/user/name', value: 'First' });
		queuePatch('/user/name', { op: 'set', path: '/user/name', value: 'Second' });

		const applied: unknown[] = [];
		clearDirty('/user/name', (op) => applied.push(op));

		expect(applied).toEqual([
			{ op: 'set', path: '/user/name', value: 'First' },
			{ op: 'set', path: '/user/name', value: 'Second' },
		]);
	});
});
