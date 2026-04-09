<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
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
		if (action) {
			sendAction(action.name, action.payload as Record<string, unknown> | undefined);
		} else {
			sendAction('navigate', { path: (props.href as string) ?? '' });
		}
	}
</script>

<button
	class="flex w-full items-center rounded-md px-3 py-2 text-sm {isActive ? 'bg-accent text-accent-foreground font-medium' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
	onclick={handleClick}
>
	{props.label ?? ''}
</button>
