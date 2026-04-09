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
	let level = $derived((props.level as number) ?? 2);
</script>

{#if level === 1}
	<h1 class="text-xl font-semibold leading-[1.2] text-foreground">{text}</h1>
{:else if level === 2}
	<h2 class="text-xl font-semibold leading-[1.2] text-foreground">{text}</h2>
{:else if level === 3}
	<h3 class="text-base font-semibold leading-[1.2] text-foreground">{text}</h3>
{:else if level === 4}
	<h4 class="text-base font-semibold leading-[1.2] text-foreground">{text}</h4>
{:else if level === 5}
	<h5 class="text-base font-semibold leading-[1.2] text-foreground">{text}</h5>
{:else}
	<h6 class="text-base font-semibold leading-[1.2] text-foreground">{text}</h6>
{/if}
