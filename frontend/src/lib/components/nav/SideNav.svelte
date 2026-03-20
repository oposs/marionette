<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { Sidebar, SidebarWrapper } from 'flowbite-svelte';

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

<Sidebar ariaLabel={props.ariaLabel as string | undefined} alwaysOpen={true} position="static">
	<SidebarWrapper>
		{@render children?.()}
	</SidebarWrapper>
</Sidebar>
