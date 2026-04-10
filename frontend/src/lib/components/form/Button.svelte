<script lang="ts">
	import { Button as ShadcnButton } from '$lib/components/ui/button';
	import { getIcon } from '$lib/registry/icons';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import { sendAction } from '$lib/transport/dispatcher';
	import { getAllData } from '$lib/store/data.svelte';
	import type { ComponentAction, PatchOperation } from '$lib/transport/messages';
	import type { Snippet } from 'svelte';

	let {
		props = {},
		bind,
		action,
		surface,
		children,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
		children?: Snippet;
	} = $props();

	let variant = $derived(
		(props.color as string) === 'red' ? 'destructive' as const
		: (props.outline as boolean) ? 'outline' as const
		: 'default' as const
	);

	let isIconOnly = $derived(!props.label && !!props.icon);
	let isLoading = $derived(!!props.loading);

	function handleClick() {
		if (action) {
			const optimisticField = action.optimistic as
				| { patch: PatchOperation[] }
				| undefined;
			// Include all surface data as payload so the backend receives form values
			const surfaceData = getAllData(surface) ?? {};
			const payload = {
				...(action.payload as Record<string, unknown> ?? {}),
				...surfaceData
			};
			sendAction(
				// Do NOT fall back to action.type here: `type` is a protocol
				// classifier (e.g. 'destructive'), not a backend action name.
				// If the server omits `name`, use a generic literal.
				action.name ?? 'button-click',
				payload,
				action.target,
				optimisticField ? { patch: optimisticField.patch } : undefined
			);
		}
	}
</script>

<ShadcnButton
	{variant}
	size={isIconOnly ? 'icon' : 'default'}
	disabled={isLoading || (props.disabled as boolean)}
	onclick={handleClick}
	class={props.icon && props.label ? 'gap-2' : ''}
	aria-label={isIconOnly ? (props.ariaLabel as string) ?? (props.label as string) ?? (props.icon as string) : undefined}
>
	{#if isLoading}
		<Loader2 class="size-4 animate-spin" />
	{:else if props.icon}
		{@const IconComp = getIcon(props.icon as string)}
		<IconComp class="size-4" />
	{/if}
	{#if props.label}
		{props.label}
	{/if}
</ShadcnButton>
