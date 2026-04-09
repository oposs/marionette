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
</script>

<div class="mt-2 space-y-1">
	{#if props.label}
		<p class="px-3 py-1 text-xs font-semibold uppercase text-muted-foreground">{props.label}</p>
	{/if}
	{@render children?.()}
</div>
