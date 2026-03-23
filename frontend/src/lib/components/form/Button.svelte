<script lang="ts">
	import { Button as FlowbiteButton } from 'flowbite-svelte';
	import type { ButtonProps } from 'flowbite-svelte';
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
				action.name ?? action.type,
				payload,
				action.target,
				optimisticField ? { patch: optimisticField.patch } : undefined
			);
		}
	}
</script>

<FlowbiteButton
	color={(props.color as ButtonProps['color']) ?? 'blue'}
	size={(props.size as ButtonProps['size']) ?? 'md'}
	disabled={props.disabled as boolean}
	outline={props.outline as boolean}
	onclick={handleClick}
>
	{props.label ?? ''}
</FlowbiteButton>
