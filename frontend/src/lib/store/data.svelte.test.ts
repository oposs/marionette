import { describe, it, expect, beforeEach } from 'vitest';
import { getStore, getData, setData, setFullState, applyPatch, resetStore } from './data.svelte.js';
import { resetDirty, markDirty, isDirty } from './dirty.svelte.js';

describe('data store', () => {
	beforeEach(() => {
		resetStore('main');
		resetDirty();
	});

	it('getStore returns an object with data property', () => {
		const store = getStore('main');
		expect(store).toHaveProperty('data');
		expect(store.data).toEqual({});
	});

	it('getStore called twice returns same reference', () => {
		const a = getStore('main');
		const b = getStore('main');
		expect(a).toBe(b);
	});

	it('getData returns value set at that path', () => {
		setData('main', '/user/name', 'Alice');
		expect(getData('main', '/user/name')).toBe('Alice');
	});

	it('setData then getData round-trips', () => {
		setData('main', '/user/name', 'Alice');
		expect(getData('main', '/user/name')).toBe('Alice');
	});

	it('setData with nested path creates intermediate objects', () => {
		setData('main', '/contacts/c1/email', 'a@b.com');
		expect(getData('main', '/contacts/c1/email')).toBe('a@b.com');
	});

	it('setFullState replaces entire data', () => {
		setData('main', '/old', 'value');
		setFullState('main', { user: { name: 'Bob' } });
		expect(getData('main', '/user/name')).toBe('Bob');
		expect(getData('main', '/old')).toBeUndefined();
	});

	it('applyPatch applies array of PatchOperations', () => {
		applyPatch('main', [
			{ path: '/user/name', value: 'Charlie' },
			{ path: '/user/age', value: 30 },
		]);
		expect(getData('main', '/user/name')).toBe('Charlie');
		expect(getData('main', '/user/age')).toBe(30);
	});

	it('applyPatch skips patches to dirty paths', () => {
		setData('main', '/user/name', 'Original');
		markDirty('/user/name');
		applyPatch('main', [{ path: '/user/name', value: 'ServerValue' }]);
		expect(getData('main', '/user/name')).toBe('Original');
	});

	it('applyPatch with value null deletes the key', () => {
		setData('main', '/user/name', 'Alice');
		applyPatch('main', [{ path: '/user/name', value: null }]);
		expect(getData('main', '/user/name')).toBeUndefined();
	});

	it('resetStore clears all data for a surface', () => {
		setData('main', '/user/name', 'Alice');
		resetStore('main');
		const store = getStore('main');
		expect(store.data).toEqual({});
	});
});
