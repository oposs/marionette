<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { getData } from '$lib/store/data.svelte';

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

	let text = $derived(
		bind ? (getData(surface, bind) as string) ?? '' : (props.text as string) ?? ''
	);
	let isLabel = $derived((props.variant as string) === 'label');
	let isMuted = $derived(Boolean(props.muted));
	let isInline = $derived(Boolean(props.inline));

	let colorClass = $derived(isMuted ? 'text-muted-foreground' : 'text-foreground');
	let textClass = $derived(
		isLabel
			? `text-sm font-semibold leading-[1.4] ${colorClass}`
			: `text-sm leading-[1.5] ${colorClass}`
	);
</script>

{#if isInline}
	<span class={textClass}>{text}</span>
{:else}
	<p class={textClass}>{text}</p>
{/if}
