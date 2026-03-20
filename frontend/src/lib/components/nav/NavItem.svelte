<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { SidebarItem } from 'flowbite-svelte';
	import { sendAction } from '$lib/transport/dispatcher';
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

	let isActive = $derived(
		bind ? Boolean(getData(surface, bind)) : Boolean(props.active)
	);

	function handleClick(e: Event) {
		e.preventDefault();
		sendAction('navigate', { path: (props.href as string) ?? '' });
	}
</script>

<SidebarItem
	label={props.label as string ?? ''}
	href={props.href as string | undefined}
	active={isActive}
	onclick={handleClick}
/>
