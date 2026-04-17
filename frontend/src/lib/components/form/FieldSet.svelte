<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import type { ComponentAction } from '$lib/transport/messages';
	import type { Snippet } from 'svelte';

	// SDUI contract declares all four standard props for NodeRenderer
	// invocation uniformity. `bind` and `action` are unused — FieldSet is a
	// passive structural container (D-C1). Children arrive via the Snippet
	// after NodeRenderer resolves the adjacency-list child ids.
	let {
		props = {},
		bind,
		action,
		surface,
		children,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
		children?: Snippet;
	} = $props();

	let cols = $derived(props.cols as number | undefined);

	// D-C3 default: responsive 1-col mobile, 2-col desktop (md:768px+).
	// D-C4 override: explicit `cols` uses inline `grid-template-columns`
	// with a static `grid gap-4` class. Pitfall #1 — Tailwind v4 JIT cannot
	// resolve dynamic `grid-cols-{N}` class names, so inline style is the
	// required workaround. Truthy check treats both `undefined` and `0` as
	// "use default"; D-C4 does not allow 0 as a valid column count.
	let gridClass = $derived(cols ? 'grid gap-4' : 'grid grid-cols-1 md:grid-cols-2 gap-4');
	let gridStyle = $derived(
		cols ? `grid-template-columns: repeat(${cols}, minmax(0, 1fr))` : undefined
	);
</script>

<Field.Set>
	{#if props.legend}
		<Field.Legend class="font-semibold">{props.legend}</Field.Legend>
	{/if}
	{#if props.description}
		<Field.Description>{props.description as string}</Field.Description>
	{/if}
	<Field.Group class={gridClass} style={gridStyle}>
		{@render children?.()}
	</Field.Group>
</Field.Set>
