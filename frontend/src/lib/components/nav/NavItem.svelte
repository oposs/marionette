<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { sendAction } from '$lib/transport/dispatcher';
	import { getData } from '$lib/store/data.svelte';
	import { Button as ShadcnButton } from '$lib/components/ui/button';
	import { getIcon } from '$lib/registry/icons';

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

	let IconComponent = $derived(
		props.icon ? getIcon(props.icon as string) : undefined
	);

	function handleClick(e: Event) {
		e.preventDefault();
		if (action) {
			// Fall back through name -> type -> literal 'navigate' so we never
			// dispatch an undefined action name to the backend.
			sendAction(
				action.name ?? action.type ?? 'navigate',
				action.payload as Record<string, unknown> | undefined
			);
		} else {
			sendAction('navigate', { path: (props.href as string) ?? '' });
		}
	}
</script>

<ShadcnButton
	variant="ghost"
	class="w-full justify-start gap-2 {isActive ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'text-muted-foreground'}"
	onclick={handleClick}
>
	{#if IconComponent}
		<IconComponent class="size-4" />
	{/if}
	{props.label ?? ''}
</ShadcnButton>
