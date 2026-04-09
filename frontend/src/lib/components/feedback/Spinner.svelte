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

<svg
	class="animate-spin text-primary size-{(props.size as string) ?? '6'}"
	xmlns="http://www.w3.org/2000/svg"
	fill="none"
	viewBox="0 0 24 24"
>
	<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
	<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
</svg>
