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

<nav aria-label={props.ariaLabel as string ?? 'Sidebar'} class="flex flex-col h-full bg-sidebar">
	<div class="h-full overflow-y-auto py-4 px-3">
		{@render children?.()}
	</div>
</nav>
