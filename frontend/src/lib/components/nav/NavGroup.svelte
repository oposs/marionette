<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { SidebarGroup } from 'flowbite-svelte';

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

<SidebarGroup>
	{@render children?.()}
</SidebarGroup>
