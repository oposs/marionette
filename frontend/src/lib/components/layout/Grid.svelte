<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';

	let {
		props = {},
		bind,
		action,
		surface,
		children
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
		children?: Snippet;
	} = $props();

	const gapMap: Record<string, string> = {
		none: 'gap-0',
		sm: 'gap-2',
		md: 'gap-4',
		lg: 'gap-6',
		xl: 'gap-8'
	};

	let cols = $derived((props.cols as number) ?? 1);
	let gapClass = $derived(gapMap[(props.gap as string) ?? 'md'] ?? 'gap-4');
	let useFlex = $derived(Boolean(props.flex));
	let flow = $derived((props.flow as string) ?? 'row');

	let layoutClass = $derived(
		useFlex
			? `flex flex-wrap ${gapClass}`
			: `grid grid-cols-${cols} ${gapClass} ${flow === 'col' ? 'grid-flow-col' : ''}`
	);
</script>

<div class={layoutClass}>
	{@render children?.()}
</div>
