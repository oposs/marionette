/**
 * Svelte 5 rune-based wrapper around @tanstack/virtual-core's Virtualizer.
 *
 * Why this exists:
 *   @tanstack/svelte-virtual@3.13.23 ships a svelte/store-based adapter
 *   that does NOT play well with Svelte 5's fine-grained reactivity inside
 *   `{#each $store.getVirtualItems() ...}` blocks — the derived store fires
 *   `onChange` but the auto-subscribed `getVirtualItems()` getter is only
 *   re-read on the initial mount, leaving the table empty. See
 *   https://github.com/TanStack/virtual/issues/866.
 *
 * Workaround:
 *   Instantiate the headless Virtualizer directly from @tanstack/virtual-core
 *   (also available as a transitive dep of @tanstack/svelte-virtual) and
 *   bridge its `onChange` callback to a Svelte 5 `$state` tick counter,
 *   which forces consumers using the returned accessors to re-read on every
 *   notify. This is the same pattern the TanStack Solid adapter uses.
 *
 * Usage (in a .svelte file):
 *
 *   <script lang="ts">
 *     import { createRuneVirtualizer } from '$lib/utils/virtualizer.svelte';
 *     let scrollRef = $state<HTMLDivElement | undefined>();
 *     const vr = createRuneVirtualizer<HTMLDivElement, HTMLDivElement>(() => ({
 *       count: rows.length,
 *       getScrollElement: () => scrollRef ?? null,
 *       estimateSize: () => 40,
 *       overscan: 5,
 *     }));
 *   </script>
 *
 *   <div bind:this={scrollRef} style="height: 300px; overflow: auto;">
 *     <div style="height: {vr.totalSize}px; position: relative;">
 *       {#each vr.virtualItems as item (item.key)}
 *         <div style="transform: translateY({item.start}px);">Row {item.index}</div>
 *       {/each}
 *     </div>
 *   </div>
 *
 * Lifecycle:
 *   - The caller MUST call `vr.mount()` once the scroll element is bound
 *     (typically from an `$effect` that fires after `scrollRef` becomes truthy).
 *   - Call `vr.destroy()` from the component's `onDestroy` (or returned `$effect`
 *     cleanup) to disconnect observers.
 *   - Call `vr.setOptions(newPartial)` to update count/estimateSize without
 *     re-creating the virtualizer instance.
 */

import {
	Virtualizer,
	observeElementOffset,
	observeElementRect,
	elementScroll,
	type VirtualItem,
	type VirtualizerOptions,
} from '@tanstack/virtual-core';

type RequiredKeys = 'count' | 'getScrollElement' | 'estimateSize';
type UserOptions<TScroll extends Element, TItem extends Element> = Pick<
	VirtualizerOptions<TScroll, TItem>,
	RequiredKeys
> &
	Partial<Omit<VirtualizerOptions<TScroll, TItem>, RequiredKeys>>;

export interface RuneVirtualizer<
	TScroll extends Element,
	TItem extends Element,
> {
	/** Instance access (use sparingly — prefer the reactive accessors). */
	readonly instance: Virtualizer<TScroll, TItem>;
	/** Reactive total virtual size in pixels. */
	readonly totalSize: number;
	/** Reactive array of visible (windowed) virtual items. */
	readonly virtualItems: VirtualItem[];
	/** Call after the scroll element is bound to start observers. */
	mount: () => void;
	/** Call on component teardown to disconnect observers. */
	destroy: () => void;
	/** Update options (e.g. new `count`) without re-creating the instance. */
	setOptions: (
		partial: Partial<VirtualizerOptions<TScroll, TItem>>,
	) => void;
}

export function createRuneVirtualizer<
	TScroll extends Element,
	TItem extends Element,
>(getOptions: () => UserOptions<TScroll, TItem>): RuneVirtualizer<TScroll, TItem> {
	// Reactivity tick — bumped in onChange so consumers re-read accessors.
	let tick = $state(0);

	const buildOptions = (): VirtualizerOptions<TScroll, TItem> => {
		const user = getOptions();
		return {
			observeElementRect,
			observeElementOffset,
			scrollToFn: elementScroll,
			...user,
			onChange: (inst, sync) => {
				tick++;
				user.onChange?.(inst, sync);
			},
		} as VirtualizerOptions<TScroll, TItem>;
	};

	const instance = new Virtualizer<TScroll, TItem>(buildOptions());
	let unsub: (() => void) | undefined;
	let mounted = false;

	return {
		instance,
		get totalSize() {
			// Touch tick so consumers that read .totalSize re-run on every notify.
			void tick;
			return instance.getTotalSize();
		},
		get virtualItems() {
			void tick;
			return instance.getVirtualItems();
		},
		mount() {
			// Idempotent — safe to call repeatedly from `$effect`.
			if (mounted) return;
			mounted = true;
			// _didMount wires up element observers (ResizeObserver on scroll el
			// + scroll listener). onChange will bump `tick` once measurements
			// arrive; do NOT bump here or we create a loop with the owning $effect.
			unsub = instance._didMount();
			instance._willUpdate();
		},
		destroy() {
			unsub?.();
			unsub = undefined;
			mounted = false;
		},
		setOptions(partial) {
			instance.setOptions({ ...instance.options, ...partial });
		},
	};
}
