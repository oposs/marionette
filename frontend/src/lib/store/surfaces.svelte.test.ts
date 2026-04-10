import { describe, test, expect, beforeEach } from 'vitest';
import {
	setSurfaceTree,
	getSurfaceTree,
	clearSurfaceTree,
	setNode,
	deleteNode,
	setChildren,
	insertChild,
	removeChild,
	gcOrphans,
} from './surfaces.svelte';
import type { ComponentNode } from '$lib/transport/messages';

const SURFACE = 'test-surface';

function textNode(label: string, bind?: string): ComponentNode {
	return { type: 'text-input', props: { label }, bind };
}

function container(children: string[]): ComponentNode {
	return { type: 'container', children };
}

beforeEach(() => {
	clearSurfaceTree(SURFACE);
});

describe('surfaces.svelte.ts fine-grained mutation API', () => {
	test('setSurfaceTree establishes a tree that getSurfaceTree reads back', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: textNode('A'),
		});
		const tree = getSurfaceTree(SURFACE);
		expect(tree?.root).toBe('root');
		expect(tree?.nodes['a'].type).toBe('text-input');
	});

	test('setNode mutates nodes[id] in place', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: textNode('A'),
		});
		const treeBefore = getSurfaceTree(SURFACE);
		const nodesRefBefore = treeBefore!.nodes;

		setNode(SURFACE, 'a', textNode('A renamed'));

		const treeAfter = getSurfaceTree(SURFACE);
		expect(treeAfter!.nodes).toBe(nodesRefBefore); // same reference
		expect(treeAfter!.nodes['a'].props?.label).toBe('A renamed');
	});

	test('deleteNode removes entry from tree.nodes', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: textNode('A'),
		});
		deleteNode(SURFACE, 'a');
		expect(getSurfaceTree(SURFACE)!.nodes['a']).toBeUndefined();
	});

	test('setChildren replaces parent.children with new order', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a', 'b', 'c']),
			a: textNode('A'),
			b: textNode('B'),
			c: textNode('C'),
		});
		setChildren(SURFACE, 'root', ['c', 'a', 'b']);
		expect(getSurfaceTree(SURFACE)!.nodes['root'].children).toEqual(['c', 'a', 'b']);
	});

	test('insertChild inserts at index', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a', 'c']),
			a: textNode('A'),
			c: textNode('C'),
		});
		insertChild(SURFACE, 'root', 1, 'b');
		expect(getSurfaceTree(SURFACE)!.nodes['root'].children).toEqual(['a', 'b', 'c']);
	});

	test('removeChild removes first matching id', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a', 'b', 'c']),
			a: textNode('A'),
			b: textNode('B'),
			c: textNode('C'),
		});
		removeChild(SURFACE, 'root', 'b');
		expect(getSurfaceTree(SURFACE)!.nodes['root'].children).toEqual(['a', 'c']);
	});

	test('gcOrphans deletes unreachable nodes via BFS from root', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: textNode('A'),
			// Orphans — never referenced by root's children transitively
			ghost1: textNode('Ghost 1'),
			ghost2: textNode('Ghost 2'),
		});
		gcOrphans(SURFACE);
		const nodes = getSurfaceTree(SURFACE)!.nodes;
		expect(Object.keys(nodes).sort()).toEqual(['a', 'root']);
		expect(nodes['ghost1']).toBeUndefined();
		expect(nodes['ghost2']).toBeUndefined();
	});

	test('gcOrphans preserves deep descendants', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: container(['a1', 'a2']),
			a1: textNode('A1'),
			a2: container(['a2a']),
			a2a: textNode('A2a'),
			orphan: textNode('Orphan'),
		});
		gcOrphans(SURFACE);
		const keys = Object.keys(getSurfaceTree(SURFACE)!.nodes).sort();
		expect(keys).toEqual(['a', 'a1', 'a2', 'a2a', 'root']);
	});

	test('mutators on a non-existent surface are no-ops (no throw)', () => {
		expect(() => setNode('missing', 'x', textNode('X'))).not.toThrow();
		expect(() => deleteNode('missing', 'x')).not.toThrow();
		expect(() => setChildren('missing', 'x', [])).not.toThrow();
		expect(() => insertChild('missing', 'x', 0, 'y')).not.toThrow();
		expect(() => removeChild('missing', 'x', 'y')).not.toThrow();
		expect(() => gcOrphans('missing')).not.toThrow();
	});
});
