<!--
  svelte-virtual Svelte 5 compatibility decision (Phase 13 Wave 0 smoke test):
  - Path chosen: VIRTUAL-CORE-DIRECT
  - Reason: @tanstack/svelte-virtual@3.13.23's svelte/store-based adapter
    calls `onChange` → `writable.set(instance)` but consumers reading
    `$store.getVirtualItems()` inside `{#each ...}` only get the initial
    (empty) snapshot under Svelte 5 — the table renders empty even though
    `$store.getTotalSize()` correctly reports 4000px. See
    https://github.com/TanStack/virtual/issues/866.
  - Workaround: Use `$lib/utils/virtualizer.svelte.ts`'s `createRuneVirtualizer`,
    which wraps `@tanstack/virtual-core`'s `Virtualizer` directly and bridges
    `onChange` to a Svelte 5 `$state` tick counter. Proven by this smoke test.
  - Downstream DataTable.svelte (Plan 05) MUST use the same virtual-core-direct
    path via `createRuneVirtualizer`, NOT `createVirtualizer` from
    `@tanstack/svelte-virtual`.
-->
<script lang="ts">
	import { onDestroy } from 'svelte';
	import { createRuneVirtualizer } from '$lib/utils/virtualizer.svelte';

	let scrollRef = $state<HTMLDivElement | undefined>();
	const count = 100;

	const vr = createRuneVirtualizer<HTMLDivElement, HTMLDivElement>(() => ({
		count,
		getScrollElement: () => scrollRef ?? null,
		estimateSize: () => 40,
		overscan: 5,
	}));

	// Mount observers after the scroll element is bound.
	$effect(() => {
		if (scrollRef) {
			vr.mount();
		}
	});

	onDestroy(() => vr.destroy());
</script>

<div bind:this={scrollRef} style="height: 300px; overflow: auto;" data-testid="scroll">
	<div
		style="height: {vr.totalSize}px; position: relative; width: 100%;"
		data-testid="inner"
	>
		{#each vr.virtualItems as item (item.key)}
			<div
				data-testid="row-{item.index}"
				style="position: absolute; top: 0; left: 0; width: 100%; height: {item.size}px; transform: translateY({item.start}px);"
			>
				Row {item.index}
			</div>
		{/each}
	</div>
</div>
