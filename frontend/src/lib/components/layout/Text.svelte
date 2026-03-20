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
	let isInline = $derived(Boolean(props.inline));
</script>

{#if isInline}
	<span class={isLabel ? 'text-sm font-semibold leading-[1.4]' : 'text-sm leading-[1.5]'}>{text}</span>
{:else}
	<p class={isLabel ? 'text-sm font-semibold leading-[1.4]' : 'text-sm leading-[1.5]'}>{text}</p>
{/if}
